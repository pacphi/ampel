//! CLI command structure and argument parsing.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod coverage;
pub mod doctor;
pub mod export;
pub mod generate_types;
pub mod import;
pub mod init;
pub mod missing;
pub mod report;
pub mod sync;
pub mod translate;
pub mod validate;

#[derive(Parser)]
#[command(name = "ampel-i18n")]
#[command(bin_name = "cargo i18n")]
#[command(about = "Translation automation for Ampel", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive setup wizard for first-time users
    Init(InitArgs),

    /// Run health checks and validate configuration
    Doctor(DoctorArgs),

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

    /// List missing translation keys per language
    Missing(MissingArgs),

    /// Generate coverage reports in various formats
    Report(ReportArgs),

    /// Generate TypeScript type definitions from translations
    GenerateTypes(GenerateTypesArgs),
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

    /// Force retranslation of all keys (ignores existing translations)
    #[arg(long)]
    pub force: bool,

    /// Detect and retranslate keys with English/source language values
    #[arg(long)]
    pub detect_untranslated: bool,
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

    /// Minimum coverage threshold as percentage (e.g., 95 for 95%) or fraction (e.g., 0.95)
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
    /// OpenAI GPT-5-mini (requires OPENAI_API_KEY) - Tier 4
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

#[derive(Parser)]
pub struct MissingArgs {
    /// Check specific language only
    #[arg(short, long)]
    pub lang: Option<String>,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,
}

#[derive(Parser)]
pub struct ReportArgs {
    /// Output format
    #[arg(short, long, value_enum, default_value = "markdown")]
    pub format: ReportFormat,

    /// Check specific language only
    #[arg(short, long)]
    pub lang: Option<String>,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ReportFormat {
    /// JSON format for programmatic access
    Json,
    /// Markdown format for documentation
    Markdown,
}

#[derive(Parser)]
pub struct GenerateTypesArgs {
    /// Output file path for TypeScript types
    #[arg(short, long, default_value = "frontend/src/i18n/types.ts")]
    pub output: PathBuf,

    /// Path to translation directory
    #[arg(long, default_value = "frontend/public/locales")]
    pub translation_dir: PathBuf,
}

#[derive(Parser)]
pub struct InitArgs {
    /// Skip interactive prompts and use defaults
    #[arg(long)]
    pub non_interactive: bool,

    /// Project framework (react, vue, rust, etc.)
    #[arg(long)]
    pub framework: Option<String>,

    /// Target languages (comma-separated, e.g., "fr,de,es")
    #[arg(long)]
    pub languages: Option<String>,

    /// Translation provider (openai, deepl, google, systran)
    #[arg(long)]
    pub provider: Option<String>,

    /// Translation directory path
    #[arg(long)]
    pub translation_dir: Option<PathBuf>,
}

#[derive(Parser)]
pub struct DoctorArgs {
    /// Show detailed diagnostic information
    #[arg(long)]
    pub verbose: bool,

    /// Attempt to fix common issues automatically
    #[arg(long)]
    pub fix: bool,
}
