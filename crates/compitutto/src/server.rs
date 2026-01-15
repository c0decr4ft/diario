use axum::{response::Html, routing::get, Router};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::data;
use crate::html;
use crate::types::HomeworkEntry;

/// Application state shared across requests
pub struct AppState {
    pub entries: RwLock<Vec<HomeworkEntry>>,
    pub output_dir: PathBuf,
}

impl AppState {
    /// Create a new AppState with the given entries and output directory
    pub fn new(entries: Vec<HomeworkEntry>, output_dir: PathBuf) -> Self {
        Self {
            entries: RwLock::new(entries),
            output_dir,
        }
    }
}

/// Create the router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/api/entries", get(entries_handler))
        .route("/api/refresh", get(refresh_handler))
        .with_state(state)
}

/// Initialize server state by loading data from disk
pub fn init_server_state(output_dir: PathBuf) -> anyhow::Result<Arc<AppState>> {
    println!("Scanning data directory...");
    let entries = data::process_all_exports(&output_dir)?;

    Ok(Arc::new(AppState::new(entries, output_dir)))
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
    println!("\nServer running at http://{}", addr);
    println!("Watching data/ for changes...");
    println!("Press Ctrl+C to stop\n");

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
    /// Log the result to stdout/stderr
    pub fn log(&self) {
        match self {
            RefreshResult::Updated {
                old_count,
                new_count,
            } => {
                println!(
                    "Updated: {} entries ({:+})",
                    new_count,
                    *new_count as i64 - *old_count as i64
                );
            }
            RefreshResult::NoChange { .. } => {
                println!("No new entries found");
            }
            RefreshResult::Error(e) => {
                eprintln!("Failed to refresh: {}", e);
            }
        }
    }
}

/// Process a refresh, updating entries and returning the result
pub async fn process_refresh(state: &AppState) -> RefreshResult {
    match data::process_all_exports(&state.output_dir) {
        Ok(new_entries) => {
            let mut entries = state.entries.write().await;
            let old_count = entries.len();
            *entries = new_entries;
            let new_count = entries.len();
            if new_count != old_count {
                RefreshResult::Updated {
                    old_count,
                    new_count,
                }
            } else {
                RefreshResult::NoChange { count: new_count }
            }
        }
        Err(e) => RefreshResult::Error(e.to_string()),
    }
}

/// Start watching the data directory for changes
fn start_file_watcher(state: Arc<AppState>) -> anyhow::Result<()> {
    let data_dir = PathBuf::from("data");

    if ensure_data_dir(&data_dir)? {
        println!("Created data/ directory");
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
            println!("\nDetected changes in data/...");
            let result = process_refresh(&state).await;
            result.log();
        }
    });

    Ok(())
}

/// Serve the main HTML page
async fn index_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Html<String> {
    let entries = state.entries.read().await;
    let markup = html::render_page(&entries);
    Html(markup.into_string())
}

/// Return entries as JSON
async fn entries_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::Json<Vec<HomeworkEntry>> {
    let entries = state.entries.read().await;
    axum::Json(entries.clone())
}

