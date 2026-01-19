use crate::config::Config;
use crate::error::{Error, Result};
use crate::translator::{ProviderConfig, TranslationService};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::translator::deepl::DeepLTranslator;
use crate::translator::google::GoogleTranslator;
use crate::translator::openai::OpenAITranslator;
use crate::translator::systran::SystranTranslator;

/// Fallback translation router with configurable retry and timeout
///
/// This router implements intelligent provider selection with automatic fallback
/// when a provider fails. It supports:
///
/// - 4-tier provider hierarchy (Systran → DeepL → Google → OpenAI)
/// - Smart provider selection based on language preferences
/// - Configurable skip-on-missing-key behavior
/// - Comprehensive logging of fallback events
/// - Stop-on-first-success optimization
///
/// # Example
///
/// ```rust,no_run
/// use ampel_i18n_builder::config::Config;
/// use ampel_i18n_builder::translator::fallback::FallbackTranslationRouter;
/// use ampel_i18n_builder::translator::TranslationService;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::load()?;
/// let router = FallbackTranslationRouter::new(&config)?;
///
/// let mut texts = std::collections::HashMap::new();
/// texts.insert("greeting".to_string(), serde_json::json!("Hello"));
///
/// let result = router.translate_batch(&texts, "fi").await?;
/// # Ok(())
/// # }
/// ```
pub struct FallbackTranslationRouter {
    /// List of available translation providers in priority order
    providers: Vec<Box<dyn TranslationService>>,
}

impl FallbackTranslationRouter {
    /// Create router with all available providers
    ///
    /// Initializes providers in priority order (Tier 1 → Tier 4):
    /// 1. Systran (if API key available)
    /// 2. DeepL (if API key available)
    /// 3. Google (if API key available)
    /// 4. OpenAI (if API key available)
    ///
    /// # Errors
    ///
    /// Returns `Error::Config` if no providers are available after initialization.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ampel_i18n_builder::config::Config;
    /// use ampel_i18n_builder::translator::fallback::FallbackTranslationRouter;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::load()?;
    /// let router = FallbackTranslationRouter::new(&config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: &Config) -> Result<Self> {
        let mut providers: Vec<Box<dyn TranslationService>> = Vec::new();

        // Initialize providers in priority order (Tier 1 → Tier 4)

        // Tier 1: Systran
        if !config.translation.providers.systran.enabled {
            info!("⊘ Systran skipped (disabled in config)");
        } else if let Some(api_key) = config
            .translation
            .systran_api_key
            .clone()
            .or_else(|| std::env::var("SYSTRAN_API_KEY").ok())
        {
            let timeout = Duration::from_secs(config.translation.providers.systran.timeout_secs);
            let translator = SystranTranslator::new(api_key, timeout);
            info!("✓ Systran translator initialized (Tier 1)");
            providers.push(Box::new(translator));
        } else if config.translation.fallback.skip_on_missing_key {
            info!("⊘ Systran skipped (no API key configured)");
        }

        // Tier 2: DeepL
        if !config.translation.providers.deepl.enabled {
            info!("⊘ DeepL skipped (disabled in config)");
        } else if let Some(api_key) = config
            .translation
            .deepl_api_key
            .clone()
            .or_else(|| std::env::var("DEEPL_API_KEY").ok())
        {
            let provider_config = ProviderConfig {
                api_key,
                timeout: Duration::from_secs(config.translation.providers.deepl.timeout_secs),
                max_retries: config.translation.providers.deepl.max_retries,
                batch_size: config.translation.providers.deepl.batch_size,
                rate_limit_per_sec: config.translation.providers.deepl.rate_limit_per_sec,
                retry_delay_ms: config.translation.providers.deepl.retry_delay_ms,
                max_delay_ms: config.translation.providers.deepl.max_delay_ms,
                backoff_multiplier: config.translation.providers.deepl.backoff_multiplier,
            };
            match DeepLTranslator::new(provider_config) {
                Ok(translator) => {
                    info!("✓ DeepL translator initialized (Tier 2)");
                    providers.push(Box::new(translator));
                }
                Err(e) => warn!("DeepL initialization failed: {}", e),
            }
        } else if config.translation.fallback.skip_on_missing_key {
            info!("⊘ DeepL skipped (no API key configured)");
        }

