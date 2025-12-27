//! Code generation for type-safe translations
//!
//! This module provides code generation functionality for creating type-safe
//! translation interfaces in TypeScript and Rust.

use async_trait::async_trait;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

// Re-export types from formats module
pub use crate::formats::{PluralForms, TranslationValue};

// Type alias for translation maps
pub type TranslationMap = BTreeMap<String, TranslationValue>;

pub mod rust;
pub mod typescript;

/// Generator configuration options
#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    /// Pretty print output
    pub pretty_print: bool,
    /// Include metadata comments
    pub include_metadata: bool,
    /// Split by namespace
    pub split_by_namespace: bool,
    /// Create index file
    pub create_index: bool,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            pretty_print: true,
            include_metadata: true,
            split_by_namespace: false,
            create_index: true,
        }
    }
}

/// Generator result with statistics
#[derive(Debug)]
pub struct GeneratorResult {
    /// Files that were created
    pub files_created: Vec<PathBuf>,
    /// Number of languages processed
    pub languages_processed: u32,
    /// Number of keys written
    pub keys_written: u32,
}

/// Code generator trait for different target languages
#[async_trait]
pub trait CodeGenerator: Send + Sync {
    /// Generate code from translation map
    async fn generate(
        &self,
        translations: &TranslationMap,
        language: &str,
        output_dir: &Path,
        options: GeneratorOptions,
    ) -> Result<GeneratorResult, GeneratorError>;

    /// Get the file extension for generated files
    fn file_extension(&self) -> &str;

    /// Get the language name (e.g., "TypeScript", "Rust")
    fn language_name(&self) -> &str;
}

/// Errors that can occur during code generation
#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid translation key: {0}")]
    InvalidKey(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Code generation failed: {0}")]
    GenerationFailed(String),
}

/// Helper function to flatten nested translations into dot-notation keys
pub fn flatten_translations(
    translations: &TranslationMap,
) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    flatten_recursive(translations, "", &mut result);
    result
}

fn flatten_recursive(
    map: &TranslationMap,
    prefix: &str,
    result: &mut BTreeMap<String, String>,
) {
    for (key, value) in map {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            TranslationValue::String(s) => {
                result.insert(full_key, s.clone());
            }
            TranslationValue::Plural(forms) => {
                // For plural forms, we store the "other" form as the base key
                result.insert(full_key.clone(), forms.other.clone());
                // And also store each plural form with suffix
                if let Some(ref zero) = forms.zero {
                    result.insert(format!("{}_zero", full_key), zero.clone());
                }
                if let Some(ref one) = forms.one {
                    result.insert(format!("{}_one", full_key), one.clone());
                }
                if let Some(ref two) = forms.two {
                    result.insert(format!("{}_two", full_key), two.clone());
                }
                if let Some(ref few) = forms.few {
                    result.insert(format!("{}_few", full_key), few.clone());
                }
                if let Some(ref many) = forms.many {
                    result.insert(format!("{}_many", full_key), many.clone());
                }
            }
            TranslationValue::Nested(nested) => {
                flatten_recursive(nested, &full_key, result);
            }
        }
    }
}

/// Helper function to validate TypeScript/Rust identifier
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_char = s.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Convert a translation key to a valid TypeScript/Rust identifier
pub fn sanitize_key(key: &str) -> String {
    let sanitized: String = key
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Ensure it doesn't start with a number
    if let Some(first_char) = sanitized.chars().next() {
        if first_char.is_numeric() {
            return format!("_{}", sanitized);
        }
    }
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_translations_simple() {
        let mut translations = BTreeMap::new();
        translations.insert(
            "hello".to_string(),
            TranslationValue::String("Hello".to_string()),
        );
        translations.insert(
            "world".to_string(),
            TranslationValue::String("World".to_string()),
        );

        let flattened = flatten_translations(&translations);
        assert_eq!(flattened.len(), 2);
        assert_eq!(flattened.get("hello").unwrap(), "Hello");
        assert_eq!(flattened.get("world").unwrap(), "World");
    }

    #[test]
    fn test_flatten_translations_nested() {
        let mut translations = BTreeMap::new();
        let mut nested = BTreeMap::new();
        nested.insert(
            "greeting".to_string(),
            TranslationValue::String("Hello".to_string()),
        );

        translations.insert("common".to_string(), TranslationValue::Nested(nested));

        let flattened = flatten_translations(&translations);
        assert_eq!(flattened.len(), 1);
        assert_eq!(flattened.get("common.greeting").unwrap(), "Hello");
    }

    #[test]
    fn test_flatten_translations_plural() {
        let mut translations = BTreeMap::new();
        let plural = PluralForms {
            zero: None,
            one: Some("1 item".to_string()),
            two: None,
            few: None,
            many: None,
            other: "{{count}} items".to_string(),
        };
        translations.insert("items".to_string(), TranslationValue::Plural(plural));

        let flattened = flatten_translations(&translations);
        assert!(flattened.contains_key("items"));
        assert!(flattened.contains_key("items_one"));
        assert_eq!(flattened.get("items").unwrap(), "{{count}} items");
        assert_eq!(flattened.get("items_one").unwrap(), "1 item");
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("hello"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("CamelCase"));
        assert!(is_valid_identifier("snake_case"));
        assert!(is_valid_identifier("with123"));

        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123start"));
        assert!(!is_valid_identifier("with-dash"));
        assert!(!is_valid_identifier("with.dot"));
        assert!(!is_valid_identifier("with space"));
    }

    #[test]
    fn test_sanitize_key() {
        assert_eq!(sanitize_key("hello"), "hello");
        assert_eq!(sanitize_key("hello.world"), "hello_world");
        assert_eq!(sanitize_key("hello-world"), "hello_world");
        assert_eq!(sanitize_key("hello world"), "hello_world");
        assert_eq!(sanitize_key("123start"), "_123start");
        assert_eq!(sanitize_key("app.name"), "app_name");
    }
}
