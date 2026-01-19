//! # Ampel i18n Builder CLI
//!
//! Command-line interface for translation automation workflows.

use anyhow::Result;
use clap::Parser;

use ampel_i18n_builder::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present (system env vars take precedence)
    // Silent failure if .env doesn't exist - it's optional
    #[cfg(debug_assertions)]
    if let Err(e) = dotenv::dotenv() {
        eprintln!("Note: .env file not found or error loading: {}", e);
    }
    #[cfg(not(debug_assertions))]
    let _ = dotenv::dotenv();

    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => {
            ampel_i18n_builder::cli::init::execute(args).await?;
        }
        Commands::Doctor(args) => {
            ampel_i18n_builder::cli::doctor::execute(args).await?;
        }
        Commands::Translate(args) => {
            ampel_i18n_builder::cli::translate::execute(args).await?;
        }
        Commands::Sync(args) => {
            ampel_i18n_builder::cli::sync::execute(args).await?;
        }
        Commands::Validate(args) => {
            ampel_i18n_builder::cli::validate::execute(args).await?;
        }
        Commands::Coverage(args) => {
            ampel_i18n_builder::cli::coverage::execute(args).await?;
        }
        Commands::Export(args) => {
            ampel_i18n_builder::cli::export::execute(args).await?;
        }
        Commands::Import(args) => {
            ampel_i18n_builder::cli::import::execute(args).await?;
        }
        Commands::Missing(args) => {
            ampel_i18n_builder::cli::missing::execute(args).await?;
        }
        Commands::Report(args) => {
            ampel_i18n_builder::cli::report::execute(args).await?;
        }
        Commands::GenerateTypes(args) => {
            ampel_i18n_builder::cli::generate_types::execute(args).await?;
        }
        Commands::Extract(args) => {
            ampel_i18n_builder::cli::extract::execute(args).await?;
        }
        Commands::Refactor(args) => {
            ampel_i18n_builder::cli::refactor::execute(args).await?;
        }
    }

    Ok(())
}
