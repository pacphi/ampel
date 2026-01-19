//! TypeScript/React string extractor
//!
//! Extracts translatable strings from TypeScript and React (JSX/TSX) files using regex patterns.

use crate::extraction::extractor::{ExtractedString, ExtractionError, Extractor, StringContext};
use async_trait::async_trait;
use regex::Regex;
use std::fs;
use std::path::Path;

/// TypeScript/React extractor
pub struct TypeScriptExtractor {
    // Regex patterns (compiled once for performance)
    jsx_text: Regex,
    jsx_attr: Regex,
    template_string: Regex,
    string_literal: Regex,
}

impl TypeScriptExtractor {
    pub fn new() -> Self {
        Self {
            // JSX text content: <Tag>Text here</Tag>
            jsx_text: Regex::new(r">([^<>{}\n]+)<").unwrap(),

            // JSX attributes: prop="value" or prop='value' (including hyphenated like aria-label)
            jsx_attr: Regex::new(r#"([\w-]+)=["']([^"']+)["']"#).unwrap(),

            // Template strings: `text ${var} more`
            template_string: Regex::new(r"`([^`]+)`").unwrap(),

            // String literals: "text" or 'text'
            string_literal: Regex::new(r#"["']([^"'\n]+)["']"#).unwrap(),
        }
    }

    /// Detect variables in template strings
    ///
    /// Example: `Hello ${userName}!` → ["userName"]
    fn extract_template_variables(&self, template: &str) -> Vec<String> {
        let var_pattern = Regex::new(r"\$\{(\w+)\}").unwrap();
        var_pattern
            .captures_iter(template)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    /// Infer context from JSX attribute name
    fn infer_context_from_attr(&self, attr_name: &str) -> StringContext {
        let lower = attr_name.to_lowercase();
        let lower_str = lower.as_str();

        if lower_str == "aria-label" || lower_str == "arialabel" {
            return StringContext::AriaLabel;
        }

        match lower_str {
            "placeholder" => StringContext::Placeholder,
            "title" => StringContext::PageTitle,
            "label" => StringContext::ButtonLabel,
            _ => StringContext::UiText,
        }
    }

    /// Check if line contains i18n function call (already translated)
    fn is_already_translated(&self, line: &str) -> bool {
        line.contains("t(")
            || line.contains("t(\"")
            || line.contains("t('")
            || line.contains("useTranslation(")
    }
}

impl Default for TypeScriptExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Extractor for TypeScriptExtractor {
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
            if line.trim().starts_with("//")
                || line.trim().starts_with("/*")
                || line.trim().starts_with('*')
            {
                continue;
            }

            // Extract JSX text content
            for cap in self.jsx_text.captures_iter(line) {
                if let Some(text_match) = cap.get(1) {
                    let text = text_match.as_str().trim();

                    if self.is_translatable(text, &StringContext::UiText) {
                        extracted.push(ExtractedString {
                            text: text.to_string(),
                            file: path.to_path_buf(),
                            line: line_num,
                            column: text_match.start(),
                            context: StringContext::UiText,
                            existing_key: None,
                            variables: Vec::new(),
                        });
                    }
                }
            }

            // Extract JSX attributes
            for cap in self.jsx_attr.captures_iter(line) {
                if let (Some(attr_name), Some(attr_value)) = (cap.get(1), cap.get(2)) {
                    let text = attr_value.as_str().trim();
                    let context = self.infer_context_from_attr(attr_name.as_str());

                    if self.is_translatable(text, &context) {
                        extracted.push(ExtractedString {
                            text: text.to_string(),
                            file: path.to_path_buf(),
                            line: line_num,
                            column: attr_value.start(),
                            context,
                            existing_key: None,
                            variables: Vec::new(),
                        });
                    }
                }
            }

            // Extract template strings
            for cap in self.template_string.captures_iter(line) {
                if let Some(template_match) = cap.get(1) {
                    let template = template_match.as_str();
                    let variables = self.extract_template_variables(template);

                    // Only extract if it contains variables (otherwise it's a simple string)
                    if !variables.is_empty()
                        && self.is_translatable(template, &StringContext::UiText)
                    {
                        extracted.push(ExtractedString {
                            text: template.to_string(),
                            file: path.to_path_buf(),
                            line: line_num,
                            column: template_match.start(),
                            context: StringContext::UiText,
                            existing_key: None,
                            variables,
                        });
                    }
                }
            }

            // Extract string literals (only in specific contexts to avoid false positives)
            // Look for patterns like: const error = "Message"
            let line_lower = line.to_lowercase();
            if line_lower.contains("error")
                || line_lower.contains("message")
                || line_lower.contains("label")
                || line_lower.contains("validation")
            {
                for cap in self.string_literal.captures_iter(line) {
                    if let Some(text_match) = cap.get(1) {
                        let text = text_match.as_str().trim();

                        // Infer context from surrounding code
                        let context = if line_lower.contains("error") {
                            StringContext::ErrorMessage
                        } else if line_lower.contains("validation") {
                            StringContext::ValidationMessage
                        } else {
                            StringContext::UiText
                        };

                        if self.is_translatable(text, &context) {
                            extracted.push(ExtractedString {
                                text: text.to_string(),
                                file: path.to_path_buf(),
                                line: line_num,
                                column: text_match.start(),
                                context,
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
        &["ts", "tsx", "js", "jsx"]
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

        let extractor = TypeScriptExtractor::new();
        extractor.extract_file(file.path()).await.unwrap()
    }

    #[tokio::test]
    async fn test_extract_jsx_text() {
        let content = r#"
            <Button>Click me</Button>
            <div>Hello world</div>
        "#;

        let extracted = extract_from_content(content).await;
        assert!(extracted.iter().any(|e| e.text == "Click me"));
        assert!(extracted.iter().any(|e| e.text == "Hello world"));
    }

    #[tokio::test]
    async fn test_extract_jsx_attributes() {
        let content = r#"
            <Input placeholder="Enter your name" />
            <Button aria-label="Close dialog" />
        "#;

        let extracted = extract_from_content(content).await;

        let placeholder = extracted
            .iter()
            .find(|e| e.text == "Enter your name")
            .unwrap();
        assert_eq!(placeholder.context, StringContext::Placeholder);

        let aria = extracted.iter().find(|e| e.text == "Close dialog").unwrap();
        assert_eq!(aria.context, StringContext::AriaLabel);
    }

    #[tokio::test]
    async fn test_extract_template_strings() {
        let content = r#"
            const msg = `Welcome, ${userName}!`;
        "#;

        let extracted = extract_from_content(content).await;

        let template = extracted
            .iter()
            .find(|e| e.text.contains("Welcome"))
            .unwrap();
        assert!(template.variables.contains(&"userName".to_string()));
    }

    #[tokio::test]
    async fn test_skip_translated() {
        let content = r#"
            <Button>{t('button.save')}</Button>
            const { t } = useTranslation('common');
        "#;

        let extracted = extract_from_content(content).await;
        assert!(extracted.is_empty());
    }

    #[tokio::test]
    async fn test_context_inference() {
        let content = r#"
            const error = "Invalid email address";
            const validation = "Password too short";
        "#;

        let extracted = extract_from_content(content).await;

        let error_msg = extracted
            .iter()
            .find(|e| e.text == "Invalid email address")
            .unwrap();
        assert_eq!(error_msg.context, StringContext::ErrorMessage);

        let validation_msg = extracted
            .iter()
            .find(|e| e.text == "Password too short")
            .unwrap();
        assert_eq!(validation_msg.context, StringContext::ValidationMessage);
    }

    #[tokio::test]
    async fn test_filter_short_strings() {
        let content = r#"
            <Button>OK</Button>
            <Button>Save Changes</Button>
        "#;

        let extracted = extract_from_content(content).await;

        // "OK" should be filtered (too short)
        assert!(!extracted.iter().any(|e| e.text == "OK"));
        // "Save Changes" should be extracted
        assert!(extracted.iter().any(|e| e.text == "Save Changes"));
    }

    #[test]
    fn test_template_variable_extraction() {
        let extractor = TypeScriptExtractor::new();
        let vars = extractor
            .extract_template_variables("Welcome, ${userName}! You have ${count} messages.");

        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"userName".to_string()));
        assert!(vars.contains(&"count".to_string()));
    }
}
