# Agent Guide for Compitutto

This document provides guidance for AI agents working on this codebase.

> **IMPORTANT: Always run `just ci` before committing.** This runs fmt-check, type-check, clippy, and tests. Do not commit if CI fails.

## Project Overview

Compitutto is a Rust application that parses homework calendar exports (Excel XML format) and generates a web view. It features:

- Excel XML (SpreadsheetML) parsing
- SQLite database for persistent storage (with migrations)
- Data deduplication and merging
- HTML generation with maud templating
- Web server with file watching (axum + notify)
- JSON API endpoints
- Automated Classe Viva export fetching (raschietto crate)

## Project Structure

```
crates/compitutto/
├── src/
│   ├── main.rs         # CLI entry point (clap), default port 9000
│   ├── types.rs        # HomeworkEntry struct
│   ├── parser.rs       # Excel XML parsing
│   ├── data.rs         # Data processing: study sessions, work reminders
│   ├── db.rs           # SQLite database operations + settings
│   ├── html/
│   │   ├── mod.rs      # render_page, render_date_group, generate_html
│   │   ├── assets.rs   # CSS and JAVASCRIPT constants
│   │   ├── calendar.rs # Calendar view: render_calendar, month_name, entries_to_json
│   │   └── settings.rs # render_settings_page
│   └── server.rs       # Web server (axum), all route handlers
├── db/
│   └── migrations/
│       ├── 001_initial_schema.sql  # entries table
│       └── 002_settings.sql        # settings table (work_days, etc.)
└── Cargo.toml

crates/raschietto/
├── src/
│   ├── main.rs     # CLI entry point
│   ├── browser.rs  # Playwright browser launch
│   ├── config.rs   # Credentials from env (CLASSEVIVA_USER / CLASSEVIVA_PASSWORD)
│   └── scraper.rs  # Login, email nag dismissal, export dialog, download via reqwest
└── Cargo.toml

data/               # Export files (export_*.xls) and homework.db
justfile            # Task runner commands
.env                # CLASSEVIVA_USER, CLASSEVIVA_PASSWORD, RUST_LOG
```

## Common Commands

```bash
# Development
just check          # Type check
just test           # Run tests
just lint           # Run clippy
just fmt            # Format code
just ci             # Run ALL checks (REQUIRED before committing)

# Running
just s              # Start web server (port 9000)
just serve 3000     # Start on custom port
just html           # Generate static HTML

# Fetching exports (raschietto)
just fetch          # Headless fetch from Classe Viva
just fetch-debug    # Headed browser (shows window — good for debugging)
just fetch-dry      # Login only, verify credentials
just go             # fetch + serve + open browser

# Setup
just setup-browser  # Install Playwright Chromium (run once)

# Code Coverage
just cov            # Full text coverage report
just cov-summary    # Summary table
just cov-report     # JSON format (agent-readable)
just cov-html       # HTML report in browser
```

## Routes and Pages

| Route | Method | Description |
|-------|--------|-------------|
| `/` | GET | Main homework list + calendar view |
| `/settings` | GET | Settings page (work days, reminder timing) |
| `/api/entries` | GET, POST | List all / create entry |
| `/api/entries/{id}` | GET, PUT, DELETE | Single entry CRUD |
| `/api/entries/{id}/children` | GET | Child study sessions |
| `/api/entries/{id}/cascade` | DELETE | Delete entry + all children |
| `/api/refresh` | GET | Re-scan exports and regenerate auto-entries |
| `/api/settings/work-days` | GET, PUT | `{"days": [1,2,3,4,5]}` |
| `/api/settings/homework-days-ahead` | GET, PUT | `{"value": 2}` |
| `/api/settings/study-days-before` | GET, PUT | `{"value": 4}` |

## Key Types

### HomeworkEntry

```rust
pub struct HomeworkEntry {
    pub id: String,                     // UUID-style or "study_…" / "lavoro_…" prefix
    pub source_id: Option<String>,      // Content hash for import deduplication
    pub entry_type: String,             // "compiti" | "nota" | "verifica" | "studio" | "lavoro" | ...
    pub date: String,                   // "YYYY-MM-DD"
    pub subject: String,
    pub task: String,
    pub completed: bool,
    pub position: i32,                  // Within-day ordering
    pub parent_id: Option<String>,      // Links study/lavoro entries to parent
    pub created_at: String,             // RFC 3339
    pub updated_at: String,             // RFC 3339
}
```

