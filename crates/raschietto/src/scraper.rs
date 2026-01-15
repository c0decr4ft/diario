//! Classe Viva page interactions: login, navigation, modal handling, download.

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use playwright::api::frame::FrameState;
use playwright::api::page::{Event, EventType};
use playwright::api::{BrowserContext, Page};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info};

use crate::config::Credentials;

/// URLs for Classe Viva.
const AGENDA_URL: &str = "https://web.spaggiari.eu/fml/app/default/agenda_studenti.php";

/// CSS selectors for page elements.
mod selectors {
    pub const LOGIN_USERNAME: &str = "#login";
    pub const LOGIN_PASSWORD: &str = "#password";
    pub const LOGIN_SUBMIT: &str = "button[type='submit']";
    /// Export button - an <a> tag with class "export" and alt="scarica"
    pub const EXPORT_BUTTON: &str = "a.export[alt='scarica']";
    pub const EXPORT_DIALOG: &str = "div.ui-dialog[role='dialog']";
    pub const DATE_FROM: &str = "#dal";
    pub const DATE_TO: &str = "#al";
    pub const CONFIRM_BUTTON: &str = "div.ui-dialog button:has-text('Conferma')";
}

/// Date range for export.
#[derive(Debug, Clone)]
pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl DateRange {
    /// Create a new date range.
    pub fn new(from: NaiveDate, to: NaiveDate) -> Self {
        Self { from, to }
    }

    /// Create default date range: 7 days ago to 15 days ahead.
    pub fn default_range() -> Self {
        let today = chrono::Local::now().date_naive();
        let from = today - chrono::Duration::days(7);
        let to = today + chrono::Duration::days(15);
        Self { from, to }
    }

    /// Format date for Classe Viva input fields (DD-MM-YYYY).
    fn format_date(date: NaiveDate) -> String {
        date.format("%d-%m-%Y").to_string()
    }
}

/// Scraper for Classe Viva homework export.
pub struct ClasseVivaScraper {
    context: BrowserContext,
    credentials: Credentials,
}

impl ClasseVivaScraper {
    /// Create a new scraper with the given browser context and credentials.
    pub fn new(context: BrowserContext, credentials: Credentials) -> Self {
        Self {
            context,
            credentials,
        }
    }

    /// Perform login and return the page.
    pub async fn login(&self) -> Result<Page> {
        info!("Navigating to Classe Viva agenda page");
        let page = self
            .context
            .new_page()
            .await
            .context("Failed to create new page")?;

        // Navigate to agenda - will redirect to login if not authenticated
        page.goto_builder(AGENDA_URL)
            .goto()
            .await
            .context("Failed to navigate to agenda page")?;

        // Wait for login form to appear
        debug!("Waiting for login form");
        page.wait_for_selector_builder(selectors::LOGIN_USERNAME)
            .wait_for_selector()
            .await
            .context("Login form did not appear")?;

        // Fill credentials
        info!("Filling login credentials");
        page.fill_builder(selectors::LOGIN_USERNAME, &self.credentials.username)
            .fill()
            .await
            .context("Failed to fill username")?;

        page.fill_builder(selectors::LOGIN_PASSWORD, &self.credentials.password)
            .fill()
            .await
            .context("Failed to fill password")?;

        // Submit form
        debug!("Submitting login form");
        page.click_builder(selectors::LOGIN_SUBMIT)
            .click()
            .await
            .context("Failed to click login button")?;

        // Wait for navigation to complete (either success or error)
        // We wait for the agenda page to load by checking for the export button
        tokio::time::sleep(Duration::from_secs(2)).await;

        info!("Login submitted, waiting for page to load");
        Ok(page)
    }

    /// Open the export dialog on the agenda page.
    pub async fn open_export_dialog(&self, page: &Page) -> Result<()> {
        info!("Opening export dialog");

        // Wait for the export button to be visible and stable
        debug!("Waiting for export button to appear");
        page.wait_for_selector_builder(selectors::EXPORT_BUTTON)
            .state(FrameState::Visible)
            .wait_for_selector()
            .await
            .context("Export button not found - login may have failed")?;

        // Small delay to ensure the page is fully interactive
        // This helps with race conditions where the element exists but isn't clickable
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Click with force option to bypass actionability checks if needed
        debug!("Clicking export button");
        page.click_builder(selectors::EXPORT_BUTTON)
            .force(true)
            .click()
            .await
            .context("Failed to click export button")?;

        // Wait for dialog to appear
        debug!("Waiting for export dialog");
        page.wait_for_selector_builder(selectors::EXPORT_DIALOG)
            .state(FrameState::Visible)
            .wait_for_selector()
            .await
            .context("Export dialog did not appear")?;

        info!("Export dialog opened");
        Ok(())
    }

