use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod data;
mod db;
mod html;
mod parser;
mod server;
mod types;

#[derive(Parser, Debug)]
#[command(name = "compitutto")]
#[command(about = "Parse homework calendar exports and generate a web view")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output directory for generated files
    #[arg(short, long, default_value = ".", global = true)]
    output: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", global = true)]
    log_level: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the web server (default)
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Process files and generate static HTML (no server)
    Build,

    /// Process a specific file
    Parse {
        /// Path to the Excel XML file
        file: PathBuf,
    },
}

fn init_tracing(log_level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level))
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("tower_http=warn".parse().unwrap());

    fmt().with_env_filter(filter).with_target(false).init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    init_tracing(&args.log_level);

    match args.command {
        // Default to serve if no command specified
        None | Some(Commands::Serve { port: 8080 }) => {
            server::serve(8080, args.output).await?;
        }
        Some(Commands::Serve { port }) => {
            server::serve(port, args.output).await?;
        }
        Some(Commands::Build) => {
            let entries = data::parse_all_exports()?;
            let html_path = args.output.join("index.html");
            html::generate_html(&entries, &html_path)?;
            info!(path = %html_path.display(), "HTML saved");
        }
        Some(Commands::Parse { file }) => {
            let entries = parser::parse_excel_xml(&file)?;
            info!(count = entries.len(), file = %file.display(), "Found entries");
            for entry in &entries {
                info!(
                    date = %entry.date,
                    subject = %entry.subject,
                    entry_type = %entry.entry_type,
                    "Entry"
                );
            }
        }
    }

    Ok(())
}
