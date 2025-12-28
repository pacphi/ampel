//! CLI command structure and argument parsing.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod translate;
pub mod sync;
pub mod validate;
pub mod coverage;
pub mod export;
pub mod import;

#[derive(Parser)]
#[command(name = "cargo-i18n")]
#[command(bin_name = "cargo i18n")]
#[command(about = "Translation automation for Ampel", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Translate missing keys using AI translation service
    Translate(TranslateArgs),

    /// Sync all languages from source language
    Sync(SyncArgs),

    /// Validate translation files for errors
    Validate(ValidateArgs),

    /// Check translation coverage statistics
    Coverage(CoverageArgs),

    /// Export translations for external translation service
    Export(ExportArgs),

    /// Import translations from external translation service
    Import(ImportArgs),
}

#[derive(Parser)]
pub struct TranslateArgs {
    /// Target language code (e.g., "fi", "sv", "es")
    #[arg(short, long)]
    pub lang: String,

    /// Translation provider to use (deprecated - use --no-fallback instead)
    #[arg(short, long, value_enum)]
    pub provider: Option<TranslationProvider>,

    /// Only translate specific namespace (e.g., "dashboard", "settings")
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// Preview changes without writing files
    #[arg(long)]
    pub dry_run: bool,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,

    /// Override global timeout (seconds)
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Override batch size
    #[arg(long)]
    pub batch_size: Option<usize>,

    /// Override max retry attempts
    #[arg(long)]
    pub max_retries: Option<usize>,

    /// Disable specific providers (can be repeated)
    #[arg(long = "disable-provider")]
    pub disabled_providers: Vec<String>,

    /// Disable fallback (use only primary provider)
    #[arg(long)]
    pub no_fallback: bool,
}

#[derive(Parser)]
pub struct SyncArgs {
    /// Source language code
    #[arg(short, long, default_value = "en")]
    pub source: String,

    /// Translation provider to use
    #[arg(short, long, value_enum)]
    pub provider: TranslationProvider,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,

    /// Preview changes without writing files
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser)]
pub struct CoverageArgs {
    /// Check specific language only
    #[arg(short, long)]
    pub lang: Option<String>,

    /// Minimum coverage threshold (0.0-1.0)
    #[arg(long)]
    pub min_coverage: Option<f32>,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,
}

#[derive(Parser)]
pub struct ValidateArgs {
    /// Validate all languages
    #[arg(long)]
    pub all: bool,

    /// Validate specific language only
    #[arg(short, long)]
    pub lang: Option<String>,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,
}

#[derive(Parser)]
pub struct ExportArgs {
    /// Target language code
    #[arg(short, long)]
    pub lang: String,

    /// Export format
    #[arg(short, long, value_enum)]
    pub format: ExportFormat,

    /// Output file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,
}

#[derive(Parser)]
pub struct ImportArgs {
    /// Target language code
    #[arg(short, long)]
    pub lang: String,

    /// Import format
    #[arg(short, long, value_enum)]
    pub format: ExportFormat,

    /// Input file path
    #[arg(short, long)]
    pub input: PathBuf,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,

    /// Preview changes without writing files
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TranslationProvider {
    /// Systran Translation API (requires SYSTRAN_API_KEY) - Tier 1
    Systran,
    /// DeepL API (requires DEEPL_API_KEY) - Tier 2
    DeepL,
    /// Google Cloud Translation API (requires GOOGLE_API_KEY) - Tier 3
    Google,
    /// OpenAI GPT-4 (requires OPENAI_API_KEY) - Tier 4
    OpenAI,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ExportFormat {
    /// XLIFF 1.2 format for translation memory tools
    Xliff,
    /// CSV format for spreadsheet editing
    Csv,
    /// JSON format for custom tools
    Json,
}
