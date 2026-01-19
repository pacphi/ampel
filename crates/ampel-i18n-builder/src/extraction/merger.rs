//! Merge extracted strings with existing translations

use crate::extraction::extractor::ExtractedString;
use crate::extraction::key_generator::{create_generator, KeyStrategy};
use crate::formats::{TranslationMap, TranslationValue};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Report of merge operation
#[derive(Debug, Clone, PartialEq)]
pub struct MergeReport {
    /// Number of new keys added
    pub added: usize,

    /// Number of existing keys skipped
    pub skipped: usize,

    /// Keys that had conflicts (same key, different value)
    pub conflicts: Vec<String>,

    /// Map of generated keys to extracted strings
    pub key_mapping: HashMap<String, ExtractedString>,
}

/// Errors that can occur during merging
#[derive(Debug, Error)]
pub enum MergeError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Merge conflict: {0}")]
    Conflict(String),
}

/// Merges extracted strings with existing translations
pub struct Merger {
    strategy: KeyStrategy,
}

impl Merger {
    pub fn new(strategy: KeyStrategy) -> Self {
        Self { strategy }
    }

    /// Merge extracted strings into an existing translation map
    ///
    /// Returns the merged map and a report
    pub fn merge(
        &self,
        existing: &TranslationMap,
        extracted: Vec<ExtractedString>,
    ) -> Result<(TranslationMap, MergeReport), MergeError> {
        let mut merged = existing.clone();
        let mut generator = create_generator(self.strategy);
        let existing_keys = self.collect_keys(existing);

        let mut report = MergeReport {
            added: 0,
            skipped: 0,
            conflicts: Vec::new(),
            key_mapping: HashMap::new(),
        };

        let mut current_keys = existing_keys.clone();

        for extracted_str in extracted {
            // Check if string already exists in translations
            if self.string_exists_in_map(existing, &extracted_str.text) {
                report.skipped += 1;
                continue;
            }

            // Generate a unique key
            let key = generator.generate_key(&extracted_str, &current_keys);
            current_keys.insert(key.clone());

            // Insert into map (supports dot-notation nesting)
            self.insert_key(&mut merged, &key, &extracted_str.text)?;

            report.added += 1;
            report.key_mapping.insert(key, extracted_str);
        }

        Ok((merged, report))
    }

    /// Collect all keys from a translation map (flattened with dot-notation)
    fn collect_keys(&self, map: &TranslationMap) -> HashSet<String> {
        let mut keys = HashSet::new();
        self.collect_keys_recursive(map, "", &mut keys);
        keys
    }

    fn collect_keys_recursive(
        &self,
        map: &TranslationMap,
        prefix: &str,
        keys: &mut HashSet<String>,
    ) {
        for (key, value) in map {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            match value {
                TranslationValue::String(_) | TranslationValue::Plural(_) => {
                    keys.insert(full_key);
                }
                TranslationValue::Nested(nested) => {
                    self.collect_keys_recursive(nested, &full_key, keys);
                }
            }
        }
    }