    /// Fill the date range in the export dialog.
    ///
    /// Uses Playwright's fill method and JavaScript to handle jQuery datepicker fields.
    pub async fn fill_date_range(&self, page: &Page, range: &DateRange) -> Result<()> {
        let from_str = DateRange::format_date(range.from);
        let to_str = DateRange::format_date(range.to);

        info!("Setting date range: {} to {}", from_str, to_str);

        // Helper to fill a date field, handling jQuery datepicker if present
        async fn fill_date_field(page: &Page, selector: &str, value: &str) -> Result<()> {
            // First, try to use Playwright's fill which clears and types
            page.fill_builder(selector, value)
                .fill()
                .await
                .context("Failed to fill date field")?;

            // Also trigger jQuery datepicker update if it exists
            // The datepicker stores its value internally and we need to sync it
            let js_sync_datepicker = r#"
                ([selector, value]) => {
                    const el = document.querySelector(selector);
                    if (el && typeof jQuery !== 'undefined' && jQuery(el).datepicker) {
                        // Parse DD-MM-YYYY format
                        const parts = value.split('-');
                        if (parts.length === 3) {
                            const date = new Date(parts[2], parts[1] - 1, parts[0]);
                            jQuery(el).datepicker('setDate', date);
                        }
                    }
                }
            "#;

            page.evaluate::<_, ()>(js_sync_datepicker, serde_json::json!([selector, value]))
                .await
                .context("Failed to sync datepicker")?;

            Ok(())
        }

        // Set the "from" date
        debug!("Setting from date: {}", from_str);
        fill_date_field(page, selectors::DATE_FROM, &from_str).await?;

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Set the "to" date
        debug!("Setting to date: {}", to_str);
        fill_date_field(page, selectors::DATE_TO, &to_str).await?;

        // Pause after setting dates to let UI fully update before clicking confirm
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(())
    }

    /// Trigger the download and save the file.
    ///
    /// Uses reqwest to download the file directly after capturing the download URL
    /// from Playwright. This works around bugs in playwright-rust's download API.
    ///
    /// Returns the path to the downloaded file.
    pub async fn trigger_download(&self, page: &Page, output_dir: &Path) -> Result<PathBuf> {
        info!("Triggering download");

        // Generate output filename with timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("export_{}.xls", timestamp);
        let output_path = output_dir
            .canonicalize()
            .context("Failed to resolve output directory path")?
            .join(&filename);

        // Start listening for download event before clicking
        let download_future = page.expect_event(EventType::Download);

        // Click confirm button to trigger download
        debug!("Clicking confirm button");
        page.click_builder(selectors::CONFIRM_BUTTON)
            .click()
            .await
            .context("Failed to click confirm button")?;

        // Wait for the download event to get the URL
        debug!("Waiting for download event");
        let event = tokio::time::timeout(Duration::from_secs(30), download_future)
            .await
            .context("Timeout waiting for download")?
            .context("Failed to receive download event")?;

        // Extract the download URL from the event
        let download = match event {
            Event::Download(d) => d,
            _ => return Err(anyhow!("Unexpected event type, expected Download")),
        };

        let download_url = download.url().to_string();
        info!("Download URL captured: {}", download_url);

        // Get cookies from browser context for authentication
        debug!("Extracting cookies from browser");
        let cookies = self
            .context
            .cookies(&[AGENDA_URL.to_string()])
            .await
            .context("Failed to get cookies")?;

        // Build cookie header string
        let cookie_header: String = cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ");

        debug!("Using {} cookies for download", cookies.len());

        // Download the file directly with reqwest
        let client = reqwest::Client::new();
        let response = client
            .get(&download_url)
            .header("Cookie", cookie_header)
            .send()
            .await
            .context("Failed to request download")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Download request failed with status: {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read download response")?;

        // Save to file
        std::fs::write(&output_path, &bytes).context("Failed to write download file")?;

        info!(
            "Download saved to: {:?} ({} bytes)",
            output_path,
            bytes.len()
        );
        Ok(output_path)
    }

    /// Perform the complete fetch operation.
    ///
    /// If `dry_run` is true, stops after login without downloading.
    pub async fn fetch(
        &self,
        range: DateRange,
        output_dir: &Path,
        dry_run: bool,
    ) -> Result<Option<PathBuf>> {
        // Step 1: Login
        let page = self.login().await?;

        if dry_run {
            info!("Dry run mode - stopping after login");
            return Ok(None);
        }

        // Step 2: Open export dialog
        self.open_export_dialog(&page).await?;

        // Step 3: Fill date range
        self.fill_date_range(&page, &range).await?;

        // Step 4: Trigger download
        let output_path = self.trigger_download(&page, output_dir).await?;

        Ok(Some(output_path))
    }
}
