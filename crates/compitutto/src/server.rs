use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get},
    Json, Router,
};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, error, info};

use crate::data::{self, generate_study_sessions, is_test_or_quiz};
use crate::db::{self, EntryUpdate};
use crate::html;
use crate::types::HomeworkEntry;

/// Application state shared across requests
pub struct AppState {
    pub conn: Mutex<Connection>,
    pub output_dir: PathBuf,
}

impl AppState {
    /// Create a new AppState with a database connection and output directory
    pub fn new(conn: Connection, output_dir: PathBuf) -> Self {
        Self {
            conn: Mutex::new(conn),
            output_dir,
        }
    }
}

// ========== Request/Response Types ==========

#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub entry_type: String,
    pub date: String,
    pub subject: String,
    pub task: String,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntryRequest {
    pub date: Option<String>,
    pub completed: Option<bool>,
    pub position: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub had_children: bool,
    pub children_orphaned: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CascadeDeleteResponse {
    pub success: bool,
    pub deleted_count: usize,
}

/// Create the router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route(
            "/api/entries",
            get(entries_handler).post(create_entry_handler),
        )
        .route(
            "/api/entries/{id}",
            get(get_entry_handler)
                .put(update_entry_handler)
                .delete(delete_entry_handler),
        )
        .route("/api/entries/{id}/children", get(get_children_handler))
        .route("/api/entries/{id}/cascade", delete(cascade_delete_handler))
        .route("/api/refresh", get(refresh_handler))
        .with_state(state)
}

/// Initialize server state by setting up the database
pub fn init_server_state(output_dir: PathBuf) -> anyhow::Result<Arc<AppState>> {
    // Determine paths
    let db_path = output_dir.join("data").join("homework.db");
    let migrations_dir = get_migrations_dir();

    info!(path = %db_path.display(), "Initializing database");

    // Initialize database
    let conn = db::init_db(&db_path, &migrations_dir)?;

    // Process any export files and import new entries
    debug!("Scanning for export files");
    match data::process_all_exports(&output_dir) {
        Ok(entries) => {
            let imported = db::import_entries(&conn, &entries)?;
            if imported > 0 {
                info!(count = imported, "Imported entries from exports");
            }

            // Generate study sessions for any tests
            let today = chrono::Local::now().date_naive();
            let mut study_sessions_created = 0;
            for entry in &entries {
                if is_test_or_quiz(entry) {
                    let sessions = generate_study_sessions(entry, today);
                    for session in sessions {
                        if db::insert_entry_if_not_exists(&conn, &session)? {
                            study_sessions_created += 1;
                        }
                    }
                }
            }
            if study_sessions_created > 0 {
                info!(count = study_sessions_created, "Created study sessions");
            }
        }
        Err(e) => {
            // Not fatal - we might just have no export files yet
            debug!(error = %e, "No export files processed");
        }
    }

    let total = db::count_entries(&conn)?;
    info!(count = total, "Database initialized");

    Ok(Arc::new(AppState::new(conn, output_dir)))
}

/// Get the migrations directory path
fn get_migrations_dir() -> PathBuf {
    // In development, use the relative path from the crate
    // This could be made configurable for production deployments
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir).join("db").join("migrations")
}

/// Create a socket address for the server
pub fn create_server_addr(port: u16) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// Start the web server with file watching
pub async fn serve(port: u16, output_dir: PathBuf) -> anyhow::Result<()> {
    let state = init_server_state(output_dir)?;

    // Start file watcher
    let watcher_state = state.clone();
    start_file_watcher(watcher_state)?;

    let app = create_router(state);

    let addr = create_server_addr(port);
    info!(url = %format!("http://{}", addr), "Server running");
    info!("Watching data/ for changes");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Check if a path is an export file that should trigger a refresh
pub fn is_export_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("export_") && n.contains(".xls"))
        .unwrap_or(false)
}

/// Ensure the data directory exists, creating it if necessary
pub fn ensure_data_dir(data_dir: &Path) -> anyhow::Result<bool> {
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir)?;
        Ok(true) // Created
    } else {
        Ok(false) // Already existed
    }
}

