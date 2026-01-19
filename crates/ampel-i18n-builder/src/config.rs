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

    /// Systran API key (or from SYSTRAN_API_KEY env var)
    pub systran_api_key: Option<String>,

    /// Request timeout in seconds (global default)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Batch size for translation requests (global default)
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Default timeout for all providers (seconds)
    #[serde(default = "default_timeout")]
    pub default_timeout_secs: u64,

    /// Default batch size for all providers
    #[serde(default = "default_batch_size")]
    pub default_batch_size: usize,

    /// Default maximum retry attempts for all providers
    #[serde(default = "default_max_retries")]
    pub default_max_retries: usize,

    /// Per-provider configuration
    #[serde(default)]
    pub providers: ProvidersConfig,

    /// Fallback behavior configuration
    #[serde(default)]
    pub fallback: FallbackConfig,
}

/// Configuration for all translation providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub systran: ProviderConfig,

    #[serde(default)]
    pub deepl: ProviderConfig,

    #[serde(default)]
    pub google: ProviderConfig,

    #[serde(default)]
    pub openai: ProviderConfig,
}

/// Configuration for a single translation provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Enable or disable this provider
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Provider priority (1 = highest, 4 = lowest)
    #[serde(default = "default_priority")]
    pub priority: u8,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Maximum retry attempts on failure
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Batch size for translation requests
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Rate limit (requests per second)
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_sec: u32,

    /// Initial retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,

    /// Backoff multiplier for exponential backoff
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// Languages where this provider performs best (optional)
    /// If set, provider gets priority for these languages
    /// If empty or None, provider is used based on priority alone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_languages: Option<Vec<String>>,

    /// OpenAI model to use (OpenAI provider only)
    /// Valid models: gpt-5-mini, gpt-5-mini-2025-08-07, gpt-4o, gpt-4o-mini, gpt-4-turbo, etc.
    /// Default: gpt-5-mini
    /// See: https://platform.openai.com/docs/models
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Fallback behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Skip providers without API keys instead of failing
    #[serde(default = "default_skip_on_missing_key")]
    pub skip_on_missing_key: bool,

    /// Stop trying providers after first success
    #[serde(default = "default_stop_on_first_success")]
    pub stop_on_first_success: bool,

    /// Log fallback events (provider failures and fallback attempts)
    #[serde(default = "default_log_fallback_events")]
    pub log_fallback_events: bool,
}

// Default values for Config
impl Default for Config {
    fn default() -> Self {
        Self {
            translation_dir: default_translation_dir(),
            translation: TranslationConfig::default(),
        }
    }
}

// Default values for TranslationConfig
impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            deepl_api_key: None,
            google_api_key: None,
            openai_api_key: None,
            systran_api_key: None,
            timeout_secs: default_timeout(),
            batch_size: default_batch_size(),
            default_timeout_secs: default_timeout(),
            default_batch_size: default_batch_size(),
            default_max_retries: default_max_retries(),
            providers: ProvidersConfig::default(),
            fallback: FallbackConfig::default(),
        }
    }
}

// Default values for ProvidersConfig
impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            systran: ProviderConfig::systran_defaults(),
            deepl: ProviderConfig::deepl_defaults(),
            google: ProviderConfig::google_defaults(),
            openai: ProviderConfig::openai_defaults(),
        }
    }
}

// Default values for ProviderConfig
impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            priority: default_priority(),
            timeout_secs: default_timeout(),
            max_retries: default_max_retries(),
            batch_size: default_batch_size(),
            rate_limit_per_sec: default_rate_limit(),
            retry_delay_ms: default_retry_delay(),
            max_delay_ms: default_max_delay(),
            backoff_multiplier: default_backoff_multiplier(),
            preferred_languages: None,
            model: None, // No default model for other providers
        }
    }
}

// Provider-specific defaults
impl ProviderConfig {
    /// Default configuration for Systran (Tier 1)
    pub fn systran_defaults() -> Self {
        Self {
            enabled: true,
            priority: 1,
            timeout_secs: 45,
            max_retries: 3,
            batch_size: 50,
            rate_limit_per_sec: 100,
            retry_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            preferred_languages: None,
            model: None, // Not applicable
        }
    }

    /// Default configuration for DeepL (Tier 2)
    pub fn deepl_defaults() -> Self {
        Self {
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
            model: None, // Not applicable
        }
    }

    /// Default configuration for Google (Tier 3)
    pub fn google_defaults() -> Self {
        Self {
            enabled: true,
            priority: 3,
            timeout_secs: 30,
            max_retries: 3,
            batch_size: 100,
            rate_limit_per_sec: 100,
            retry_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            preferred_languages: None,
            model: None, // Not applicable
        }
    }

