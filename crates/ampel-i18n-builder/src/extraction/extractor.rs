//! Core extraction trait and types for string extraction

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Represents a string extracted from source code
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedString {
    /// The actual text content
    pub text: String,

    /// Source file path
    pub file: PathBuf,

    /// Line number in source file (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// Context where the string was found
    pub context: StringContext,

    /// If this string is already wrapped in an i18n call, store the existing key
    pub existing_key: Option<String>,

    /// Variables found in template strings (e.g., ${userName})
    pub variables: Vec<String>,
}

/// Context information about where a string was found
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StringContext {
    /// Button label: <Button>Click me</Button>
    ButtonLabel,

    /// Input placeholder: <Input placeholder="Enter name" />
    Placeholder,

    /// Error message in code
    ErrorMessage,

    /// Form validation message
    ValidationMessage,

    /// Page or section title
    PageTitle,

    /// Heading text (h1-h6)
    Heading,

    /// ARIA label for accessibility
    AriaLabel,

    /// Generic UI text
    UiText,

    /// Log message (console.log, println!, etc.)
    LogMessage,

    /// Unknown context
    Unknown,
}

impl StringContext {
    /// Get a prefix for semantic key generation
    pub fn key_prefix(&self) -> &str {
        match self {
            StringContext::ButtonLabel => "button",
            StringContext::Placeholder => "placeholder",
            StringContext::ErrorMessage => "error",
            StringContext::ValidationMessage => "validation",
            StringContext::PageTitle => "title",
            StringContext::Heading => "heading",
            StringContext::AriaLabel => "aria",
            StringContext::UiText => "ui",
            StringContext::LogMessage => "log",
            StringContext::Unknown => "str",
        }
    }
}

/// Errors that can occur during string extraction
#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse file: {0}")]
    ParseError(String),

    #[error("Invalid regex pattern: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
}

/// Trait for language-specific string extractors
#[async_trait]
pub trait Extractor: Send + Sync {
    /// Extract translatable strings from a single file
    async fn extract_file(&self, path: &Path) -> Result<Vec<ExtractedString>, ExtractionError>;

    /// Get supported file extensions (e.g., ["ts", "tsx", "js", "jsx"])
    fn supported_extensions(&self) -> &[&str];

    /// Check if a string should be translated based on content and context
    ///
    /// Returns false for:
    /// - Very short strings (< 3 characters)
    /// - Numeric-only strings
    /// - Technical strings (URLs, file paths, SQL queries)
    /// - Test data
    fn is_translatable(&self, text: &str, context: &StringContext) -> bool {
        // Skip very short strings
        if text.trim().len() < 3 {
            return false;
        }

        // Skip numeric-only strings
        if text.chars().all(|c| c.is_numeric() || c.is_whitespace()) {
            return false;
        }

        // Skip URLs
        if text.starts_with("http://") || text.starts_with("https://") || text.starts_with("//") {
            return false;
        }

        // Skip file paths
        if text.starts_with('/') || text.starts_with("./") || text.starts_with("../") {
            return false;
        }

        // Skip SQL queries
        if text.to_uppercase().starts_with("SELECT ")
            || text.to_uppercase().starts_with("INSERT ")
            || text.to_uppercase().starts_with("UPDATE ")
            || text.to_uppercase().starts_with("DELETE ") {
            return false;
        }

        // Log messages are typically not user-facing
        if matches!(context, StringContext::LogMessage) {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExtractor;

    #[async_trait]
    impl Extractor for MockExtractor {
        async fn extract_file(&self, _path: &Path) -> Result<Vec<ExtractedString>, ExtractionError> {
            Ok(vec![])
        }

        fn supported_extensions(&self) -> &[&str] {
            &["mock"]
        }
    }

    #[test]
    fn test_is_translatable_short_strings() {
        let extractor = MockExtractor;
        assert!(!extractor.is_translatable("OK", &StringContext::ButtonLabel));
        assert!(!extractor.is_translatable("  ", &StringContext::UiText));
    }

    #[test]
    fn test_is_translatable_numeric() {
        let extractor = MockExtractor;
        assert!(!extractor.is_translatable("123", &StringContext::UiText));
        assert!(!extractor.is_translatable("42", &StringContext::UiText));
    }

    #[test]
    fn test_is_translatable_urls() {
        let extractor = MockExtractor;
        assert!(!extractor.is_translatable("https://example.com", &StringContext::UiText));
        assert!(!extractor.is_translatable("http://test.com", &StringContext::UiText));
    }

    #[test]
    fn test_is_translatable_valid() {
        let extractor = MockExtractor;
        assert!(extractor.is_translatable("Click me", &StringContext::ButtonLabel));
        assert!(extractor.is_translatable("Enter your name", &StringContext::Placeholder));
    }

    #[test]
    fn test_context_key_prefix() {
        assert_eq!(StringContext::ButtonLabel.key_prefix(), "button");
        assert_eq!(StringContext::ErrorMessage.key_prefix(), "error");
        assert_eq!(StringContext::Placeholder.key_prefix(), "placeholder");
    }
}
