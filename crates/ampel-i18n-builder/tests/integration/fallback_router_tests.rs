//! Integration tests for FallbackTranslationRouter
//!
//! This test suite validates the fallback translation router functionality:
//! - Provider ordering and tier-based selection
//! - Fallback behavior when providers fail
//! - Stop-on-first-success optimization
//! - Comprehensive error handling
//!
//! These tests use mock providers to simulate different scenarios without
//! requiring actual API keys or network calls.

use ampel_i18n_builder::config::Config;
use ampel_i18n_builder::error::{Error, Result};
use ampel_i18n_builder::translator::fallback::FallbackTranslationRouter;
use ampel_i18n_builder::translator::TranslationService;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock provider that can be configured to succeed or fail
#[derive(Clone)]
struct MockProvider {
    name: String,
    tier: u8,
    call_count: Arc<Mutex<usize>>,
    should_fail: bool,
    delay_ms: u64,
}

impl MockProvider {
    fn new(name: &str, tier: u8, should_fail: bool) -> Self {
        Self {
            name: name.to_string(),
            tier,
            call_count: Arc::new(Mutex::new(0)),
            should_fail,
            delay_ms: 0,
        }
    }

    fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait]
impl TranslationService for MockProvider {
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Increment call count
        {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
        }

        // Simulate delay if configured
        if self.delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        }

        // Fail if configured to do so
        if self.should_fail {
            return Err(Error::Translation(format!(
                "{} provider failed (simulated)",
                self.name
            )));
        }

        // Success: return translated texts
        let mut result = HashMap::new();
        for (key, value) in texts {
            let text = value.as_str().unwrap_or("");
            let translated = format!("{}-{}-{}", text, target_lang, self.name);
            result.insert(key.clone(), serde_json::json!(translated));
        }
        Ok(result)
    }

    fn provider_name(&self) -> &str {
        &self.name
    }

    fn provider_tier(&self) -> u8 {
        self.tier
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[test]
fn test_new_with_no_providers_fails() {
    let config = Config::default();
    let result = FallbackTranslationRouter::new(&config);

    assert!(result.is_err());
    match result {
        Err(Error::Config(msg)) => {
            assert!(msg.contains("No translation providers available"));
        }
        _ => panic!("Expected Config error"),
    }
}

#[tokio::test]
async fn test_fallback_on_first_provider_failure() {
    // This test will be fully functional when provider initialization is implemented
    // For now, we document the expected behavior:

    // Setup:
    // - Provider 1 (Tier 1): Configured to fail
    // - Provider 2 (Tier 2): Configured to succeed
    //
    // Expected behavior:
    // 1. Router attempts Provider 1 → fails
    // 2. Router falls back to Provider 2 → succeeds
    // 3. Result contains translations from Provider 2
    // 4. Provider 1 call count = 1
    // 5. Provider 2 call count = 1

    // TODO: Implement once provider initialization is added to FallbackTranslationRouter::new()
}

#[tokio::test]
async fn test_all_providers_fail() {
    // This test will verify error handling when all providers fail

    // Setup:
    // - Provider 1 (Tier 1): Configured to fail
    // - Provider 2 (Tier 2): Configured to fail
    // - Provider 3 (Tier 3): Configured to fail
    //
    // Expected behavior:
    // 1. Router attempts all providers sequentially
    // 2. All providers fail
    // 3. Router returns Error::Translation with descriptive message
    // 4. Error message mentions "All providers failed"
    // 5. All provider call counts = 1

    // TODO: Implement once provider initialization is added to FallbackTranslationRouter::new()
}

#[tokio::test]
async fn test_first_provider_succeeds() {
    // This test verifies optimization when first provider succeeds

    // Setup:
    // - Provider 1 (Tier 1): Configured to succeed
    // - Provider 2 (Tier 2): Configured to succeed
    //
    // Expected behavior:
    // 1. Router attempts Provider 1 → succeeds
    // 2. Router returns immediately (stop_on_first_success)
    // 3. Provider 1 call count = 1
    // 4. Provider 2 call count = 0 (never called)

    // TODO: Implement once provider initialization is added to FallbackTranslationRouter::new()
}

#[tokio::test]
async fn test_provider_tier_ordering() {
    // This test verifies providers are tried in tier order (1, 2, 3, 4)

    // Setup:
    // - Provider A (Tier 3): Configured to fail
    // - Provider B (Tier 1): Configured to succeed
    // - Provider C (Tier 2): Configured to succeed
    // - Provider D (Tier 4): Configured to succeed
    //
    // Expected execution order: B(1), C(2), A(3), D(4)
    // Expected result: Provider B succeeds (Tier 1)

    // TODO: Implement once provider initialization is added to FallbackTranslationRouter::new()
}

#[tokio::test]
async fn test_language_preference_ordering() {
    // This test verifies language preference affects provider ordering

    // Setup (for Finnish "fi"):
    // - DeepL (Tier 2): preferred_languages = ["fi", "sv", "de"]
    // - Systran (Tier 1): no preference
    // - Google (Tier 3): preferred_languages = ["ar", "th"]
    //
    // Expected order for "fi": DeepL(2), Systran(1), Google(3)
    // Expected order for "ar": Google(3), Systran(1), DeepL(2)

    // TODO: Implement once ProviderConfig includes preferred_languages field
}

#[tokio::test]
async fn test_concurrent_requests() {
    // This test verifies router handles concurrent translation requests safely

    // Setup:
    // - Single router instance
    // - Multiple concurrent translation requests
    // - Provider with simulated delay
    //
    // Expected behavior:
    // - All requests complete successfully
    // - No race conditions or panics
    // - Each request gets independent results

    // TODO: Implement once provider initialization is added to FallbackTranslationRouter::new()
}

#[tokio::test]
async fn test_empty_texts_batch() {
    // This test verifies router handles empty input gracefully

    // Setup:
    // - Empty HashMap of texts
    //
    // Expected behavior:
    // - Router succeeds without calling providers
    // - Returns empty HashMap
    // - OR router calls provider and gets empty result

    // TODO: Implement once provider initialization is added to FallbackTranslationRouter::new()
}

#[tokio::test]
async fn test_large_batch_translation() {
    // This test verifies router handles large batches efficiently

    // Setup:
    // - 1000 text entries in batch
    // - Provider configured to succeed
    //
    // Expected behavior:
    // - Router processes entire batch
    // - Returns 1000 translated entries
    // - Respects provider batch size limits (if configured)

    // TODO: Implement once provider initialization is added to FallbackTranslationRouter::new()
}

/// Helper function to create test texts batch
#[allow(dead_code)]
fn create_test_texts(count: usize) -> HashMap<String, serde_json::Value> {
    let mut texts = HashMap::new();
    for i in 0..count {
        texts.insert(
            format!("key_{}", i),
            serde_json::json!(format!("Text {}", i)),
        );
    }
    texts
}

/// Helper function to verify translation results
#[allow(dead_code)]
fn verify_translations(
    result: &HashMap<String, serde_json::Value>,
    expected_provider: &str,
    target_lang: &str,
) {
    for (key, value) in result {
        let text = value.as_str().unwrap();
        assert!(
            text.contains(expected_provider),
            "Translation should be from {}",
            expected_provider
        );
        assert!(
            text.contains(target_lang),
            "Translation should be for language {}",
            target_lang
        );
        assert!(
            text.starts_with("Text") || text.starts_with("Hello") || text.starts_with("Goodbye"),
            "Translation should start with original text"
        );
    }
}

#[test]
fn test_mock_provider_basic() {
    // Verify mock provider works correctly
    let provider = MockProvider::new("TestProvider", 2, false);

    assert_eq!(provider.provider_name(), "TestProvider");
    assert_eq!(provider.provider_tier(), 2);
    assert!(provider.is_available());
    assert_eq!(provider.call_count(), 0);
}

#[tokio::test]
async fn test_mock_provider_success() {
    let provider = MockProvider::new("TestProvider", 2, false);

    let mut texts = HashMap::new();
    texts.insert("greeting".to_string(), serde_json::json!("Hello"));
    texts.insert("farewell".to_string(), serde_json::json!("Goodbye"));

    let result = provider.translate_batch(&texts, "fi").await;

    assert!(result.is_ok());
    let translated = result.unwrap();
    assert_eq!(translated.len(), 2);
    assert_eq!(
        translated.get("greeting").unwrap().as_str().unwrap(),
        "Hello-fi-TestProvider"
    );
    assert_eq!(
        translated.get("farewell").unwrap().as_str().unwrap(),
        "Goodbye-fi-TestProvider"
    );
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn test_mock_provider_failure() {
    let provider = MockProvider::new("FailingProvider", 3, true);

    let mut texts = HashMap::new();
    texts.insert("greeting".to_string(), serde_json::json!("Hello"));

    let result = provider.translate_batch(&texts, "fi").await;

    assert!(result.is_err());
    match result {
        Err(Error::Translation(msg)) => {
            assert!(msg.contains("FailingProvider"));
            assert!(msg.contains("failed"));
        }
        _ => panic!("Expected Translation error"),
    }
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn test_mock_provider_delay() {
    use std::time::Instant;

    let provider = MockProvider::new("SlowProvider", 1, false).with_delay(100);

    let mut texts = HashMap::new();
    texts.insert("greeting".to_string(), serde_json::json!("Hello"));

    let start = Instant::now();
    let result = provider.translate_batch(&texts, "fi").await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(
        duration.as_millis() >= 100,
        "Should take at least 100ms, took {}ms",
        duration.as_millis()
    );
}

// ========================================
// REAL PROVIDER INITIALIZATION TESTS
// ========================================

use ampel_i18n_builder::config::{FallbackConfig, ProviderConfig as ConfigProviderConfig, ProvidersConfig, TranslationConfig};

/// Test that FallbackTranslationRouter initializes with all 4 providers when API keys are configured
#[test]
fn test_router_initializes_all_four_providers() {
    let config = Config {
        translation_dir: std::path::PathBuf::from("./locales"),
        translation: TranslationConfig {
            // Provide all API keys (fake keys for testing initialization logic)
            systran_api_key: Some("fake_systran_key".to_string()),
            deepl_api_key: Some("fake_deepl_key".to_string()),
            google_api_key: Some("fake_google_key".to_string()),
            openai_api_key: Some("fake_openai_key".to_string()),

            timeout_secs: 30,
            batch_size: 50,
            default_timeout_secs: 30,
            default_batch_size: 50,
            default_max_retries: 3,

            providers: ProvidersConfig {
                systran: ConfigProviderConfig {
                    enabled: true,
                    priority: 1,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 100,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                deepl: ConfigProviderConfig {
                    enabled: true,
                    priority: 2,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 10,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                google: ConfigProviderConfig {
                    enabled: true,
                    priority: 3,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 100,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                openai: ConfigProviderConfig {
                    enabled: true,
                    priority: 4,
                    timeout_secs: 60,
                    max_retries: 3,
                    batch_size: 20,
                    rate_limit_per_sec: 50,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
            },

            fallback: FallbackConfig {
                skip_on_missing_key: true,
                stop_on_first_success: true,
                log_fallback_events: true,
            },
        },
    };

    let router = FallbackTranslationRouter::new(&config)
        .expect("Router should initialize with all providers");

    // Verify router is available
    assert!(router.is_available(), "Router should be available");
    assert_eq!(router.provider_name(), "FallbackRouter");
    assert_eq!(router.provider_tier(), 0);
}

/// Test that FallbackTranslationRouter respects the enabled flag
#[test]
fn test_router_respects_enabled_flag() {
    let config = Config {
        translation_dir: std::path::PathBuf::from("./locales"),
        translation: TranslationConfig {
            // Provide keys for all, but only enable Google and OpenAI
            systran_api_key: Some("fake_systran_key".to_string()),
            deepl_api_key: Some("fake_deepl_key".to_string()),
            google_api_key: Some("fake_google_key".to_string()),
            openai_api_key: Some("fake_openai_key".to_string()),

            timeout_secs: 30,
            batch_size: 50,
            default_timeout_secs: 30,
            default_batch_size: 50,
            default_max_retries: 3,

            providers: ProvidersConfig {
                systran: ConfigProviderConfig {
                    enabled: false, // Disabled
                    priority: 1,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 100,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                deepl: ConfigProviderConfig {
                    enabled: false, // Disabled
                    priority: 2,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 10,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                google: ConfigProviderConfig {
                    enabled: true, // Enabled
                    priority: 3,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 100,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                openai: ConfigProviderConfig {
                    enabled: true, // Enabled
                    priority: 4,
                    timeout_secs: 60,
                    max_retries: 3,
                    batch_size: 20,
                    rate_limit_per_sec: 50,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
            },

            fallback: FallbackConfig {
                skip_on_missing_key: true,
                stop_on_first_success: true,
                log_fallback_events: true,
            },
        },
    };

    let router = FallbackTranslationRouter::new(&config)
        .expect("Router should initialize with enabled providers only");

    // Verify router is available with at least one provider
    assert!(router.is_available(), "Router should be available");
}

/// Test that FallbackTranslationRouter reads API keys from environment variables
#[test]
fn test_router_reads_api_keys_from_env() {
    // Set environment variables
    std::env::set_var("SYSTRAN_API_KEY", "env_systran_key");
    std::env::set_var("DEEPL_API_KEY", "env_deepl_key");

    let config = Config {
        translation_dir: std::path::PathBuf::from("./locales"),
        translation: TranslationConfig {
            // No API keys in config, should read from env
            systran_api_key: None,
            deepl_api_key: None,
            google_api_key: None,
            openai_api_key: None,

            timeout_secs: 30,
            batch_size: 50,
            default_timeout_secs: 30,
            default_batch_size: 50,
            default_max_retries: 3,

            providers: ProvidersConfig {
                systran: ConfigProviderConfig {
                    enabled: true,
                    priority: 1,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 100,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                deepl: ConfigProviderConfig {
                    enabled: true,
                    priority: 2,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 10,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                google: ConfigProviderConfig {
                    enabled: false, // Disabled (no API key)
                    priority: 3,
                    timeout_secs: 30,
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 100,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
                openai: ConfigProviderConfig {
                    enabled: false, // Disabled (no API key)
                    priority: 4,
                    timeout_secs: 60,
                    max_retries: 3,
                    batch_size: 20,
                    rate_limit_per_sec: 50,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                    preferred_languages: None,
                },
            },

            fallback: FallbackConfig {
                skip_on_missing_key: true,
                stop_on_first_success: true,
                log_fallback_events: true,
            },
        },
    };

    let router = FallbackTranslationRouter::new(&config)
        .expect("Router should initialize with env var API keys");

    // Verify router is available
    assert!(router.is_available(), "Router should be available");

    // Clean up env vars
    std::env::remove_var("SYSTRAN_API_KEY");
    std::env::remove_var("DEEPL_API_KEY");
}
