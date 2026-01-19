//! Refactor command - automatically replace hardcoded strings with i18n calls

use crate::cli::RefactorArgs;
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat};
use crate::refactor::{refactor_directory, refactor_file, RefactorOptions};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;

pub async fn execute(args: RefactorArgs) -> Result<()> {
    println!(
        "{} Starting code refactoring...",
        "→".cyan().bold()
    );

    // Load translation mapping from file
    let mapping = load_mapping(&args.mapping)?;

    if mapping.is_empty() {
        println!("{} No translations found in mapping file", "!".yellow());
        return Ok(());
    }

    println!(
        "{} Loaded {} translation mappings",
        "✓".green().bold(),
        mapping.len()
    );

    // Create refactor options
    let options = RefactorOptions {
        dry_run: args.dry_run,
        create_backup: !args.no_backup,
        namespace: args.namespace.clone(),
        translation_map: mapping,
        auto_inject_imports: true,
    };

    // Refactor target (file or directory)
    if args.target.is_file() {
        let result = refactor_file(&args.target, &options)?;

        if args.dry_run {
            println!("\n{} Dry-run preview:", "→".cyan().bold());
            if let Some(preview) = result.preview {
                println!("{}", preview);
            }
        } else {
            println!("\n{} Refactoring complete:", "✓".green().bold());
            println!("  Files modified: {}", result.files_modified);
            println!("  Strings replaced: {}", result.strings_replaced);
            println!("  Imports added: {}", result.imports_added);

            if let Some(backup) = result.backup_path {
                println!(
                    "  Backup created: {}",
                    backup.display().to_string().cyan()
                );
            }
        }
    } else if args.target.is_dir() {
        let results = refactor_directory(&args.target, &args.patterns, &options)?;

        if results.is_empty() {
            println!("{} No files needed refactoring", "!".yellow());
            return Ok(());
        }

        let total_files = results.len();
        let total_strings: usize = results.iter().map(|r| r.strings_replaced).sum();
        let total_imports: usize = results.iter().map(|r| r.imports_added).sum();

        println!("\n{} Refactoring complete:", "✓".green().bold());
        println!("  Files modified: {}", total_files);
        println!("  Strings replaced: {}", total_strings);
        println!("  Imports added: {}", total_imports);

        if !args.no_backup {
            println!(
                "  Backups in: {}",
                ".ampel-i18n-backups/".cyan()
            );
        }
    } else {
        return Err(crate::error::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Target not found: {}", args.target.display()),
        )));
    }

    if args.dry_run {
        println!("\n{} Dry-run mode: no files were modified", "!".yellow().bold());
    }

    Ok(())
}

/// Load translation mapping from JSON file
///
/// Expected format: { "text": { "generated_key": "key.path" } }
/// or simpler: { "text": "key.path" }
fn load_mapping(path: &std::path::Path) -> Result<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let format = JsonFormat::new();
    let translation_map = format.parse(&content)?;

    // Flatten the translation map to text → key mapping
    let mut mapping = HashMap::new();
    flatten_for_mapping(&translation_map, "", &mut mapping);

    Ok(mapping)
}

/// Flatten translation map for refactoring
fn flatten_for_mapping(
    map: &crate::formats::TranslationMap,
    prefix: &str,
    result: &mut HashMap<String, String>,
) {
    use crate::formats::TranslationValue;

    for (key, value) in map {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            TranslationValue::String(text) => {
                // Store text → key mapping (reverse of normal translation)
                result.insert(text.clone(), full_key);
            }
            TranslationValue::Nested(nested) => {
                flatten_for_mapping(nested, &full_key, result);
            }
            TranslationValue::Plural(_) => {
                // Skip plural forms for now
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_flatten_for_mapping() {
        use crate::formats::{TranslationMap, TranslationValue};

        let mut map = TranslationMap::new();
        map.insert(
            "button".to_string(),
            TranslationValue::Nested({
                let mut nested = BTreeMap::new();
                nested.insert(
                    "save".to_string(),
                    TranslationValue::String("Save Changes".to_string()),
                );
                nested
            }),
        );

        let mut result = HashMap::new();
        flatten_for_mapping(&map, "", &mut result);

        assert_eq!(result.get("Save Changes"), Some(&"button.save".to_string()));
    }

    #[tokio::test]
    async fn test_refactor_command_dry_run() {
        use tempfile::TempDir;

        // Create temp directory
        let temp_dir = TempDir::new().unwrap();

        // Create test file with .tsx extension
        let test_file = temp_dir.path().join("Test.tsx");
        fs::write(&test_file, "<Button>Save</Button>").unwrap();

        // Create mapping file
        let mapping_file = temp_dir.path().join("mapping.json");
        fs::write(
            &mapping_file,
            r#"{"button": {"save": "Save"}}"#
        )
        .unwrap();

        let args = RefactorArgs {
            target: test_file.clone(),
            mapping: mapping_file.clone(),
            namespace: "common".to_string(),
            dry_run: true,
            no_backup: false,
            patterns: vec!["*.tsx".to_string()],
        };

        // Should not error in dry-run
        execute(args).await.unwrap();
    }
}
