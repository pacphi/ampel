use crate::error::{Error, Result};
use crate::translator::TranslationService;
use async_trait::async_trait;
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use lru::LruCache;
use nonzero_ext::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, warn};

/// Cache key for translation lookups
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    text: String,
    source_lang: String,
    target_lang: String,
}

/// Retry policy configuration
#[derive(Debug, Clone)]
struct RetryPolicy {
    max_retries: usize,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Google Cloud Translation API client with production-grade features:
/// - Batch translation (up to 100 texts per request)
/// - Exponential backoff retry logic (3 attempts)
/// - Token bucket rate limiting (100 req/sec)
/// - LRU caching (1000 entries)
/// - Usage metrics tracking
pub struct GoogleTranslator {
    client: reqwest::Client,
    api_key: String,
    cache: Arc<Mutex<LruCache<CacheKey, String>>>,
    rate_limiter: Arc<GovernorRateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
    retry_policy: RetryPolicy,
    usage_chars: Arc<Mutex<u64>>,
    usage_calls: Arc<Mutex<u64>>,
    cache_hits: Arc<Mutex<u64>>,
}

#[derive(Serialize)]
struct GoogleRequest {
    q: Vec<String>,
    target: String,
    source: String,
    format: String,
}

#[derive(Deserialize)]
struct GoogleResponse {
    data: GoogleData,
}

#[derive(Deserialize)]
struct GoogleData {
    translations: Vec<GoogleTranslation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslation {
    translated_text: String,
}

impl GoogleTranslator {
    pub fn new(api_key: String, timeout: Duration) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("ampel-i18n-builder/1.0.0")
            .build()
            .map_err(|e| Error::Config(format!("Failed to build HTTP client: {}", e)))?;

        // Rate limiter: 100 requests per second (Google API limit is much higher)
        let rate_limiter = Arc::new(GovernorRateLimiter::direct(Quota::per_second(nonzero!(100u32))));

        // LRU cache with 1000 entries
        // SAFETY: 1000 is a non-zero constant
        let cache_capacity = NonZeroUsize::new(1000).expect("1000 is non-zero");
        let cache = Arc::new(Mutex::new(LruCache::new(cache_capacity)));

        Ok(Self {
            client,
            api_key,
            cache,
            rate_limiter,
            retry_policy: RetryPolicy::default(),
            usage_chars: Arc::new(Mutex::new(0)),
            usage_calls: Arc::new(Mutex::new(0)),
            cache_hits: Arc::new(Mutex::new(0)),
        })
    }

    /// Get or create cache key
    fn cache_key(&self, text: &str, source_lang: &str, target_lang: &str) -> CacheKey {
        CacheKey {
            text: text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
        }
    }

    /// Make API request with exponential backoff retry
    async fn translate_with_retry(&self, texts: &[String], target_lang: &str) -> Result<Vec<String>> {
        let mut attempt = 0;
        let mut delay = self.retry_policy.initial_delay_ms;

        loop {
            // Wait for rate limiter token
            self.rate_limiter.until_ready().await;

            let request = GoogleRequest {
                q: texts.to_vec(),
                target: target_lang.to_string(),
                source: "en".to_string(),
                format: "text".to_string(),
            };

            let url = format!(
                "https://translation.googleapis.com/language/translate/v2?key={}",
                self.api_key
            );

            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    if status.is_success() {
                        let google_response: GoogleResponse = resp.json().await?;
                        return Ok(google_response
                            .data
                            .translations
                            .into_iter()
                            .map(|t| t.translated_text)
                            .collect());
                    }

                    let error_body = resp.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());

                    // Check if retryable
                    let is_retryable = matches!(status.as_u16(), 408 | 429 | 500 | 502 | 503 | 504);

                    if !is_retryable {
                        return Err(Error::Api(format!(
                            "Google Translation API error {}: {}",
                            status, error_body
                        )));
                    }

                    attempt += 1;
                    if attempt >= self.retry_policy.max_retries {
                        return Err(Error::Api(format!(
                            "Max retries ({}) exceeded. Last error: {} {}",
                            self.retry_policy.max_retries, status, error_body
                        )));
                    }

                    warn!(
                        "Google API request failed (attempt {}/{}): {} {}",
                        attempt, self.retry_policy.max_retries, status, error_body
                    );
                }
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.retry_policy.max_retries {
                        return Err(Error::Api(format!("Network error after {} attempts: {}", attempt, e)));
                    }