    /// Default configuration for OpenAI (Tier 4)
    pub fn openai_defaults() -> Self {
        Self {
            enabled: true,
            priority: 4,
            timeout_secs: 60,
            max_retries: 2,
            batch_size: 0,         // Unlimited (context window limited)
            rate_limit_per_sec: 0, // No rate limiting
            retry_delay_ms: 2000,
            max_delay_ms: 60000,
            backoff_multiplier: 2.0,
            preferred_languages: None,
            model: Some("gpt-5-mini".to_string()), // Default to gpt-5-mini
        }
    }

    /// Validate provider configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.priority == 0 {
            return Err(crate::error::Error::Config(
                "Provider priority must be >= 1".to_string(),
            ));
        }

        if self.timeout_secs == 0 {
            return Err(crate::error::Error::Config(
                "Provider timeout must be > 0".to_string(),
            ));
        }

        if self.backoff_multiplier < 1.0 {
            return Err(crate::error::Error::Config(
                "Backoff multiplier must be >= 1.0".to_string(),
            ));
        }

        if self.retry_delay_ms == 0 {
            return Err(crate::error::Error::Config(
                "Retry delay must be > 0".to_string(),
            ));
        }

        if self.max_delay_ms < self.retry_delay_ms {
            return Err(crate::error::Error::Config(
                "Max delay must be >= retry delay".to_string(),
            ));
        }

        Ok(())
    }
}

// Default values for FallbackConfig
impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            skip_on_missing_key: default_skip_on_missing_key(),
            stop_on_first_success: default_stop_on_first_success(),
            log_fallback_events: default_log_fallback_events(),
        }
    }
}

impl Config {
    /// Find .ampel-i18n.yaml by searching up the directory tree
    /// Also checks AMPEL_I18N_CONFIG environment variable
    fn find_config_file() -> Option<PathBuf> {
        // First check environment variable
        if let Ok(config_path) = std::env::var("AMPEL_I18N_CONFIG") {
            let path = PathBuf::from(config_path);
            if path.exists() {
                return Some(path);
            }
            tracing::warn!(
                "AMPEL_I18N_CONFIG points to non-existent file: {}",
                path.display()
            );
        }

        // Search up the directory tree from current working directory
        let config_name = ".ampel-i18n.yaml";
        let mut current_dir = std::env::current_dir().ok()?;

        loop {
            let config_path = current_dir.join(config_name);
            if config_path.exists() {
                return Some(config_path);
            }

            // Move to parent directory
            if !current_dir.pop() {
                // Reached filesystem root
                break;
            }
        }

        None
    }

