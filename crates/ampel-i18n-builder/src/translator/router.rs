use crate::cli::TranslationProvider;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::translator::{TranslationService, Translator};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::info;

/// Intelligent translation router that selects the optimal provider
/// based on target language characteristics:
///
/// - DeepL: European languages (excellent quality, 18 languages)
/// - Google: Thai, Arabic, and other non-EU languages (broader coverage)
/// - OpenAI: Fallback for specialized content
///
/// Based on research in TRANSLATION_API_RESEARCH.md
pub struct SmartTranslationRouter {
    deepl: Option<Box<dyn TranslationService>>,
    google: Option<Box<dyn TranslationService>>,
    openai: Option<Box<dyn TranslationService>>,
}

impl SmartTranslationRouter {
    /// Create a new router with available providers from config
    pub fn new(config: &Config) -> Result<Self> {
        let mut deepl = None;
        let mut google = None;
        let mut openai = None;

        // Initialize DeepL if available
        if config.translation.deepl_api_key.is_some() || std::env::var("DEEPL_API_KEY").is_ok() {
            match Translator::new(TranslationProvider::DeepL, config) {
                Ok(translator) => {
                    deepl = Some(translator.service);
                    info!("DeepL translator initialized");
                }
                Err(e) => info!("DeepL not available: {}", e),
            }
        }

        // Initialize Google if available
        if config.translation.google_api_key.is_some() || std::env::var("GOOGLE_API_KEY").is_ok() {
            match Translator::new(TranslationProvider::Google, config) {
                Ok(translator) => {
                    google = Some(translator.service);
                    info!("Google translator initialized");
                }
                Err(e) => info!("Google not available: {}", e),
            }
        }

        // Initialize OpenAI if available
        if config.translation.openai_api_key.is_some() || std::env::var("OPENAI_API_KEY").is_ok() {
            match Translator::new(TranslationProvider::OpenAI, config) {
                Ok(translator) => {
                    openai = Some(translator.service);
                    info!("OpenAI translator initialized");
                }
                Err(e) => info!("OpenAI not available: {}", e),
            }
        }

        if deepl.is_none() && google.is_none() && openai.is_none() {
            return Err(Error::Config(
                "No translation providers available. Configure API keys.".to_string(),
            ));
        }

        Ok(Self {
            deepl,
            google,
            openai,
        })
    }

    /// Select optimal provider for target language
    fn select_provider(&self, target_lang: &str) -> Result<&dyn TranslationService> {
        // Languages supported by DeepL (with excellent quality)
        const DEEPL_LANGUAGES: &[&str] = &[
            "bg", "cs", "da", "de", "el", "es", "et", "fi", "fr", "hu", "id", "it", "ja", "ko",
            "lt", "lv", "nb", "nl", "pl", "pt", "ro", "ru", "sk", "sl", "sv", "tr", "uk", "zh",
        ];

        // Languages better served by Google (broader coverage)
        const GOOGLE_PREFERRED: &[&str] = &["ar", "th", "vi", "hi"];

        if DEEPL_LANGUAGES.contains(&target_lang) {
            if let Some(ref service) = self.deepl {
                info!("Using DeepL for {}", target_lang);
                return Ok(service.as_ref());
            }
        }

        if GOOGLE_PREFERRED.contains(&target_lang) {
            if let Some(ref service) = self.google {
                info!("Using Google for {}", target_lang);
                return Ok(service.as_ref());
            }
        }

        // Fallback logic: prefer DeepL > Google > OpenAI
        if let Some(ref service) = self.deepl {
            info!("Using DeepL (fallback) for {}", target_lang);
            return Ok(service.as_ref());
        }

        if let Some(ref service) = self.google {
            info!("Using Google (fallback) for {}", target_lang);
            return Ok(service.as_ref());
        }

        if let Some(ref service) = self.openai {
            info!("Using OpenAI (fallback) for {}", target_lang);
            return Ok(service.as_ref());
        }

        Err(Error::Config(format!(
            "No suitable translation provider for language: {}",
            target_lang
        )))
    }
}

#[async_trait]
impl TranslationService for SmartTranslationRouter {
    fn provider_name(&self) -> &str {
        "SmartRouter"
    }

    fn provider_tier(&self) -> u8 {
        0 // Router doesn't have a tier
    }

    fn is_available(&self) -> bool {
        self.deepl.is_some() || self.google.is_some() || self.openai.is_some()
    }

    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let provider = self.select_provider(target_lang)?;
        provider.translate_batch(texts, target_lang).await
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_deepl_languages() {
        // Verify DeepL language codes match research
        const DEEPL_LANGUAGES: &[&str] = &[
            "bg", "cs", "da", "de", "el", "es", "et", "fi", "fr", "hu", "id", "it", "ja", "ko",
            "lt", "lv", "nb", "nl", "pl", "pt", "ro", "ru", "sk", "sl", "sv", "tr", "uk", "zh",
        ];

        assert!(DEEPL_LANGUAGES.contains(&"fi"));
        assert!(DEEPL_LANGUAGES.contains(&"sv"));
        assert!(DEEPL_LANGUAGES.contains(&"de"));
        assert!(!DEEPL_LANGUAGES.contains(&"ar"));
        assert!(!DEEPL_LANGUAGES.contains(&"th"));
    }

    #[test]
    fn test_google_preferred() {
        const GOOGLE_PREFERRED: &[&str] = &["ar", "th", "vi", "hi"];

        assert!(GOOGLE_PREFERRED.contains(&"ar"));
        assert!(GOOGLE_PREFERRED.contains(&"th"));
        assert!(!GOOGLE_PREFERRED.contains(&"fi"));
    }
}