**Entry types:**
- `compiti` — homework due on `date`. Gets a 📋 Due badge + red left border.
- `nota` — general note
- `verifica` / `interrogazione` — test/quiz — triggers study session generation
- `studio` — auto-generated study reminder (child of verifica)
- `lavoro` — auto-generated "do it" reminder for compiti (child of compiti). Gets a ✏️ Do it badge + amber left border + link to parent due date.

**Deduplication:** based on `source_id` (hash of date+subject+task). Moving an entry in the UI changes its `date` in the DB but leaves `source_id` unchanged, so re-imports are safely skipped.

**Generated entries:** `is_generated()` returns true when `id` starts with `"study_"` or `"lavoro_"`. `is_orphaned()` returns true for generated entries whose `parent_id` is `None`.

### AppState

```rust
pub struct AppState {
    pub conn: Mutex<Connection>,  // Single SQLite connection, mutex-guarded
}
```

### Settings (DB keys in `settings` table)

| Key | Default | Description |
|-----|---------|-------------|
| `work_days` | `[1,2,3,4,5]` | Weekday numbers (1=Mon…5=Fri) allowed for work reminders. Weekends always allowed. |
| `homework_days_ahead` | `2` | Days before due date to place lavoro reminder (1 or 2) |
| `study_days_before` | `4` | Study sessions to generate before a verifica (min 3) |

## Auto-generated Entries

### Study sessions (type: `studio`)
Generated for any entry where `is_test_or_quiz()` is true (task contains "verifica", "prova", "test", "interrogazione"). Creates up to `study_days_before` entries on consecutive days before the test.

### Work reminders (type: `lavoro`)
Generated for `compiti` entries that are ≥ `homework_days_ahead` days in the future. Placed on the last allowed work day at least `homework_days_ahead` days before due. Weekends always count as allowed.

**Important:** Both are generated from **DB entries** (not freshly-parsed exports) to ensure `parent_id` FK references are valid. IDs are deterministic hashes so re-runs are idempotent.

## Database

### Schema

```sql
-- entries: all homework + auto-generated items
CREATE TABLE entries (
    id TEXT PRIMARY KEY,
    source_id TEXT,
    entry_type TEXT NOT NULL,
    date TEXT NOT NULL,
    subject TEXT NOT NULL DEFAULT '',
    task TEXT NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    position INTEGER NOT NULL DEFAULT 0,
    parent_id TEXT,                          -- FK → entries(id) ON DELETE SET NULL
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- settings: key/value user preferences
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### DB helper functions (db.rs)

```rust
init_db(path, migrations_dir) -> Result<Connection>
import_entries(conn, entries) -> Result<usize>   // skips source_id duplicates
insert_entry(conn, entry) -> Result<()>
insert_entry_if_not_exists(conn, entry) -> Result<bool>
get_all_entries(conn) -> Result<Vec<HomeworkEntry>>
get_entry(conn, id) -> Result<Option<HomeworkEntry>>
update_entry(conn, id, updates) -> Result<bool>
delete_entry(conn, id) -> Result<bool>
delete_with_children(conn, id) -> Result<usize>
get_children(conn, parent_id) -> Result<Vec<HomeworkEntry>>
count_entries(conn) -> Result<usize>