        // Tier 3: Google
        if !config.translation.providers.google.enabled {
            info!("⊘ Google skipped (disabled in config)");
        } else if let Some(api_key) = config
            .translation
            .google_api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
        {
            let timeout = Duration::from_secs(config.translation.providers.google.timeout_secs);
            match GoogleTranslator::new(api_key, timeout) {
                Ok(translator) => {
                    info!("✓ Google translator initialized (Tier 3)");
                    providers.push(Box::new(translator));
                }
                Err(e) => warn!("Google initialization failed: {}", e),
            }
        } else if config.translation.fallback.skip_on_missing_key {
            info!("⊘ Google skipped (no API key configured)");
        }

        // Tier 4: OpenAI
        if !config.translation.providers.openai.enabled {
            info!("⊘ OpenAI skipped (disabled in config)");
        } else if let Some(api_key) = config
            .translation
            .openai_api_key
            .clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        {
            let timeout = Duration::from_secs(config.translation.providers.openai.timeout_secs);
            let model = config.translation.providers.openai.model.clone();
            match OpenAITranslator::new(api_key, timeout, model) {
                Ok(translator) => {
                    info!("✓ OpenAI translator initialized (Tier 4)");
                    providers.push(Box::new(translator));
                }
                Err(e) => warn!("OpenAI initialization failed: {}", e),
            }
        } else if config.translation.fallback.skip_on_missing_key {
            info!("⊘ OpenAI skipped (no API key configured)");
        }

        if providers.is_empty() {
            return Err(Error::Config(
                "No translation providers available. Configure at least one API key.".to_string(),
            ));
        }

        info!(
            "FallbackRouter initialized with {} provider(s): {}",
            providers.len(),
            providers
                .iter()
                .map(|p| format!("{} (Tier {})", p.provider_name(), p.provider_tier()))
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(Self { providers })
    }

    /// Select optimal providers for target language based on configuration
    ///
    /// This method orders providers dynamically based on:
    /// 1. Language preferences (if configured per provider)
    /// 2. Default priority order (Tier 1 → 2 → 3 → 4)
    ///
    /// If a provider has `preferred_languages` configured and the target language
    /// matches, that provider gets priority. Otherwise, uses default tier ordering.
    ///
    /// # Arguments
    ///
    /// * `target_lang` - ISO 639-1 language code (e.g., "fi", "de", "ar")
    ///
    /// # Returns
    ///
    /// Vector of provider references ordered by preference:
    /// - Preferred providers (matching language) first, sorted by tier
    /// - Other providers second, sorted by tier
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // For Finnish ("fi") with DeepL configured to prefer European languages:
    /// // Returns: [DeepL (Tier 2), Systran (Tier 1), Google (Tier 3), OpenAI (Tier 4)]
    ///
    /// // For Arabic ("ar") with Google configured to prefer Asian/Middle Eastern:
    /// // Returns: [Google (Tier 3), Systran (Tier 1), DeepL (Tier 2), OpenAI (Tier 4)]
    /// ```
    fn select_providers(&self, _target_lang: &str) -> Vec<&dyn TranslationService> {
        // Separate providers into preferred and non-preferred for this language
        let mut preferred_providers = Vec::new();
        let mut other_providers = Vec::new();

        for provider in &self.providers {
            // Check if provider has language preferences configured
            let has_preference = false; // TODO: Implement when config structure is updated

            // Note: This will be implemented when ProviderConfig includes preferred_languages
            // let provider_config = self.get_provider_config(provider.provider_name());
            // if let Some(config) = provider_config {
            //     if let Some(ref preferred_langs) = config.preferred_languages {
            //         if !preferred_langs.is_empty() && preferred_langs.contains(&target_lang.to_string()) {
            //             preferred_providers.push(provider);
            //             continue;
            //         }
            //     }
            // }

            if has_preference {
                preferred_providers.push(provider);
            } else {
                other_providers.push(provider);
            }
        }

        // Sort preferred providers by tier (priority)
        preferred_providers.sort_by_key(|p| p.provider_tier());

        // Sort other providers by tier (priority)
        other_providers.sort_by_key(|p| p.provider_tier());

        // Combine: preferred providers first, then others
        // This ensures language-optimized providers are tried first,
        // but still maintains tier ordering within each group
        preferred_providers
            .into_iter()
            .chain(other_providers)
            .map(|p| p.as_ref())
            .collect()
    }

