# Diario

A homework calendar system for ClasseViva. Includes two crates:

- **compitutto** - Web viewer for homework exports
- **raschietto** - Automated fetcher for ClasseViva exports

## Setup

1. Install Rust (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Build the project:
```bash
cargo build --release
```

3. (Optional) For automated fetching, install Playwright:
```bash
just setup-browser
```

## Compitutto (Viewer)

Drop Excel export files into the `data/` directory and view them in a styled web interface.

### Start the Server

```bash
just s
```

Or without just:
```bash
cargo run -p compitutto --release
```

This will:
- Scan `data/` for export files
- Import entries into `data/homework.db` (SQLite)
- Start a web server at http://localhost:8080
- Watch for new files and auto-update

### Commands

| Command | Description |
|---------|-------------|
| `just s` | Start web server |
| `just serve 3000` | Start on custom port |
| `just html` | Generate static HTML only |
| `just status` | Show data status |

### CLI

```bash
compitutto              # Start server (default)
compitutto serve -p 80  # Custom port
compitutto build        # Static HTML only
```

## Raschietto (Fetcher)

Automated fetcher that logs into ClasseViva and downloads homework exports.

### Setup

1. Create a `.env` file with your credentials:
```bash
CLASSEVIVA_USER=your_username
CLASSEVIVA_PASSWORD=your_password
```

2. Install the Playwright browser:
```bash
just setup-browser
```

### Commands

| Command | Description |
|---------|-------------|
| `just fetch` | Fetch new exports (headless) |
| `just fetch-debug` | Fetch with visible browser |
| `just fetch-dry` | Verify login only (no download) |

### CLI

```bash
raschietto fetch                    # Default date range (7 days ago to 15 days ahead)
raschietto fetch --from 2025-01-01  # Custom start date
raschietto fetch --to 2025-02-01    # Custom end date
raschietto fetch --headed           # Show browser window
raschietto fetch --dry-run          # Verify credentials only
raschietto fetch -o ./exports       # Custom output directory
```

## Workflow

### Quick Start
```bash
just go
```
This fetches new exports, starts the server, and opens the browser - all in one command.

### Manual
1. Export homework from ClasseViva as Excel (.xls)
2. Drop the file into the `data/` directory
3. The server auto-detects new files and updates
4. View at http://localhost:8080

### Automated
1. Run `just fetch` to download new exports
2. Files are saved to `data/` automatically
3. The server picks them up if running

Files are deduplicated automatically, so you can fetch overlapping date ranges without creating duplicates.

## Output

- `data/homework.db` - SQLite database with all entries
- `data/export_*.xls` - Downloaded export files
- `index.html` - Generated when using `build` command

## API Endpoints

- `GET /` - The homework calendar UI
- `GET /api/entries` - JSON data
- `GET /api/refresh` - Manual refresh trigger
