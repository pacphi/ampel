use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory containing translation files
    #[serde(default = "default_translation_dir")]
    pub translation_dir: PathBuf,

    /// Translation API configuration
    #[serde(default)]
    pub translation: TranslationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    /// DeepL API key (or from DEEPL_API_KEY env var)
    pub deepl_api_key: Option<String>,

    /// Google Cloud Translation API key (or from GOOGLE_API_KEY env var)
    pub google_api_key: Option<String>,

    /// OpenAI API key (or from OPENAI_API_KEY env var)
    pub openai_api_key: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Batch size for translation requests
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            translation_dir: default_translation_dir(),
            translation: TranslationConfig::default(),
        }
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            deepl_api_key: None,
            google_api_key: None,
            openai_api_key: None,
            timeout_secs: default_timeout(),
            batch_size: default_batch_size(),
        }
    }
}

impl Config {
    /// Load configuration from .ampel-i18n.yaml in current directory
    pub fn load() -> crate::error::Result<Self> {
        let config_path = PathBuf::from(".ampel-i18n.yaml");

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let mut config: Config = serde_yaml::from_str(&content)?;

        // Override with environment variables
        if let Ok(key) = std::env::var("DEEPL_API_KEY") {
            config.translation.deepl_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("GOOGLE_API_KEY") {
            config.translation.google_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            config.translation.openai_api_key = Some(key);
        }

        Ok(config)
    }
}

fn default_translation_dir() -> PathBuf {
    PathBuf::from("frontend/public/locales")
}

fn default_timeout() -> u64 {
    30
}

fn default_batch_size() -> usize {
    50
}
