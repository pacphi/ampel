use crate::cli::TranslationProvider;
use crate::config::Config;
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

pub mod cache;
pub(crate) mod deepl;
pub mod fallback;
pub(crate) mod google;
pub(crate) mod openai;
pub mod router;
pub(crate) mod systran;

/// Provider configuration for a single translation provider
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// API key for the provider
    pub api_key: String,

    /// Request timeout duration
    pub timeout: Duration,

    /// Maximum retry attempts on failure
    pub max_retries: usize,

    /// Batch size for translation requests
    pub batch_size: usize,

    /// Rate limit (requests per second)
    pub rate_limit_per_sec: u32,

    /// Initial retry delay in milliseconds
    pub retry_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,

    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl ProviderConfig {
    /// Create default config with API key
    pub fn new(api_key: String, timeout: Duration) -> Self {
        Self {
            api_key,
            timeout,
            max_retries: 3,
            batch_size: 50,
            rate_limit_per_sec: 10,
            retry_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

#[async_trait]
pub trait TranslationService: Send + Sync {
    /// Translate a batch of texts to target language
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>>;

    /// Get provider name (for logging)
    fn provider_name(&self) -> &str;

    /// Get provider tier (1-4)
    fn provider_tier(&self) -> u8;

    /// Check if provider is available (API key configured)
    fn is_available(&self) -> bool;
}

pub struct Translator {
    service: Box<dyn TranslationService>,
}

impl Translator {
    pub fn new(provider: TranslationProvider, config: &Config) -> Result<Self> {
        let service: Box<dyn TranslationService> = match provider {
            TranslationProvider::Systran => {
                let api_key = config
                    .translation
                    .systran_api_key
                    .clone()
                    .or_else(|| std::env::var("SYSTRAN_API_KEY").ok())
                    .ok_or_else(|| {
                        Error::Config(
                            "Systran API key not found. Set SYSTRAN_API_KEY env var or config"
                                .to_string(),
                        )
                    })?;

                Box::new(systran::SystranTranslator::new(
                    api_key,
                    Duration::from_secs(config.translation.timeout_secs),
                ))
            }
            TranslationProvider::DeepL => {
                let api_key = config
                    .translation
                    .deepl_api_key
                    .clone()
                    .or_else(|| std::env::var("DEEPL_API_KEY").ok())
                    .ok_or_else(|| {
                        Error::Config(
                            "DeepL API key not found. Set DEEPL_API_KEY env var or config"
                                .to_string(),
                        )
                    })?;

                let provider_config = ProviderConfig {
                    api_key,
                    timeout: Duration::from_secs(config.translation.timeout_secs),
                    max_retries: 3,
                    batch_size: 50,
                    rate_limit_per_sec: 10,
                    retry_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                };

                Box::new(deepl::DeepLTranslator::new(provider_config)?)
            }
            TranslationProvider::Google => {
                let api_key = config
                    .translation
                    .google_api_key
                    .clone()
                    .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
                    .ok_or_else(|| {
                        Error::Config(
                            "Google API key not found. Set GOOGLE_API_KEY env var or config"
                                .to_string(),
                        )
                    })?;

                Box::new(google::GoogleTranslator::new(
                    api_key,
                    Duration::from_secs(config.translation.timeout_secs),
                )?)
            }
            TranslationProvider::OpenAI => {
                let api_key = config
                    .translation
                    .openai_api_key
                    .clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or_else(|| {
                        Error::Config(
                            "OpenAI API key not found. Set OPENAI_API_KEY env var or config"
                                .to_string(),
                        )
                    })?;

                let model = config.translation.providers.openai.model.clone();

                Box::new(openai::OpenAITranslator::new(
                    api_key,
                    Duration::from_secs(config.translation.timeout_secs),
                    model,
                )?)
            }
        };

        Ok(Self { service })
    }

    pub async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        self.service.translate_batch(texts, target_lang).await
    }
}

#[async_trait]
impl TranslationService for Translator {
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        self.service.translate_batch(texts, target_lang).await
    }

    fn provider_name(&self) -> &str {
        self.service.provider_name()
    }

    fn provider_tier(&self) -> u8 {
        self.service.provider_tier()
    }

    fn is_available(&self) -> bool {
        self.service.is_available()
    }
}