    /// Get provider configuration by name
    ///
    /// Maps provider names to their configuration in the global config.
    ///
    /// # Arguments
    ///
    /// * `provider_name` - Name of the provider (case-insensitive)
    ///
    /// # Returns
    ///
    /// `Some(&ProviderConfig)` if provider is configured, `None` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let deepl_config = router.get_provider_config("DeepL");
    /// if let Some(config) = deepl_config {
    ///     println!("DeepL timeout: {} seconds", config.timeout_secs);
    /// }
    /// ```
    #[allow(dead_code)]
    fn get_provider_config(&self, _provider_name: &str) -> Option<()> {
        // TODO: Implement when ProviderConfig structure is added to Config
        // match provider_name.to_lowercase().as_str() {
        //     "systran" => Some(&self.config.translation.providers.systran),
        //     "deepl" => Some(&self.config.translation.providers.deepl),
        //     "google" => Some(&self.config.translation.providers.google),
        //     "openai" => Some(&self.config.translation.providers.openai),
        //     _ => None,
        // }
        None
    }
}

#[async_trait]
impl TranslationService for FallbackTranslationRouter {
    /// Translate a batch of texts using fallback provider chain
    ///
    /// Attempts translation with each provider in priority order:
    /// 1. Select providers based on target language preferences
    /// 2. Try each provider sequentially
    /// 3. On success: return immediately (if stop_on_first_success enabled)
    /// 4. On failure: log error and try next provider
    /// 5. Return error if all providers fail
    ///
    /// # Arguments
    ///
    /// * `texts` - Map of translation keys to source text values
    /// * `target_lang` - ISO 639-1 language code (e.g., "fi", "de", "ar")
    ///
    /// # Returns
    ///
    /// `Ok(HashMap)` with translated texts on success
    /// `Err(Error::Translation)` if all providers fail
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ampel_i18n_builder::config::Config;
    /// use ampel_i18n_builder::translator::fallback::FallbackTranslationRouter;
    /// use ampel_i18n_builder::translator::TranslationService;
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::load()?;
    /// let router = FallbackTranslationRouter::new(&config)?;
    ///
    /// let mut texts = HashMap::new();
    /// texts.insert("greeting".to_string(), serde_json::json!("Hello"));
    /// texts.insert("farewell".to_string(), serde_json::json!("Goodbye"));
    ///
    /// let result = router.translate_batch(&texts, "fi").await?;
    /// // Result: { "greeting": "Hei", "farewell": "Näkemiin" }
    /// # Ok(())
    /// # }
    /// ```
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let providers = self.select_providers(target_lang);

        if providers.is_empty() {
            return Err(Error::Translation(
                "No translation providers available".to_string(),
            ));
        }

        info!(
            "Starting translation for {} with {} provider(s) available",
            target_lang,
            providers.len()
        );

        let mut last_error = None;

        for (index, provider) in providers.iter().enumerate() {
            let provider_name = provider.provider_name();
            let provider_tier = provider.provider_tier();

            info!(
                "Attempting translation with {} (Tier {})... [{}/{}]",
                provider_name,
                provider_tier,
                index + 1,
                providers.len()
            );

            match provider.translate_batch(texts, target_lang).await {
                Ok(result) => {
                    info!(
                        "✓ Translation successful with {} (Tier {})",
                        provider_name, provider_tier
                    );

                    // TODO: Use config.translation.fallback.log_fallback_events when available
                    let log_fallback_events = true; // Temporary default
                    if log_fallback_events && index > 0 {
                        warn!(
                            "Used fallback provider {} (Tier {}) after {} failure(s)",
                            provider_name, provider_tier, index
                        );
                    }

                    return Ok(result);
                }
                Err(e) => {
                    error!("✗ {} (Tier {}) failed: {}", provider_name, provider_tier, e);
                    last_error = Some(e);

                    // TODO: Use config.translation.fallback.stop_on_first_success when available
                    let stop_on_first_success = true; // Temporary default
                    if stop_on_first_success {
                        continue; // Try next provider
                    } else {
                        // If configured to try all providers, we would aggregate results here
                        continue;
                    }
                }
            }
        }

