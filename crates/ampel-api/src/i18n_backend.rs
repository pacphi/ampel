//! Custom rust-i18n backend that supports nested YAML files in subdirectories.
//!
//! This backend reads locale files from `locales/{lang}/*.yml` and flattens
//! nested YAML keys into dot-notation (e.g., `errors.auth.invalid_credentials`).

use rust_i18n::Backend;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Backend that loads nested YAML from language subdirectories.
///
/// File structure expected:
/// ```text
/// locales/
/// ├── en/
/// │   ├── common.yml
/// │   ├── errors.yml
/// │   └── ...
/// ├── de/
/// │   ├── common.yml
/// │   └── ...
/// ```
pub struct NestedYamlBackend {
    /// Translations: locale -> (flattened_key -> value)
    translations: HashMap<String, HashMap<String, String>>,
}

impl NestedYamlBackend {
    /// Create a new backend by loading all locale files from the given directory.
    pub fn new<P: AsRef<Path>>(locales_dir: P) -> Self {
        let mut translations = HashMap::new();
        let locales_path = locales_dir.as_ref();

        if let Ok(entries) = fs::read_dir(locales_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(locale) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip hidden directories
                        if locale.starts_with('.') {
                            continue;
                        }
                        let locale_translations = Self::load_locale_dir(&path);
                        if !locale_translations.is_empty() {
                            translations.insert(locale.to_string(), locale_translations);
                        }
                    }
                }
            }
        }

        Self { translations }
    }

    /// Load all YAML files from a locale directory and merge them.
    fn load_locale_dir(dir: &Path) -> HashMap<String, String> {
        let mut translations = HashMap::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "yml" || ext == "yaml") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(yaml) = serde_yaml::from_str::<Value>(&content) {
                            Self::flatten_yaml(&yaml, String::new(), &mut translations);
                        }
                    }
                }
            }
        }

        translations
    }

    /// Recursively flatten nested YAML into dot-notation keys.
    fn flatten_yaml(value: &Value, prefix: String, result: &mut HashMap<String, String>) {
        match value {
            Value::Mapping(map) => {
                for (k, v) in map {
                    if let Value::String(key) = k {
                        // Skip YAML comments (keys starting with #)
                        if key.starts_with('#') {
                            continue;
                        }
                        let new_prefix = if prefix.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", prefix, key)
                        };
                        Self::flatten_yaml(v, new_prefix, result);
                    }
                }
            }
            Value::String(s) => {
                if !prefix.is_empty() {
                    result.insert(prefix, s.clone());
                }
            }
            // Handle other scalar types
            Value::Number(n) => {
                if !prefix.is_empty() {
                    result.insert(prefix, n.to_string());
                }
            }
            Value::Bool(b) => {
                if !prefix.is_empty() {
                    result.insert(prefix, b.to_string());
                }
            }
            _ => {}
        }
    }
}

impl Backend for NestedYamlBackend {
    fn available_locales(&self) -> Vec<&str> {
        self.translations.keys().map(|k| k.as_str()).collect()
    }

    fn translate(&self, locale: &str, key: &str) -> Option<&str> {
        self.translations
            .get(locale)
            .and_then(|t| t.get(key))
            .map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_simple() {
        let yaml: Value = serde_yaml::from_str(
            r#"
            errors:
              auth:
                invalid: "Invalid credentials"
            "#,
        )
        .unwrap();

        let mut result = HashMap::new();
        NestedYamlBackend::flatten_yaml(&yaml, String::new(), &mut result);

        assert_eq!(
            result.get("errors.auth.invalid"),
            Some(&"Invalid credentials".to_string())
        );
    }

    #[test]
    fn test_flatten_multiple_levels() {
        let yaml: Value = serde_yaml::from_str(
            r#"
            app:
              name: "Test App"
              settings:
                theme: "dark"
            "#,
        )
        .unwrap();

        let mut result = HashMap::new();
        NestedYamlBackend::flatten_yaml(&yaml, String::new(), &mut result);

        assert_eq!(result.get("app.name"), Some(&"Test App".to_string()));
        assert_eq!(result.get("app.settings.theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_load_actual_locales() {
        // Test loading actual locale files from the project
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let locales_path = std::path::Path::new(manifest_dir).join("locales");
        let backend = NestedYamlBackend::new(&locales_path);

        // Check that English locale is available
        let locales = backend.available_locales();
        assert!(locales.contains(&"en"), "English locale should be available");

        // Check that a known translation key works
        let translation = backend.translate("en", "errors.auth.invalid_credentials");
        assert_eq!(
            translation,
            Some("Invalid email or password"),
            "Should find English translation for errors.auth.invalid_credentials"
        );

        // Check German translation exists
        let de_translation = backend.translate("de", "errors.auth.invalid_credentials");
        assert!(
            de_translation.is_some(),
            "German translation should exist for errors.auth.invalid_credentials"
        );
    }

    #[test]
    fn test_t_macro_integration() {
        // Test that the t! macro works with our backend
        use rust_i18n::t;

        let result = t!("errors.auth.invalid_credentials");
        assert_eq!(
            result, "Invalid email or password",
            "t! macro should return the correct translation"
        );
    }
}
