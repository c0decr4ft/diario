# Compitutto - Homework Calendar

# List commands
default:
    @just --list

# Start web server
s:
    cargo run -p compitutto --release

# Start server on custom port
serve port="8080":
    cargo run -p compitutto --release -- serve --port {{port}}

# Build release binary
build:
    cargo build -p compitutto --release

# Generate static HTML (no server)
html:
    cargo run -p compitutto --release -- build

# Show data status
status:
    @echo "Export files in data/:"
    @ls -lh data/export_*.xls* 2>/dev/null || echo "  (none)"
    @echo ""
    @if [ -f data/homework.db ]; then \
        echo "Database: $$(sqlite3 data/homework.db 'SELECT COUNT(*) FROM entries' 2>/dev/null || echo 0) entries"; \
    else \
        echo "Database: not created yet"; \
    fi

# Clean build artifacts
clean:
    cargo clean
    rm -f coverage.json lcov.info

# Run all CI checks
ci: fmt-check check lint test
    @echo "CI passed!"

# Dev commands
check:
    cargo check

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

test:
    cargo test

# ========== Code Coverage ==========

# Run tests with coverage and show text summary (human readable)
cov:
    cargo llvm-cov --text

# Run tests with coverage and show summary only
cov-summary:
    cargo llvm-cov report --summary-only

# Generate JSON coverage report (full LLVM format)
cov-json:
    cargo llvm-cov --json --output-path coverage.json
    @echo "Coverage report saved to coverage.json"

# Generate coverage report in simple JSON format (agent readable)
# Outputs a compact JSON with per-file and total coverage stats
cov-report:
    #!/usr/bin/env bash
    set -euo pipefail
    # Run tests and generate JSON coverage (suppress test output)
    cargo llvm-cov --json 2>/dev/null | jq '{
        generated: now | strftime("%Y-%m-%dT%H:%M:%SZ"),
        summary: {
            lines: .data[0].totals.lines,
            functions: .data[0].totals.functions,
            regions: .data[0].totals.regions
        },
        files: [.data[0].files[] | {
            file: .filename | split("/") | last,
            lines: .summary.lines,
            functions: .summary.functions,
            regions: .summary.regions
        }]
    }'

# Generate LCOV coverage report (for IDE integration)
cov-lcov:
    cargo llvm-cov --lcov --output-path lcov.info
    @echo "LCOV report saved to lcov.info"

# Generate HTML coverage report and open in browser
cov-html:
    cargo llvm-cov --html
    @echo "Opening coverage report..."
    open target/llvm-cov/html/index.html 2>/dev/null || xdg-open target/llvm-cov/html/index.html 2>/dev/null || echo "HTML report at target/llvm-cov/html/index.html"

# Run all coverage formats
cov-all: cov-json cov-lcov
    @just cov-summary

# Clean coverage artifacts
cov-clean:
    cargo llvm-cov clean --workspace

# ========== Raschietto (Classe Viva Fetcher) ==========

# Fetch new exports from Classe Viva
fetch:
    cargo run -p raschietto -- fetch

# Fetch with visible browser (for debugging)
fetch-debug:
    cargo run -p raschietto -- fetch --headed

# Fetch (dry run - verify login only)
fetch-dry:
    cargo run -p raschietto -- fetch --dry-run

# Setup Playwright browser (run once)
setup-browser:
    npx playwright install chromium

# Fetch new exports, start server, and open browser
go:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Fetching new exports..."
    cargo run -p raschietto --release -- fetch
    echo "Starting server and opening browser..."
    open http://localhost:8080 2>/dev/null || xdg-open http://localhost:8080 2>/dev/null &
    cargo run -p compitutto --release