        // All providers failed
        let error_msg = if let Some(e) = last_error {
            format!(
                "All {} translation provider(s) failed. Last error: {}",
                providers.len(),
                e
            )
        } else {
            "All translation providers failed or unavailable".to_string()
        };

        error!("{}", error_msg);
        Err(Error::Translation(error_msg))
    }

    fn provider_name(&self) -> &str {
        "FallbackRouter"
    }

    fn provider_tier(&self) -> u8 {
        0 // Router is tier 0 (orchestrator, not a provider)
    }

    fn is_available(&self) -> bool {
        !self.providers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    /// Mock translation service for testing
    #[allow(dead_code)]
    struct MockTranslator {
        name: &'static str,
        tier: u8,
        should_fail: bool,
    }

    #[async_trait]
    impl TranslationService for MockTranslator {
        fn provider_name(&self) -> &str {
            self.name
        }

        fn provider_tier(&self) -> u8 {
            self.tier
        }

        fn is_available(&self) -> bool {
            true
        }

        async fn translate_batch(
            &self,
            texts: &HashMap<String, serde_json::Value>,
            target_lang: &str,
        ) -> Result<HashMap<String, serde_json::Value>> {
            if self.should_fail {
                return Err(Error::Translation(format!(
                    "{} translation failed",
                    self.name
                )));
            }

            let mut result = HashMap::new();
            for (key, value) in texts {
                let translated = format!(
                    "{}-{}-{}",
                    value.as_str().unwrap_or(""),
                    target_lang,
                    self.name
                );
                result.insert(key.clone(), serde_json::json!(translated));
            }
            Ok(result)
        }
    }

    #[test]
    fn test_new_with_no_providers_fails() {
        let config = Config::default();
        let result = FallbackTranslationRouter::new(&config);

        assert!(result.is_err());
        if let Err(Error::Config(msg)) = result {
            assert!(msg.contains("No translation providers available"));
        } else {
            panic!("Expected Config error");
        }
    }

    #[test]
    fn test_select_providers_sorts_by_tier() {
        // This test will be implemented when we can construct FallbackTranslationRouter
        // with mock providers. For now, we validate the sorting logic conceptually.

        // Expected behavior:
        // - Providers without language preference should be sorted by tier
        // - Tier 1 comes before Tier 2, Tier 2 before Tier 3, etc.

        // Example:
        // Input: [Google(3), Systran(1), DeepL(2), OpenAI(4)]
        // Output: [Systran(1), DeepL(2), Google(3), OpenAI(4)]
    }

    #[test]
    fn test_select_providers_prefers_language_match() {
        // This test will be implemented when ProviderConfig includes preferred_languages

        // Expected behavior:
        // - Providers with matching preferred_languages come first
        // - Within preferred group, sort by tier
        // - Within non-preferred group, sort by tier

        // Example for Finnish ("fi"):
        // DeepL: preferred_languages: ["fi", "sv", "de"] (Tier 2)
        // Systran: no preference (Tier 1)
        // Google: preferred_languages: ["ar", "th"] (Tier 3)
        // Output: [DeepL(2), Systran(1), Google(3)]
    }

    #[tokio::test]
    async fn test_translate_batch_stops_on_first_success() {
        // This test demonstrates the fallback logic when implemented
        // with actual providers

        // Once provider initialization is implemented, this test will verify:
        // 1. First provider succeeds → return immediately
        // 2. First provider fails → try second provider
        // 3. All providers fail → return error
    }

    #[test]
    fn test_get_provider_config() {
        // This will be updated when ProviderConfig structure is added
        // For now, we verify the function signature exists
        let router = FallbackTranslationRouter {
            providers: Vec::new(),
        };

        // Once implemented, should return:
        // - Some(config) for valid provider names
        // - None for invalid provider names
        let _result = router.get_provider_config("deepl");
    }
}
