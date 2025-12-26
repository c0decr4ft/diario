# Compitutto - Homework Calendar Commands

# Default: show available commands
default:
    @just --list

# Download export from ClasseViva and parse it
all:
    @echo "📥 Downloading and parsing homework..."
    @python3 parse_homework.py --download

# Download export from ClasseViva
download:
    @echo "📥 Downloading export from ClasseViva..."
    @python3 download_export.py

# Download export in interactive mode (visible browser)
download-interactive:
    @echo "📥 Downloading export (interactive mode)..."
    @python3 download_export.py --interactive

# Parse existing export file
parse:
    @echo "📊 Parsing homework from existing export..."
    python3 parse_homework.py

# Parse a specific file
parse-file FILE:
    @echo "📊 Parsing {{FILE}}..."
    python3 parse_homework.py --file {{FILE}}

# Open the HTML view in browser
open:
    @echo "🌐 Opening homework calendar in browser..."
    @if [ -f index.html ]; then \
        open index.html; \
    else \
        echo "❌ index.html not found. Run 'just parse' or 'just all' first."; \
        exit 1; \
    fi

# Download, parse, and open in one command
update:
    @echo "🔄 Updating homework calendar..."
    @just all
    @just open

# Clean up old export files (keeps last 7 days)
clean:
    @echo "🧹 Cleaning up old export files..."
    @find data -name "export_*.xls*" -mtime +7 -not -name "$(ls -t data/export_*.xls* 2>/dev/null | head -1 | xargs basename)" -delete 2>/dev/null || true
    @echo "✅ Cleanup complete"

# Show status of files
status:
    @echo "📊 Current status:"
    @echo ""
    @if [ -f index.html ]; then \
        echo "✅ HTML file exists"; \
        echo "   Last modified: $(stat -f '%Sm' index.html 2>/dev/null || stat -c '%y' index.html 2>/dev/null)"; \
    else \
        echo "❌ HTML file not found"; \
    fi
    @echo ""
    @if [ -f homework.json ]; then \
        echo "✅ JSON file exists"; \
        echo "   Last modified: $(stat -f '%Sm' homework.json 2>/dev/null || stat -c '%y' homework.json 2>/dev/null)"; \
    else \
        echo "❌ JSON file not found"; \
    fi
    @echo ""
    @echo "📁 Export files in data/:"
    @ls -lh data/export_*.xls* 2>/dev/null | awk '{print "   " $$9 " (" $$5 ")"}' || echo "   No export files found"

# Install dependencies
install:
    @echo "📦 Installing dependencies..."
    pip3 install -r requirements.txt
    @echo ""
    @echo "📦 Installing Playwright browsers..."
    @echo "   (This may take a few minutes the first time)"
    python3 -m playwright install chromium

# Install Playwright browsers (required first time)
install-browsers:
    @echo "🌐 Installing Playwright browsers..."
    python3 -m playwright install chromium

# Check if Playwright browsers are installed
check-playwright:
    @echo "🔍 Checking for Playwright..."
    @if python3 -c "import playwright" 2>/dev/null; then \
        echo "✅ Playwright is installed"; \
        python3 -c "from playwright.sync_api import sync_playwright; print('Playwright version:', __import__('playwright').__version__)"; \
    else \
        echo "❌ Playwright not found"; \
        echo "   Install with: pip3 install playwright && python3 -m playwright install chromium"; \
    fi

# Fix .env file permissions and extended attributes
fix-env:
    @echo "🔧 Fixing .env file permissions..."
    @if [ -f .env ]; then \
        echo "Removing extended attributes (if any)..."; \
        xattr -d com.apple.quarantine .env 2>/dev/null || true; \
        xattr -c .env 2>/dev/null || true; \
        echo "Setting correct permissions (600)..."; \
        chmod 600 .env; \
        echo "✅ .env file permissions fixed"; \
        echo ""; \
        echo "Note: You can also set credentials via environment variables:"; \
        echo "  export CLASSEVIVA_USERNAME=... CLASSEVIVA_PASSWORD=..."; \
        echo "  Or pass inline: CLASSEVIVA_USERNAME=... CLASSEVIVA_PASSWORD=... just all"; \
    else \
        echo "❌ .env file not found"; \
        echo "   Create it with: CLASSEVIVA_USERNAME=your_username"; \
        echo "                   CLASSEVIVA_PASSWORD=your_password"; \
    fi

# Help command
help:
    @echo "Compitutto - Homework Calendar Commands"
    @echo ""
    @echo "Available commands:"
    @echo "  just all              - Download and parse homework"
    @echo "  just download         - Download export from ClasseViva"
    @echo "  just download-interactive - Download with visible browser"
    @echo "  just parse            - Parse existing export file"
    @echo "  just parse-file FILE  - Parse a specific file"
    @echo "  just open             - Open HTML view in browser"
    @echo "  just update           - Download, parse, and open"
    @echo "  just clean            - Clean up old export files"
    @echo "  just status           - Show current file status"
    @echo "  just install          - Install Python dependencies"
    @echo "  just install-browsers   - Install Playwright browsers (first time setup)"
    @echo "  just check-playwright    - Check if Playwright is installed"
    @echo "  just fix-env           - Fix .env file permissions and extended attributes"
    @echo "  just help             - Show this help message"
    @echo ""
    @echo "Quick start:"
    @echo "  1. Set credentials (choose one):"
    @echo "     - Create .env file: CLASSEVIVA_USERNAME=... CLASSEVIVA_PASSWORD=..."
    @echo "     - Export in shell: export CLASSEVIVA_USERNAME=... CLASSEVIVA_PASSWORD=..."
    @echo "     - Pass inline: CLASSEVIVA_USERNAME=... CLASSEVIVA_PASSWORD=... just all"
    @echo "  2. Optional: Fix .env permissions with: just fix-env"
    @echo "  3. Run: just update"

