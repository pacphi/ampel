//! Code refactoring for automatic i18n integration
//!
//! This module provides AST-based code transformation to automatically
//! replace hardcoded strings with i18n function calls.

pub mod backup;
pub mod rust_syn;
pub mod typescript_oxc;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use backup::{BackupError, BackupManager};

/// Options for refactoring operations
#[derive(Debug, Clone)]
pub struct RefactorOptions {
    /// Preview changes without modifying files
    pub dry_run: bool,

    /// Create backups before modifying files
    pub create_backup: bool,

    /// Default namespace for generated keys
    pub namespace: String,

    /// Mapping of text → translation key
    pub translation_map: HashMap<String, String>,

    /// Automatically inject necessary imports
    pub auto_inject_imports: bool,
}

impl Default for RefactorOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            create_backup: true,
            namespace: "common".to_string(),
            translation_map: HashMap::new(),
            auto_inject_imports: true,
        }
    }
}

/// Result of refactoring operation
#[derive(Debug, Clone)]
pub struct RefactorResult {
    /// Number of files modified
    pub files_modified: usize,

    /// Number of strings replaced with i18n calls
    pub strings_replaced: usize,

    /// Number of import statements added
    pub imports_added: usize,

    /// Number of hooks injected (React only)
    pub hooks_injected: usize,

    /// Path to backup file (if created)
    pub backup_path: Option<PathBuf>,

    /// Modified source code (for dry-run preview)
    pub preview: Option<String>,
}

/// Errors that can occur during refactoring
#[derive(Debug, Error)]
pub enum RefactorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Transform error: {0}")]
    TransformError(String),

    #[error("Generate error: {0}")]
    GenerateError(String),

    #[error("Backup error: {0}")]
    Backup(#[from] BackupError),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

/// Refactor a single file based on detected language
pub fn refactor_file(
    file: &Path,
    options: &RefactorOptions,
) -> Result<RefactorResult, RefactorError> {
    // Detect language from extension
    let extension = file
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| RefactorError::UnsupportedLanguage("Unknown extension".to_string()))?;

    match extension {
        "ts" | "tsx" | "js" | "jsx" => typescript_oxc::refactor_typescript_file(file, options),
        "rs" => rust_syn::refactor_rust_file(file, options),
        _ => Err(RefactorError::UnsupportedLanguage(format!(
            "Extension '{}' not supported",
            extension
        ))),
    }
}

/// Refactor all files in a directory matching patterns
pub fn refactor_directory(
    dir: &Path,
    patterns: &[String],
    options: &RefactorOptions,
) -> Result<Vec<RefactorResult>, RefactorError> {
    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() && matches_pattern(path, patterns) {
            let result = refactor_file(path, options)?;
            if result.strings_replaced > 0 || result.imports_added > 0 {
                results.push(result);
            }
        }
    }

    Ok(results)
}

/// Check if file matches any of the glob patterns
fn matches_pattern(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return true;
    }

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    for pattern in patterns {
        if let Some(ext) = pattern.strip_prefix("*.") {
            if file_name.ends_with(ext) {
                return true;
            }
        } else if pattern == file_name {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern(
            Path::new("test.tsx"),
            &["*.tsx".to_string()]
        ));
        assert!(matches_pattern(Path::new("test.rs"), &["*.rs".to_string()]));
        assert!(!matches_pattern(
            Path::new("test.txt"),
            &["*.tsx".to_string()]
        ));
    }

    #[test]
    fn test_refactor_options_default() {
        let options = RefactorOptions::default();
        assert!(!options.dry_run);
        assert!(options.create_backup);
        assert_eq!(options.namespace, "common");
        assert!(options.auto_inject_imports);
    }
}
