# Translation API Integration - Phase 2 Implementation

## Overview

This document describes the production-ready translation API integration implemented for the Ampel i18n system. The implementation includes DeepL and Google Cloud Translation API clients with intelligent routing, caching, and robust error handling.

## Architecture

### Components

1. **Translation Providers**
   - `src/translator/deepl.rs` - DeepL API client
   - `src/translator/google.rs` - Google Cloud Translation API client
   - `src/translator/openai.rs` - OpenAI GPT-4 fallback (existing)

2. **Intelligent Routing**
   - `src/translator/router.rs` - Smart provider selection based on language
   - Routes EU languages to DeepL (superior quality for 18 languages)
   - Routes Thai/Arabic to Google (broader language coverage)
   - Automatic fallback chain: DeepL → Google → OpenAI

3. **Caching Layer**
   - `src/translator/cache.rs` - File-based translation cache
   - Avoids redundant API calls across sessions
   - Source text validation (invalidates on change)
   - Batch operations for efficiency

## Features

### DeepL Integration

```rust
pub struct DeepLTranslator {
    // Production-grade features:
    // - Batch translation (up to 50 texts per request)
    // - Exponential backoff retry (3 attempts max)
    // - Token bucket rate limiting (10 req/sec)
    // - LRU caching (1000 entries)
    // - Usage metrics tracking
}
```

**Supported Languages (18):**

- European: bg, cs, da, de, el, es, et, fi, fr, hu, it, lt, lv, nl, pl, pt, ro, ru, sk, sl, sv
- Asian: id, ja, ko, tr, uk, zh

**API Limits:**

- Batch size: 50 texts per request
- Rate limit: 10 requests/second
- Retryable errors: 408, 429, 500, 502, 503, 504

### Google Translation Integration

```rust
pub struct GoogleTranslator {
    // Production-grade features:
    // - Batch translation (up to 100 texts per request)
    // - Exponential backoff retry (3 attempts max)
    // - Token bucket rate limiting (100 req/sec)
    // - LRU caching (1000 entries)
    // - Usage metrics tracking
}
```

**Preferred Languages:**

- Arabic (ar), Thai (th), Vietnamese (vi), Hindi (hi)

**API Limits:**

- Batch size: 100 texts per request
- Rate limit: 100 requests/second
- Retryable errors: 408, 429, 500, 502, 503, 504

### Intelligent Routing

The `SmartTranslationRouter` automatically selects the optimal provider:

```rust
// Language-based routing
if language in [fi, sv, de, es, fr, ...] {
    use DeepL  // Superior quality for EU languages
} else if language in [ar, th, vi, hi] {
    use Google  // Better coverage for non-EU languages
} else {
    fallback: DeepL → Google → OpenAI
}
```

### File-Based Caching

Cache structure:

```
.ampel-i18n-cache/
├── fi/
│   ├── dashboard.json
│   └── settings.json
└── sv/
    ├── dashboard.json
    └── settings.json
```

**Features:**

- Source text validation (invalidates on change)
- Batch set operations for efficiency
- Per-language and per-namespace clearing
- Usage statistics (entries, namespaces, providers)

**Cache Entry Format:**

```json
{
  "entries": {
    "greeting": {
      "source_text": "Hello, world!",
      "translated_text": "Terve, maailma!",
      "provider": "deepl",
      "timestamp": 1735339200,
      "metadata": {}
    }
  },
  "version": 1
}
```

## Error Handling

### Exponential Backoff Retry

Both DeepL and Google clients implement exponential backoff:

```rust
struct RetryPolicy {
    max_retries: 3,
    initial_delay_ms: 1000,
    max_delay_ms: 30000,
    backoff_multiplier: 2.0,
}
```

**Retry Logic:**

1. Attempt 1: Immediate
2. Attempt 2: 1000ms + jitter (10%)
3. Attempt 3: 2000ms + jitter (10%)
4. Fail: Return error to caller

**Retryable Status Codes:**

- 408: Request Timeout
- 429: Too Many Requests
- 500: Internal Server Error
- 502: Bad Gateway
- 503: Service Unavailable
- 504: Gateway Timeout

**Non-retryable (fail immediately):**

- 400: Bad Request (invalid parameters)
- 401: Unauthorized (invalid API key)
- 403: Forbidden (quota exceeded)
- 404: Not Found

## Configuration

### API Keys

API keys can be provided via:

1. Configuration file (`.ampel-i18n.yaml`)
2. Environment variables (recommended for security)

```yaml
# .ampel-i18n.yaml
translation:
  deepl_api_key: 'your-deepl-key' # Or DEEPL_API_KEY env var
  google_api_key: 'your-google-key' # Or GOOGLE_API_KEY env var
  timeout_secs: 30
  batch_size: 50
```

**Environment Variables:**

```bash
export DEEPL_API_KEY="your-deepl-api-key"
export GOOGLE_API_KEY="your-google-cloud-api-key"
export OPENAI_API_KEY="your-openai-api-key"  # Optional fallback
```

### Security

API keys are handled securely:

- Use `secrecy` crate for API key management
- Never log API keys
- Support for credential rotation
- Environment variable precedence over config file

## CLI Usage

### Translate Command

Translate missing keys for a specific language:

```bash
# Using DeepL for Finnish
cargo i18n translate --lang fi --provider deepl

# Using Google for Thai
cargo i18n translate --lang th --provider google

# Dry run (preview changes)
cargo i18n translate --lang sv --provider deepl --dry-run

# Translate specific namespace only
cargo i18n translate --lang de --provider deepl --namespace dashboard
```

### Sync Command

Synchronize all languages from source:

```bash
# Sync all languages using DeepL
cargo i18n sync --provider deepl

# Sync with dry run
cargo i18n sync --provider google --dry-run
```

### Cache Management

```rust
use ampel_i18n_builder::translator::cache::FileCache;

let cache = FileCache::default();

// Get statistics
let stats = cache.stats("fi");
println!("Entries: {}, Namespaces: {}", stats.total_entries, stats.total_namespaces);

// Clear specific namespace
cache.clear("fi", "dashboard")?;

// Clear entire language
cache.clear_language("fi")?;

// Clear all cache
cache.clear_all()?;
```

## Performance Metrics

### Batch Translation

**DeepL:**

- Batch size: 50 texts
- API latency: ~500ms per batch
- Rate limit: 10 req/sec = 500 texts/sec

**Google:**

- Batch size: 100 texts
- API latency: ~300ms per batch
- Rate limit: 100 req/sec = 10,000 texts/sec

### Caching

**Cache Hit Ratio:**

- First run: 0% (all API calls)
- Subsequent runs: 95%+ (only new/changed keys)

**Storage:**

- Average entry: ~200 bytes
- 1000 entries: ~200KB per language

## Testing

### Unit Tests

```bash
# Test DeepL translator
cargo test --lib translator::deepl::tests

# Test Google translator
cargo test --lib translator::google::tests

# Test router
cargo test --lib translator::router::tests

# Test cache
cargo test --lib translator::cache::tests
```

### Integration Tests

```bash
# Test with mocked API responses
cargo test --test integration translation_api_tests

# Run all integration tests
cargo test --test integration
```

**Test Coverage:**

- ✅ Successful translation
- ✅ Rate limit retry (429)
- ✅ Server error retry (500)
- ✅ Non-retryable errors (400)
- ✅ Cache operations (get, set, batch, clear)
- ✅ Cache invalidation on source change
- ✅ Provider routing by language
- ✅ Batch size limits
- ✅ Concurrent cache access

## Migration Guide

### From Manual Translation

**Before:**

1. Export English translations to XLIFF
2. Send to translation agency
3. Wait 1-2 weeks
4. Import translated XLIFF
5. Manual review and fixes

**After:**

1. Run `cargo i18n translate --lang fi --provider deepl`
2. Review automated translations (optional)
3. Done in minutes

### From OpenAI-only

**Before:**

```bash
cargo i18n translate --lang fi --provider openai
# Cost: $0.10 per 1000 translations
# Quality: Good, but variable
```

**After:**

```bash
cargo i18n translate --lang fi --provider deepl
# Cost: $0.02 per 1000 translations (5x cheaper)
# Quality: Excellent for EU languages
```

## Monitoring

### Usage Metrics

Track API usage via internal metrics:

```rust
// DeepL stats (in-memory)
let stats = deepl_translator.get_stats();
println!("API calls: {}", stats.total_calls);
println!("Characters: {}", stats.total_chars);
println!("Cache hits: {}", stats.cache_hits);
```

### Cost Estimation

**DeepL Pricing:**

- Free tier: 500,000 chars/month
- Pro tier: $5.49 per 1M chars

**Google Pricing:**

- Free tier: 500,000 chars/month (Cloud Translation API)
- Paid tier: $20 per 1M chars

**Example Project (Ampel):**

- English keys: ~500 strings = ~10,000 chars
- 18 target languages = 180,000 chars
- Cost: $0.98 (DeepL) or $3.60 (Google)

## Troubleshooting

### API Key Errors

```
Error: DeepL API key not found. Set DEEPL_API_KEY env var or config
```

**Solution:**

```bash
export DEEPL_API_KEY="your-api-key-here"
```

### Rate Limit Errors

```
Error: Max retries (3) exceeded. Last error: 429 Too Many Requests
```

**Solution:**

- Reduce batch size in config
- Add delay between namespaces
- Upgrade API tier

### Cache Issues

```
Warning: Failed to load cache: JSON parse error
```

**Solution:**

```bash
# Clear corrupted cache
rm -rf .ampel-i18n-cache
```

## Future Enhancements

### Phase 3 (Planned)

- [ ] Redis cache support for distributed systems
- [ ] Translation memory (TM) integration
- [ ] Human-in-the-loop review workflow
- [ ] A/B testing different providers
- [ ] Cost optimization algorithms
- [ ] Glossary/terminology management
- [ ] Context-aware translation (UI screenshots)

### API Provider Additions

- [ ] Microsoft Azure Translator
- [ ] Amazon Translate
- [ ] Yandex.Translate
- [ ] LibreTranslate (self-hosted)

## References

- [DeepL API Documentation](https://www.deepl.com/docs-api)
- [Google Cloud Translation API](https://cloud.google.com/translate/docs)
- [Translation API Research](../TRANSLATION_API_RESEARCH.md)
- [i18n System Architecture](../../../docs/i18n/ARCHITECTURE.md)

## Support

For issues or questions:

- GitHub Issues: https://github.com/pacphi/ampel/issues
- Internal Docs: `/docs/i18n/`
- Team Slack: #i18n-support