/// Describes the result of processing a file change event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshResult {
    /// Entries were updated with a count change
    Updated { old_count: usize, new_count: usize },
    /// No new entries were found
    NoChange { count: usize },
    /// Refresh failed with an error message
    Error(String),
}

impl RefreshResult {
    /// Log the result using tracing
    pub fn log(&self) {
        match self {
            RefreshResult::Updated {
                old_count,
                new_count,
            } => {
                let delta = *new_count as i64 - *old_count as i64;
                info!(count = new_count, delta = delta, "Entries updated");
            }
            RefreshResult::NoChange { count } => {
                debug!(count = count, "No new entries found");
            }
            RefreshResult::Error(e) => {
                error!(error = %e, "Failed to refresh");
            }
        }
    }
}

/// Process a refresh, updating the database and returning the result
pub fn process_refresh(state: &AppState) -> RefreshResult {
    let conn = match state.conn.lock() {
        Ok(c) => c,
        Err(e) => return RefreshResult::Error(format!("Lock error: {}", e)),
    };

    let old_count = db::count_entries(&conn).unwrap_or(0);

    match data::process_all_exports(&state.output_dir) {
        Ok(entries) => {
            let imported = db::import_entries(&conn, &entries).unwrap_or(0);

            // Generate study sessions for any new tests
            let today = chrono::Local::now().date_naive();
            for entry in &entries {
                if is_test_or_quiz(entry) {
                    let sessions = generate_study_sessions(entry, today);
                    for session in sessions {
                        let _ = db::insert_entry_if_not_exists(&conn, &session);
                    }
                }
            }

            let new_count = db::count_entries(&conn).unwrap_or(0);

            if new_count != old_count || imported > 0 {
                RefreshResult::Updated {
                    old_count,
                    new_count,
                }
            } else {
                RefreshResult::NoChange { count: new_count }
            }
        }
        Err(e) => {
            // If no exports but we have data, that's fine
            let count = db::count_entries(&conn).unwrap_or(0);
            if count > 0 {
                RefreshResult::NoChange { count }
            } else {
                RefreshResult::Error(e.to_string())
            }
        }
    }
}

/// Start watching the data directory for changes
fn start_file_watcher(state: Arc<AppState>) -> anyhow::Result<()> {
    let data_dir = PathBuf::from("data");

    if ensure_data_dir(&data_dir)? {
        info!("Created data/ directory");
    }

    // Create a channel to receive events
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // Spawn a blocking task for the file watcher
    let watch_dir = data_dir.clone();
    std::thread::spawn(move || {
        let tx_clone = tx.clone();
        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            move |result: DebounceEventResult| {
                if let Ok(events) = result {
                    let has_export = events.iter().any(|e| is_export_file(&e.path));

                    if has_export {
                        let _ = tx_clone.blocking_send(());
                    }
                }
            },
        )
        .expect("Failed to create debouncer");

        debouncer
            .watcher()
            .watch(&watch_dir, RecursiveMode::NonRecursive)
            .expect("Failed to watch directory");

        // Keep the watcher alive
        loop {
            std::thread::sleep(Duration::from_secs(60));
        }
    });

    // Spawn a task to handle file change notifications
    tokio::spawn(async move {
        while rx.recv().await.is_some() {
            info!("Detected changes in data/");
            let result = process_refresh(&state);
            result.log();
        }
    });

    Ok(())
}

