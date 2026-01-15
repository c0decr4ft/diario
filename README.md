# Compitutto

A homework calendar viewer for ClasseViva exports. Drop Excel export files into the `data/` directory and view them in a styled web interface.

## Setup

1. Install Rust (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Build the project:
```bash
cargo build --release
```

## Usage

### Start the Server

```bash
just
```

Or without just:
```bash
cargo run --release
```

This will:
- Scan `data/` for export files
- Import entries into `data/homework.db` (SQLite)
- Start a web server at http://localhost:8080
- Watch for new files and auto-update

### Commands

| Command | Description |
|---------|-------------|
| `just` | Start web server (default) |
| `just serve` | Same as above |
| `just serve-port 3000` | Start on custom port |
| `just build-html` | Generate static HTML only |
| `just parse FILE` | Parse a specific file |
| `just status` | Show data status |

### CLI

```bash
compitutto              # Start server (default)
compitutto serve -p 80  # Custom port
compitutto build        # Static HTML only
compitutto parse FILE   # Parse specific file
```

## Workflow

1. Export homework from ClasseViva as Excel (.xls)
2. Drop the file into the `data/` directory
3. The server auto-detects new files and updates
4. View at http://localhost:8080

Files are deduplicated automatically, so you can drop overlapping exports without creating duplicates.

## Output

- `data/homework.db` - SQLite database with all entries
- `index.html` - Generated when using `build` command
- `data/` - Place export files here

## API Endpoints

- `GET /` - The homework calendar UI
- `GET /api/entries` - JSON data
- `GET /api/refresh` - Manual refresh trigger