    /// Load configuration from .ampel-i18n.yaml
    /// Searches up the directory tree from current working directory
    /// Also supports AMPEL_I18N_CONFIG environment variable for explicit path
    pub fn load() -> crate::error::Result<Self> {
        let config_path = match Self::find_config_file() {
            Some(path) => {
                tracing::info!("Loading config from: {}", path.display());
                path
            }
            None => {
                tracing::warn!(
                    "No .ampel-i18n.yaml found in directory tree. Using default configuration with ALL providers enabled. \
                     Create .ampel-i18n.yaml in your project root or set AMPEL_I18N_CONFIG environment variable."
                );
                return Ok(Self::default());
            }
        };

        let content = std::fs::read_to_string(&config_path)?;
        let mut config: Config = serde_yaml::from_str(&content)?;

        // Override with environment variables (backward compatibility)
        if let Ok(key) = std::env::var("DEEPL_API_KEY") {
            config.translation.deepl_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("GOOGLE_API_KEY") {
            config.translation.google_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            config.translation.openai_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("SYSTRAN_API_KEY") {
            config.translation.systran_api_key = Some(key);
        }

        // Validate provider configurations
        config.validate()?;

        Ok(config)
    }

    /// Validate entire configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        // Validate each provider config
        self.translation.providers.systran.validate()?;
        self.translation.providers.deepl.validate()?;
        self.translation.providers.google.validate()?;
        self.translation.providers.openai.validate()?;

        // Ensure at least one provider is enabled
        let enabled_count = [
            &self.translation.providers.systran,
            &self.translation.providers.deepl,
            &self.translation.providers.google,
            &self.translation.providers.openai,
        ]
        .iter()
        .filter(|p| p.enabled)
        .count();

        if enabled_count == 0 {
            return Err(crate::error::Error::Config(
                "At least one provider must be enabled".to_string(),
            ));
        }

        // Validate priority uniqueness (warn if duplicates)
        let priorities: Vec<u8> = [
            self.translation.providers.systran.priority,
            self.translation.providers.deepl.priority,
            self.translation.providers.google.priority,
            self.translation.providers.openai.priority,
        ]
        .to_vec();

        let mut sorted_priorities = priorities.clone();
        sorted_priorities.sort_unstable();
        sorted_priorities.dedup();

        if sorted_priorities.len() != priorities.len() {
            // Duplicate priorities found - this is allowed but may cause unexpected behavior
            tracing::warn!(
                "Duplicate provider priorities detected. Providers with same priority will be ordered non-deterministically."
            );
        }

        Ok(())
    }
}

// Default value functions
fn default_translation_dir() -> PathBuf {
    PathBuf::from("frontend/public/locales")
}

fn default_timeout() -> u64 {
    30
}

fn default_batch_size() -> usize {
    50
}

fn default_max_retries() -> usize {
    3
}

fn default_enabled() -> bool {
    true
}

fn default_priority() -> u8 {
    1
}

fn default_rate_limit() -> u32 {
    10
}

fn default_retry_delay() -> u64 {
    1000
}

fn default_max_delay() -> u64 {
    30000
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_skip_on_missing_key() -> bool {
    true
}

fn default_stop_on_first_success() -> bool {
    true
}

fn default_log_fallback_events() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.translation.timeout_secs, 30);
        assert_eq!(config.translation.batch_size, 50);
        assert_eq!(config.translation.default_timeout_secs, 30);
        assert_eq!(config.translation.default_batch_size, 50);
        assert_eq!(config.translation.default_max_retries, 3);
    }

    #[test]
    fn test_provider_defaults() {
        let systran = ProviderConfig::systran_defaults();
        assert_eq!(systran.priority, 1);
        assert_eq!(systran.timeout_secs, 45);
        assert_eq!(systran.batch_size, 50);
        assert_eq!(systran.rate_limit_per_sec, 100);
        assert!(systran.enabled);

        let deepl = ProviderConfig::deepl_defaults();
        assert_eq!(deepl.priority, 2);
        assert_eq!(deepl.timeout_secs, 30);
        assert_eq!(deepl.batch_size, 50);
        assert_eq!(deepl.rate_limit_per_sec, 10);

        let google = ProviderConfig::google_defaults();
        assert_eq!(google.priority, 3);
        assert_eq!(google.batch_size, 100);
        assert_eq!(google.rate_limit_per_sec, 100);

        let openai = ProviderConfig::openai_defaults();
        assert_eq!(openai.priority, 4);
        assert_eq!(openai.timeout_secs, 60);
        assert_eq!(openai.batch_size, 0);
        assert_eq!(openai.rate_limit_per_sec, 0);
    }

    #[test]
    fn test_fallback_config_defaults() {
        let fallback = FallbackConfig::default();
        assert!(fallback.skip_on_missing_key);
        assert!(fallback.stop_on_first_success);
        assert!(fallback.log_fallback_events);
    }

    #[test]
    fn test_yaml_deserialization_minimal() {
        let yaml = r#"
translation:
  deepl_api_key: "test-key"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            config.translation.deepl_api_key,
            Some("test-key".to_string())
        );
        assert_eq!(config.translation.timeout_secs, 30);
        assert_eq!(config.translation.providers.deepl.priority, 2);
    }

    #[test]
    fn test_yaml_deserialization_full() {
        let yaml = r#"
translation:
  systran_api_key: "systran-key"
  deepl_api_key: "deepl-key"
  google_api_key: "google-key"
  openai_api_key: "openai-key"

  default_timeout_secs: 45
  default_batch_size: 100
  default_max_retries: 5

  providers:
    systran:
      enabled: true
      priority: 1
      timeout_secs: 60
      max_retries: 4
      batch_size: 50
      rate_limit_per_sec: 100
      retry_delay_ms: 1000
      max_delay_ms: 30000
      backoff_multiplier: 2.0
      preferred_languages: ["de", "fr", "fi"]

    deepl:
      enabled: true
      priority: 2
      timeout_secs: 30
      preferred_languages: ["sv", "pl", "cs"]

    google:
      enabled: true
      priority: 3
      preferred_languages: ["ar", "th", "vi"]

    openai:
      enabled: false
      priority: 4

  fallback:
    skip_on_missing_key: true
    stop_on_first_success: true
    log_fallback_events: true
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            config.translation.systran_api_key,
            Some("systran-key".to_string())
        );
        assert_eq!(config.translation.default_timeout_secs, 45);
        assert_eq!(config.translation.default_batch_size, 100);
        assert_eq!(config.translation.default_max_retries, 5);

        assert_eq!(config.translation.providers.systran.priority, 1);
        assert_eq!(config.translation.providers.systran.timeout_secs, 60);
        assert_eq!(
            config.translation.providers.systran.preferred_languages,
            Some(vec!["de".to_string(), "fr".to_string(), "fi".to_string()])
        );

        assert_eq!(config.translation.providers.deepl.priority, 2);
        assert_eq!(
            config.translation.providers.deepl.preferred_languages,
            Some(vec!["sv".to_string(), "pl".to_string(), "cs".to_string()])
        );

        assert!(!config.translation.providers.openai.enabled);
        assert!(config.translation.fallback.skip_on_missing_key);
    }

    #[test]
    fn test_provider_validation_zero_priority() {
        let provider = ProviderConfig {
            priority: 0,
            ..Default::default()
        };
        assert!(provider.validate().is_err());
    }

    #[test]
    fn test_provider_validation_zero_timeout() {
        let provider = ProviderConfig {
            timeout_secs: 0,
            ..Default::default()
        };
        assert!(provider.validate().is_err());
    }

    #[test]
    fn test_provider_validation_invalid_backoff() {
        let provider = ProviderConfig {
            backoff_multiplier: 0.5,
            ..Default::default()
        };
        assert!(provider.validate().is_err());
    }

    #[test]
    fn test_provider_validation_invalid_delays() {
        let provider = ProviderConfig {
            retry_delay_ms: 5000,
            max_delay_ms: 1000,
            ..Default::default()
        };
        assert!(provider.validate().is_err());
    }

    #[test]
    fn test_provider_validation_valid() {
        let provider = ProviderConfig::systran_defaults();
        assert!(provider.validate().is_ok());
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_config_validation_no_enabled_providers() {
        let mut config = Config::default();
        config.translation.providers.systran.enabled = false;
        config.translation.providers.deepl.enabled = false;
        config.translation.providers.google.enabled = false;
        config.translation.providers.openai.enabled = false;
        assert!(config.validate().is_err());
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_config_validation_at_least_one_enabled() {
        let mut config = Config::default();
        config.translation.providers.systran.enabled = false;
        config.translation.providers.deepl.enabled = false;
        config.translation.providers.google.enabled = false;
        config.translation.providers.openai.enabled = true;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preferred_languages_serialization() {
        let provider = ProviderConfig {
            preferred_languages: Some(vec!["de".to_string(), "fr".to_string()]),
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&provider).unwrap();
        assert!(yaml.contains("preferred_languages"));
        assert!(yaml.contains("de"));
        assert!(yaml.contains("fr"));
    }

    #[test]
    fn test_preferred_languages_none_skipped() {
        let provider = ProviderConfig::default();
        let yaml = serde_yaml::to_string(&provider).unwrap();
        // Should not serialize preferred_languages if None
        assert!(!yaml.contains("preferred_languages"));
    }

    #[test]
    fn test_backward_compatibility_old_fields() {
        // Ensure old config fields still work
        let yaml = r#"
translation:
  deepl_api_key: "test-key"
  timeout_secs: 60
  batch_size: 100
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            config.translation.deepl_api_key,
            Some("test-key".to_string())
        );
        assert_eq!(config.translation.timeout_secs, 60);
        assert_eq!(config.translation.batch_size, 100);
        // New fields should use defaults
        assert_eq!(config.translation.default_timeout_secs, 30);
    }

    #[test]
    fn test_find_config_file_returns_none_when_not_found() {
        // Temporarily change to a directory without config
        let temp_dir = std::env::temp_dir().join("ampel-i18n-test-no-config");
        std::fs::create_dir_all(&temp_dir).ok();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Clear env var
        std::env::remove_var("AMPEL_I18N_CONFIG");

        let result = Config::find_config_file();
        assert!(result.is_none());

        // Restore
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_find_config_file_uses_env_var() {
        let temp_dir = std::env::temp_dir().join("ampel-i18n-test-env");
        std::fs::create_dir_all(&temp_dir).ok();
        let config_path = temp_dir.join(".ampel-i18n.yaml");
        std::fs::write(&config_path, "translation:\n  timeout_secs: 30").unwrap();

        std::env::set_var("AMPEL_I18N_CONFIG", config_path.to_str().unwrap());

        let result = Config::find_config_file();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), config_path);

        // Cleanup
        std::env::remove_var("AMPEL_I18N_CONFIG");
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_find_config_file_searches_parent_dirs() {
        let temp_dir = std::env::temp_dir().join("ampel-i18n-test-parent");
        let nested_dir = temp_dir.join("sub1").join("sub2");
        std::fs::create_dir_all(&nested_dir).ok();

        // Create config in parent (temp_dir)
        let config_path = temp_dir.join(".ampel-i18n.yaml");
        std::fs::write(&config_path, "translation:\n  timeout_secs: 30").unwrap();

        // Clear env var and change to nested dir
        std::env::remove_var("AMPEL_I18N_CONFIG");
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&nested_dir).unwrap();

        let result = Config::find_config_file();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), config_path);

        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