/// Refresh data from disk (manual trigger)
async fn refresh_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> &'static str {
    println!("\nManual refresh triggered...");

    match data::process_all_exports(&state.output_dir) {
        Ok(new_entries) => {
            let mut entries = state.entries.write().await;
            *entries = new_entries;
            "OK"
        }
        Err(e) => {
            eprintln!("Refresh failed: {}", e);
            "ERROR"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use std::sync::Mutex;
    use tower::ServiceExt;

    // Mutex to prevent concurrent directory changes in tests
    static DIR_LOCK: Mutex<()> = Mutex::new(());

    /// Helper to create test entries
    fn make_entry(entry_type: &str, date: &str, subject: &str, task: &str) -> HomeworkEntry {
        HomeworkEntry::new(
            entry_type.to_string(),
            date.to_string(),
            subject.to_string(),
            task.to_string(),
        )
    }

    /// Helper to create a test app state
    fn test_state(entries: Vec<HomeworkEntry>) -> Arc<AppState> {
        Arc::new(AppState::new(entries, PathBuf::from(".")))
    }

    /// Helper to get response body as string
    async fn body_to_string(body: Body) -> String {
        let bytes = body.collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    /// Helper to run async test with changed directory
    async fn with_temp_dir_async<F, Fut, T>(temp_dir: &tempfile::TempDir, f: F) -> T
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
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let state = AppState::new(entries.clone(), PathBuf::from("/test/path"));

        assert_eq!(state.output_dir, PathBuf::from("/test/path"));
        // Can't easily test RwLock contents in sync test, covered by async tests
    }

    #[tokio::test]
    async fn test_app_state_entries_read() {
        let entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
        ];
        let state = AppState::new(entries.clone(), PathBuf::from("."));

        let read_entries = state.entries.read().await;
        assert_eq!(read_entries.len(), 2);
        assert_eq!(read_entries[0].subject, "MATEMATICA");
    }

    #[tokio::test]
    async fn test_app_state_entries_write() {
        let state = AppState::new(vec![], PathBuf::from("."));

        {
            let mut entries = state.entries.write().await;
            entries.push(make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"));
        }

        let read_entries = state.entries.read().await;
        assert_eq!(read_entries.len(), 1);
    }

    // ========== Router tests ==========

    #[test]
    fn test_create_router() {
        let state = test_state(vec![]);
        let router = create_router(state);
        // Router created successfully - routes are tested via handler tests
        assert!(true, "Router created: {:?}", router);
    }

    // ========== index_handler tests ==========

    #[tokio::test]
    async fn test_index_handler_empty_entries() {
        let state = test_state(vec![]);
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
        let state = test_state(entries);
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
        let state = test_state(vec![]);
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
        let state = test_state(vec![]);
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
        let state = test_state(entries);
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
        let state = test_state(vec![]);
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
        let state = test_state(entries);
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
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        let state = Arc::new(AppState::new(vec![], temp_dir.path().to_path_buf()));
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
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir(&data_dir).unwrap();

        // Create existing homework.json
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let json = serde_json::to_string(&entries).unwrap();
        std::fs::write(temp_dir.path().join("homework.json"), json).unwrap();

        let state = Arc::new(AppState::new(vec![], temp_dir.path().to_path_buf()));
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

        // Verify state was updated
        let read_entries = state.entries.read().await;
        assert_eq!(read_entries.len(), 1);
    }

    // ========== 404 tests ==========

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let state = test_state(vec![]);
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

    #[tokio::test]
    async fn test_concurrent_reads() {
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let state = test_state(entries);

        // Simulate multiple concurrent reads
        let mut handles = vec![];
        for _ in 0..10 {
            let state_clone = state.clone();
            handles.push(tokio::spawn(async move {
                let entries = state_clone.entries.read().await;
                entries.len()
            }));
        }

        for handle in handles {
            let count = handle.await.unwrap();
            assert_eq!(count, 1);
        }
    }

    #[tokio::test]
    async fn test_read_write_consistency() {
        let state = test_state(vec![]);

        // Write some entries
        {
            let mut entries = state.entries.write().await;
            entries.push(make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"));
            entries.push(make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"));
        }

        // Read and verify
        let entries = state.entries.read().await;
        assert_eq!(entries.len(), 2);
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

    #[tokio::test]
    async fn test_init_server_state_with_data() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("data")).unwrap();

        // Create homework.json
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let json = serde_json::to_string(&entries).unwrap();
        std::fs::write(temp_dir.path().join("homework.json"), json).unwrap();

        let state = with_temp_dir_async(&temp_dir, || async {
            init_server_state(temp_dir.path().to_path_buf()).unwrap()
        })
        .await;

        let read_entries = state.entries.read().await;
        assert_eq!(read_entries.len(), 1);
        assert_eq!(state.output_dir, temp_dir.path().to_path_buf());
    }

    #[tokio::test]
    async fn test_init_server_state_no_data() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("data")).unwrap();
        // No homework.json

        let result = with_temp_dir_async(&temp_dir, || async {
            init_server_state(temp_dir.path().to_path_buf())
        })
        .await;

        // Should fail because no data exists
        assert!(result.is_err());
    }

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

    #[tokio::test]
    async fn test_process_refresh_with_new_entries() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("data")).unwrap();

        // Create initial state with no entries
        let state = AppState::new(vec![], temp_dir.path().to_path_buf());

        // Create homework.json with one entry
        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let json = serde_json::to_string(&entries).unwrap();
        std::fs::write(temp_dir.path().join("homework.json"), json).unwrap();

        let result =
            with_temp_dir_async(&temp_dir, || async { process_refresh(&state).await }).await;

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

        // Verify state was updated
        let read_entries = state.entries.read().await;
        assert_eq!(read_entries.len(), 1);
    }

    #[tokio::test]
    async fn test_process_refresh_no_change() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("data")).unwrap();

        let entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];

        // Create homework.json
        let json = serde_json::to_string(&entries).unwrap();
        std::fs::write(temp_dir.path().join("homework.json"), json).unwrap();

        // Create state with same entries
        let state = AppState::new(entries.clone(), temp_dir.path().to_path_buf());

        let result =
            with_temp_dir_async(&temp_dir, || async { process_refresh(&state).await }).await;

        match result {
            RefreshResult::NoChange { count } => {
                assert_eq!(count, 1);
            }
            _ => panic!("Expected NoChange result, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_process_refresh_error() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("data")).unwrap();
        // No homework.json - will cause error

        let state = AppState::new(vec![], temp_dir.path().to_path_buf());

        let result =
            with_temp_dir_async(&temp_dir, || async { process_refresh(&state).await }).await;

        match result {
            RefreshResult::Error(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected Error result, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_process_refresh_decrease_entries() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("data")).unwrap();

        // Start with 2 entries
        let initial_entries = vec![
            make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1"),
            make_entry("nota", "2025-01-16", "ITALIANO", "Task 2"),
        ];
        let state = AppState::new(initial_entries, temp_dir.path().to_path_buf());

        // homework.json has only 1 entry
        let new_entries = vec![make_entry("compiti", "2025-01-15", "MATEMATICA", "Task 1")];
        let json = serde_json::to_string(&new_entries).unwrap();
        std::fs::write(temp_dir.path().join("homework.json"), json).unwrap();

        let result =
            with_temp_dir_async(&temp_dir, || async { process_refresh(&state).await }).await;

        match result {
            RefreshResult::Updated {
                old_count,
                new_count,
            } => {
                assert_eq!(old_count, 2);
                assert_eq!(new_count, 1);
            }
            _ => panic!("Expected Updated result, got {:?}", result),
        }
    }
}
