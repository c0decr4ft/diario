use axum::{response::Html, routing::get, Router};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::net::SocketAddr;
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

/// Start the web server with file watching
pub async fn serve(port: u16, output_dir: PathBuf) -> anyhow::Result<()> {
    // Process data on startup
    println!("Scanning data directory...");
    let entries = data::process_all_exports(&output_dir)?;

    let state = Arc::new(AppState {
        entries: RwLock::new(entries),
        output_dir: output_dir.clone(),
    });

    // Start file watcher
    let watcher_state = state.clone();
    start_file_watcher(watcher_state)?;

    let app = create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("\nServer running at http://{}", addr);
    println!("Watching data/ for changes...");
    println!("Press Ctrl+C to stop\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Start watching the data directory for changes
fn start_file_watcher(state: Arc<AppState>) -> anyhow::Result<()> {
    let data_dir = PathBuf::from("data");

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
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
                    // Check if any event is for an export file
                    let has_export = events.iter().any(|e| {
                        e.path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.starts_with("export_") && n.contains(".xls"))
                            .unwrap_or(false)
                    });

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
            match data::process_all_exports(&state.output_dir) {
                Ok(new_entries) => {
                    let mut entries = state.entries.write().await;
                    let old_count = entries.len();
                    *entries = new_entries;
                    let new_count = entries.len();
                    if new_count != old_count {
                        println!(
                            "Updated: {} entries ({:+})",
                            new_count,
                            new_count as i64 - old_count as i64
                        );
                    } else {
                        println!("No new entries found");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to refresh: {}", e);
                }
            }
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
}
