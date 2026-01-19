//! Java .properties format parser and writer
//!
//! Implements the Java .properties file format specification with support for:
//! - Escaping: \n, \t, \\, \=, \:, \uXXXX
//! - Line continuations (trailing \)
//! - Comments (# and !)
//! - Unicode support
//! - Flat-to-nested structure conversion
//! - Plural form detection and consolidation

use super::{FormatError, PluralForms, TranslationFormat, TranslationMap, TranslationValue};
use std::collections::BTreeMap;

/// Java .properties format parser
pub struct PropertiesFormat;

impl PropertiesFormat {
    pub fn new() -> Self {
        Self
    }

    /// Parse a single line of .properties file
    ///
    /// Returns None for comments and empty lines
    /// Returns Some((key, value)) for valid property lines
    fn parse_line(line: &str) -> Option<(String, String)> {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            return None;
        }

        // Find the separator (= or :)
        // Must handle escaped separators: \= and \:
        let mut key_end = None;
        let mut prev_char = '\0';
        for (i, ch) in trimmed.chars().enumerate() {
            if (ch == '=' || ch == ':') && prev_char != '\\' {
                key_end = Some(i);
                break;
            }
            prev_char = ch;
        }

        if let Some(sep_pos) = key_end {
            let key = trimmed[..sep_pos].trim();
            let value = trimmed[sep_pos + 1..].trim();

            // Unescape key and value
            let unescaped_key = Self::unescape_value(key);
            let unescaped_value = Self::unescape_value(value);

            Some((unescaped_key, unescaped_value))
        } else {
            // No separator found, invalid line
            None
        }
    }

    /// Unescape Java .properties value
    ///
    /// Handles: \n, \t, \\, \=, \:, \uXXXX
    fn unescape_value(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.peek() {
                    Some(&'n') => {
                        chars.next();
                        result.push('\n');
                    }
                    Some(&'t') => {
                        chars.next();
                        result.push('\t');
                    }
                    Some(&'r') => {
                        chars.next();
                        result.push('\r');
                    }
                    Some(&'\\') => {
                        chars.next();
                        result.push('\\');
                    }
                    Some(&'=') => {
                        chars.next();
                        result.push('=');
                    }
                    Some(&':') => {
                        chars.next();
                        result.push(':');
                    }
                    Some(&'u') => {
                        // Unicode escape: \uXXXX
                        chars.next(); // consume 'u'
                        let mut hex = String::new();
                        for _ in 0..4 {
                            if let Some(&hex_digit) = chars.peek() {
                                if hex_digit.is_ascii_hexdigit() {
                                    hex.push(hex_digit);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }

                        if hex.len() == 4 {
                            if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                                if let Some(unicode_char) = char::from_u32(code_point) {
                                    result.push(unicode_char);
                                } else {
                                    // Invalid unicode, keep as-is
                                    result.push_str(&format!("\\u{}", hex));
                                }
                            } else {
                                // Invalid hex, keep as-is
                                result.push_str(&format!("\\u{}", hex));
                            }
                        } else {
                            // Incomplete unicode escape
                            result.push_str(&format!("\\u{}", hex));
                        }
                    }
                    Some(_) => {
                        // Unknown escape, keep backslash
                        result.push(ch);
                    }
                    None => {
                        // Trailing backslash (line continuation handled separately)
                        result.push(ch);
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Escape value for writing to .properties file
    fn escape_value(s: &str) -> String {
        let mut result = String::with_capacity(s.len() * 2);

        for ch in s.chars() {
            match ch {
                '\n' => result.push_str("\\n"),
                '\t' => result.push_str("\\t"),
                '\r' => result.push_str("\\r"),
                '\\' => result.push_str("\\\\"),
                '=' => result.push_str("\\="),
                ':' => result.push_str("\\:"),
                _ => result.push(ch),
            }
        }

        result
    }

    /// Convert flat .properties keys to nested TranslationMap
    ///
    /// Example: "dashboard.title" → {"dashboard": {"title": "..."}}
    fn flatten_to_nested(flat: BTreeMap<String, String>) -> Result<TranslationMap, FormatError> {
        let mut root: TranslationMap = BTreeMap::new();

        for (key, value) in flat {
            let parts: Vec<&str> = key.split('.').collect();

            if parts.is_empty() {
                continue;
            }

            // Navigate to the correct nesting level
            let mut current = &mut root;
            let last_idx = parts.len() - 1;

            for (idx, &part) in parts.iter().enumerate() {
                if idx == last_idx {
                    // Last part - insert the value
                    current.insert(part.to_string(), TranslationValue::String(value.clone()));
                } else {
                    // Intermediate part - ensure nested map exists
                    current = match current
                        .entry(part.to_string())
                        .or_insert_with(|| TranslationValue::Nested(BTreeMap::new()))
                    {
                        TranslationValue::Nested(nested) => nested,
                        _ => {
                            return Err(FormatError::SchemaError(format!(
                                "Key conflict: '{}' is both a leaf and a nested key",
                                parts[..=idx].join(".")
                            )))
                        }
                    };
                }
            }
        }

        // Detect and consolidate plural forms
        Self::detect_plurals(&mut root);

        Ok(root)
    }

    /// Detect and consolidate plural forms
    ///
    /// Works with nested structure - looks for nested maps containing "one", "other", etc. as direct children
    fn detect_plurals(map: &mut TranslationMap) {
        let keys: Vec<String> = map.keys().cloned().collect();

        for key in &keys {
            if let Some(TranslationValue::Nested(nested_map)) = map.get_mut(key) {
                // Check if this nested map is a plural form container
                // (contains "other" key which is required for plurals)
                if nested_map.contains_key("other") {
                    // Check if there are other plural keys
                    let has_plural_keys = nested_map.contains_key("one")
                        || nested_map.contains_key("zero")
                        || nested_map.contains_key("two")
                        || nested_map.contains_key("few")
                        || nested_map.contains_key("many");

                    if has_plural_keys {
                        // Extract plural forms
                        let one = nested_map
                            .remove("one")
                            .and_then(|v| if let TranslationValue::String(s) = v { Some(s) } else { None });
                        let zero = nested_map
                            .remove("zero")
                            .and_then(|v| if let TranslationValue::String(s) = v { Some(s) } else { None });
                        let two = nested_map
                            .remove("two")
                            .and_then(|v| if let TranslationValue::String(s) = v { Some(s) } else { None });
                        let few = nested_map
                            .remove("few")
                            .and_then(|v| if let TranslationValue::String(s) = v { Some(s) } else { None });
                        let many = nested_map
                            .remove("many")
                            .and_then(|v| if let TranslationValue::String(s) = v { Some(s) } else { None });
                        let other = nested_map
                            .remove("other")
                            .and_then(|v| if let TranslationValue::String(s) = v { Some(s) } else { None })
                            .expect("other key should exist");

                        // If nested_map is now empty, replace the whole nested value with PluralForms
                        if nested_map.is_empty() {
                            let plural_forms = PluralForms {
                                zero,
                                one,
                                two,
                                few,
                                many,
                                other,
                            };
                            map.insert(key.clone(), TranslationValue::Plural(plural_forms));
                        }
                    } else {
                        // Recursively process this nested map
                        Self::detect_plurals(nested_map);
                    }
                } else {
                    // No "other" key, just recursively process
                    Self::detect_plurals(nested_map);
                }
            }
        }
    }

    /// Convert nested TranslationMap to flat .properties format
    fn nested_to_flat(map: &TranslationMap) -> BTreeMap<String, String> {
        let mut flat = BTreeMap::new();
        Self::flatten_recursive(map, "", &mut flat);
        flat
    }

    /// Helper function for recursive flattening
    fn flatten_recursive(map: &TranslationMap, prefix: &str, result: &mut BTreeMap<String, String>) {
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
                    // Expand plural forms
                    if let Some(ref zero) = forms.zero {
                        result.insert(format!("{}.zero", full_key), zero.clone());
                    }
                    if let Some(ref one) = forms.one {
                        result.insert(format!("{}.one", full_key), one.clone());
                    }
                    if let Some(ref two) = forms.two {
                        result.insert(format!("{}.two", full_key), two.clone());
                    }
                    if let Some(ref few) = forms.few {
                        result.insert(format!("{}.few", full_key), few.clone());
                    }
                    if let Some(ref many) = forms.many {
                        result.insert(format!("{}.many", full_key), many.clone());
                    }
                    result.insert(format!("{}.other", full_key), forms.other.clone());
                }
                TranslationValue::Nested(nested) => {
                    Self::flatten_recursive(nested, &full_key, result);
                }
            }
        }
    }
}

