# Agent Guide for Compitutto

This document provides guidance for AI agents working on this codebase.

## Project Overview

Compitutto is a Rust application that parses homework calendar exports (Excel XML format) and generates a web view. It features:

- Excel XML (SpreadsheetML) parsing
- SQLite database for persistent storage
- Data deduplication and merging
- HTML generation with maud templating
- Web server with file watching (axum + notify)
- JSON API endpoints

## Project Structure

```
crates/compitutto/
├── src/
│   ├── main.rs      # CLI entry point (clap)
│   ├── types.rs     # HomeworkEntry struct
│   ├── parser.rs    # Excel XML parsing
│   ├── data.rs      # Data processing, merging, study session generation
│   ├── db.rs        # SQLite database operations
│   ├── html.rs      # HTML generation (maud)
│   └── server.rs    # Web server (axum)
├── db/
│   └── migrations/  # SQL migration files
├── Cargo.toml
data/                # Export files (export_*.xls) and homework.db
justfile             # Task runner commands
```

## Common Commands

```bash
# Development
just check          # Type check
just test           # Run tests
just lint           # Run clippy
just fmt            # Format code
just ci             # Run all CI checks

# Running
just s              # Start web server (port 8080)
just serve 3000     # Start on custom port
just html           # Generate static HTML

# Code Coverage
just cov            # Full text coverage report
just cov-summary    # Summary table
just cov-report     # JSON format (agent-readable)
just cov-html       # HTML report in browser
```

## Code Coverage

Current coverage: **94.75% line coverage**

Run `just cov-report` for agent-readable JSON output:

```json
{
  "summary": {
    "lines": { "count": 2038, "covered": 1931, "percent": 94.75 },
    "functions": { "count": 233, "covered": 224, "percent": 96.14 }
  },
  "files": [
    { "file": "types.rs", "lines": { "percent": 100.0 } },
    { "file": "parser.rs", "lines": { "percent": 99.67 } },
    { "file": "html.rs", "lines": { "percent": 100.0 } },
    { "file": "data.rs", "lines": { "percent": 100.0 } },
    { "file": "server.rs", "lines": { "percent": 91.70 } }
  ]
}
```

Note: The remaining uncovered code in `server.rs` (~8%) is infrastructure code that runs indefinitely (file watcher loop, `axum::serve().await`). This is intentionally not tested as it requires integration-level testing.

## Testing Patterns

### Unit Tests

Each module has inline tests in a `#[cfg(test)] mod tests` block. Run with:

```bash
cargo test                    # All tests
cargo test data::tests        # Specific module
cargo test test_name          # Specific test
```

### Test Helpers

Common patterns used in tests:

```rust
// Create test entries
fn make_entry(entry_type: &str, date: &str, subject: &str, task: &str) -> HomeworkEntry

// Create test Excel XML files
fn create_test_excel_xml(path: &Path, entries: &[(&str, &str, &str, &str)])

// Run with temp directory (thread-safe)
fn with_temp_dir<F, T>(temp_dir: &TempDir, f: F) -> T
```

### Server Testing

Server handlers are tested using `tower::ServiceExt::oneshot`:

```rust
let state = Arc::new(AppState::new(entries, PathBuf::from(".")));
let app = create_router(state);

let response = app
    .oneshot(Request::builder().uri("/api/entries").body(Body::empty()).unwrap())
    .await
    .unwrap();
```

## Key Types

### HomeworkEntry

```rust
pub struct HomeworkEntry {
    pub entry_type: String,  // "compiti", "nota", etc.
    pub date: String,        // "YYYY-MM-DD"
    pub subject: String,     // "MATEMATICA", etc.
    pub task: String,        // Task description
}
```

Deduplication is based on `(date, subject, task)` - entry_type is ignored.

### AppState

```rust
pub struct AppState {
    pub entries: RwLock<Vec<HomeworkEntry>>,
    pub output_dir: PathBuf,
}
```

### Server Helper Functions

The server module exposes testable helper functions:

```rust
// Check if a path matches the export file pattern
pub fn is_export_file(path: &Path) -> bool

// Create data directory if missing, returns whether it was created
pub fn ensure_data_dir(data_dir: &Path) -> anyhow::Result<bool>

// Create a socket address for the server
pub fn create_server_addr(port: u16) -> SocketAddr

// Initialize server state by loading data from disk
pub fn init_server_state(output_dir: PathBuf) -> anyhow::Result<Arc<AppState>>

// Process a refresh, updating entries and returning the result
pub async fn process_refresh(state: &AppState) -> RefreshResult
```

### RefreshResult

```rust
pub enum RefreshResult {
    Updated { old_count: usize, new_count: usize },
    NoChange { count: usize },
    Error(String),
}
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | HTML page with all entries |
| `/api/entries` | GET | JSON array of entries |
| `/api/refresh` | GET | Reload data from disk |

## Excel XML Format

The parser expects SpreadsheetML format with these column headers (case-insensitive):

- `tipo` / `type` - Entry type
- `data_inizio` / `data` / `date` - Date
- `materia` / `subject` / `corso` - Subject
- `nota` / `descrizione` / `task` / `compito` - Task description

## Data Flow

1. Export files placed in `data/export_*.xls`
2. `process_all_exports()` finds and parses all exports
3. New entries imported into SQLite database via `db::import_entries()`
4. Duplicates detected via `source_id` field (entries with same source_id are skipped)
5. Results sorted by date and position when retrieved

## File Watching

The server watches `data/` for new export files:
- Debounced (2 second delay)
- Only triggers on `export_*.xls` files
- Auto-refreshes entries in memory

## Common Tasks for Agents

### Adding a New Field

1. Add field to `HomeworkEntry` in `types.rs`
2. Update `parse_row()` in `parser.rs`
3. Update `map_columns()` if new header names needed
4. Update HTML rendering in `html.rs`
5. Add tests for new functionality
6. Run `just ci` to verify

### Adding a New API Endpoint

1. Add handler function in `server.rs`
2. Add route in `create_router()`
3. Add tests using tower oneshot pattern
4. Run `just cov-report` to check coverage

### Debugging Parser Issues

1. Check column mapping with sample file
2. Use `cargo run -- parse <file>` to test single file
3. Parser prints found columns for debugging
4. Check `normalize_date()` for date format issues

## Dependencies

Key dependencies:
- `axum` - Web framework
- `maud` - HTML templating
- `quick-xml` - XML parsing
- `serde` / `serde_json` - Serialization
- `notify` - File watching
- `clap` - CLI parsing
- `chrono` - Date handling

Dev dependencies:
- `tempfile` - Temporary directories for tests
- `tower` - Handler testing
- `http-body-util` - Response body reading
