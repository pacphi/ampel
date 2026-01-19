//! Key generation strategies for translation keys

use crate::extraction::extractor::ExtractedString;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// Strategy for generating translation keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStrategy {
    /// Generate semantic keys based on context and text (e.g., "button.clickMe")
    Semantic,

    /// Generate hash-based keys (e.g., "str_a3f2b1c4")
    Hash,

    /// Generate incremental keys (e.g., "str_001", "str_002")
    Incremental,
}

/// Trait for generating translation keys
pub trait KeyGenerator {
    /// Generate a unique key for an extracted string
    ///
    /// The generator should ensure uniqueness by checking against existing_keys
    fn generate_key(
        &mut self,
        extracted: &ExtractedString,
        existing_keys: &HashSet<String>,
    ) -> String;
}

/// Semantic key generator
///
/// Generates keys like: "button.clickMe", "error.invalidEmail"
pub struct SemanticKeyGenerator {
    counter: std::collections::HashMap<String, usize>,
}

impl SemanticKeyGenerator {
    pub fn new() -> Self {
        Self {
            counter: std::collections::HashMap::new(),
        }
    }

    /// Sanitize text for use in a key
    ///
    /// - Convert to camelCase
    /// - Remove special characters
    /// - Ensure it starts with a letter
    fn sanitize_text(text: &str) -> String {
        let words: Vec<String> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_lowercase().chain(chars).collect(),
                }
            })
            .collect();

        if words.is_empty() {
            return "unknown".to_string();
        }

        // CamelCase: first word lowercase, rest capitalized
        let mut result = words[0].to_lowercase();
        for word in &words[1..] {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push_str(&first.to_uppercase().to_string());
                result.push_str(&chars.as_str().to_lowercase());
            }
        }

        // Ensure it starts with a letter
        if !result.chars().next().is_some_and(|c| c.is_alphabetic()) {
            result.insert(0, 'x');
        }

        // Truncate if too long
        if result.len() > 50 {
            result.truncate(50);
        }

        result
    }
}

impl Default for SemanticKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyGenerator for SemanticKeyGenerator {
    fn generate_key(
        &mut self,
        extracted: &ExtractedString,
        existing_keys: &HashSet<String>,
    ) -> String {
        let prefix = extracted.context.key_prefix();
        let sanitized = Self::sanitize_text(&extracted.text);
        let base_key = format!("{}.{}", prefix, sanitized);

        // Check for uniqueness
        if !existing_keys.contains(&base_key) {
            return base_key;
        }

        // Generate numbered variant
        let entry = self.counter.entry(base_key.clone()).or_insert(1);
        loop {
            *entry += 1;
            let numbered_key = format!("{}_{}", base_key, entry);
            if !existing_keys.contains(&numbered_key) {
                return numbered_key;
            }
        }
    }
}

/// Hash-based key generator
///
/// Generates keys like: "str_a3f2b1c4"
pub struct HashKeyGenerator;

impl HashKeyGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HashKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyGenerator for HashKeyGenerator {
    fn generate_key(
        &mut self,
        extracted: &ExtractedString,
        _existing_keys: &HashSet<String>,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(extracted.text.as_bytes());
        let result = hasher.finalize();

        // Use first 8 hex characters
        let hex = format!("{:x}", result);
        format!("str_{}", &hex[..8])
    }
}

/// Incremental key generator
///
/// Generates keys like: "str_001", "str_002"
pub struct IncrementalKeyGenerator {
    counter: usize,
}

impl IncrementalKeyGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn with_start(start: usize) -> Self {
        Self { counter: start }
    }
}

impl Default for IncrementalKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyGenerator for IncrementalKeyGenerator {
    fn generate_key(
        &mut self,
        _extracted: &ExtractedString,
        existing_keys: &HashSet<String>,
    ) -> String {
        loop {
            self.counter += 1;
            let key = format!("str_{:03}", self.counter);
            if !existing_keys.contains(&key) {
                return key;
            }
        }
    }
}

/// Create a key generator based on strategy
pub fn create_generator(strategy: KeyStrategy) -> Box<dyn KeyGenerator> {
    match strategy {
        KeyStrategy::Semantic => Box::new(SemanticKeyGenerator::new()),
        KeyStrategy::Hash => Box::new(HashKeyGenerator::new()),
        KeyStrategy::Incremental => Box::new(IncrementalKeyGenerator::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extraction::extractor::StringContext;
    use std::path::PathBuf;

    fn make_extracted(text: &str, context: StringContext) -> ExtractedString {
        ExtractedString {
            text: text.to_string(),
            file: PathBuf::from("test.tsx"),
            line: 1,
            column: 1,
            context,
            existing_key: None,
            variables: vec![],
        }
    }

    #[test]
    fn test_semantic_sanitize() {
        assert_eq!(SemanticKeyGenerator::sanitize_text("Click me!"), "clickMe");
        assert_eq!(
            SemanticKeyGenerator::sanitize_text("Save Changes"),
            "saveChanges"
        );
        assert_eq!(
            SemanticKeyGenerator::sanitize_text("Enter your name"),
            "enterYourName"
        );
    }

    #[test]
    fn test_semantic_generator() {
        let mut gen = SemanticKeyGenerator::new();
        let existing = HashSet::new();

        let extracted = make_extracted("Click me", StringContext::ButtonLabel);
        let key = gen.generate_key(&extracted, &existing);
        assert_eq!(key, "button.clickMe");
    }

    #[test]
    fn test_semantic_generator_duplicate() {
        let mut gen = SemanticKeyGenerator::new();
        let mut existing = HashSet::new();

        let extracted = make_extracted("Click me", StringContext::ButtonLabel);
        let key1 = gen.generate_key(&extracted, &existing);
        existing.insert(key1.clone());

        let key2 = gen.generate_key(&extracted, &existing);
        assert_ne!(key1, key2);
        assert!(key2.starts_with("button.clickMe_"));
    }

    #[test]
    fn test_hash_generator() {
        let mut gen = HashKeyGenerator::new();
        let existing = HashSet::new();

        let extracted = make_extracted("Click me", StringContext::ButtonLabel);
        let key = gen.generate_key(&extracted, &existing);
        assert!(key.starts_with("str_"));
        assert_eq!(key.len(), 12); // "str_" + 8 hex chars
    }

    #[test]
    fn test_hash_generator_deterministic() {
        let mut gen1 = HashKeyGenerator::new();
        let mut gen2 = HashKeyGenerator::new();
        let existing = HashSet::new();

        let extracted = make_extracted("Click me", StringContext::ButtonLabel);
        let key1 = gen1.generate_key(&extracted, &existing);
        let key2 = gen2.generate_key(&extracted, &existing);
        assert_eq!(key1, key2); // Same text should generate same hash
    }

    #[test]
    fn test_incremental_generator() {
        let mut gen = IncrementalKeyGenerator::new();
        let existing = HashSet::new();

        let extracted = make_extracted("Click me", StringContext::ButtonLabel);
        let key = gen.generate_key(&extracted, &existing);
        assert_eq!(key, "str_001");
    }

    #[test]
    fn test_incremental_generator_sequence() {
        let mut gen = IncrementalKeyGenerator::new();
        let existing = HashSet::new();

        let extracted1 = make_extracted("Click me", StringContext::ButtonLabel);
        let key1 = gen.generate_key(&extracted1, &existing);

        let extracted2 = make_extracted("Save", StringContext::ButtonLabel);
        let key2 = gen.generate_key(&extracted2, &existing);

        assert_eq!(key1, "str_001");
        assert_eq!(key2, "str_002");
    }
}
