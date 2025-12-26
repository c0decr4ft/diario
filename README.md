# Compitutto

A modern, Gen Alpha-styled homework calendar viewer for ClasseViva exports.

## Setup

1. Install dependencies:
```bash
pip3 install -r requirements.txt
```

   **Note:** On macOS, use `pip3` instead of `pip`. If you get "command not found", make sure Python 3 is installed.

2. Install Playwright browsers (required for automation):
```bash
python3 -m playwright install chromium
```

This will automatically download the Chromium browser that Playwright uses. No separate ChromeDriver needed!

3. Create a `.env` file in the project root with your ClasseViva credentials:
```
CLASSEVIVA_USERNAME=your_username
CLASSEVIVA_PASSWORD=your_password
```

## Usage

### Quick Commands (using Justfile)

The easiest way to use this project is with `just` commands:

```bash
# Download, parse, and open in browser (recommended)
just update

# Just download and parse
just all

# Download export only
just download

# Parse existing export
just parse

# Open HTML view in browser
just open

# Check status of files
just status

# See all available commands
just --list
```

### Manual Commands

If you don't have `just` installed, you can use the Python scripts directly:

```bash
# Download and parse automatically
python3 parse_homework.py --download

# Parse existing export file
python3 parse_homework.py

# Parse specific file
python3 parse_homework.py --file data/export_20251226.xls

# Just download (without parsing)
python3 download_export.py

# Interactive mode (see what's happening)
python3 download_export.py --interactive
```

**Note:** 
- Install `just` with `brew install just` for easier command management
- First time setup: Run `playwright install chromium` after installing dependencies

## Output

- `index.html` - The web view of your homework calendar
- `homework.json` - JSON export of all entries
- `data/export_YYYYMMDD.xls` - Downloaded export files (e.g., `export_20251226.xls`)

**File Management:**
- Files are downloaded to the `data/` directory
- Each file is named with the date: `export_YYYYMMDD.xls`
- Old export files (older than 7 days) are automatically cleaned up
- If you download on the same day, the old file is replaced

Open `index.html` in your browser to view your homework with checkboxes to track completion!

