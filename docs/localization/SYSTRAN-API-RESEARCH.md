# SYSTRAN Translation API - Comprehensive Research Documentation

**Research Date**: 2025-12-28
**Purpose**: Integration into ampel-i18n-builder crate
**Researcher**: Research Agent

---

## Table of Contents

1. [API Overview](#api-overview)
2. [Authentication](#authentication)
3. [API Endpoints](#api-endpoints)
4. [Request/Response Structure](#requestresponse-structure)
5. [Batch Translation Capabilities](#batch-translation-capabilities)
6. [Rate Limits & Error Handling](#rate-limits--error-handling)
7. [Timeout Best Practices](#timeout-best-practices)
8. [Language Codes & Mappings](#language-codes--mappings)
9. [Rust Implementation Recommendations](#rust-implementation-recommendations)
10. [Integration Checklist](#integration-checklist)

---

## API Overview

**Base URL**: `https://api-translate.systran.net`

SYSTRAN Translate API is a RESTful API that provides:

- Pure Neural Machine Translation with human-quality accuracy
- Support for 100+ language pairs
- Text and file translation capabilities
- Batch processing and async translation
- HTTPS-secured communication

### Key Features

- Neural machine translation engine
- RESTful architecture
- JSON request/response format
- Both synchronous and asynchronous translation modes
- OAuth2 and API key authentication
- Comprehensive language pair support

---

## Authentication

SYSTRAN supports two primary authentication methods:

### 1. API Key Authentication

**Method**: Include API key in request
**Location**: Query parameter or Authorization header
**Format**: `key=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`

**Getting Your API Key**:

- Available from user preferences at `https://SERVER/user`
- Private key unique to each user account

**Example**:

```bash
# Query parameter
GET https://api-translate.systran.net/translation/text/translate?key=YOUR_API_KEY

# Header (recommended)
Authorization: ApiKey YOUR_API_KEY
```

### 2. OAuth2 Authentication

**Supported from**: Version 9.5.0+
**OAuth2 Flows**:

- Client Credentials Flow
- PKCE (Proof Key for Code Exchange)

**Required Credentials**:

- Client ID
- Client Secret

**Token Usage**:

```bash
Authorization: Bearer <access_token>
```

**Token Endpoint**: Authorization server (ses-console)

**Rust Implementation**:

```rust
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

// API Key
let mut headers = HeaderMap::new();
headers.insert(
    AUTHORIZATION,
    HeaderValue::from_str(&format!("ApiKey {}", api_key))?
);

// OAuth2 Bearer Token
headers.insert(
    AUTHORIZATION,
    HeaderValue::from_str(&format!("Bearer {}", access_token))?
);
```

---

## API Endpoints

### 1. Text Translation

**Endpoint**: `/translation/text/translate`
**Method**: POST
**Content-Type**: `application/json`

**Parameters**:

- `source` (string): Source language code (ISO 639-1)
- `target` (string): Target language code (ISO 639-1)
- `input` (string or array): Text to translate (can be repeated, max 50000 paragraphs / 50 MB)
- `profile` (optional): Translation profile identifier
- `async` (optional boolean): Enable asynchronous mode
- `batchId` (optional): Batch identifier for grouping requests

**Example Request**:

```json
{
  "source": "en",
  "target": "fr",
  "input": ["Hello world", "How are you?"]
}
```

### 2. File Translation

**Endpoint**: `/translation/file/translate`
**Method**: POST
**Content-Type**: `multipart/form-data`

**Parameters**:

- `source`: Source language code
- `target`: Target language code
- `file`: File to translate
- `async`: Boolean for asynchronous processing
- `batchId`: Optional batch identifier

**Async Mode Response**:

```json
{
  "requestId": "uuid-request-identifier"
}
```

### 3. Supported Languages

**Endpoint**: `/translation/supportedLanguages`
**Method**: GET

**Optional Filters**:

- `source`: Filter by source language
- `target`: Filter by target language

**Response**:

```json
{
  "languagePairs": [
    {
      "source": "en",
      "target": "fr",
      "profiles": [...]
    }
  ]
}
```

### 4. API Documentation

**Endpoint**: `/translation/doc?key=YOUR_API_KEY`
**Method**: GET
**Returns**: Embedded API documentation

---

## Request/Response Structure

### Text Translation Response

**Success Response** (200 OK):

```json
{
  "translations": [
    {
      "detectedSourceLanguage": "en",
      "model": "neural",
      "translatedText": "Bonjour le monde"
    },
    {
      "detectedSourceLanguage": "en",
      "model": "neural",
      "translatedText": "Comment allez-vous?"
    }
  ]
}
```

**Fields**:

- `detectedSourceLanguage`: Auto-detected source language (if not specified)
- `model`: Translation model used (e.g., "neural")
- `translatedText`: The translated output

### Error Response

**Rate Limit Exceeded** (429 Too Many Requests):

```json
{
  "error": {
    "code": 429,
    "message": "Rate limit exceeded",
    "details": "Too many requests. Please retry after the specified time."
  }
}
```

**Common Error Codes**:

- `400`: Bad Request - Invalid parameters
- `401`: Unauthorized - Invalid or missing API key
- `403`: Forbidden - Insufficient permissions
- `404`: Not Found - Invalid endpoint or resource
- `429`: Too Many Requests - Rate limit exceeded
- `500`: Internal Server Error - Server-side error
- `503`: Service Unavailable - Temporary service outage

### Response Headers

**Rate Limit Information**:

- `X-RateLimit-Limit`: Maximum requests allowed in time window
- `X-RateLimit-Remaining`: Remaining requests in current window
- `X-RateLimit-Reset`: Unix timestamp when limit resets
- `Retry-After`: Seconds to wait before retrying (on 429 errors)

---

## Batch Translation Capabilities

SYSTRAN supports batch translation for processing multiple documents efficiently.

### Batch Features

1. **Batch Identification**: Each batch has a unique identifier
2. **Asynchronous Processing**: Large batches processed in background
3. **Request Grouping**: Group related translation requests
4. **Batch Management**: Open, close, and track batch status

### Batch Workflow

1. **Create Batch**: Assign a unique `batchId`
2. **Add Requests**: Send translation requests with the same `batchId`
3. **Close Batch**: Prevent new requests from being added
4. **Monitor Progress**: Track ongoing translations
5. **Retrieve Results**: Get completed translations

### Batch Parameters

**Maximum Input**:

- 50,000 paragraphs per request
- 50 MB per request

**Async Mode**:

```json
{
  "source": "en",
  "target": "fr",
  "input": ["..."],
  "async": true,
  "batchId": "batch-2025-12-28-001"
}
```

**Response**:

```json
{
  "requestId": "req-uuid-12345",
  "batchId": "batch-2025-12-28-001",
  "status": "processing"
}
```

### Batch Best Practices

1. **Use Async Mode**: For large batches (>100 items)
2. **Optimal Batch Size**: 500-1000 items per batch for best performance
3. **Polling Interval**: Check status every 2-5 seconds
4. **Error Handling**: Retry failed items individually
5. **Idempotency**: Use unique request IDs to prevent duplicates

---

## Rate Limits & Error Handling

### Rate Limiting Strategy

**HTTP Status Code**: `429 Too Many Requests`

**Detection**:

```rust
if response.status() == 429 {
    // Handle rate limit
    let retry_after = response.headers()
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60); // Default 60 seconds

    tokio::time::sleep(Duration::from_secs(retry_after)).await;
}
```

### Exponential Backoff with Jitter

**Recommended Strategy**:

```rust
async fn retry_with_backoff<F, T>(
    mut operation: F,
    max_retries: u32,
) -> Result<T, Error>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, Error>>>>,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries && e.is_retryable() => {
                attempt += 1;
                let delay = calculate_backoff_delay(attempt);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

fn calculate_backoff_delay(attempt: u32) -> Duration {
    let base_delay = 2_u64.pow(attempt); // Exponential: 2^attempt
    let jitter = rand::thread_rng().gen_range(0..1000); // Random 0-1000ms
    Duration::from_millis(base_delay * 1000 + jitter)
}
```

### Error Classification

**Retryable Errors** (Temporary):

- `429`: Rate limit exceeded
- `500`: Internal server error
- `503`: Service unavailable
- Network timeouts
- Connection errors

**Non-Retryable Errors** (Permanent):

- `400`: Bad request (fix input)
- `401`: Unauthorized (check credentials)
- `403`: Forbidden (insufficient permissions)
- `404`: Not found (wrong endpoint)
- `405`: Method not allowed

### Response Headers for Rate Limiting

Monitor these headers to prevent rate limits:

```rust
fn check_rate_limit_headers(headers: &HeaderMap) -> RateLimitInfo {
    RateLimitInfo {
        limit: parse_header(headers, "X-RateLimit-Limit"),
        remaining: parse_header(headers, "X-RateLimit-Remaining"),
        reset: parse_header(headers, "X-RateLimit-Reset"),
    }
}

// Warn when approaching limit
if rate_limit_info.remaining < rate_limit_info.limit / 10 {
    warn!("Approaching rate limit: {} remaining", rate_limit_info.remaining);
}
```

---

## Timeout Best Practices

### Timeout Configuration

**Connection Timeout**: Time to establish connection
**Request Timeout**: Maximum time for complete request/response

**Recommended Values**:

```rust
use reqwest::Client;
use std::time::Duration;

let client = Client::builder()
    .connect_timeout(Duration::from_secs(10))  // Connection timeout
    .timeout(Duration::from_secs(60))           // Request timeout
    .pool_idle_timeout(Duration::from_secs(90)) // Keep-alive
    .build()?;
```

### Dynamic Timeout Adjustment

**For Text Translation**:

- Short texts (<100 chars): 10-15 seconds
- Medium texts (100-1000 chars): 20-30 seconds
- Long texts (>1000 chars): 45-60 seconds

**For Batch Operations**:

- Use async mode for batches >100 items
- Poll for results with 5-10 second intervals
- Overall batch timeout: 5-10 minutes

```rust
fn calculate_timeout(text_length: usize) -> Duration {
    let base_timeout = 10;
    let additional_time = (text_length / 100) * 2; // 2 sec per 100 chars
    Duration::from_secs((base_timeout + additional_time).min(60))
}
```

### Circuit Breaker Pattern

Prevent cascading failures:

```rust
pub struct CircuitBreaker {
    failure_threshold: u32,
    failure_count: AtomicU32,
    state: AtomicU8, // 0=Closed, 1=Open, 2=HalfOpen
    last_failure: Mutex<Instant>,
}

impl CircuitBreaker {
    pub fn can_attempt(&self) -> bool {
        match self.state.load(Ordering::Relaxed) {
            0 => true,  // Closed: Allow requests
            1 => {      // Open: Check if timeout elapsed
                let elapsed = self.last_failure.lock().elapsed();
                if elapsed > Duration::from_secs(60) {
                    self.state.store(2, Ordering::Relaxed); // HalfOpen
                    true
                } else {
                    false
                }
            }
            2 => true,  // HalfOpen: Allow single test request
            _ => false,
        }
    }
}
```

---

## Language Codes & Mappings

### Language Code Format

**Standard**: ISO 639-1:2002 (two-letter codes)
**Reference**: http://www.loc.gov/standards/iso639-2/php/code_list.php

### Common Language Codes

| Code | Language   | Code | Language   |
| ---- | ---------- | ---- | ---------- |
| `en` | English    | `zh` | Chinese    |
| `fr` | French     | `ja` | Japanese   |
| `de` | German     | `ko` | Korean     |
| `es` | Spanish    | `ar` | Arabic     |
| `it` | Italian    | `ru` | Russian    |
| `pt` | Portuguese | `hi` | Hindi      |
| `nl` | Dutch      | `th` | Thai       |
| `pl` | Polish     | `vi` | Vietnamese |
| `cs` | Czech      | `he` | Hebrew     |
| `fi` | Finnish    | `sr` | Serbian    |

### Language Pair Support

**Total Pairs**: 100+

**Retrieve Available Pairs**:

```bash
GET https://api-translate.systran.net/translation/supportedLanguages?key=YOUR_API_KEY
```

**Response Structure**:

```json
{
  "languagePairs": [
    {
      "source": "en",
      "target": "fr",
      "profiles": ["general", "technical", "medical"]
    },
    {
      "source": "en",
      "target": "de",
      "profiles": ["general"]
    }
  ]
}
```

### Rust Language Code Mapping

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePair {
    pub source: String,
    pub target: String,
    pub profiles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LanguageMapper {
    pairs: HashMap<(String, String), LanguagePair>,
}

impl LanguageMapper {
    pub async fn fetch_supported_languages(
        client: &reqwest::Client,
        api_key: &str,
    ) -> Result<Self, Error> {
        let response = client
            .get("https://api-translate.systran.net/translation/supportedLanguages")
            .query(&[("key", api_key)])
            .send()
            .await?;

        let data: SupportedLanguagesResponse = response.json().await?;

        let mut pairs = HashMap::new();
        for pair in data.language_pairs {
            let key = (pair.source.clone(), pair.target.clone());
            pairs.insert(key, pair);
        }

        Ok(Self { pairs })
    }

    pub fn is_supported(&self, source: &str, target: &str) -> bool {
        self.pairs.contains_key(&(source.to_string(), target.to_string()))
    }

    pub fn get_profiles(&self, source: &str, target: &str) -> Option<&[String]> {
        self.pairs
            .get(&(source.to_string(), target.to_string()))
            .map(|p| p.profiles.as_slice())
    }
}
```

### Language Detection

SYSTRAN can auto-detect source language:

```json
{
  "target": "fr",
  "input": ["Hello world"]
}
```

Response includes detected language:

```json
{
  "translations": [
    {
      "detectedSourceLanguage": "en",
      "translatedText": "Bonjour le monde"
    }
  ]
}
```

---

## Rust Implementation Recommendations

### HTTP Client Selection

**Recommended**: `reqwest` (most popular, built on `hyper`)

**Alternatives**:

- `hyper`: Lower-level, more control
- `rustify`: Lightweight wrapper for HTTP APIs

### Complete Rust Client Implementation

```rust
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystranError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Rate limit exceeded. Retry after {0} seconds")]
    RateLimitExceeded(u64),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Translation failed: {0}")]
    TranslationFailed(String),

    #[error("Language pair not supported: {source} -> {target}")]
    UnsupportedLanguagePair { source: String, target: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub source: String,
    pub target: String,
    pub input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranslationResponse {
    pub translations: Vec<Translation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Translation {
    pub detected_source_language: Option<String>,
    pub model: String,
    pub translated_text: String,
}

pub struct SystranClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl SystranClient {
    pub fn new(api_key: String) -> Result<Self, SystranError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://api-translate.systran.net".to_string(),
        })
    }

    pub async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, SystranError> {
        let url = format!("{}/translation/text/translate", self.base_url);

        let response = self.client
            .post(&url)
            .query(&[("key", &self.api_key)])
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn translate_batch(
        &self,
        source: String,
        target: String,
        texts: Vec<String>,
        batch_size: usize,
    ) -> Result<Vec<String>, SystranError> {
        let mut results = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(batch_size) {
            let request = TranslationRequest {
                source: source.clone(),
                target: target.clone(),
                input: chunk.to_vec(),
                profile: None,
                r#async: None,
                batch_id: None,
            };

            let response = self.translate_with_retry(request, 3).await?;
            results.extend(
                response.translations.into_iter()
                    .map(|t| t.translated_text)
            );
        }

        Ok(results)
    }

    async fn translate_with_retry(
        &self,
        request: TranslationRequest,
        max_retries: u32,
    ) -> Result<TranslationResponse, SystranError> {
        let mut attempt = 0;

        loop {
            match self.translate(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(SystranError::RateLimitExceeded(retry_after)) => {
                    if attempt >= max_retries {
                        return Err(SystranError::RateLimitExceeded(retry_after));
                    }
                    attempt += 1;
                    tokio::time::sleep(Duration::from_secs(retry_after)).await;
                }
                Err(e) if attempt < max_retries && e.is_retryable() => {
                    attempt += 1;
                    let delay = self.calculate_backoff_delay(attempt);
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn handle_response(
        &self,
        response: Response,
    ) -> Result<TranslationResponse, SystranError> {
        match response.status().as_u16() {
            200..=299 => {
                let translation = response.json::<TranslationResponse>().await?;
                Ok(translation)
            }
            429 => {
                let retry_after = response.headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(60);
                Err(SystranError::RateLimitExceeded(retry_after))
            }
            401 | 403 => {
                Err(SystranError::AuthenticationFailed(
                    "Invalid or missing API key".to_string()
                ))
            }
            _ => {
                let error_text = response.text().await?;
                Err(SystranError::TranslationFailed(error_text))
            }
        }
    }

    fn calculate_backoff_delay(&self, attempt: u32) -> Duration {
        let base_delay = 2_u64.pow(attempt.min(5)); // Cap at 2^5 = 32 seconds
        let jitter = rand::random::<u64>() % 1000; // 0-1000ms jitter
        Duration::from_millis(base_delay * 1000 + jitter)
    }
}

impl SystranError {
    fn is_retryable(&self) -> bool {
        matches!(
            self,
            SystranError::RequestFailed(_) | SystranError::TranslationFailed(_)
        )
    }
}
```

### Integration with ampel-i18n-builder

```rust
// In crates/ampel-i18n-builder/src/translator/systran.rs

use super::TranslationProvider;
use async_trait::async_trait;

pub struct SystranTranslator {
    client: SystranClient,
    cache: Option<Arc<TranslationCache>>,
}

#[async_trait]
impl TranslationProvider for SystranTranslator {
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String, TranslationError> {
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(text, source_lang, target_lang).await? {
                return Ok(cached);
            }
        }

        // Translate
        let request = TranslationRequest {
            source: source_lang.to_string(),
            target: target_lang.to_string(),
            input: vec![text.to_string()],
            profile: None,
            r#async: None,
            batch_id: None,
        };

        let response = self.client.translate(request).await
            .map_err(|e| TranslationError::ProviderError(e.to_string()))?;

        let translated = response.translations
            .first()
            .ok_or_else(|| TranslationError::EmptyResponse)?
            .translated_text
            .clone();

        // Store in cache
        if let Some(cache) = &self.cache {
            cache.set(text, source_lang, target_lang, &translated).await?;
        }

        Ok(translated)
    }

    async fn translate_batch(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>, TranslationError> {
        self.client
            .translate_batch(
                source_lang.to_string(),
                target_lang.to_string(),
                texts.to_vec(),
                50, // Batch size
            )
            .await
            .map_err(|e| TranslationError::ProviderError(e.to_string()))
    }
}
```

---

## Integration Checklist

### Phase 1: Setup & Authentication

- [ ] Add `reqwest` dependency to `Cargo.toml`
- [ ] Implement `SystranClient` with API key authentication
- [ ] Add configuration for API key (environment variable `SYSTRAN_API_KEY`)
- [ ] Test basic authentication with simple translation request

### Phase 2: Core Translation

- [ ] Implement `TranslationRequest` and `TranslationResponse` structs
- [ ] Create `translate()` method for single text translation
- [ ] Add error handling for common HTTP status codes
- [ ] Test translation with various language pairs

### Phase 3: Batch Processing

- [ ] Implement `translate_batch()` with chunking (50 items per chunk)
- [ ] Add support for async mode (`async: true`)
- [ ] Implement batch tracking with unique `batchId`
- [ ] Test batch translation with 100+ items

### Phase 4: Error Handling & Resilience

- [ ] Implement exponential backoff with jitter
- [ ] Add retry logic for retryable errors (429, 500, 503)
- [ ] Implement circuit breaker pattern
- [ ] Add timeout configuration (connection + request)
- [ ] Test error recovery scenarios

### Phase 5: Rate Limiting

- [ ] Parse rate limit headers (`X-RateLimit-*`)
- [ ] Implement rate limit tracking
- [ ] Add warnings when approaching limits
- [ ] Test rate limit handling with deliberate over-requesting

### Phase 6: Caching

- [ ] Integrate with existing translation cache
- [ ] Implement cache key generation (text + source + target)
- [ ] Add cache hit/miss metrics
- [ ] Test cache performance improvement

### Phase 7: Language Support

- [ ] Fetch supported language pairs from API
- [ ] Implement language pair validation
- [ ] Add language code mapping for common locales
- [ ] Test with all supported language pairs in ampel

### Phase 8: Integration with ampel-i18n-builder

- [ ] Implement `TranslationProvider` trait
- [ ] Add Systran to provider enum/router
- [ ] Update CLI to support `--provider systran`
- [ ] Add configuration validation

### Phase 9: Testing

- [ ] Unit tests for request/response serialization
- [ ] Integration tests with real API (feature-gated)
- [ ] Mock tests for error scenarios
- [ ] Performance benchmarks vs other providers

### Phase 10: Documentation

- [ ] Add API key setup instructions
- [ ] Document configuration options
- [ ] Add usage examples in README
- [ ] Create troubleshooting guide

---

## References & Sources

### Official Documentation

- [SYSTRAN Translate API Reference](https://docs.systran.net/translateAPI/en/)
- [REST Translation API Documentation](https://docs.systran.net/translateAPI/translation/)
- [API Embedded Documentation Guide](https://help.systrangroup.com/hc/en-us/articles/360012964900--REST-API-API-embedded-documentation)
- [OAuth2 Authentication Setup](https://help.systrangroup.com/hc/en-us/articles/360020115279--SPN9-STS10-How-to-set-up-and-use-Oauth2-as-an-authentication-method-for-API-requests)

### Technical Resources

- [SYSTRAN Translation Products - API](https://www.systransoft.com/translation-products/translate-api/)
- [SYSTRAN GitHub Organization](https://github.com/SYSTRAN)

### Best Practices References

- [API Rate Limiting & Timeouts Best Practices](https://www.apyflux.com/blogs/api-integration/api-rate-limiting-timeouts)
- [Rate Limiting Best Practices in REST API Design](https://www.speakeasy.com/api-design/rate-limiting)
- [Error Handling in API Connectors Best Practices](https://latenode.com/blog/error-handling-in-api-connectors-best-practices)
- [API Rate Limits Explained: Best Practices for 2025](https://orq.ai/blog/api-rate-limit)

### Rust Implementation Resources

- [How To Write A REST Client In Rust](https://www.lpalmieri.com/posts/how-to-write-a-rest-client-in-rust-with-reqwest-and-wiremock/)
- [How to choose the right Rust HTTP client](https://blog.logrocket.com/the-state-of-rust-http-clients/)
- [Idiomatic REST API clients in Rust](https://users.rust-lang.org/t/idiomatic-rest-api-clients-in-rust/27136)

---

**Document Version**: 1.0
**Last Updated**: 2025-12-28
**Maintained By**: Ampel Development Team
