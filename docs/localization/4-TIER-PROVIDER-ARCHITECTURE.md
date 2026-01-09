# 4-Tier Translation Provider Architecture

**Version**: 1.0
**Date**: 2025-12-28
**Status**: Design Phase
**Author**: System Architecture Designer

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Architecture Analysis](#current-architecture-analysis)
3. [Proposed Architecture](#proposed-architecture)
4. [Configuration Schema](#configuration-schema)
5. [Provider Interface Specifications](#provider-interface-specifications)
6. [Fallback Flow Diagrams](#fallback-flow-diagrams)
7. [Implementation Plan](#implementation-plan)
8. [Quality Attributes](#quality-attributes)
9. [Trade-offs and Risks](#trade-offs-and-risks)
10. [Testing Strategy](#testing-strategy)

---

## Executive Summary

This document defines a robust 4-tier translation provider architecture with intelligent fallback, configurable retry mechanisms, and per-provider timeout controls. The architecture addresses reliability concerns by gracefully degrading through multiple translation providers when failures occur.

### Key Features

- **4-Tier Provider Hierarchy**: Systran (Tier 1) → DeepL (Tier 2) → Google (Tier 3) → OpenAI (Tier 4)
- **Smart Fallback**: Automatic provider selection based on language characteristics and availability
- **Configurable Retry**: Per-provider retry attempts with exponential backoff
- **Flexible Timeouts**: Independent timeout configuration for each provider
- **Batch Size Control**: Configurable batch sizes to prevent timeout issues
- **Skip-on-Missing**: Gracefully skip providers without API keys

---

## Current Architecture Analysis

### Strengths

1. **Trait-Based Abstraction**: Clean `TranslationService` trait allows easy provider addition
2. **Retry Logic**: Exponential backoff implemented in Google/DeepL providers
3. **Rate Limiting**: Token bucket rate limiting prevents API quota exhaustion
4. **LRU Caching**: 1000-entry cache reduces redundant API calls
5. **Smart Router**: Language-based provider selection in `SmartTranslationRouter`

### Weaknesses

1. **Single Provider Failure**: If selected provider fails, entire translation fails
2. **No Fallback Chain**: Router selects one provider; no retry with alternatives
3. **Hardcoded Defaults**: Retry/timeout values embedded in provider implementations
4. **Limited Configurability**: No CLI override for batch size, retries, timeouts
5. **Missing Systran**: No support for Systran Translation API (Tier 1 requirement)

### Current Provider Capabilities

| Provider | Batch Size | Retry      | Rate Limit  | Timeout | Cache |
| -------- | ---------- | ---------- | ----------- | ------- | ----- |
| DeepL    | 50 texts   | 3 attempts | 10 req/sec  | 30s     | ✅    |
| Google   | 100 texts  | 3 attempts | 100 req/sec | 30s     | ✅    |
| OpenAI   | N/A        | ❌         | ❌          | 30s     | ❌    |

---

## Proposed Architecture

### Architecture Principles

1. **Resilience**: System continues operating despite individual provider failures
2. **Configurability**: All parameters adjustable via YAML, TOML, or CLI flags
3. **Observability**: Comprehensive logging of provider selection and fallback events
4. **Extensibility**: Easy addition of new providers without modifying core logic
5. **Performance**: Minimize overhead while maintaining reliability

### Component Diagram (C4 Model - Level 2)

```
┌─────────────────────────────────────────────────────────────────┐
│                   Translation CLI Application                   │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
         ┌───────────────────────────────────────┐
         │   FallbackTranslationRouter           │
         │   - Provider priority queue           │
         │   - Retry/timeout coordination        │
         │   - Batch size management             │
         └───┬───────────────────────────────────┘
             │
    ┌────────┴──────────┬──────────┬─────────────┐
    ▼                   ▼          ▼             ▼
┌─────────┐       ┌─────────┐  ┌────────┐  ┌──────────┐
│ Systran │       │  DeepL  │  │ Google │  │  OpenAI  │
│ Tier 1  │       │ Tier 2  │  │ Tier 3 │  │  Tier 4  │
└─────────┘       └─────────┘  └────────┘  └──────────┘
   100 req/s         10 req/s    100 req/s   Unlimited
   50 texts/batch    50 texts    100 texts   N/A
   Enterprise        High Quality Broad       Fallback
```

### Provider Tier Definitions

#### Tier 1: Systran Translation API (NEW)

- **Purpose**: Primary enterprise translation with neural MT
- **Strengths**: High accuracy, domain-specific models, fast response
- **Rate Limit**: 100 requests/sec (estimated)
- **Batch Size**: 50 texts per request
- **Best For**: All languages when API key available
- **Cost**: Enterprise pricing (pay-per-character)

#### Tier 2: DeepL API

- **Purpose**: High-quality European language translation
- **Strengths**: Best-in-class quality for 28 languages
- **Rate Limit**: 10 requests/sec
- **Batch Size**: 50 texts per request
- **Best For**: EU languages (de, fr, fi, sv, pl, cs, etc.)
- **Cost**: Free tier available, paid tier recommended

#### Tier 3: Google Translate API

- **Purpose**: Broad language coverage and reliability
- **Strengths**: 133+ languages, robust infrastructure
- **Rate Limit**: 100 requests/sec
- **Batch Size**: 100 texts per request
- **Best For**: Asian/Middle Eastern languages (ar, th, vi, zh, ja)
- **Cost**: Pay-per-character

#### Tier 4: OpenAI GPT-4.5 API

- **Purpose**: Fallback for specialized content and edge cases
- **Strengths**: Context-aware translation, placeholder preservation
- **Rate Limit**: No specific limit (token-based)
- **Batch Size**: Unlimited (limited by context window)
- **Best For**: Technical content, complex placeholders
- **Cost**: High (token-based pricing)

---

## Configuration Schema

### YAML Configuration (.ampel-i18n.yaml)

```yaml
translation:
  # Provider API Keys (also read from env vars)
  systran_api_key: '${SYSTRAN_API_KEY}' # Optional: use env var
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'

  # Global Defaults
  default_timeout_secs: 30
  default_batch_size: 50
  default_max_retries: 3

  # Provider-Specific Configuration
  providers:
    systran:
      enabled: true
      priority: 1 # Tier 1 (highest)
      timeout_secs: 45
      max_retries: 3
      batch_size: 50
      rate_limit_per_sec: 100
      retry_delay_ms: 1000
      max_delay_ms: 30000
      backoff_multiplier: 2.0
      # Optional: Specify languages where this provider excels
      # If not set, provider is used for all languages based on priority
      # preferred_languages: ["de", "fr", "fi", "sv", "pl", "cs"]

    deepl:
      enabled: true
      priority: 2 # Tier 2
      timeout_secs: 30
      max_retries: 3
      batch_size: 50
      rate_limit_per_sec: 10
      retry_delay_ms: 1000
      max_delay_ms: 30000
      backoff_multiplier: 2.0
      # Optional: Language preferences for this provider
      # Uncomment and customize based on provider strengths
      # preferred_languages: ["bg", "cs", "da", "de", "el", "es", "et", "fi", "fr", "hu", "id", "it", "ja", "ko", "lt", "lv", "nb", "nl", "pl", "pt", "ro", "ru", "sk", "sl", "sv", "tr", "uk", "zh"]

    google:
      enabled: true
      priority: 3 # Tier 3
      timeout_secs: 30
      max_retries: 3
      batch_size: 100
      rate_limit_per_sec: 100
      retry_delay_ms: 1000
      max_delay_ms: 30000
      backoff_multiplier: 2.0
      # Optional: Language preferences for this provider
      # Google excels at Asian and Middle Eastern languages
      # preferred_languages: ["ar", "th", "vi", "hi", "zh", "ja", "ko"]

    openai:
      enabled: true
      priority: 4 # Tier 4 (fallback)
      timeout_secs: 60 # Higher timeout for LLM
      max_retries: 2
      batch_size: 0 # Unlimited (context window limited)
      rate_limit_per_sec: 0 # No rate limiting
      retry_delay_ms: 2000
      max_delay_ms: 60000
      backoff_multiplier: 2.0
      model: 'gpt-4o' # Model selection
      temperature: 0.3
      # Optional: Language preferences (typically not needed for OpenAI)
      # OpenAI handles all languages well but is expensive, so use as fallback
      # preferred_languages: []

  # Fallback Strategy
  fallback:
    skip_on_missing_key: true # Skip providers without API keys
    stop_on_first_success: true # Don't try more providers after success
    log_fallback_events: true # Log when falling back to next tier
```

### TOML Configuration (Alternative)

```toml
[translation]
systran_api_key = "${SYSTRAN_API_KEY}"
deepl_api_key = "${DEEPL_API_KEY}"
google_api_key = "${GOOGLE_API_KEY}"
openai_api_key = "${OPENAI_API_KEY}"

default_timeout_secs = 30
default_batch_size = 50
default_max_retries = 3

[translation.providers.systran]
enabled = true
priority = 1
timeout_secs = 45
max_retries = 3
batch_size = 50
rate_limit_per_sec = 100

[translation.providers.deepl]
enabled = true
priority = 2
timeout_secs = 30
max_retries = 3
batch_size = 50
rate_limit_per_sec = 10

[translation.providers.google]
enabled = true
priority = 3
timeout_secs = 30
max_retries = 3
batch_size = 100
rate_limit_per_sec = 100

[translation.providers.openai]
enabled = true
priority = 4
timeout_secs = 60
max_retries = 2
batch_size = 0
rate_limit_per_sec = 0
model = "gpt-4o"
temperature = 0.3

[translation.fallback]
skip_on_missing_key = true
stop_on_first_success = true
log_fallback_events = true
```

### CLI Parameter Overrides

```bash
# Override timeout for all providers
cargo i18n translate --lang fi \
  --timeout 60

# Override batch size
cargo i18n translate --lang fi \
  --batch-size 25

# Override retry attempts
cargo i18n translate --lang fi \
  --max-retries 5

# Disable specific providers
cargo i18n translate --lang fi \
  --disable-provider openai

# Force specific provider (no fallback)
cargo i18n translate --lang fi \
  --provider systran \
  --no-fallback

# Override per-provider settings
cargo i18n translate --lang fi \
  --systran-timeout 60 \
  --deepl-retries 5 \
  --google-batch-size 50
```

---

## Provider Interface Specifications

### Core Trait (Updated)

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

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

/// Translation service trait (unchanged)
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
```

### Systran Provider Implementation

```rust
use crate::error::{Error, Result};
use crate::translator::{ProviderConfig, TranslationService};
use async_trait::async_trait;
use governor::{Quota, RateLimiter};
use lru::LruCache;
use nonzero_ext::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Systran Translation API client (Tier 1)
pub struct SystranTranslator {
    client: reqwest::Client,
    config: ProviderConfig,
    cache: Arc<Mutex<LruCache<CacheKey, String>>>,
    rate_limiter: Arc<RateLimiter</*...*/>> ,
    usage_chars: Arc<Mutex<u64>>,
    usage_calls: Arc<Mutex<u64>>,
    cache_hits: Arc<Mutex<u64>>,
}

#[derive(Serialize)]
struct SystranRequest {
    input: Vec<String>,
    target: String,
    source: String,
}

#[derive(Deserialize)]
struct SystranResponse {
    outputs: Vec<SystranOutput>,
}

#[derive(Deserialize)]
struct SystranOutput {
    output: String,
}

impl SystranTranslator {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent("ampel-i18n-builder/1.0.0")
            .build()
            .expect("Failed to build HTTP client");

        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(nonzero!(config.rate_limit_per_sec))
        ));

        let cache_capacity = NonZeroUsize::new(1000).unwrap();
        let cache = Arc::new(Mutex::new(LruCache::new(cache_capacity)));

        Self {
            client,
            config,
            cache,
            rate_limiter,
            usage_chars: Arc::new(Mutex::new(0)),
            usage_calls: Arc::new(Mutex::new(0)),
            cache_hits: Arc::new(Mutex::new(0)),
        }
    }

    async fn translate_with_retry(
        &self,
        texts: &[String],
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let mut attempt = 0;
        let mut delay = self.config.retry_delay_ms;

        loop {
            // Wait for rate limiter
            self.rate_limiter.until_ready().await;

            let request = SystranRequest {
                input: texts.to_vec(),
                target: target_lang.to_string(),
                source: "en".to_string(),
            };

            let response = self
                .client
                .post("https://api-translate.systran.net/translation/text/translate")
                .header("Authorization", format!("Key {}", self.config.api_key))
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    if status.is_success() {
                        let systran_response: SystranResponse = resp.json().await?;
                        return Ok(systran_response
                            .outputs
                            .into_iter()
                            .map(|o| o.output)
                            .collect());
                    }

                    let error_body = resp.text().await.unwrap_or_default();
                    let is_retryable = matches!(
                        status.as_u16(),
                        408 | 429 | 500 | 502 | 503 | 504
                    );

                    if !is_retryable {
                        return Err(Error::Api(format!(
                            "Systran API error {}: {}",
                            status, error_body
                        )));
                    }

                    attempt += 1;
                    if attempt >= self.config.max_retries {
                        return Err(Error::Api(format!(
                            "Systran: Max retries ({}) exceeded. Last error: {} {}",
                            self.config.max_retries, status, error_body
                        )));
                    }

                    warn!(
                        "Systran API request failed (attempt {}/{}): {} {}",
                        attempt, self.config.max_retries, status, error_body
                    );
                }
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.config.max_retries {
                        return Err(Error::Api(format!(
                            "Systran: Network error after {} attempts: {}",
                            attempt, e
                        )));
                    }

                    warn!(
                        "Systran API network error (attempt {}/{}): {}",
                        attempt, self.config.max_retries, e
                    );
                }
            }

            // Exponential backoff with jitter
            let jitter = (delay as f64 * 0.1 * random_f64()) as u64;
            let sleep_duration = std::cmp::min(
                delay + jitter,
                self.config.max_delay_ms
            );

            debug!("Retrying Systran in {}ms...", sleep_duration);
            tokio::time::sleep(std::time::Duration::from_millis(sleep_duration)).await;

            delay = (delay as f64 * self.config.backoff_multiplier) as u64;
        }
    }
}

#[async_trait]
impl TranslationService for SystranTranslator {
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Implementation similar to DeepL/Google
        // (Extract texts, check cache, batch translate, merge results)
        todo!("Implement batch translation logic")
    }

    fn provider_name(&self) -> &str {
        "Systran"
    }

    fn provider_tier(&self) -> u8 {
        1
    }

    fn is_available(&self) -> bool {
        !self.config.api_key.is_empty()
    }
}
```

### Fallback Router Implementation

```rust
use crate::config::Config;
use crate::error::{Error, Result};
use crate::translator::TranslationService;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Fallback translation router with configurable retry and timeout
pub struct FallbackTranslationRouter {
    providers: Vec<Box<dyn TranslationService>>,
    config: Config,
}

impl FallbackTranslationRouter {
    /// Create router with all available providers
    pub fn new(config: &Config) -> Result<Self> {
        let mut providers: Vec<Box<dyn TranslationService>> = Vec::new();

        // Initialize providers in priority order (Tier 1 → Tier 4)

        // Tier 1: Systran
        if let Some(api_key) = config.translation.systran_api_key.clone()
            .or_else(|| std::env::var("SYSTRAN_API_KEY").ok())
        {
            if config.translation.providers.systran.enabled {
                let provider_config = ProviderConfig::from_config(
                    &config.translation.providers.systran,
                    api_key
                );
                match SystranTranslator::new(provider_config) {
                    Ok(translator) => {
                        info!("✓ Systran translator initialized (Tier 1)");
                        providers.push(Box::new(translator));
                    }
                    Err(e) => warn!("Systran initialization failed: {}", e),
                }
            }
        } else if config.translation.fallback.skip_on_missing_key {
            info!("⊘ Systran skipped (no API key configured)");
        }

        // Tier 2: DeepL
        if let Some(api_key) = config.translation.deepl_api_key.clone()
            .or_else(|| std::env::var("DEEPL_API_KEY").ok())
        {
            if config.translation.providers.deepl.enabled {
                let provider_config = ProviderConfig::from_config(
                    &config.translation.providers.deepl,
                    api_key
                );
                match DeepLTranslator::new(provider_config) {
                    Ok(translator) => {
                        info!("✓ DeepL translator initialized (Tier 2)");
                        providers.push(Box::new(translator));
                    }
                    Err(e) => warn!("DeepL initialization failed: {}", e),
                }
            }
        } else if config.translation.fallback.skip_on_missing_key {
            info!("⊘ DeepL skipped (no API key configured)");
        }

        // Tier 3: Google
        if let Some(api_key) = config.translation.google_api_key.clone()
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
        {
            if config.translation.providers.google.enabled {
                let provider_config = ProviderConfig::from_config(
                    &config.translation.providers.google,
                    api_key
                );
                match GoogleTranslator::new(provider_config) {
                    Ok(translator) => {
                        info!("✓ Google translator initialized (Tier 3)");
                        providers.push(Box::new(translator));
                    }
                    Err(e) => warn!("Google initialization failed: {}", e),
                }
            }
        } else if config.translation.fallback.skip_on_missing_key {
            info!("⊘ Google skipped (no API key configured)");
        }

        // Tier 4: OpenAI
        if let Some(api_key) = config.translation.openai_api_key.clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        {
            if config.translation.providers.openai.enabled {
                let provider_config = ProviderConfig::from_config(
                    &config.translation.providers.openai,
                    api_key
                );
                match OpenAITranslator::new(provider_config) {
                    Ok(translator) => {
                        info!("✓ OpenAI translator initialized (Tier 4)");
                        providers.push(Box::new(translator));
                    }
                    Err(e) => warn!("OpenAI initialization failed: {}", e),
                }
            }
        } else if config.translation.fallback.skip_on_missing_key {
            info!("⊘ OpenAI skipped (no API key configured)");
        }

        if providers.is_empty() {
            return Err(Error::Config(
                "No translation providers available. Configure at least one API key."
                    .to_string(),
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

        Ok(Self {
            providers,
            config: config.clone(),
        })
    }

    /// Select optimal providers for target language based on configuration
    ///
    /// This method orders providers dynamically based on:
    /// 1. Language preferences (if configured per provider)
    /// 2. Default priority order (Tier 1 → 2 → 3 → 4)
    ///
    /// If a provider has `preferred_languages` configured and the target language
    /// matches, that provider gets priority. Otherwise, uses default tier ordering.
    fn select_providers(&self, target_lang: &str) -> Vec<&Box<dyn TranslationService>> {
        // Separate providers into preferred and non-preferred for this language
        let mut preferred_providers = Vec::new();
        let mut other_providers = Vec::new();

        for provider in &self.providers {
            // Check if provider has language preferences configured
            let provider_config = self.get_provider_config(provider.provider_name());

            if let Some(config) = provider_config {
                if let Some(ref preferred_langs) = config.preferred_languages {
                    if !preferred_langs.is_empty() && preferred_langs.contains(&target_lang.to_string()) {
                        // This provider prefers this language
                        preferred_providers.push(provider);
                        continue;
                    }
                }
            }

            // No language preference or language not in preferences
            other_providers.push(provider);
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
            .chain(other_providers.into_iter())
            .collect()
    }

    /// Get provider configuration by name
    fn get_provider_config(&self, provider_name: &str) -> Option<&ProviderConfig> {
        match provider_name.to_lowercase().as_str() {
            "systran" => Some(&self.config.translation.providers.systran),
            "deepl" => Some(&self.config.translation.providers.deepl),
            "google" => Some(&self.config.translation.providers.google),
            "openai" => Some(&self.config.translation.providers.openai),
            _ => None,
        }
    }
}

#[async_trait]
impl TranslationService for FallbackTranslationRouter {
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let providers = self.select_providers(target_lang);

        let mut last_error = None;

        for (index, provider) in providers.iter().enumerate() {
            let provider_name = provider.provider_name();
            let provider_tier = provider.provider_tier();

            info!(
                "Attempting translation with {} (Tier {})...",
                provider_name, provider_tier
            );

            match provider.translate_batch(texts, target_lang).await {
                Ok(result) => {
                    info!(
                        "✓ Translation successful with {} (Tier {})",
                        provider_name, provider_tier
                    );

                    if self.config.translation.fallback.log_fallback_events && index > 0 {
                        warn!(
                            "Used fallback provider {} (Tier {}) after {} failure(s)",
                            provider_name, provider_tier, index
                        );
                    }

                    return Ok(result);
                }
                Err(e) => {
                    error!(
                        "✗ {} (Tier {}) failed: {}",
                        provider_name, provider_tier, e
                    );
                    last_error = Some(e);

                    if self.config.translation.fallback.stop_on_first_success {
                        continue; // Try next provider
                    }
                }
            }
        }

        // All providers failed
        Err(last_error.unwrap_or_else(|| {
            Error::Translation(
                "All translation providers failed or unavailable".to_string()
            )
        }))
    }

    fn provider_name(&self) -> &str {
        "FallbackRouter"
    }

    fn provider_tier(&self) -> u8 {
        0 // Router is tier 0 (orchestrator)
    }

    fn is_available(&self) -> bool {
        !self.providers.is_empty()
    }
}
```

---

## Fallback Flow Diagrams

### Provider Selection Flow

```
┌─────────────────────────────────────────────────────┐
│          Translation Request Received                │
│         (target_lang, texts)                         │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
      ┌────────────────────────────────┐
      │  Load Configuration            │
      │  - Provider priorities         │
      │  - Timeout/retry settings      │
      │  - Batch size limits           │
      │  - Language preferences        │
      └─────────────┬──────────────────┘
                    │
                    ▼
      ┌────────────────────────────────┐
      │  Select Provider Order         │
      │  For Each Provider:            │
      │  1. Check preferred_languages  │
      │  2. Match target language?     │
      │  3. Group into:                │
      │     - Preferred (matched)      │
      │     - Other (not matched)      │
      │  4. Sort each group by tier    │
      │  5. Preferred providers first  │
      └─────────────┬──────────────────┘
                    │
                    ▼
      ┌────────────────────────────────┐
      │  Filter Available Providers    │
      │  - Skip if API key missing     │
      │  - Skip if disabled in config  │
      └─────────────┬──────────────────┘
                    │
                    ▼
      ┌────────────────────────────────┐
      │  Iterate Through Providers     │
      │  (Tier 1 → Tier 2 → ... → 4)   │
      └─────────────┬──────────────────┘
                    │
                    ▼
         ╔══════════════════════════════════╗
         ║  For Each Provider:               ║
         ╠══════════════════════════════════╣
         ║  1. Check cache                   ║
         ║  2. Apply rate limiting           ║
         ║  3. Send API request              ║
         ║  4. Retry on failure (0-5x)       ║
         ║  5. Return on success             ║
         ╚════════┬═════════════════════════╝
                  │
       ┌──────────┴──────────┐
       │                     │
       ▼                     ▼
   Success?              Failure?
       │                     │
       ▼                     ▼
  ┌────────┐         ┌──────────────┐
  │ Return │         │ Try Next Tier│
  │ Result │         │ (if exists)  │
  └────────┘         └──────┬───────┘
                            │
                            ▼
                     ┌──────────────┐
                     │ All Failed?  │
                     └──────┬───────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │ Return Error    │
                   └─────────────────┘
```

### Retry Flow with Exponential Backoff

```
┌──────────────────────────────────────┐
│  API Request Sent to Provider        │
└────────────┬─────────────────────────┘
             │
             ▼
      ┌──────────────┐
      │  Response?   │
      └──────┬───────┘
             │
    ┌────────┴────────┐
    │                 │
    ▼                 ▼
Success          HTTP Error
    │                 │
    │                 ▼
    │          ┌────────────────┐
    │          │  Retryable?    │
    │          │  (408/429/5xx) │
    │          └────────┬───────┘
    │                   │
    │           ┌───────┴────────┐
    │           │                │
    │           ▼                ▼
    │         Yes              No
    │           │                │
    │           │                ▼
    │           │         ┌──────────────┐
    │           │         │ Return Error │
    │           │         └──────────────┘
    │           │
    │           ▼
    │    ┌─────────────────┐
    │    │ Attempt < Max?  │
    │    └────────┬────────┘
    │             │
    │      ┌──────┴──────┐
    │      │             │
    │      ▼             ▼
    │     Yes           No
    │      │             │
    │      │             ▼
    │      │      ┌──────────────┐
    │      │      │ Return Error │
    │      │      └──────────────┘
    │      │
    │      ▼
    │   ┌─────────────────────────┐
    │   │ Calculate Backoff Delay │
    │   │ delay = initial * 2^n   │
    │   │ + jitter (10%)          │
    │   └────────┬────────────────┘
    │            │
    │            ▼
    │   ┌─────────────────────────┐
    │   │ Sleep(delay)            │
    │   │ Max: 30s (configurable) │
    │   └────────┬────────────────┘
    │            │
    │            ▼
    │   ┌─────────────────────────┐
    │   │ Increment Attempt       │
    │   └────────┬────────────────┘
    │            │
    │            └────────┐
    │                     │
    │◄────────────────────┘
    │   (Loop back to API request)
    │
    ▼
┌────────────────┐
│ Return Success │
└────────────────┘
```

### Batch Size Management Flow

```
┌──────────────────────────────────────┐
│  Texts to Translate (N texts)        │
└────────────┬─────────────────────────┘
             │
             ▼
   ┌──────────────────────────┐
   │ Get Provider Batch Size  │
   │ - Systran: 50            │
   │ - DeepL: 50              │
   │ - Google: 100            │
   │ - OpenAI: Unlimited      │
   └────────────┬─────────────┘
                │
                ▼
   ┌──────────────────────────┐
   │ Split into Chunks        │
   │ chunk_size = min(        │
   │   N,                     │
   │   provider.batch_size,   │
   │   cli_override           │
   │ )                        │
   └────────────┬─────────────┘
                │
                ▼
   ┌──────────────────────────┐
   │ For Each Chunk:          │
   │ 1. Check cache           │
   │ 2. Translate uncached    │
   │ 3. Merge results         │
   └────────────┬─────────────┘
                │
                ▼
   ┌──────────────────────────┐
   │ Combine All Chunks       │
   │ into Final Result        │
   └────────────┬─────────────┘
                │
                ▼
         ┌──────────────┐
         │ Return Result│
         └──────────────┘
```

---

## Implementation Plan

### Phase 1: Configuration Infrastructure (1 day)

#### File: `crates/ampel-i18n-builder/src/config.rs`

**Changes**:

1. Add `ProviderConfig` struct for per-provider settings
2. Add `FallbackConfig` struct for fallback behavior
3. Extend `TranslationConfig` with `providers` field
4. Update `Config::load()` to parse new YAML structure
5. Add validation for provider priorities

**New Structs**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub priority: u8,
    pub timeout_secs: u64,
    pub max_retries: usize,
    pub batch_size: usize,
    pub rate_limit_per_sec: u32,
    pub retry_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    #[serde(default)]
    pub preferred_languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub skip_on_missing_key: bool,
    pub stop_on_first_success: bool,
    pub log_fallback_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub systran: ProviderConfig,
    pub deepl: ProviderConfig,
    pub google: ProviderConfig,
    pub openai: ProviderConfig,
}
```

**Tests**:

- `test_config_parse_yaml()`: Parse complete YAML with all providers
- `test_config_defaults()`: Verify default values
- `test_config_env_override()`: Environment variable precedence
- `test_invalid_priority()`: Reject invalid priority values

---

### Phase 2: Provider Base Infrastructure (1 day)

#### File: `crates/ampel-i18n-builder/src/translator/mod.rs`

**Changes**:

1. Add `ProviderConfig` struct (moved from config)
2. Update `TranslationService` trait with new methods
3. Create `ProviderFactory` for provider instantiation

**New Code**:

```rust
/// Factory for creating translation providers
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create_systran(config: ProviderConfig) -> Result<Box<dyn TranslationService>> {
        Ok(Box::new(systran::SystranTranslator::new(config)?))
    }

    pub fn create_deepl(config: ProviderConfig) -> Result<Box<dyn TranslationService>> {
        Ok(Box::new(deepl::DeepLTranslator::new(config)?))
    }

    pub fn create_google(config: ProviderConfig) -> Result<Box<dyn TranslationService>> {
        Ok(Box::new(google::GoogleTranslator::new(config)?))
    }

    pub fn create_openai(config: ProviderConfig) -> Result<Box<dyn TranslationService>> {
        Ok(Box::new(openai::OpenAITranslator::new(config)?))
    }
}
```

**Tests**:

- `test_provider_factory()`: Create all provider types
- `test_provider_trait_methods()`: Verify trait implementation

---

### Phase 3: Systran Provider Implementation (2 days)

#### File: `crates/ampel-i18n-builder/src/translator/systran.rs` (NEW)

**Implementation**:

1. `SystranTranslator` struct with `ProviderConfig`
2. HTTP client with configurable timeout
3. Rate limiting (100 req/sec default)
4. LRU caching (1000 entries)
5. Exponential backoff retry logic
6. Batch translation (50 texts per request)
7. Usage metrics tracking

**API Endpoint**: `https://api-translate.systran.net/translation/text/translate`

**Request Format**:

```json
{
  "input": ["Hello", "World"],
  "source": "en",
  "target": "fi"
}
```

**Response Format**:

```json
{
  "outputs": [{ "output": "Hei" }, { "output": "Maailma" }]
}
```

**Tests**:

- `test_systran_translate_batch()`: Basic translation
- `test_systran_retry_on_429()`: Retry logic
- `test_systran_cache_hit()`: Cache behavior
- `test_systran_rate_limiting()`: Rate limiter
- `test_systran_timeout()`: Timeout handling

---

### Phase 4: Update Existing Providers (1 day)

#### Files to Update:

- `crates/ampel-i18n-builder/src/translator/deepl.rs`
- `crates/ampel-i18n-builder/src/translator/google.rs`
- `crates/ampel-i18n-builder/src/translator/openai.rs`

**Changes for Each**:

1. Replace hardcoded values with `ProviderConfig`
2. Implement new trait methods (`provider_name()`, `provider_tier()`, `is_available()`)
3. Update constructor to accept `ProviderConfig`
4. Make retry/timeout/batch_size configurable

**Example Diff for DeepL**:

```diff
- pub fn new(api_key: String, timeout: Duration) -> Self {
+ pub fn new(config: ProviderConfig) -> Result<Self> {
      let client = reqwest::Client::builder()
-         .timeout(timeout)
+         .timeout(config.timeout)
          .user_agent("ampel-i18n-builder/1.0.0")
          .build()
          .expect("Failed to build HTTP client");

-     let rate_limiter = Arc::new(RateLimiter::direct(Quota::per_second(nonzero!(10u32))));
+     let rate_limiter = Arc::new(RateLimiter::direct(
+         Quota::per_second(nonzero!(config.rate_limit_per_sec))
+     ));

      Self {
          client,
-         api_key,
+         config,
          cache,
          rate_limiter,
-         retry_policy: RetryPolicy::default(),
+         retry_policy: RetryPolicy::from_config(&config),
          // ...
      }
  }
```

**Tests**:

- Update existing tests to use `ProviderConfig`
- Add tests for configurable parameters

---

### Phase 5: Fallback Router Implementation (2 days)

#### File: `crates/ampel-i18n-builder/src/translator/fallback.rs` (NEW)

**Implementation**:

1. `FallbackTranslationRouter` struct
2. Provider initialization with priority ordering
3. Language-based provider selection
4. Fallback loop with error handling
5. Comprehensive logging of fallback events

**Key Methods**:

```rust
impl FallbackTranslationRouter {
    pub fn new(config: &Config) -> Result<Self>;
    fn select_providers(&self, target_lang: &str) -> Vec<&Box<dyn TranslationService>>;
    async fn translate_with_fallback(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>>;
}
```

**Tests**:

- `test_fallback_priority_order()`: Verify tier ordering
- `test_fallback_on_failure()`: Fallback to next tier
- `test_fallback_skip_missing_key()`: Skip unconfigured providers
- `test_fallback_language_preference()`: Language-based selection
- `test_fallback_all_providers_fail()`: Error handling

---

### Phase 6: CLI Integration (1 day)

#### File: `crates/ampel-i18n-builder/src/cli/mod.rs`

**New CLI Arguments**:

```rust
#[derive(Parser, Debug, Clone)]
pub struct TranslateArgs {
    // ... existing fields ...

    /// Override global timeout (seconds)
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Override batch size
    #[arg(long)]
    pub batch_size: Option<usize>,

    /// Override max retry attempts
    #[arg(long)]
    pub max_retries: Option<usize>,

    /// Disable specific providers (can be repeated)
    #[arg(long = "disable-provider")]
    pub disabled_providers: Vec<String>,

    /// Force specific provider (no fallback)
    #[arg(long)]
    pub provider: Option<TranslationProvider>,

    /// Disable fallback (use only primary provider)
    #[arg(long)]
    pub no_fallback: bool,

    /// Per-provider timeout overrides
    #[arg(long)]
    pub systran_timeout: Option<u64>,
    #[arg(long)]
    pub deepl_timeout: Option<u64>,
    #[arg(long)]
    pub google_timeout: Option<u64>,
    #[arg(long)]
    pub openai_timeout: Option<u64>,

    /// Per-provider retry overrides
    #[arg(long)]
    pub systran_retries: Option<usize>,
    #[arg(long)]
    pub deepl_retries: Option<usize>,
    #[arg(long)]
    pub google_retries: Option<usize>,
    #[arg(long)]
    pub openai_retries: Option<usize>,

    /// Per-provider batch size overrides
    #[arg(long)]
    pub systran_batch_size: Option<usize>,
    #[arg(long)]
    pub deepl_batch_size: Option<usize>,
    #[arg(long)]
    pub google_batch_size: Option<usize>,
    #[arg(long)]
    pub openai_batch_size: Option<usize>,
}
```

#### File: `crates/ampel-i18n-builder/src/cli/translate.rs`

**Changes**:

1. Replace `SmartTranslationRouter` with `FallbackTranslationRouter`
2. Apply CLI overrides to configuration
3. Add verbose logging for provider selection

**Example**:

```rust
pub async fn execute(args: TranslateArgs) -> Result<()> {
    let mut config = Config::load()?;

    // Apply CLI overrides
    if let Some(timeout) = args.timeout {
        config.translation.default_timeout_secs = timeout;
    }
    if let Some(batch_size) = args.batch_size {
        config.translation.default_batch_size = batch_size;
    }
    // ... more overrides ...

    let router = if args.no_fallback {
        // Use single provider
        let provider = args.provider.ok_or_else(|| {
            Error::Config("--no-fallback requires --provider".to_string())
        })?;
        SingleProviderRouter::new(provider, &config)?
    } else {
        // Use fallback router
        FallbackTranslationRouter::new(&config)?
    };

    // ... rest of translation logic ...
}
```

**Tests**:

- `test_cli_timeout_override()`: CLI timeout takes precedence
- `test_cli_disable_provider()`: Provider exclusion
- `test_cli_no_fallback()`: Single provider mode

---

### Phase 7: Documentation (1 day)

#### Files to Create/Update:

1. `docs/localization/PROVIDER-CONFIGURATION.md`: Configuration guide
2. `docs/localization/FALLBACK-STRATEGY.md`: Fallback behavior
3. `crates/ampel-i18n-builder/README.md`: Update with new features
4. `.ampel-i18n.example.yaml`: Example configuration
5. `CHANGELOG.md`: Document breaking changes

#### File: `crates/ampel-i18n-builder/examples/custom_config.yaml`

**Example Configuration**:

```yaml
# Example: Production configuration with all providers
translation:
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'

  default_timeout_secs: 45
  default_batch_size: 50
  default_max_retries: 3

  providers:
    systran:
      enabled: true
      priority: 1
      timeout_secs: 60
      max_retries: 4
      batch_size: 50

    deepl:
      enabled: true
      priority: 2
      timeout_secs: 30
      max_retries: 3

    google:
      enabled: true
      priority: 3

    openai:
      enabled: false # Disabled to save costs

  fallback:
    skip_on_missing_key: true
    stop_on_first_success: true
    log_fallback_events: true
```

---

### Phase 8: Integration Testing (1 day)

#### File: `crates/ampel-i18n-builder/tests/integration/fallback_tests.rs` (NEW)

**Test Scenarios**:

1. **Happy Path**: Systran succeeds, no fallback
2. **Single Fallback**: Systran fails, DeepL succeeds
3. **Multiple Fallbacks**: Systran/DeepL fail, Google succeeds
4. **All Fail**: All providers fail, error returned
5. **Missing Keys**: Skip providers without API keys
6. **Language Preference (Configured)**: Provider with matching `preferred_languages` gets priority
7. **Language Preference (Default)**: No language preferences, use tier ordering
8. **Timeout Handling**: Provider timeout triggers fallback
9. **Batch Size**: Large batches split correctly

**Example Test**:

```rust
#[tokio::test]
async fn test_fallback_chain_systran_fails_deepl_succeeds() {
    let config = Config {
        translation: TranslationConfig {
            systran_api_key: Some("invalid-key".to_string()),
            deepl_api_key: Some(env::var("DEEPL_API_KEY").unwrap()),
            google_api_key: None,
            openai_api_key: None,
            providers: ProvidersConfig {
                systran: ProviderConfig {
                    enabled: true,
                    priority: 1,
                    max_retries: 1, // Fast fail
                    // ...
                },
                deepl: ProviderConfig {
                    enabled: true,
                    priority: 2,
                    // ...
                },
                // ...
            },
            fallback: FallbackConfig {
                skip_on_missing_key: true,
                stop_on_first_success: true,
                log_fallback_events: true,
            },
            // ...
        },
        // ...
    };

    let router = FallbackTranslationRouter::new(&config).unwrap();

    let mut texts = HashMap::new();
    texts.insert("greeting".to_string(), json!("Hello"));

    let result = router.translate_batch(&texts, "fi").await.unwrap();

    assert_eq!(result.get("greeting").unwrap(), "Hei");
    // Verify logs show fallback from Systran to DeepL
}
```

---

### Phase 9: Migration and Deprecation (0.5 days)

#### Breaking Changes

1. **`Translator::new()` signature changed**:
   - Old: `Translator::new(provider: TranslationProvider, config: &Config)`
   - New: Removed (use `FallbackTranslationRouter::new(config)`)

2. **`SmartTranslationRouter` deprecated**:
   - Replaced by `FallbackTranslationRouter`

3. **Configuration schema changed**:
   - Old: Flat `timeout_secs`, `batch_size`
   - New: Per-provider configuration under `providers.*`

#### Migration Guide

**File**: `docs/localization/MIGRATION-v2.md`

````markdown
# Migration Guide: v1 → v2 (4-Tier Architecture)

## Configuration Changes

### Before (v1)

```yaml
translation:
  deepl_api_key: 'xxx'
  google_api_key: 'yyy'
  timeout_secs: 30
  batch_size: 50
```
````

### After (v2)

```yaml
translation:
  deepl_api_key: 'xxx'
  google_api_key: 'yyy'
  systran_api_key: 'zzz' # NEW

  default_timeout_secs: 30
  default_batch_size: 50

  providers: # NEW
    systran:
      enabled: true
      priority: 1
    deepl:
      enabled: true
      priority: 2
    google:
      enabled: true
      priority: 3
    openai:
      enabled: false
```

## Code Changes

### Before (v1)

```rust
let translator = Translator::new(TranslationProvider::DeepL, &config)?;
let result = translator.translate_batch(&texts, "fi").await?;
```

### After (v2)

```rust
let router = FallbackTranslationRouter::new(&config)?;
let result = router.translate_batch(&texts, "fi").await?;
```

## CLI Changes

### Before (v1)

```bash
cargo i18n translate --lang fi --provider deepl
```

### After (v2)

```bash
# Uses all available providers with fallback (recommended)
cargo i18n translate --lang fi

# Force specific provider (no fallback)
cargo i18n translate --lang fi --provider deepl --no-fallback

# Disable specific providers
cargo i18n translate --lang fi --disable-provider openai
```

---

## Quality Attributes

### Reliability

**Target**: 99.9% success rate for translations

**Mechanisms**:

- 4-tier fallback ensures multiple providers available
- Exponential backoff retry (3 attempts per provider)
- Graceful degradation when providers fail
- Skip providers without API keys

**Metrics**:

- Success rate per provider
- Fallback frequency (should be < 5%)
- Mean time to recovery (MTTR)

---

### Performance

**Target**: < 5 seconds for 50-text batch translation

**Mechanisms**:

- Rate limiting prevents quota exhaustion
- LRU caching (1000 entries) reduces redundant API calls
- Batch translation (50-100 texts per request)
- Configurable timeouts prevent hung requests

**Metrics**:

- P50/P95/P99 latency per provider
- Cache hit rate (target: > 40%)
- API call count (minimize redundant calls)

---

### Observability

**Target**: Full visibility into provider selection and failures

**Mechanisms**:

- Structured logging (tracing crate)
- Per-provider usage metrics
- Fallback event logging
- Retry attempt tracking

**Log Levels**:

- `INFO`: Provider selection, success events
- `WARN`: Fallback events, retry attempts
- `ERROR`: Provider failures, all-providers-failed errors
- `DEBUG`: Cache hits, rate limiting

**Example Logs**:

```

INFO FallbackRouter initialized with 3 providers: Systran (Tier 1), DeepL (Tier 2), Google (Tier 3)
INFO Attempting translation with Systran (Tier 1)...
ERROR ✗ Systran (Tier 1) failed: API error 429: Rate limit exceeded
WARN Retrying in 1024ms...
WARN Used fallback provider DeepL (Tier 2) after 1 failure(s)
INFO ✓ Translation successful with DeepL (Tier 2)

```

---

### Configurability

**Target**: All parameters adjustable without code changes

**Configuration Layers** (precedence high → low):

1. CLI arguments (`--timeout`, `--batch-size`, etc.)
2. Environment variables (`SYSTRAN_API_KEY`, etc.)
3. YAML configuration (`.ampel-i18n.yaml`)
4. Default values (hardcoded fallbacks)

**Configurable Parameters** (per provider):

- API key
- Enabled/disabled flag
- Priority (1-4)
- Timeout (seconds)
- Max retries (0-10)
- Batch size (1-100+)
- Rate limit (requests/sec)
- Retry delay (milliseconds)
- Max delay (milliseconds)
- Backoff multiplier (1.0-3.0)

---

## Trade-offs and Risks

### Trade-off 1: Complexity vs. Reliability

**Decision**: Implement 4-tier fallback architecture

**Pros**:

- High reliability (99.9%+ success rate)
- Graceful degradation under failures
- Flexibility to use multiple providers

**Cons**:

- Increased code complexity (~500 LOC)
- More configuration surface area
- Longer debugging time for failures

**Mitigation**:

- Comprehensive unit/integration tests (> 80% coverage)
- Clear documentation and migration guide
- Structured logging for observability

---

### Trade-off 2: Performance vs. Cost

**Decision**: Use caching and batch translation

**Pros**:

- Reduce redundant API calls by 40-60%
- Lower API costs (pay per character)
- Faster translation (cache hits are instant)

**Cons**:

- Cache invalidation complexity
- Memory overhead (1000-entry LRU cache ~500KB)
- Stale translations if source changes

**Mitigation**:

- Cache key includes source text, target lang
- Configurable cache size
- Cache is per-process (no shared state)

---

### Trade-off 3: Provider Agnosticism vs. Optimization

**Decision**: Generic `TranslationService` trait

**Pros**:

- Easy to add new providers
- Consistent API across providers
- Clean separation of concerns

**Cons**:

- Can't leverage provider-specific features
- Lowest common denominator API
- May miss optimization opportunities

**Mitigation**:

- Per-provider configuration allows optimization
- Providers can implement internal optimizations
- Can extend trait with optional methods

---

### Risk 1: API Key Leakage

**Likelihood**: Medium
**Impact**: High

**Mitigation**:

1. Never commit API keys to version control
2. Use environment variables for secrets
3. Add `.ampel-i18n.yaml` to `.gitignore`
4. Document secure key management practices

---

### Risk 2: Rate Limit Exhaustion

**Likelihood**: Medium
**Impact**: Medium

**Mitigation**:

1. Token bucket rate limiting per provider
2. Exponential backoff on 429 errors
3. Configurable rate limits
4. Warning logs when approaching limits

---

### Risk 3: Provider API Changes

**Likelihood**: Low
**Impact**: High

**Mitigation**:

1. Integration tests with real API calls (in CI)
2. Version pinning for API endpoints
3. Provider abstraction layer isolates changes
4. Fallback to other providers on failures

---

### Risk 4: Translation Quality Variance

**Likelihood**: Medium
**Impact**: Medium

**Mitigation**:

1. Language-based provider selection (DeepL for EU, Google for Asian)
2. Tier ordering prioritizes quality (Systran/DeepL before Google/OpenAI)
3. Manual review of critical translations
4. Placeholder preservation validation

---

## Testing Strategy

### Unit Tests (80%+ coverage)

**Files**:

- `src/config.rs`: Configuration parsing, validation
- `src/translator/systran.rs`: Systran provider logic
- `src/translator/deepl.rs`: DeepL provider logic (update)
- `src/translator/google.rs`: Google provider logic (update)
- `src/translator/openai.rs`: OpenAI provider logic (update)
- `src/translator/fallback.rs`: Fallback router logic

**Test Coverage**:

- Configuration loading (YAML, TOML, env vars)
- Provider initialization (success, failure, missing key)
- Retry logic (exponential backoff, max retries)
- Rate limiting (token bucket behavior)
- Cache behavior (hit, miss, eviction)
- Batch splitting (chunk sizes, edge cases)

---

### Integration Tests

**File**: `tests/integration/fallback_tests.rs`

**Test Scenarios**:

1. **Happy Path**: First provider succeeds
2. **Single Fallback**: First fails, second succeeds
3. **Multiple Fallbacks**: First two fail, third succeeds
4. **All Fail**: All providers fail, error returned
5. **Missing Keys**: Skip unconfigured providers
6. **Language Preference**: Provider selection by language
7. **Timeout**: Provider timeout triggers fallback
8. **Large Batch**: 500 texts split into chunks
9. **CLI Overrides**: CLI parameters override config
10. **Cache Persistence**: Cache survives multiple calls

**Prerequisites**:

- Real API keys in environment variables (CI secrets)
- Or mockito for stubbed API responses

---

### End-to-End Tests

**Command**: `cargo i18n translate --lang fi`

**Scenarios**:

1. Translate complete namespace (common.json, dashboard.json)
2. Verify placeholder preservation ({{count}}, {{provider}})
3. Verify plural forms (zero, one, two, few, many, other)
4. Verify nested keys (auth.login.title)
5. Verify fallback logging (check logs for fallback events)

**Validation**:

- All keys translated
- No placeholder corruption
- Valid JSON output
- Correct language code (fi, de, ar, etc.)

---

### Performance Tests

**Tool**: `cargo bench` (criterion.rs)

**Benchmarks**:

1. **Translation Latency**: 50 texts, measure P50/P95/P99
2. **Cache Hit Rate**: 1000 texts, 50% duplicates
3. **Batch Splitting**: 500 texts, measure overhead
4. **Rate Limiting**: 100 concurrent requests, measure throughput

**Targets**:

- P50 latency: < 2s
- P95 latency: < 5s
- Cache hit rate: > 40%
- Throughput: > 100 texts/sec (with caching)

---

### Security Tests

**Scenarios**:

1. **API Key Redaction**: Verify keys not logged
2. **Injection Attack**: Malicious text inputs
3. **Config Validation**: Reject invalid configurations
4. **Timeout Prevention**: Requests don't hang indefinitely

---

## Appendix: Example Configuration Files

### Minimal Configuration (.ampel-i18n.yaml)

```yaml
translation:
  # Only one provider configured (will use this for all translations)
  deepl_api_key: '${DEEPL_API_KEY}'

  # Use defaults for everything else
```

### Development Configuration

```yaml
translation:
  # All providers enabled for testing
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'

  default_timeout_secs: 60 # Higher timeout for debugging
  default_batch_size: 10 # Smaller batches for testing

  providers:
    systran:
      enabled: true
      priority: 1
      max_retries: 1 # Fast fail for development
      # Language preferences disabled for testing all languages
      # preferred_languages: []

    deepl:
      enabled: true
      priority: 2
      # Test language-specific routing
      # preferred_languages: ["de", "fr", "fi"]

    google:
      enabled: true
      priority: 3
      # preferred_languages: ["ar", "th", "vi"]

    openai:
      enabled: true
      priority: 4
      # No language preferences (use as universal fallback)

  fallback:
    skip_on_missing_key: false # Fail if key missing
    stop_on_first_success: true
    log_fallback_events: true # Verbose logging
```

### Production Configuration

```yaml
translation:
  # Use Systran for enterprise quality, DeepL as fallback
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'

  # Disable OpenAI to save costs
  openai_api_key: null

  default_timeout_secs: 45
  default_batch_size: 50
  default_max_retries: 3

  providers:
    systran:
      enabled: true
      priority: 1
      timeout_secs: 60
      max_retries: 4
      batch_size: 50
      rate_limit_per_sec: 100
      # No language preferences - use Systran for all languages as primary

    deepl:
      enabled: true
      priority: 2
      timeout_secs: 30
      max_retries: 3
      batch_size: 50
      rate_limit_per_sec: 10
      # Optimize for European languages where DeepL excels
      # Uncomment to enable language-specific routing:
      # preferred_languages: ["bg", "cs", "da", "de", "el", "es", "et", "fi", "fr", "hu", "it", "lt", "lv", "nb", "nl", "pl", "pt", "ro", "ru", "sk", "sl", "sv"]

    google:
      enabled: true
      priority: 3
      timeout_secs: 30
      max_retries: 3
      batch_size: 100
      rate_limit_per_sec: 100
      # Optimize for Asian and Middle Eastern languages
      # Uncomment to enable language-specific routing:
      # preferred_languages: ["ar", "th", "vi", "hi", "zh", "ja", "ko", "id", "tr", "uk"]

    openai:
      enabled: false # Disabled to save costs

  fallback:
    skip_on_missing_key: true
    stop_on_first_success: true
    log_fallback_events: true
```

---

## Appendix: Provider Comparison Matrix

| Feature                | Systran | DeepL | Google | OpenAI   |
| ---------------------- | ------- | ----- | ------ | -------- |
| **Tier**               | 1       | 2     | 3      | 4        |
| **Languages**          | 55+     | 28    | 133+   | All      |
| **Quality (EU)**       | ★★★★★   | ★★★★★ | ★★★★   | ★★★★     |
| **Quality (Asian)**    | ★★★★★   | ★★★   | ★★★★★  | ★★★★     |
| **Batch Translation**  | ✅ 50   | ✅ 50 | ✅ 100 | ❌       |
| **Rate Limit**         | 100/s   | 10/s  | 100/s  | ∞        |
| **Free Tier**          | ❌      | ✅    | ❌     | ❌       |
| **Enterprise**         | ✅      | ✅    | ✅     | ✅       |
| **Placeholder Safety** | ✅      | ✅    | ✅     | ✅✅     |
| **Context-Aware**      | ❌      | ❌    | ❌     | ✅✅     |
| **Cost ($/1M chars)**  | $20     | $25   | $20    | $500     |
| **Best For**           | All     | EU    | Asian  | Fallback |

---

## Conclusion

This 4-tier translation provider architecture provides a robust, configurable, and maintainable solution for the Ampel i18n builder. The design balances reliability (4 fallback tiers), performance (caching, batching, rate limiting), and flexibility (per-provider configuration).

**Next Steps**:

1. Review and approve this design
2. Implement Phase 1-9 (9 days estimated)
3. Run integration tests with real API keys
4. Update documentation
5. Deploy to staging environment
6. Migrate production configuration

**Success Metrics**:

- 99.9% translation success rate
- < 5s P95 latency for 50-text batches
- < 5% fallback rate (most requests use Tier 1/2)
- Zero API key leakage incidents
- 80%+ code coverage

---

**Document Version**: 1.0
**Last Updated**: 2025-12-28
**Review Status**: Pending Approval