                    warn!(
                        "Google API network error (attempt {}/{}): {}",
                        attempt, self.retry_policy.max_retries, e
                    );
                }
            }

            // Exponential backoff with jitter
            let jitter = (delay as f64 * 0.1 * random_f64()) as u64;
            let sleep_duration = std::cmp::min(delay + jitter, self.retry_policy.max_delay_ms);

            debug!("Retrying in {}ms...", sleep_duration);
            tokio::time::sleep(Duration::from_millis(sleep_duration)).await;

            delay = (delay as f64 * self.retry_policy.backoff_multiplier) as u64;
        }
    }

    /// Get usage statistics (for future metrics/monitoring)
    #[allow(dead_code)]
    pub fn get_stats(&self) -> Result<GoogleStats> {
        Ok(GoogleStats {
            total_chars: *self.usage_chars.lock()
                .map_err(|e| Error::Internal(format!("Usage chars lock poisoned: {}", e)))?,
            total_calls: *self.usage_calls.lock()
                .map_err(|e| Error::Internal(format!("Usage calls lock poisoned: {}", e)))?,
            cache_hits: *self.cache_hits.lock()
                .map_err(|e| Error::Internal(format!("Cache hits lock poisoned: {}", e)))?,
        })
    }
}

/// Usage statistics for Google Translator (for future metrics/monitoring)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GoogleStats {
    pub total_chars: u64,
    pub total_calls: u64,
    pub cache_hits: u64,
}

// Simple random number generator for jitter
fn random_f64() -> f64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u8(0);
    (hasher.finish() % 100) as f64 / 100.0
}

#[async_trait]
impl TranslationService for GoogleTranslator {
    fn provider_name(&self) -> &str {
        "Google"
    }

    fn provider_tier(&self) -> u8 {
        3
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        const MAX_BATCH_SIZE: usize = 100;

        // Extract text values with their keys
        let text_entries: Vec<(String, String)> = texts
            .iter()
            .filter_map(|(key, value)| {
                if let serde_json::Value::String(s) = value {
                    Some((key.clone(), s.clone()))
                } else {
                    None
                }
            })
            .collect();

        if text_entries.is_empty() {
            return Ok(HashMap::new());
        }

        // Check cache and separate into cached/uncached
        let mut cached_results: HashMap<String, String> = HashMap::new();
        let mut uncached_entries = Vec::new();

        for (key, text) in &text_entries {
            let cache_key = self.cache_key(text, "en", target_lang);

            let cached = self.cache.lock()
                .map_err(|e| Error::Internal(format!("Cache lock poisoned: {}", e)))?
                .get(&cache_key)
                .cloned();

            if let Some(cached_text) = cached {
                cached_results.insert(key.clone(), cached_text);
                *self.cache_hits.lock()
                    .map_err(|e| Error::Internal(format!("Cache hits lock poisoned: {}", e)))? += 1;
            } else {
                uncached_entries.push((key.clone(), text.clone()));
            }
        }

        debug!(
            "Translating {} texts ({} from cache, {} from API)",
            text_entries.len(),
            cached_results.len(),
            uncached_entries.len()
        );

        // Translate uncached texts in batches
        let mut all_translations = Vec::new();

        for chunk in uncached_entries.chunks(MAX_BATCH_SIZE) {
            let chunk_texts: Vec<String> = chunk.iter().map(|(_, text)| text.clone()).collect();

            let translations = self.translate_with_retry(&chunk_texts, target_lang).await?;

            // Update usage metrics
            let chars: usize = chunk_texts.iter().map(|s| s.len()).sum();
            *self.usage_chars.lock()
                .map_err(|e| Error::Internal(format!("Usage chars lock poisoned: {}", e)))? += chars as u64;
            *self.usage_calls.lock()
                .map_err(|e| Error::Internal(format!("Usage calls lock poisoned: {}", e)))? += 1;

            // Cache results
            for (text, translation) in chunk_texts.iter().zip(translations.iter()) {
                let cache_key = self.cache_key(text, "en", target_lang);
                self.cache.lock()
                    .map_err(|e| Error::Internal(format!("Cache lock poisoned: {}", e)))?
                    .put(cache_key, translation.clone());
            }

            all_translations.extend(
                chunk
                    .iter()
                    .zip(translations.iter())
                    .map(|((key, _), translation)| (key.clone(), translation.clone())),
            );
        }

        // Merge cached and new translations
        let mut result = HashMap::new();

        for (key, _) in &text_entries {
            if let Some(translation) = cached_results.get(key) {
                result.insert(key.clone(), serde_json::Value::String(translation.clone()));
            } else if let Some((_, translation)) = all_translations.iter().find(|(k, _)| k == key) {
                result.insert(key.clone(), serde_json::Value::String(translation.clone()));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key() {
        let translator = GoogleTranslator::new("test-key".to_string(), Duration::from_secs(30))
            .expect("Failed to create translator");
        let key1 = translator.cache_key("hello", "en", "fi");
        let key2 = translator.cache_key("hello", "en", "fi");
        let key3 = translator.cache_key("hello", "en", "sv");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_delay_ms, 1000);
        assert_eq!(policy.max_delay_ms, 30000);
        assert_eq!(policy.backoff_multiplier, 2.0);
    }
}