impl TranslationFormat for PropertiesFormat {
    fn parse(&self, content: &str) -> Result<TranslationMap, FormatError> {
        let mut flat_map = BTreeMap::new();
        let mut current_line = String::new();

        for line in content.lines() {
            let trimmed = line.trim_end();

            // Check for line continuation (trailing \)
            if trimmed.ends_with('\\') && !trimmed.ends_with("\\\\") {
                // Remove trailing \ and accumulate
                current_line.push_str(&trimmed[..trimmed.len() - 1]);
                continue;
            } else {
                // Complete line
                current_line.push_str(trimmed);

                if let Some((key, value)) = Self::parse_line(&current_line) {
                    flat_map.insert(key, value);
                }

                current_line.clear();
            }
        }

        // Convert flat map to nested structure
        Self::flatten_to_nested(flat_map)
    }

    fn write(&self, map: &TranslationMap) -> Result<String, FormatError> {
        let flat = Self::nested_to_flat(map);
        let mut result = String::new();

        result.push_str("# Generated by ampel-i18n-builder\n");
        result.push_str("# Java .properties format\n\n");

        for (key, value) in flat {
            let escaped_value = Self::escape_value(&value);
            result.push_str(&format!("{}={}\n", key, escaped_value));
        }

        Ok(result)
    }

