use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// File-based translation cache to avoid redundant API calls across sessions
///
/// Cache structure:
/// ```text
/// .ampel-i18n-cache/
///   ├── fi/
///   │   ├── dashboard.json
///   │   └── settings.json
///   └── sv/
///       ├── dashboard.json
///       └── settings.json
/// ```
#[derive(Debug)]
pub struct FileCache {
    cache_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    source_text: String,
    translated_text: String,
    provider: String,
    timestamp: i64,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NamespaceCache {
    entries: HashMap<String, CacheEntry>,
    #[serde(default)]
    version: u32,
}

impl Default for NamespaceCache {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            version: 1,
        }
    }
}

impl FileCache {
    /// Create new file cache with custom directory
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
        }
    }


    /// Get cached translation for a key
    pub fn get(
        &self,
        target_lang: &str,
        namespace: &str,
        key: &str,
        source_text: &str,
    ) -> Option<String> {
        let cache_file = self.cache_file_path(target_lang, namespace);

        if !cache_file.exists() {
            return None;
        }

        match self.load_cache(&cache_file) {
            Ok(cache) => {
                if let Some(entry) = cache.entries.get(key) {
                    // Validate source text matches (invalidate if changed)
                    if entry.source_text == source_text {
                        debug!(
                            "Cache hit: {} -> {} ({})",
                            key, target_lang, entry.provider
                        );
                        return Some(entry.translated_text.clone());
                    } else {
                        debug!(
                            "Cache miss: {} -> {} (source text changed)",
                            key, target_lang
                        );
                    }
                }
                None
            }
            Err(e) => {
                warn!("Failed to load cache {}: {}", cache_file.display(), e);
                None
            }
        }
    }

    /// Store translation in cache
    pub fn set(
        &self,
        target_lang: &str,
        namespace: &str,
        key: &str,
        source_text: &str,
        translated_text: &str,
        provider: &str,
    ) -> Result<()> {
        let cache_file = self.cache_file_path(target_lang, namespace);

        // Create directory if needed
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing cache or create new
        let mut cache = if cache_file.exists() {
            self.load_cache(&cache_file).unwrap_or_default()
        } else {
            NamespaceCache::default()
        };

        // Add/update entry
        cache.entries.insert(
            key.to_string(),
            CacheEntry {
                source_text: source_text.to_string(),
                translated_text: translated_text.to_string(),
                provider: provider.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                metadata: HashMap::new(),
            },
        );

        // Save cache
        self.save_cache(&cache_file, &cache)?;

        debug!("Cached: {} -> {} ({})", key, target_lang, provider);

        Ok(())
    }

    /// Store multiple translations in batch
    pub fn set_batch(
        &self,
        target_lang: &str,
        namespace: &str,
        translations: &[(String, String, String)], // (key, source, translation)
        provider: &str,
    ) -> Result<()> {
        let cache_file = self.cache_file_path(target_lang, namespace);

        // Create directory if needed
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing cache or create new
        let mut cache = if cache_file.exists() {
            self.load_cache(&cache_file).unwrap_or_default()
        } else {
            NamespaceCache::default()
        };

        let timestamp = chrono::Utc::now().timestamp();

        // Add all entries
        for (key, source_text, translated_text) in translations {
            cache.entries.insert(
                key.clone(),
                CacheEntry {
                    source_text: source_text.clone(),
                    translated_text: translated_text.clone(),
                    provider: provider.to_string(),
                    timestamp,
                    metadata: HashMap::new(),
                },
            );
        }

        // Save cache
        self.save_cache(&cache_file, &cache)?;

        debug!(
            "Cached {} translations: {} ({})",
            translations.len(),
            namespace,
            provider
        );

        Ok(())
    }

    /// Clear cache for specific language and namespace
    pub fn clear(&self, target_lang: &str, namespace: &str) -> Result<()> {
        let cache_file = self.cache_file_path(target_lang, namespace);

        if cache_file.exists() {
            fs::remove_file(&cache_file)?;
            debug!("Cleared cache: {} -> {}", target_lang, namespace);
        }

        Ok(())
    }

    /// Clear all cache for a language
    pub fn clear_language(&self, target_lang: &str) -> Result<()> {
        let lang_dir = self.cache_dir.join(target_lang);

        if lang_dir.exists() {
            fs::remove_dir_all(&lang_dir)?;
            debug!("Cleared all cache for: {}", target_lang);
        }

        Ok(())
    }

    /// Clear entire cache
    pub fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            debug!("Cleared entire cache");
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self, target_lang: &str) -> CacheStats {
        let lang_dir = self.cache_dir.join(target_lang);

        if !lang_dir.exists() {
            return CacheStats::default();
        }

        let mut total_entries = 0;
        let mut total_namespaces = 0;
        let mut providers: HashMap<String, usize> = HashMap::new();

        if let Ok(entries) = fs::read_dir(&lang_dir) {
            for entry in entries.flatten() {
                if let Ok(cache) = self.load_cache(&entry.path()) {
                    total_namespaces += 1;
                    total_entries += cache.entries.len();

                    for entry in cache.entries.values() {
                        *providers.entry(entry.provider.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        CacheStats {
            total_entries,
            total_namespaces,
            providers,
        }
    }

    fn cache_file_path(&self, target_lang: &str, namespace: &str) -> PathBuf {
        self.cache_dir
            .join(target_lang)
            .join(format!("{}.json", namespace))
    }

    fn load_cache(&self, path: &Path) -> Result<NamespaceCache> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn save_cache(&self, path: &Path, cache: &NamespaceCache) -> Result<()> {
        let content = serde_json::to_string_pretty(cache)?;
        fs::write(path, content)?;
        Ok(())
    }
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new(".ampel-i18n-cache")
    }
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_namespaces: usize,
    pub providers: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_set_get() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(temp_dir.path());

        cache
            .set(
                "fi",
                "dashboard",
                "greeting",
                "Hello",
                "Terve",
                "deepl",
            )
            .unwrap();

        let result = cache.get("fi", "dashboard", "greeting", "Hello");
        assert_eq!(result, Some("Terve".to_string()));

        // Source text mismatch should return None
        let result = cache.get("fi", "dashboard", "greeting", "Hi");
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_batch() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(temp_dir.path());

        let translations = vec![
            ("key1".to_string(), "Hello".to_string(), "Terve".to_string()),
            ("key2".to_string(), "World".to_string(), "Maailma".to_string()),
        ];

        cache
            .set_batch("fi", "dashboard", &translations, "deepl")
            .unwrap();

        assert_eq!(
            cache.get("fi", "dashboard", "key1", "Hello"),
            Some("Terve".to_string())
        );
        assert_eq!(
            cache.get("fi", "dashboard", "key2", "World"),
            Some("Maailma".to_string())
        );
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(temp_dir.path());

        cache
            .set("fi", "dashboard", "greeting", "Hello", "Terve", "deepl")
            .unwrap();

        cache.clear("fi", "dashboard").unwrap();

        let result = cache.get("fi", "dashboard", "greeting", "Hello");
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(temp_dir.path());

        cache
            .set("fi", "dashboard", "key1", "Hello", "Terve", "deepl")
            .unwrap();
        cache
            .set("fi", "settings", "key2", "World", "Maailma", "google")
            .unwrap();

        let stats = cache.stats("fi");
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_namespaces, 2);
        assert_eq!(stats.providers.get("deepl"), Some(&1));
        assert_eq!(stats.providers.get("google"), Some(&1));
    }
}