// Settings
get_work_days(conn) -> Result<Vec<u32>>
set_work_days(conn, days) -> Result<()>
get_homework_days_ahead(conn) -> Result<u32>     // clamped 1..=2
set_homework_days_ahead(conn, days) -> Result<()>
get_study_days_before(conn) -> Result<u32>       // min 3
set_study_days_before(conn, days) -> Result<()>
```

## HTML Module Structure

The `html` module is split into logical submodules to avoid a single giant file:

- **`html/mod.rs`** — `render_page()`, `render_date_group()`, `generate_html()`, all tests
- **`html/assets.rs`** — `CSS` and `JAVASCRIPT` string constants (large, don't edit unless styling)
- **`html/calendar.rs`** — `render_calendar()`, `month_name()`, `entries_to_json()`
- **`html/settings.rs`** — `render_settings_page()`, `SETTINGS_CSS`, `SETTINGS_JS`

## Raschietto (Automated Fetcher)

Fetches exports from Classe Viva (https://web.spaggiari.eu) using Playwright:

1. Navigate to agenda page (redirects to login)
2. Fill credentials from env vars
3. Submit login form
4. **Dismiss email nag screen** if it appears ("Continua senza associare l'email")
5. Click export button → fill date range → click Conferma
6. Capture `Download` event URL + browser cookies → download via reqwest
7. Save to `data/export_<timestamp>.xls`

Credentials: set `CLASSEVIVA_USER` and `CLASSEVIVA_PASSWORD` in `.env`.

Browser: uses Playwright Chromium from `~/Library/Caches/ms-playwright`. Run `just setup-browser` once.

The download uses reqwest (not Playwright's download API) because in headed mode the browser's native download manager intercepts the file. The `Download` event still fires and gives us the URL and we use browser cookies to authenticate the direct HTTP request.

## Testing Patterns

### Unit Tests

Each module has inline tests in a `#[cfg(test)] mod tests` block:

```bash
cargo test                      # All tests
cargo test html::tests          # html module only
cargo test data::tests          # data module only
cargo test test_name            # Specific test by name
```

### Test Helpers

```rust
// Create a HomeworkEntry
fn make_entry(entry_type: &str, date: &str, subject: &str, task: &str) -> HomeworkEntry

// Create test Excel XML
fn create_test_excel_xml(path: &Path, entries: &[(&str, &str, &str, &str)])

// Thread-safe temp dir helper
fn with_temp_dir<F, T>(temp_dir: &TempDir, f: F) -> T
```

### Server Handler Testing

```rust
let conn = db::init_db(&db_path, &migrations_dir).unwrap();
let state = Arc::new(AppState::new(conn));
let app = create_router(state);

let response = app
    .oneshot(Request::builder().uri("/api/entries").body(Body::empty()).unwrap())
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::OK);
```

## Data Flow

1. Export files land in `data/export_*.xls` (manually or via `just fetch`)
2. Server startup: `parse_all_exports()` → `import_entries()` (deduped by source_id)
3. Then loads all DB entries and generates study sessions + work reminders
4. File watcher detects new exports → triggers refresh
5. `/api/refresh` endpoint also triggers re-scan manually

## Common Tasks for Agents

### Adding a New Entry Type

1. Add entry type string handling in `types.rs` (`is_generated()` if applicable)
2. Add CSS badge color in `html/assets.rs` (`.homework-type[data-type="…"]`)
3. Add calendar color in `html/assets.rs` (`.cal-entry[data-type="…"]`)
4. Add to the `<select>` in the add-entry dialog in `html/mod.rs`
5. Run `just ci`

### Adding a New Setting

1. Add `INSERT OR IGNORE` default in `db/migrations/002_settings.sql` (or new migration)
2. Add `get_X` / `set_X` functions in `db.rs`
3. Add GET/PUT routes in `server.rs` `create_router()`
4. Add handler functions in `server.rs`
5. Add UI controls in `html/settings.rs`
6. Thread the setting through callers in `server.rs` and `data.rs`
7. Run `just ci`

### Adding a New API Endpoint

1. Add handler function in `server.rs`
2. Add route in `create_router()`
3. Add tests using tower oneshot pattern
4. Run `just ci`

### Adding a New DB Migration

1. Create `db/migrations/00N_description.sql`
2. Migrations run in filename order on startup
3. Already-applied migrations are skipped (tracked in `schema_migrations` table)

## Dependencies

Key dependencies:
- `axum` — Web framework
- `maud` — HTML templating (compile-time)
- `quick-xml` — XML parsing
- `rusqlite` — SQLite
- `serde` / `serde_json` — Serialization
- `notify-debouncer-mini` — File watching
- `clap` — CLI parsing
- `chrono` — Date handling (day names, date arithmetic)
- `playwright` — Browser automation (raschietto)
- `reqwest` — HTTP client for authenticated downloads (raschietto)

Dev dependencies:
- `tempfile` — Temporary directories for tests
- `tower` — Handler testing utilities
- `http-body-util` — Response body reading in tests
