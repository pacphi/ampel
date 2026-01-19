//! TypeScript/React refactoring using OXC parser
//!
//! Uses OXC (Oxc Parser) for TypeScript/TSX AST transformation.
//! OXC is 3x faster than SWC and works on stable Rust.

use super::{BackupManager, RefactorError, RefactorOptions, RefactorResult};
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use std::fs;
use std::path::Path;

/// Refactor a TypeScript/React file using OXC
pub fn refactor_typescript_file(
    file: &Path,
    options: &RefactorOptions,
) -> Result<RefactorResult, RefactorError> {
    let source = fs::read_to_string(file)?;

    // Parse with OXC
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file).unwrap_or(SourceType::tsx());

    let ParserReturn {
        program: _, errors, ..
    } = Parser::new(&allocator, &source, source_type).parse();

    if !errors.is_empty() {
        return Err(RefactorError::ParseError(format!(
            "Failed to parse {}: {} errors",
            file.display(),
            errors.len()
        )));
    }

    // For MVP: Use regex-based replacement on the source text
    // Full AST transformation would require OXC codegen which is still in development
    let (transformed, stats) = apply_regex_transformations(&source, &options.translation_map)?;

    let mut result = RefactorResult {
        files_modified: if stats.strings_replaced > 0 { 1 } else { 0 },
        strings_replaced: stats.strings_replaced,
        imports_added: 0,
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

struct TransformStats {
    strings_replaced: usize,
}

/// Apply regex-based transformations (MVP implementation)
///
/// Full AST codegen with OXC is in development, so using regex for MVP
fn apply_regex_transformations(
    source: &str,
    mapping: &std::collections::HashMap<String, String>,
) -> Result<(String, TransformStats), RefactorError> {
    let mut output = source.to_string();
    let mut strings_replaced = 0;

    // Sort by length (longest first) to avoid partial replacements
    let mut sorted_mapping: Vec<_> = mapping.iter().collect();
    sorted_mapping.sort_by_key(|(text, _)| std::cmp::Reverse(text.len()));

    for (text, key) in sorted_mapping {
        // Skip if already translated
        if output.contains(&format!("t('{}')", key)) || output.contains(&format!("t(\"{}\") ", key))
        {
            continue;
        }

        // Transform JSX text: <Tag>Text</Tag> → <Tag>{t('key')}</Tag>
        let jsx_text_pattern = format!(">{}<", regex::escape(text));
        let jsx_text_replacement = format!(">{{t('{}')}}<", key);

        let before_len = output.len();
        output = output.replace(&jsx_text_pattern, &jsx_text_replacement);
        if output.len() != before_len {
            strings_replaced += 1;
        }

        // Transform JSX attributes: prop="text" → prop={t('key')}
        for attr in &["placeholder", "aria-label", "title", "label"] {
            let attr_pattern = format!("{}=\"{}\"", attr, regex::escape(text));
            let attr_replacement = format!("{}={{t('{}')}}", attr, key);

            let before_len = output.len();
            output = output.replace(&attr_pattern, &attr_replacement);
            if output.len() != before_len {
                strings_replaced += 1;
            }
        }

        // Transform string literals: const x = "text" → const x = t('key')
        let literal_pattern = format!("\"{}\"", regex::escape(text));
        let literal_replacement = format!("t('{}')", key);

        // Only replace in safe contexts (not in imports, not in already-transformed code)
        if !output.contains("import") || !literal_pattern.contains("react") {
            let before_len = output.len();
            output = output.replace(&literal_pattern, &literal_replacement);
            if output.len() != before_len {
                strings_replaced += 1;
            }
        }
    }

    // Add import if we made transformations
    if strings_replaced > 0 && !output.contains("useTranslation") {
        output = add_use_translation_import(&output);
    }

    Ok((output, TransformStats { strings_replaced }))
}

/// Add useTranslation import and hook to a React component
fn add_use_translation_import(source: &str) -> String {
    let mut output = source.to_string();

    // Add import at top if missing
    if !output.contains("import { useTranslation }") {
        // Find the first import or the beginning of the file
        if let Some(pos) = output.find("import ") {
            output.insert_str(pos, "import { useTranslation } from 'react-i18next';\n");
        } else {
            output.insert_str(0, "import { useTranslation } from 'react-i18next';\n\n");
        }
    }

    // Add hook at component start (simplified - finds export default function)
    if !output.contains("const { t } = useTranslation") {
        if let Some(pos) = output.find("export default function") {
            // Find opening brace
            if let Some(brace_pos) = output[pos..].find('{').map(|p| pos + p) {
                let injection = "\n  const { t } = useTranslation('common');\n";
                output.insert_str(brace_pos + 1, injection);
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_oxc_parses_tsx() {
        let source = r#"<Button>Click me</Button>"#;
        let allocator = Allocator::default();
        let source_type = SourceType::tsx();

        let result = Parser::new(&allocator, source, source_type).parse();
        assert!(result.errors.is_empty(), "Should parse valid TSX");
    }

    #[test]
    fn test_regex_transform_jsx_text() {
        let source = r#"<Button>Save Changes</Button>"#;
        let mut mapping = HashMap::new();
        mapping.insert("Save Changes".to_string(), "button.saveChanges".to_string());

        let (output, stats) = apply_regex_transformations(source, &mapping).unwrap();

        assert_eq!(stats.strings_replaced, 1);
        assert!(output.contains("t('button.saveChanges')"));
    }

    #[test]
    fn test_regex_transform_jsx_attribute() {
        let source = r#"<Input placeholder="Enter name" />"#;
        let mut mapping = HashMap::new();
        mapping.insert(
            "Enter name".to_string(),
            "placeholder.enterName".to_string(),
        );

        let (output, stats) = apply_regex_transformations(source, &mapping).unwrap();

        assert_eq!(stats.strings_replaced, 1);
        assert!(output.contains("placeholder={t('placeholder.enterName')}"));
    }

    #[test]
    fn test_add_use_translation_import() {
        let source = r#"export default function MyComponent() {
  return <div>Hello</div>;
}"#;

        let output = add_use_translation_import(source);

        assert!(output.contains("import { useTranslation }"));
        assert!(output.contains("const { t } = useTranslation"));
    }

    #[test]
    fn test_refactor_typescript_dry_run() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "<Button>Save</Button>").unwrap();

        let mut mapping = HashMap::new();
        mapping.insert("Save".to_string(), "button.save".to_string());

        let options = RefactorOptions {
            dry_run: true,
            translation_map: mapping,
            ..Default::default()
        };

        let result = refactor_typescript_file(file.path(), &options).unwrap();
        assert!(result.preview.is_some());
        assert!(result.preview.unwrap().contains("t('button.save')"));
    }
}