/// Serve the main HTML page
async fn index_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();
    match db::get_all_entries(&conn) {
        Ok(entries) => {
            let markup = html::render_page(&entries);
            Html(markup.into_string()).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to get entries");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// Return all entries as JSON
async fn entries_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();
    match db::get_all_entries(&conn) {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => {
            error!(error = %e, "Failed to get entries");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// Get a single entry by ID
async fn get_entry_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();
    match db::get_entry(&conn, &id) {
        Ok(Some(entry)) => Json(entry).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Entry not found").into_response(),
        Err(e) => {
            error!(error = %e, id = %id, "Failed to get entry");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// Create a new entry
async fn create_entry_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEntryRequest>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();

    // Create the entry
    let mut entry = HomeworkEntry::new(req.entry_type, req.date.clone(), req.subject, req.task);

    // Set position if provided, otherwise put at end of day
    entry.position = match req.position {
        Some(pos) => pos,
        None => db::get_max_position_for_date(&conn, &req.date).unwrap_or(-1) + 1,
    };

    match db::insert_entry(&conn, &entry) {
        Ok(()) => {
            // If it's a test, generate study sessions
            if is_test_or_quiz(&entry) {
                let today = chrono::Local::now().date_naive();
                let sessions = generate_study_sessions(&entry, today);
                for session in sessions {
                    let _ = db::insert_entry_if_not_exists(&conn, &session);
                }
            }
            debug!(id = %entry.id, subject = %entry.subject, "Entry created");
            (StatusCode::CREATED, Json(entry)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create entry");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create entry").into_response()
        }
    }
}

/// Update an existing entry
async fn update_entry_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
    Json(req): Json<UpdateEntryRequest>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();

    let updates = EntryUpdate {
        date: req.date,
        completed: req.completed,
        position: req.position,
        task: None,
    };

    match db::update_entry(&conn, &id, &updates) {
        Ok(true) => {
            debug!(id = %id, "Entry updated");
            // Return the updated entry
            match db::get_entry(&conn, &id) {
                Ok(Some(entry)) => Json(entry).into_response(),
                _ => StatusCode::OK.into_response(),
            }
        }
        Ok(false) => (StatusCode::NOT_FOUND, "Entry not found").into_response(),
        Err(e) => {
            error!(error = %e, id = %id, "Failed to update entry");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update entry").into_response()
        }
    }
}

/// Delete an entry (orphans its children)
async fn delete_entry_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();

    // Check for children first
    let children = db::get_children(&conn, &id).unwrap_or_default();
    let had_children = !children.is_empty();
    let children_count = children.len();

    match db::delete_entry(&conn, &id) {
        Ok(true) => {
            debug!(id = %id, had_children = had_children, "Entry deleted");
            Json(DeleteResponse {
                success: true,
                had_children,
                children_orphaned: children_count,
            })
            .into_response()
        }
        Ok(false) => (StatusCode::NOT_FOUND, "Entry not found").into_response(),
        Err(e) => {
            error!(error = %e, id = %id, "Failed to delete entry");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete entry").into_response()
        }
    }
}

/// Get children (study sessions) for an entry
async fn get_children_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();
    match db::get_children(&conn, &id) {
        Ok(children) => Json(children).into_response(),
        Err(e) => {
            error!(error = %e, id = %id, "Failed to get children");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// Delete an entry and all its children (cascade delete)
async fn cascade_delete_handler(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();
    match db::delete_with_children(&conn, &id) {
        Ok(count) => {
            debug!(id = %id, deleted_count = count, "Cascade delete completed");
            Json(CascadeDeleteResponse {
                success: count > 0,
                deleted_count: count,
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, id = %id, "Failed to cascade delete");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete").into_response()
        }
    }
}

/// Refresh data from disk (re-process export files)
async fn refresh_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Manual refresh triggered");

    let conn = state.conn.lock().unwrap();

    match data::process_all_exports(&state.output_dir) {
        Ok(entries) => {
            let imported = db::import_entries(&conn, &entries).unwrap_or(0);

            // Generate study sessions for any new tests
            let today = chrono::Local::now().date_naive();
            let mut study_sessions_created = 0;
            for entry in &entries {
                if is_test_or_quiz(entry) {
                    let sessions = generate_study_sessions(entry, today);
                    for session in sessions {
                        if db::insert_entry_if_not_exists(&conn, &session).unwrap_or(false) {
                            study_sessions_created += 1;
                        }
                    }
                }
            }

            if imported > 0 || study_sessions_created > 0 {
                info!(
                    imported = imported,
                    study_sessions = study_sessions_created,
                    "Refresh complete"
                );
            }
            "OK"
        }
        Err(e) => {
            error!(error = %e, "Refresh failed");
            "ERROR"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use std::sync::Mutex as StdMutex;
    use tempfile::TempDir;
    use tower::ServiceExt;

    // Mutex to prevent concurrent directory changes in tests
    static DIR_LOCK: StdMutex<()> = StdMutex::new(());

    /// Helper to create test entries
    fn make_entry(entry_type: &str, date: &str, subject: &str, task: &str) -> HomeworkEntry {
        HomeworkEntry::new(
            entry_type.to_string(),
            date.to_string(),
            subject.to_string(),
            task.to_string(),
        )
    }

    /// Setup a test database with optional entries
    fn setup_test_db(entries: &[HomeworkEntry]) -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let migrations_dir = temp_dir.path().join("migrations");
        std::fs::create_dir(&migrations_dir).unwrap();

        std::fs::write(
            migrations_dir.join("001_initial_schema.sql"),
            include_str!("../db/migrations/001_initial_schema.sql"),
        )
        .unwrap();

        let conn = db::init_db(&db_path, &migrations_dir).unwrap();

        for entry in entries {
            db::insert_entry(&conn, entry).unwrap();
        }

        (temp_dir, conn)
    }

    /// Helper to create a test app state with a database
    fn test_state(entries: Vec<HomeworkEntry>) -> (TempDir, Arc<AppState>) {
        let (temp_dir, conn) = setup_test_db(&entries);
        let state = Arc::new(AppState::new(conn, temp_dir.path().to_path_buf()));
        (temp_dir, state)
    }

    /// Helper to get response body as string
    async fn body_to_string(body: Body) -> String {
        let bytes = body.collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    /// Helper to run async test with changed directory
    async fn with_temp_dir_async<F, Fut, T>(temp_dir: &TempDir, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let _lock = DIR_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let result = f().await;
        std::env::set_current_dir(original_dir).unwrap();
        result
    }

    // ========== AppState tests ==========

    #[test]
    fn test_app_state_new() {
        let (_temp_dir, conn) = setup_test_db(&[]);
        let state = AppState::new(conn, PathBuf::from("/test/path"));
        assert_eq!(state.output_dir, PathBuf::from("/test/path"));
    }

    #[test]
    fn test_app_state_db_access() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
        ];
        let (_temp_dir, state) = test_state(entries);

        let conn = state.conn.lock().unwrap();
        let read_entries = db::get_all_entries(&conn).unwrap();
        assert_eq!(read_entries.len(), 2);
    }

    // ========== Router tests ==========

    #[test]
    fn test_create_router() {
        let (_temp_dir, state) = test_state(vec![]);
        let router = create_router(state);
        // Router created successfully - routes are tested via handler tests
        assert!(true, "Router created: {:?}", router);
    }

    // ========== index_handler tests ==========

    #[tokio::test]
    async fn test_index_handler_empty_entries() {
        let (_temp_dir, state) = test_state(vec![]);
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("<!DOCTYPE html>"));
        assert!(body.contains("Compitutto"));
        assert!(body.contains("No homework entries found"));
    }

    #[tokio::test]
    async fn test_index_handler_with_entries() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Pag. 100"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Verifica"),
        ];
        let (_temp_dir, state) = test_state(entries);
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert!(body.contains("MATEMATICA"));
        assert!(body.contains("ITALIANO"));
        assert!(body.contains("Pag. 100"));
        assert!(body.contains("Verifica"));
        assert!(body.contains("2025-01-15"));
        assert!(body.contains("2025-01-16"));
    }

    #[tokio::test]
    async fn test_index_handler_content_type() {
        let (_temp_dir, state) = test_state(vec![]);
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let content_type = response.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("text/html"));
    }

    // ========== entries_handler tests ==========

    #[tokio::test]
    async fn test_entries_handler_empty() {
        let (_temp_dir, state) = test_state(vec![]);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/entries")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert_eq!(body, "[]");
    }

    #[tokio::test]
    async fn test_entries_handler_with_data() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
        ];
        let (_temp_dir, state) = test_state(entries);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/entries")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        let parsed: Vec<HomeworkEntry> = serde_json::from_str(&body).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].subject, "MATEMATICA");
        assert_eq!(parsed[1].subject, "ITALIANO");
    }

    #[tokio::test]
    async fn test_entries_handler_json_content_type() {
        let (_temp_dir, state) = test_state(vec![]);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/entries")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let content_type = response.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("application/json"));
    }

    #[tokio::test]
    async fn test_entries_handler_serialization() {
        let entries = vec![make_entry(
            "compiti",
            "2025-01-15",
            "MATEMATICA",
            "Special chars: àèìòù & \"quotes\"",
        )];
        let (_temp_dir, state) = test_state(entries);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/entries")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = body_to_string(response.into_body()).await;
        let parsed: Vec<HomeworkEntry> = serde_json::from_str(&body).unwrap();

        assert_eq!(parsed[0].task, "Special chars: àèìòù & \"quotes\"");
    }

    // ========== refresh_handler tests ==========

    #[tokio::test]
    async fn test_refresh_handler_no_data() {
        // Create a temp directory with no export files and no existing data
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        // Setup migrations for the test database
        let migrations_dir = temp_dir.path().join("migrations");
        std::fs::create_dir(&migrations_dir).unwrap();
        std::fs::write(
            migrations_dir.join("001_initial_schema.sql"),
            include_str!("../db/migrations/001_initial_schema.sql"),
        )
        .unwrap();

        let db_path = data_dir.join("homework.db");
        let conn = db::init_db(&db_path, &migrations_dir).unwrap();
        let state = Arc::new(AppState::new(conn, temp_dir.path().to_path_buf()));
        let app = create_router(state);

        let response = with_temp_dir_async(&temp_dir, || async {
            app.oneshot(
                Request::builder()
                    .uri("/api/refresh")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert_eq!(body, "ERROR"); // No data available
    }

    #[tokio::test]
    async fn test_refresh_handler_with_existing_json() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        // Setup migrations
        let migrations_dir = temp_dir.path().join("migrations");
        std::fs::create_dir(&migrations_dir).unwrap();
        std::fs::write(
            migrations_dir.join("001_initial_schema.sql"),
            include_str!("../db/migrations/001_initial_schema.sql"),
        )
        .unwrap();

        // Create existing homework.json
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let json = serde_json::to_string(&entries).unwrap();
        std::fs::write(temp_dir.path().join("homework.json"), json).unwrap();

        let db_path = data_dir.join("homework.db");
        let conn = db::init_db(&db_path, &migrations_dir).unwrap();
        let state = Arc::new(AppState::new(conn, temp_dir.path().to_path_buf()));
        let app = create_router(state.clone());

        let response = with_temp_dir_async(&temp_dir, || async {
            app.oneshot(
                Request::builder()
                    .uri("/api/refresh")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        assert_eq!(body, "OK");

        // Verify database was updated
        let conn = state.conn.lock().unwrap();
        let db_entries = db::get_all_entries(&conn).unwrap();
        assert_eq!(db_entries.len(), 1);
    }

    // ========== 404 tests ==========

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let (_temp_dir, state) = test_state(vec![]);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unknown/route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // ========== Concurrent access tests ==========

    #[test]
    fn test_concurrent_db_access() {
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let (_temp_dir, state) = test_state(entries);

        // Simulate multiple sequential reads (Mutex doesn't allow true concurrent access)
        for _ in 0..10 {
            let conn = state.conn.lock().unwrap();
            let count = db::count_entries(&conn).unwrap();
            assert_eq!(count, 1);
        }
    }

    // ========== is_export_file tests ==========

    #[test]
    fn test_is_export_file_valid() {
        assert!(is_export_file(Path::new("export_2025.xls")));
        assert!(is_export_file(Path::new("export_homework.xls")));
        assert!(is_export_file(Path::new("export_.xls")));
        assert!(is_export_file(Path::new("/path/to/export_data.xls")));
        assert!(is_export_file(Path::new("data/export_test.xls")));
    }

    #[test]
    fn test_is_export_file_xlsx() {
        // .xlsx files should also match (contains ".xls")
        assert!(is_export_file(Path::new("export_2025.xlsx")));
    }

    #[test]
    fn test_is_export_file_invalid_prefix() {
        assert!(!is_export_file(Path::new("homework.xls")));
        assert!(!is_export_file(Path::new("data.xls")));
        assert!(!is_export_file(Path::new("xport_file.xls")));
        assert!(!is_export_file(Path::new("Export_file.xls"))); // Case sensitive
    }

    #[test]
    fn test_is_export_file_invalid_extension() {
        assert!(!is_export_file(Path::new("export_data.txt")));
        assert!(!is_export_file(Path::new("export_data.csv")));
        assert!(!is_export_file(Path::new("export_data.json")));
        assert!(!is_export_file(Path::new("export_data")));
    }

    #[test]
    fn test_is_export_file_edge_cases() {
        assert!(!is_export_file(Path::new("")));
        assert!(!is_export_file(Path::new("/")));
        assert!(!is_export_file(Path::new("/path/to/")));
        assert!(!is_export_file(Path::new(".")));
        assert!(!is_export_file(Path::new("..")));
    }

    // ========== ensure_data_dir tests ==========

    #[test]
    fn test_ensure_data_dir_creates_new() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("new_data_dir");

        assert!(!data_dir.exists());
        let created = ensure_data_dir(&data_dir).unwrap();
        assert!(created);
        assert!(data_dir.exists());
    }

    #[test]
    fn test_ensure_data_dir_already_exists() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("existing_dir");
        std::fs::create_dir(&data_dir).unwrap();

        assert!(data_dir.exists());
        let created = ensure_data_dir(&data_dir).unwrap();
        assert!(!created);
        assert!(data_dir.exists());
    }

    #[test]
    fn test_ensure_data_dir_nested() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("a").join("b").join("c");

        assert!(!data_dir.exists());
        let created = ensure_data_dir(&data_dir).unwrap();
        assert!(created);
        assert!(data_dir.exists());
    }

    // ========== create_server_addr tests ==========

    #[test]
    fn test_create_server_addr() {
        let addr = create_server_addr(8080);
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[test]
    fn test_create_server_addr_different_ports() {
        assert_eq!(create_server_addr(3000).port(), 3000);
        assert_eq!(create_server_addr(0).port(), 0);
        assert_eq!(create_server_addr(65535).port(), 65535);
    }

    // ========== init_server_state tests ==========
    // Note: init_server_state requires CARGO_MANIFEST_DIR to find migrations,
    // so we test the components separately rather than the full function.

    // ========== RefreshResult tests ==========

    #[test]
    fn test_refresh_result_updated() {
        let result = RefreshResult::Updated {
            old_count: 5,
            new_count: 10,
        };
        assert_eq!(
            result,
            RefreshResult::Updated {
                old_count: 5,
                new_count: 10
            }
        );

        // Just ensure log doesn't panic
        result.log();
    }

    #[test]
    fn test_refresh_result_no_change() {
        let result = RefreshResult::NoChange { count: 5 };
        assert_eq!(result, RefreshResult::NoChange { count: 5 });
        result.log();
    }

    #[test]
    fn test_refresh_result_error() {
        let result = RefreshResult::Error("test error".to_string());
        assert_eq!(result, RefreshResult::Error("test error".to_string()));
        result.log();
    }

    #[test]
    fn test_refresh_result_debug() {
        let result = RefreshResult::Updated {
            old_count: 1,
            new_count: 2,
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("Updated"));
    }

    #[test]
    fn test_refresh_result_clone() {
        let result = RefreshResult::Updated {
            old_count: 1,
            new_count: 2,
        };
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    // ========== process_refresh tests ==========

    #[test]
    fn test_process_refresh_with_new_entries() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        // Setup migrations
        let migrations_dir = temp_dir.path().join("migrations");
        std::fs::create_dir(&migrations_dir).unwrap();
        std::fs::write(
            migrations_dir.join("001_initial_schema.sql"),
            include_str!("../db/migrations/001_initial_schema.sql"),
        )
        .unwrap();

        // Create database with no entries
        let db_path = data_dir.join("homework.db");
        let conn = db::init_db(&db_path, &migrations_dir).unwrap();
        let state = AppState::new(conn, temp_dir.path().to_path_buf());

        // Create homework.json with one entry
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let json = serde_json::to_string(&entries).unwrap();
        std::fs::write(temp_dir.path().join("homework.json"), json).unwrap();

        let _lock = DIR_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = process_refresh(&state);

        std::env::set_current_dir(original_dir).unwrap();

        match result {
            RefreshResult::Updated {
                old_count,
                new_count,
            } => {
                assert_eq!(old_count, 0);
                assert_eq!(new_count, 1);
            }
            _ => panic!("Expected Updated result, got {:?}", result),
        }
    }

    #[test]
    fn test_process_refresh_no_change() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        let migrations_dir = temp_dir.path().join("migrations");
        std::fs::create_dir(&migrations_dir).unwrap();
        std::fs::write(
            migrations_dir.join("001_initial_schema.sql"),
            include_str!("../db/migrations/001_initial_schema.sql"),
        )
        .unwrap();

        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        // Create homework.json
        let json = serde_json::to_string(&entries).unwrap();
        std::fs::write(temp_dir.path().join("homework.json"), json).unwrap();

        // Create database with same entries
        let db_path = data_dir.join("homework.db");
        let conn = db::init_db(&db_path, &migrations_dir).unwrap();
        for entry in &entries {
            db::insert_entry(&conn, entry).unwrap();
        }
        let state = AppState::new(conn, temp_dir.path().to_path_buf());

        let _lock = DIR_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = process_refresh(&state);

        std::env::set_current_dir(original_dir).unwrap();

        match result {
            RefreshResult::NoChange { count } => {
                assert_eq!(count, 1);
            }
            _ => panic!("Expected NoChange result, got {:?}", result),
        }
    }

    #[test]
    fn test_process_refresh_error() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();
        // No homework.json - will cause error

        let migrations_dir = temp_dir.path().join("migrations");
        std::fs::create_dir(&migrations_dir).unwrap();
        std::fs::write(
            migrations_dir.join("001_initial_schema.sql"),
            include_str!("../db/migrations/001_initial_schema.sql"),
        )
        .unwrap();

        let db_path = data_dir.join("homework.db");
        let conn = db::init_db(&db_path, &migrations_dir).unwrap();
        let state = AppState::new(conn, temp_dir.path().to_path_buf());

        let _lock = DIR_LOCK.lock().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = process_refresh(&state);

        std::env::set_current_dir(original_dir).unwrap();

        match result {
            RefreshResult::Error(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected Error result, got {:?}", result),
        }
    }

    // ========== New API endpoint tests ==========

    #[tokio::test]
    async fn test_get_entry_handler() {
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let entry_id = entries[0].id.clone();
        let (_temp_dir, state) = test_state(entries);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/entries/{}", entry_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        let parsed: HomeworkEntry = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed.id, entry_id);
    }

    #[tokio::test]
    async fn test_get_entry_not_found() {
        let (_temp_dir, state) = test_state(vec![]);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/entries/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_entry_handler() {
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let entry_id = entries[0].id.clone();
        let (_temp_dir, state) = test_state(entries);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/entries/{}", entry_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        let parsed: DeleteResponse = serde_json::from_str(&body).unwrap();
        assert!(parsed.success);
        assert!(!parsed.had_children);
    }

    #[tokio::test]
    async fn test_cascade_delete_handler() {
        let (_temp_dir, state) = test_state(vec![]);

        // Create a parent and child entry
        {
            let conn = state.conn.lock().unwrap();
            let parent = make_entry("compiti", "2025-01-20", "MATEMATICA", "Test");
            db::insert_entry(&conn, &parent).unwrap();

            let mut child = HomeworkEntry::with_id(
                "child1".to_string(),
                "studio".to_string(),
                "2025-01-19".to_string(),
                "MATEMATICA".to_string(),
                "Study".to_string(),
            );
            child.parent_id = Some(parent.id.clone());
            db::insert_entry(&conn, &child).unwrap();
        }

        let app = create_router(state.clone());

        // Get the parent ID
        let parent_id = {
            let conn = state.conn.lock().unwrap();
            let entries = db::get_all_entries(&conn).unwrap();
            entries
                .iter()
                .find(|e| e.entry_type == "compiti")
                .unwrap()
                .id
                .clone()
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/entries/{}/cascade", parent_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_to_string(response.into_body()).await;
        let parsed: CascadeDeleteResponse = serde_json::from_str(&body).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.deleted_count, 2);
    }
}
