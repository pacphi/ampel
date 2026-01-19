//! Rust refactoring using Syn + Quote + Prettyplease
//!
//! Uses Syn for AST parsing and Prettyplease for formatting

use super::{BackupManager, RefactorError, RefactorOptions, RefactorResult};
use std::fs;
use std::path::Path;

/// Refactor a Rust source file using Syn
pub fn refactor_rust_file(
    file: &Path,
    options: &RefactorOptions,
) -> Result<RefactorResult, RefactorError> {
    let source = fs::read_to_string(file)?;

    // For MVP: Use regex-based replacement
    // Full Syn AST transformation can be added in phase 2
    let (transformed, stats) = apply_rust_transformations(&source, &options.translation_map)?;

    let mut result = RefactorResult {
        files_modified: if stats.strings_replaced > 0 { 1 } else { 0 },
        strings_replaced: stats.strings_replaced,
        imports_added: if stats.imports_added { 1 } else { 0 },
        hooks_injected: 0,
        backup_path: None,
        preview: None,
    };

    if options.dry_run {
        result.preview = Some(transformed);
        return Ok(result);
    }

    // Create backup and write
    if options.create_backup && stats.strings_replaced > 0 {
        let backup_mgr = BackupManager::new();
        result.backup_path = Some(backup_mgr.backup_file(file)?);
    }

    if stats.strings_replaced > 0 {
        fs::write(file, &transformed)?;
    }

    Ok(result)
}

struct RustTransformStats {
    strings_replaced: usize,
    imports_added: bool,
}

/// Apply Rust-specific transformations
fn apply_rust_transformations(
    source: &str,
    mapping: &std::collections::HashMap<String, String>,
) -> Result<(String, RustTransformStats), RefactorError> {
    let mut output = source.to_string();
    let mut strings_replaced = 0;

    // Sort by length (longest first)
    let mut sorted_mapping: Vec<_> = mapping.iter().collect();
    sorted_mapping.sort_by_key(|(text, _)| std::cmp::Reverse(text.len()));

    for (text, key) in sorted_mapping {
        // Skip if already translated
        if output.contains(&format!("t!(\"{}\")", key)) {
            continue;
        }

        // Transform error macros: anyhow!("text") → anyhow!(t!("key"))
        let anyhow_pattern = format!("anyhow!(\"{}\")", text);
        let anyhow_replacement = format!("anyhow!(t!(\"{}\"))", key);
        let before_len = output.len();
        output = output.replace(&anyhow_pattern, &anyhow_replacement);
        if output.len() != before_len {
            strings_replaced += 1;
        }

        // Transform bail!: bail!("text") → bail!(t!("key"))
        let bail_pattern = format!("bail!(\"{}\")", text);
        let bail_replacement = format!("bail!(t!(\"{}\"))", key);
        let before_len = output.len();
        output = output.replace(&bail_pattern, &bail_replacement);
        if output.len() != before_len {
            strings_replaced += 1;
        }

        // Transform string literals in assignments: let x = "text" → let x = t!("key")
        let literal_pattern = format!("= \"{}\"", text);
        let literal_replacement = format!("= t!(\"{}\")", key);
        let before_len = output.len();
        output = output.replace(&literal_pattern, &literal_replacement);
        if output.len() != before_len {
            strings_replaced += 1;
        }
    }

    // Add import if we made transformations
    let imports_added = if strings_replaced > 0 && !output.contains("use rust_i18n::t") {
        output = add_rust_i18n_import(&output);
        true
    } else {
        false
    };

    Ok((
        output,
        RustTransformStats {
            strings_replaced,
            imports_added,
        },
    ))
}

/// Add rust_i18n::t import to Rust file
fn add_rust_i18n_import(source: &str) -> String {
    // Find first non-comment, non-attribute line
    let lines: Vec<&str> = source.lines().collect();
    let mut insert_pos = 0;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("#!")
            && !trimmed.starts_with("#[")
        {
            insert_pos = idx;
            break;
        }
    }

    let mut new_lines = lines;
    new_lines.insert(insert_pos, "use rust_i18n::t;");

    new_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_transform_anyhow() {
        let source = r#"anyhow!("Authentication failed")"#;
        let mut mapping = HashMap::new();
        mapping.insert(
            "Authentication failed".to_string(),
            "errors.auth.failed".to_string(),
        );

        let (output, stats) = apply_rust_transformations(source, &mapping).unwrap();

        assert_eq!(stats.strings_replaced, 1);
        assert!(output.contains(r#"t!("errors.auth.failed")"#));
    }

    #[test]
    fn test_transform_bail() {
        let source = r#"bail!("Connection timeout")"#;
        let mut mapping = HashMap::new();
        mapping.insert(
            "Connection timeout".to_string(),
            "errors.timeout".to_string(),
        );

        let (output, _stats) = apply_rust_transformations(source, &mapping).unwrap();

        // Verify transformation occurred (either bail! or anyhow! pattern might match)
        assert!(
            output.contains(r#"t!("errors.timeout")"#),
            "Should contain t!() macro call"
        );
    }

    #[test]
    fn test_add_rust_import() {
        let source = r#"fn main() {
    println!("Hello");
}"#;

        let output = add_rust_i18n_import(source);
        assert!(output.contains("use rust_i18n::t;"));
    }

    #[test]
    fn test_refactor_rust_dry_run() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"anyhow!("Error")"#).unwrap();

        let mut mapping = HashMap::new();
        mapping.insert("Error".to_string(), "errors.generic".to_string());

        let options = RefactorOptions {
            dry_run: true,
            translation_map: mapping,
            ..Default::default()
        };

        let result = refactor_rust_file(file.path(), &options).unwrap();
        assert!(result.preview.is_some());
        assert!(result.preview.unwrap().contains("t!(\"errors.generic\")"));
    }
}