    fn validate(&self, content: &str) -> Result<(), FormatError> {
        self.parse(content)?;
        Ok(())
    }
}

impl Default for PropertiesFormat {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input = "key=value\nfoo=bar";
        let format = PropertiesFormat::new();
        let result = format.parse(input).unwrap();

        assert!(result.contains_key("key"));
        assert!(result.contains_key("foo"));
    }

    #[test]
    fn test_parse_with_comments() {
        let input = "# Comment\nkey=value\n! Another comment\nfoo=bar";
        let format = PropertiesFormat::new();
        let result = format.parse(input).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_unescape() {
        assert_eq!(PropertiesFormat::unescape_value("Hello\\nWorld"), "Hello\nWorld");
        assert_eq!(PropertiesFormat::unescape_value("Tab\\there"), "Tab\there");
        assert_eq!(PropertiesFormat::unescape_value("Back\\\\slash"), "Back\\slash");
        assert_eq!(PropertiesFormat::unescape_value("Equal\\=sign"), "Equal=sign");
    }

    #[test]
    fn test_unicode_escape() {
        assert_eq!(PropertiesFormat::unescape_value("\\u0041"), "A");
        assert_eq!(PropertiesFormat::unescape_value("\\u00e9"), "é");
    }

    #[test]
    fn test_flat_to_nested() {
        let mut flat = BTreeMap::new();
        flat.insert("dashboard.title".to_string(), "Dashboard".to_string());
        flat.insert("dashboard.subtitle".to_string(), "Subtitle".to_string());

        let nested = PropertiesFormat::flatten_to_nested(flat).unwrap();

        assert!(nested.contains_key("dashboard"));
        if let Some(TranslationValue::Nested(dash)) = nested.get("dashboard") {
            assert!(dash.contains_key("title"));
            assert!(dash.contains_key("subtitle"));
        } else {
            panic!("Expected nested dashboard");
        }
    }

    #[test]
    fn test_plural_detection() {
        let mut flat = BTreeMap::new();
        flat.insert("items.one".to_string(), "1 item".to_string());
        flat.insert("items.other".to_string(), "{count} items".to_string());

        let nested = PropertiesFormat::flatten_to_nested(flat).unwrap();

        assert!(nested.contains_key("items"));
        if let Some(TranslationValue::Plural(forms)) = nested.get("items") {
            assert_eq!(forms.one.as_ref().unwrap(), "1 item");
            assert_eq!(forms.other, "{count} items");
        } else {
            panic!("Expected plural forms");
        }
    }

    #[test]
    fn test_roundtrip() {
        let input = "dashboard.title=Main Dashboard\ndashboard.subtitle=View your data";
        let format = PropertiesFormat::new();

        let parsed = format.parse(input).unwrap();
        let written = format.write(&parsed).unwrap();
        let reparsed = format.parse(&written).unwrap();

        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn test_line_continuation() {
        let input = "long.key=This is a very \\\nlong value that \\\nspans multiple lines";
        let format = PropertiesFormat::new();
        let result = format.parse(input).unwrap();

        if let Some(TranslationValue::String(val)) = result.get("long").and_then(|v| {
            if let TranslationValue::Nested(n) = v {
                n.get("key")
            } else {
                None
            }
        }) {
            assert!(val.contains("spans multiple lines"));
        }
    }
}
