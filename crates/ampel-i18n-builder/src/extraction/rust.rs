//! Rust string extractor
//!
//! Extracts translatable strings from Rust source files using regex patterns.

use crate::extraction::extractor::{ExtractedString, Extractor, ExtractionError, StringContext};
use async_trait::async_trait;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Rust extractor
pub struct RustExtractor {
    // Regex patterns (compiled once for performance)
    error_macro: Regex,
    format_macro: Regex,
    string_literal: Regex,
    thiserror_attr: Regex,
}

impl RustExtractor {
    pub fn new() -> Self {
        Self {
            // Error macro: anyhow!("message") or bail!("message")
            error_macro: Regex::new(r#"(?:anyhow|bail)!\s*\(\s*["']([^"']+)["']"#).unwrap(),

            // Format macro: format!("template {}", var) or println!("text")
            format_macro: Regex::new(r#"(?:format|println|eprintln|print|eprint)!\s*\(\s*["']([^"']+)["']"#).unwrap(),

            // String literals: "text" or String::from("text")
            string_literal: Regex::new(r#"["']([^"'\n]+)["']"#).unwrap(),

            // thiserror attribute: #[error("message")]
            thiserror_attr: Regex::new(r#"#\[error\s*\(\s*["']([^"']+)["']\s*\)\]"#).unwrap(),
        }
    }

    /// Detect placeholders in format strings
    ///
    /// Example: "Found {} items" → ["0"]
    /// Example: "User {name} logged in" → ["name"]
    fn extract_format_placeholders(&self, format_str: &str) -> Vec<String> {
        let placeholder_pattern = Regex::new(r"\{(\w*)\}").unwrap();
        placeholder_pattern
            .captures_iter(format_str)
            .enumerate()
            .map(|(idx, cap)| {
                if let Some(name_match) = cap.get(1) {
                    let name = name_match.as_str();
                    if name.is_empty() {
                        idx.to_string() // Positional: {}
                    } else {
                        name.to_string() // Named: {name}
                    }
                } else {
                    idx.to_string()
                }
            })
            .collect()
    }

    /// Check if line contains i18n macro (already translated)
    fn is_already_translated(&self, line: &str) -> bool {
        line.contains("t!(") || line.contains("rust_i18n::t")
    }
}

impl Default for RustExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Extractor for RustExtractor {
    async fn extract_file(&self, path: &Path) -> Result<Vec<ExtractedString>, ExtractionError> {
        let content = fs::read_to_string(path)?;
        let mut extracted = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1; // 1-indexed

            // Skip already translated lines
            if self.is_already_translated(line) {
                continue;
            }

            // Skip comments
            if line.trim().starts_with("//") {
                continue;
            }

            // Extract from thiserror attributes
            for cap in self.thiserror_attr.captures_iter(line) {
                if let Some(text_match) = cap.get(1) {
                    let text = text_match.as_str();
                    let variables = self.extract_format_placeholders(text);

                    if self.is_translatable(text, &StringContext::ErrorMessage) {
                        extracted.push(ExtractedString {
                            text: text.to_string(),
                            file: path.to_path_buf(),
                            line: line_num,
                            column: text_match.start(),
                            context: StringContext::ErrorMessage,
                            existing_key: None,
                            variables,
                        });
                    }
                }
            }

            // Extract from error macros (anyhow!, bail!)
            for cap in self.error_macro.captures_iter(line) {
                if let Some(text_match) = cap.get(1) {
                    let text = text_match.as_str();
                    let variables = self.extract_format_placeholders(text);

                    if self.is_translatable(text, &StringContext::ErrorMessage) {
                        extracted.push(ExtractedString {
                            text: text.to_string(),
                            file: path.to_path_buf(),
                            line: line_num,
                            column: text_match.start(),
                            context: StringContext::ErrorMessage,
                            existing_key: None,
                            variables,
                        });
                    }
                }
            }

            // Extract from format macros (format!, but skip println!/eprintln!)
            for cap in self.format_macro.captures_iter(line) {
                if let Some(text_match) = cap.get(1) {
                    let text = text_match.as_str();
                    let variables = self.extract_format_placeholders(text);

                    // Determine context - skip log macros (println, eprintln)
                    let is_log_macro = line.contains("println!") || line.contains("eprintln!") ||
                                       line.contains("print!") || line.contains("eprint!");

                    if !is_log_macro && self.is_translatable(text, &StringContext::UiText) {
                        extracted.push(ExtractedString {
                            text: text.to_string(),
                            file: path.to_path_buf(),
                            line: line_num,
                            column: text_match.start(),
                            context: StringContext::UiText,
                            existing_key: None,
                            variables,
                        });
                    }
                }
            }

            // Extract string literals in specific contexts
            // Only extract from variable assignments, not function calls
            let line_lower = line.to_lowercase();
            let is_assignment = line.contains("let") || line.contains("const");
            let is_error_context = line_lower.contains("error") && !line.contains("Error::") && !line.contains("error!");
            let is_not_log_macro = !line.contains("println!") && !line.contains("eprintln!") && !line.contains("print!") && !line.contains("eprint!");

            if is_assignment && is_error_context && is_not_log_macro {
                for cap in self.string_literal.captures_iter(line) {
                    if let Some(text_match) = cap.get(1) {
                        let text = text_match.as_str().trim();

                        if self.is_translatable(text, &StringContext::ErrorMessage) {
                            extracted.push(ExtractedString {
                                text: text.to_string(),
                                file: path.to_path_buf(),
                                line: line_num,
                                column: text_match.start(),
                                context: StringContext::ErrorMessage,
                                existing_key: None,
                                variables: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        Ok(extracted)
    }

    fn supported_extensions(&self) -> &[&str] {
        &["rs"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    async fn extract_from_content(content: &str) -> Vec<ExtractedString> {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();

        let extractor = RustExtractor::new();
        extractor.extract_file(file.path()).await.unwrap()
    }

    #[tokio::test]
    async fn test_extract_thiserror() {
        let content = r#"
            #[error("User not found")]
            UserNotFound,
            #[error("Invalid credentials: {0}")]
            InvalidCredentials(String),
        "#;

        let extracted = extract_from_content(content).await;
        assert!(extracted.iter().any(|e| e.text == "User not found"));
        assert!(extracted.iter().any(|e| e.text == "Invalid credentials: {0}"));
    }

    #[tokio::test]
    async fn test_extract_error_macros() {
        let content = r#"
            anyhow!("Authentication failed")
            bail!("Connection timeout")
        "#;

        let extracted = extract_from_content(content).await;
        assert!(extracted.iter().any(|e| e.text == "Authentication failed"));
        assert!(extracted.iter().any(|e| e.text == "Connection timeout"));
    }

    // Note: format! extraction is tested via test_format_macro_regex
    // The full extraction test can be added later after more real-world testing

    #[tokio::test]
    async fn test_skip_println() {
        let content = r#"
            println!("Debug message");
            eprintln!("Error log");
        "#;

        let extracted = extract_from_content(content).await;
        // Log messages should be filtered
        assert!(extracted.is_empty());
    }

    #[tokio::test]
    async fn test_skip_translated() {
        let content = r#"
            t!("errors.user_not_found")
            rust_i18n::t("messages.welcome")
        "#;

        let extracted = extract_from_content(content).await;
        assert!(extracted.is_empty());
    }

    #[tokio::test]
    async fn test_extract_error_context() {
        let content = r#"
            let error = "Invalid email address";
        "#;

        let extracted = extract_from_content(content).await;

        let error_msg = extracted.iter().find(|e| e.text == "Invalid email address").unwrap();
        assert_eq!(error_msg.context, StringContext::ErrorMessage);
    }

    #[test]
    fn test_format_placeholder_extraction() {
        let extractor = RustExtractor::new();

        // Positional placeholders
        let vars = extractor.extract_format_placeholders("Found {} items in {} categories");
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0], "0");
        assert_eq!(vars[1], "1");

        // Named placeholders
        let vars = extractor.extract_format_placeholders("User {name} has {count} messages");
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"count".to_string()));
    }

    #[test]
    fn test_format_macro_regex() {
        let extractor = RustExtractor::new();
        let line = r#"let msg = format!("Processing items");"#;

        let captures: Vec<_> = extractor.format_macro.captures_iter(line).collect();
        assert!(!captures.is_empty(), "Regex should match format! call");

        if let Some(cap) = captures.first() {
            let text = cap.get(1).map(|m| m.as_str());
            assert_eq!(text, Some("Processing items"));
        }
    }
}
