//! Raschietto - Automated fetcher for Classe Viva homework exports.
//!
//! Uses Playwright to automate logging into Classe Viva, navigating to the
//! agenda page, and downloading homework exports as Excel files.

mod browser;
mod config;
mod scraper;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use browser::{BrowserOptions, BrowserSession};
use config::Credentials;
use scraper::{ClasseVivaScraper, DateRange};

#[derive(Parser)]
#[command(name = "raschietto")]
#[command(about = "Automated fetcher for Classe Viva homework exports")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch homework exports from Classe Viva
    Fetch {
        /// Start date for export range (YYYY-MM-DD format)
        /// Default: 7 days ago
        #[arg(long)]
        from: Option<NaiveDate>,

        /// End date for export range (YYYY-MM-DD format)
        /// Default: 15 days from now
        #[arg(long)]
        to: Option<NaiveDate>,

        /// Show browser window instead of running headless
        #[arg(long)]
        headed: bool,

        /// Only login, don't download (verify credentials work)
        #[arg(long)]
        dry_run: bool,

        /// Output directory for downloaded files
        /// Default: ./data
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch {
            from,
            to,
            headed,
            dry_run,
            output,
        } => {
            fetch_command(from, to, headed, dry_run, output).await?;
        }
    }

    Ok(())
}

async fn fetch_command(
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    headed: bool,
    dry_run: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    // Load credentials
    let credentials = Credentials::from_env().context("Failed to load credentials")?;
    info!("Loaded credentials for user: {}", credentials.username);

    // Determine date range
    let range = match (from, to) {
        (Some(f), Some(t)) => DateRange::new(f, t),
        (Some(f), None) => {
            let default = DateRange::default_range();
            DateRange::new(f, default.to)
        }
        (None, Some(t)) => {
            let default = DateRange::default_range();
            DateRange::new(default.from, t)
        }
        (None, None) => DateRange::default_range(),
    };
    info!("Date range: {} to {}", range.from, range.to);

    // Determine output directory
    let output_dir = output.unwrap_or_else(|| PathBuf::from("data"));
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
    }
    info!("Output directory: {:?}", output_dir);

    // Launch browser
    let options = BrowserOptions { headed };
    info!(
        "Launching browser ({})",
        if headed { "headed" } else { "headless" }
    );

    let session = BrowserSession::launch(options)
        .await
        .context("Failed to launch browser")?;

    // Create browser context
    let context = session.new_context().await?;

    // Create scraper and run
    let scraper = ClasseVivaScraper::new(context, credentials);

    match scraper.fetch(range, &output_dir, dry_run).await {
        Ok(Some(path)) => {
            info!("Successfully downloaded to: {:?}", path);
        }
        Ok(None) => {
            info!("Dry run completed successfully");
        }
        Err(e) => {
            error!("Fetch failed: {}", e);
            return Err(e);
        }
    }

    // Close browser
    session.close().await?;

    Ok(())
}
