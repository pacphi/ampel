// Integration tests for config module only
// These tests run independently without requiring the full translator module to compile

use ampel_i18n_builder::config::*;

#[test]
fn test_default_config_values() {
    let config = Config::default();
    assert_eq!(config.translation.timeout_secs, 30);
    assert_eq!(config.translation.batch_size, 50);
    assert_eq!(config.translation.default_timeout_secs, 30);
    assert_eq!(config.translation.default_batch_size, 50);
    assert_eq!(config.translation.default_max_retries, 3);
}

#[test]
fn test_provider_specific_defaults() {
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
fn test_yaml_minimal_deserialization() {
    let yaml = r#"
translation:
  deepl_api_key: "test-key"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.translation.deepl_api_key, Some("test-key".to_string()));
    assert_eq!(config.translation.timeout_secs, 30);
    assert_eq!(config.translation.providers.deepl.priority, 2);
}

#[test]
fn test_yaml_full_deserialization() {
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
    assert_eq!(config.translation.systran_api_key, Some("systran-key".to_string()));
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
    assert_eq!(config.translation.deepl_api_key, Some("test-key".to_string()));
    assert_eq!(config.translation.timeout_secs, 60);
    assert_eq!(config.translation.batch_size, 100);
    // New fields should use defaults
    assert_eq!(config.translation.default_timeout_secs, 30);
}

#[test]
fn test_providers_config_defaults() {
    let providers = ProvidersConfig::default();

    // Verify all providers have correct tier priorities
    assert_eq!(providers.systran.priority, 1);
    assert_eq!(providers.deepl.priority, 2);
    assert_eq!(providers.google.priority, 3);
    assert_eq!(providers.openai.priority, 4);

    // Verify all providers are enabled by default
    assert!(providers.systran.enabled);
    assert!(providers.deepl.enabled);
    assert!(providers.google.enabled);
    assert!(providers.openai.enabled);
}

#[test]
fn test_retry_configuration() {
    let config = Config::default();

    // Verify retry configuration for each provider
    assert_eq!(config.translation.providers.systran.max_retries, 3);
    assert_eq!(config.translation.providers.systran.retry_delay_ms, 1000);
    assert_eq!(config.translation.providers.systran.max_delay_ms, 30000);
    assert_eq!(config.translation.providers.systran.backoff_multiplier, 2.0);

    assert_eq!(config.translation.providers.deepl.max_retries, 3);
    assert_eq!(config.translation.providers.google.max_retries, 3);
    assert_eq!(config.translation.providers.openai.max_retries, 2);
}

#[test]
fn test_batch_size_configuration() {
    let config = Config::default();

    // Verify batch sizes match design spec
    assert_eq!(config.translation.providers.systran.batch_size, 50);
    assert_eq!(config.translation.providers.deepl.batch_size, 50);
    assert_eq!(config.translation.providers.google.batch_size, 100);
    assert_eq!(config.translation.providers.openai.batch_size, 0); // Unlimited
}

#[test]
fn test_rate_limit_configuration() {
    let config = Config::default();

    // Verify rate limits match design spec
    assert_eq!(config.translation.providers.systran.rate_limit_per_sec, 100);
    assert_eq!(config.translation.providers.deepl.rate_limit_per_sec, 10);
    assert_eq!(config.translation.providers.google.rate_limit_per_sec, 100);
    assert_eq!(config.translation.providers.openai.rate_limit_per_sec, 0); // No limit
}

#[test]
fn test_timeout_configuration() {
    let config = Config::default();

    // Verify timeouts match design spec
    assert_eq!(config.translation.providers.systran.timeout_secs, 45);
    assert_eq!(config.translation.providers.deepl.timeout_secs, 30);
    assert_eq!(config.translation.providers.google.timeout_secs, 30);
    assert_eq!(config.translation.providers.openai.timeout_secs, 60); // Higher for LLM
}