    /// Check if a string value exists anywhere in the translation map
    fn string_exists_in_map(&self, map: &TranslationMap, text: &str) -> bool {
        for value in map.values() {
            match value {
                TranslationValue::String(s) => {
                    if s == text {
                        return true;
                    }
                }
                TranslationValue::Plural(forms) => {
                    if forms.other == text
                        || forms.one.as_ref() == Some(&text.to_string())
                        || forms.zero.as_ref() == Some(&text.to_string())
                        || forms.two.as_ref() == Some(&text.to_string())
                        || forms.few.as_ref() == Some(&text.to_string())
                        || forms.many.as_ref() == Some(&text.to_string())
                    {
                        return true;
                    }
                }
                TranslationValue::Nested(nested) => {
                    if self.string_exists_in_map(nested, text) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Insert a key with dot-notation into the map
    ///
    /// Example: "button.clickMe" → map["button"]["clickMe"]
    fn insert_key(
        &self,
        map: &mut TranslationMap,
        key: &str,
        value: &str,
    ) -> Result<(), MergeError> {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.is_empty() {
            return Err(MergeError::InvalidKey(key.to_string()));
        }

        let mut current = map;
        let last_idx = parts.len() - 1;

        for (idx, &part) in parts.iter().enumerate() {
            if idx == last_idx {
                // Last part - insert the value
                current.insert(
                    part.to_string(),
                    TranslationValue::String(value.to_string()),
                );
            } else {
                // Intermediate part - ensure nested map exists
                current = match current
                    .entry(part.to_string())
                    .or_insert_with(|| TranslationValue::Nested(std::collections::BTreeMap::new()))
                {
                    TranslationValue::Nested(nested) => nested,
                    _ => {
                        return Err(MergeError::Conflict(format!(
                            "Key conflict: '{}' is both a leaf and a nested key",
                            parts[..=idx].join(".")
                        )))
                    }
                };
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extraction::extractor::StringContext;
    use std::path::PathBuf;

    fn make_extracted(text: &str) -> ExtractedString {
        ExtractedString {
            text: text.to_string(),
            file: PathBuf::from("test.tsx"),
            line: 1,
            column: 1,
            context: StringContext::ButtonLabel,
            existing_key: None,
            variables: vec![],
        }
    }

    #[test]
    fn test_merge_empty_map() {
        let merger = Merger::new(KeyStrategy::Semantic);
        let existing = TranslationMap::new();
        let extracted = vec![make_extracted("Click me")];

        let (merged, report) = merger.merge(&existing, extracted).unwrap();

        assert_eq!(report.added, 1);
        assert_eq!(report.skipped, 0);
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_merge_skip_existing() {
        let merger = Merger::new(KeyStrategy::Semantic);
        let mut existing = TranslationMap::new();
        existing.insert(
            "button".to_string(),
            TranslationValue::Nested({
                let mut nested = std::collections::BTreeMap::new();
                nested.insert(
                    "clickMe".to_string(),
                    TranslationValue::String("Click me".to_string()),
                );
                nested
            }),
        );

        let extracted = vec![make_extracted("Click me")];

        let (_merged, report) = merger.merge(&existing, extracted).unwrap();

        assert_eq!(report.added, 0);
        assert_eq!(report.skipped, 1); // Already exists
    }

    #[test]
    fn test_merge_multiple_strings() {
        let merger = Merger::new(KeyStrategy::Semantic);
        let existing = TranslationMap::new();
        let extracted = vec![
            make_extracted("Click me"),
            make_extracted("Save"),
            make_extracted("Cancel"),
        ];

        let (merged, report) = merger.merge(&existing, extracted).unwrap();

        assert_eq!(report.added, 3);
        assert_eq!(report.skipped, 0);
        assert!(merged.contains_key("button"));
    }

    #[test]
    fn test_insert_key_nested() {
        let merger = Merger::new(KeyStrategy::Semantic);
        let mut map = TranslationMap::new();

        merger
            .insert_key(&mut map, "button.clickMe", "Click me")
            .unwrap();

        assert!(map.contains_key("button"));
        if let Some(TranslationValue::Nested(nested)) = map.get("button") {
            assert!(nested.contains_key("clickMe"));
        } else {
            panic!("Expected nested map");
        }
    }

    #[test]
    fn test_collect_keys() {
        let merger = Merger::new(KeyStrategy::Semantic);
        let mut map = TranslationMap::new();
        map.insert(
            "simple".to_string(),
            TranslationValue::String("value".to_string()),
        );

        let mut nested = std::collections::BTreeMap::new();
        nested.insert(
            "inner".to_string(),
            TranslationValue::String("nested value".to_string()),
        );
        map.insert("outer".to_string(), TranslationValue::Nested(nested));

        let keys = merger.collect_keys(&map);

        assert!(keys.contains("simple"));
        assert!(keys.contains("outer.inner"));
        assert_eq!(keys.len(), 2);
    }
}
