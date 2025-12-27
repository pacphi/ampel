use crate::cli::TranslationProvider;
use crate::config::Config;
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

mod deepl;
mod google;
mod openai;

#[async_trait]
pub trait TranslationService: Send + Sync {
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>>;
}

pub struct Translator {
    service: Box<dyn TranslationService>,
}

impl Translator {
    pub fn new(provider: TranslationProvider, config: &Config) -> Result<Self> {
        let service: Box<dyn TranslationService> = match provider {
            TranslationProvider::DeepL => {
                let api_key = config
                    .translation
                    .deepl_api_key
                    .clone()
                    .or_else(|| std::env::var("DEEPL_API_KEY").ok())
                    .ok_or_else(|| {
                        Error::Config(
                            "DeepL API key not found. Set DEEPL_API_KEY env var or config".to_string(),
                        )
                    })?;

                Box::new(deepl::DeepLTranslator::new(
                    api_key,
                    Duration::from_secs(config.translation.timeout_secs),
                ))
            }
            TranslationProvider::Google => {
                let api_key = config
                    .translation
                    .google_api_key
                    .clone()
                    .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
                    .ok_or_else(|| {
                        Error::Config(
                            "Google API key not found. Set GOOGLE_API_KEY env var or config".to_string(),
                        )
                    })?;

                Box::new(google::GoogleTranslator::new(
                    api_key,
                    Duration::from_secs(config.translation.timeout_secs),
                ))
            }
            TranslationProvider::OpenAI => {
                let api_key = config
                    .translation
                    .openai_api_key
                    .clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or_else(|| {
                        Error::Config(
                            "OpenAI API key not found. Set OPENAI_API_KEY env var or config".to_string(),
                        )
                    })?;

                Box::new(openai::OpenAITranslator::new(
                    api_key,
                    Duration::from_secs(config.translation.timeout_secs),
                ))
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
