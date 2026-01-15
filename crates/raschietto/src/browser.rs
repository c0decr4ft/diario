//! Playwright browser setup and management.

use anyhow::{Context, Result};
use playwright::api::{Browser, BrowserContext, Playwright};
use std::path::PathBuf;
use std::sync::Arc;

/// Browser configuration options.
#[derive(Debug, Clone, Default)]
pub struct BrowserOptions {
    /// Whether to show the browser window (false = headless).
    pub headed: bool,
}

/// Wrapper around Playwright browser instance.
pub struct BrowserSession {
    #[allow(dead_code)]
    playwright: Arc<Playwright>,
    browser: Browser,
}

/// Find Chromium executable from npx-installed Playwright browsers.
///
/// Searches the standard Playwright browser cache locations for an installed
/// Chromium browser. Returns the path to the executable if found.
fn find_chromium_executable() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    // Check macOS Playwright cache location
    let cache_dir = PathBuf::from(&home).join("Library/Caches/ms-playwright");
    if let Some(path) = find_chromium_in_cache(&cache_dir) {
        return Some(path);
    }

    // Check Linux Playwright cache location
    let linux_cache_dir = PathBuf::from(&home).join(".cache/ms-playwright");
    if let Some(path) = find_chromium_in_cache(&linux_cache_dir) {
        return Some(path);
    }

    None
}

/// Search for Chromium in a Playwright cache directory.
fn find_chromium_in_cache(cache_dir: &PathBuf) -> Option<PathBuf> {
    if !cache_dir.exists() {
        return None;
    }

    let entries = std::fs::read_dir(cache_dir).ok()?;
    let mut chromium_dirs: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.starts_with("chromium-") && !name.contains("headless_shell")
        })
        .collect();

    // Sort descending to get latest version first
    chromium_dirs.sort_by_key(|d| std::cmp::Reverse(d.file_name()));

    let chromium_dir = chromium_dirs.first()?;

    // Try platform-specific paths
    let candidates = [
        // macOS ARM64
        "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
        // macOS Intel
        "chrome-mac/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
        // Linux
        "chrome-linux/chrome",
    ];

    for candidate in candidates {
        let path = chromium_dir.path().join(candidate);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

impl BrowserSession {
    /// Launch a new browser session with the given options.
    pub async fn launch(options: BrowserOptions) -> Result<Self> {
        let playwright = Playwright::initialize()
            .await
            .context("Failed to initialize Playwright")?;
        let playwright = Arc::new(playwright);

        // Find Chromium from npx-installed Playwright browsers
        let chromium_path = find_chromium_executable().context(
            "Chromium not found. Run 'npm install && npx playwright install chromium' first.",
        )?;

        let browser = playwright
            .chromium()
            .launcher()
            .headless(!options.headed)
            .executable(&chromium_path)
            .launch()
            .await
            .context("Failed to launch Chromium browser")?;

        Ok(Self {
            playwright,
            browser,
        })
    }

    /// Create a new browser context (isolated session).
    ///
    /// Downloads are accepted by default so we can capture exported files.
    pub async fn new_context(&self) -> Result<BrowserContext> {
        self.browser
            .context_builder()
            .accept_downloads(true)
            .build()
            .await
            .context("Failed to create browser context")
    }

    /// Close the browser.
    pub async fn close(self) -> Result<()> {
        self.browser
            .close()
            .await
            .context("Failed to close browser")
    }
}
