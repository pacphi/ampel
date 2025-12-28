# Provider Configuration Guide

**Version**: 1.0
**Date**: 2025-12-28
**Status**: Documentation

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Configuration Methods](#configuration-methods)
4. [Provider-Specific Settings](#provider-specific-settings)
5. [Fallback Configuration](#fallback-configuration)
6. [Language Preferences](#language-preferences)
7. [CLI Overrides](#cli-overrides)
8. [Common Scenarios](#common-scenarios)
9. [Troubleshooting](#troubleshooting)

---

## Overview

The ampel-i18n-builder uses a **4-tier translation provider architecture** with configurable fallback:

- **Tier 1: Systran** - Enterprise neural MT (primary)
- **Tier 2: DeepL** - High-quality European languages
- **Tier 3: Google** - Broad language coverage
- **Tier 4: OpenAI** - Fallback for complex content

Each provider can be independently configured with:

- API keys
- Timeout limits
- Retry attempts
- Batch sizes
- Rate limits
- Language preferences

---

## Quick Start

### Minimal Setup (One Provider)

```bash
# .env file
DEEPL_API_KEY=your_deepl_key_here
```

That's it! The system will use DeepL for all translations.

### Recommended Setup (Multi-Tier with Fallback)

```bash
# .env file
SYSTRAN_API_KEY=your_systran_key      # Tier 1 (primary)
DEEPL_API_KEY=your_deepl_key          # Tier 2 (fallback)
GOOGLE_API_KEY=your_google_key        # Tier 3 (broad coverage)
OPENAI_API_KEY=your_openai_key        # Tier 4 (emergency fallback)
```

```yaml
# .ampel-i18n.yaml
translation:
  # Use environment variables for keys
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'

  # Fallback behavior
  fallback:
    skip_on_missing_key: true
    stop_on_first_success: true
    log_fallback_events: true
```

---

## Configuration Methods

### Priority Order (Highest to Lowest)

1. **CLI Parameters** - Runtime overrides
2. **Environment Variables** - System environment
3. **`.env` File** - Project-specific environment
4. **`.ampel-i18n.yaml`** - Configuration file
5. **Default Values** - Built-in fallbacks

### Method 1: Environment Variables

**Recommended for production deployments**

```bash
# API Keys
export SYSTRAN_API_KEY="xxx-xxx-xxx"
export DEEPL_API_KEY="xxx-xxx-xxx"
export GOOGLE_API_KEY="xxx-xxx-xxx"
export OPENAI_API_KEY="xxx-xxx-xxx"

# Optional: Global overrides
export AMPEL_I18N_TIMEOUT_SECS=45
export AMPEL_I18N_BATCH_SIZE=50
export AMPEL_I18N_MAX_RETRIES=5
```

### Method 2: .env File

**Recommended for development**

```bash
# Copy example and edit
cp .env.example .env

# Edit .env
SYSTRAN_API_KEY=your-key-here
DEEPL_API_KEY=your-key-here
GOOGLE_API_KEY=your-key-here
OPENAI_API_KEY=your-key-here
```

### Method 3: YAML Configuration

**Recommended for advanced settings**

Create `.ampel-i18n.yaml`:

```yaml
translation:
  # API Keys
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'

  # Global Defaults
  default_timeout_secs: 30
  default_batch_size: 50
  default_max_retries: 3

  # Per-Provider Configuration
  providers:
    systran:
      enabled: true
      priority: 1
      timeout_secs: 45
      max_retries: 3
      batch_size: 50
      rate_limit_per_sec: 100

    deepl:
      enabled: true
      priority: 2
      timeout_secs: 30
      max_retries: 3
      batch_size: 50
      rate_limit_per_sec: 10

    google:
      enabled: true
      priority: 3
      timeout_secs: 30
      max_retries: 3
      batch_size: 100
      rate_limit_per_sec: 100

    openai:
      enabled: true
      priority: 4
      timeout_secs: 60
      max_retries: 2
      model: 'gpt-4o'
      temperature: 0.3

  # Fallback Strategy
  fallback:
    skip_on_missing_key: true
    stop_on_first_success: true
    log_fallback_events: true
```

---

## Provider-Specific Settings

### Systran (Tier 1)

**API Documentation**: https://platform.systran.net/

```yaml
providers:
  systran:
    enabled: true
    priority: 1 # Tier 1 (highest priority)
    timeout_secs: 45 # Request timeout
    max_retries: 3 # Retry attempts on failure
    batch_size: 50 # Texts per request
    rate_limit_per_sec: 100 # Requests per second
    retry_delay_ms: 1000 # Initial retry delay
    max_delay_ms: 30000 # Maximum retry delay
    backoff_multiplier: 2.0 # Exponential backoff factor
```

**When to Use:**

- Primary provider for all languages
- Enterprise-grade quality required
- Fast response times needed

**Cost**: ~$20 per million characters

### DeepL (Tier 2)

**API Documentation**: https://www.deepl.com/docs-api

```yaml
providers:
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
```

**When to Use:**

- European languages (de, fr, fi, sv, pl, cs, etc.)
- High quality translations required
- Free tier available for development

**Cost**: Free tier available, ~$25 per million characters for paid

### Google Translate (Tier 3)

**API Documentation**: https://cloud.google.com/translate/docs

```yaml
providers:
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
```

**When to Use:**

- Asian languages (zh, ja, ko, vi, th)
- Middle Eastern languages (ar, he)
- Broad language coverage needed
- High throughput required

**Cost**: ~$20 per million characters

### OpenAI (Tier 4)

**API Documentation**: https://platform.openai.com/docs/api-reference

```yaml
providers:
  openai:
    enabled: true
    priority: 4 # Tier 4 (fallback)
    timeout_secs: 60 # Higher timeout for LLM
    max_retries: 2
    batch_size: 0 # Unlimited (context window limit)
    rate_limit_per_sec: 0 # No rate limiting
    retry_delay_ms: 2000
    max_delay_ms: 60000
    backoff_multiplier: 2.0
    model: 'gpt-4o' # Model selection
    temperature: 0.3 # Lower = more consistent
```

**When to Use:**

- Fallback when other providers fail
- Complex technical content
- Placeholder-heavy translations
- Context-aware translation needed

**Cost**: ~$500 per million characters (expensive!)

---

## Fallback Configuration

### Fallback Strategy Settings

```yaml
translation:
  fallback:
    # Skip providers without API keys
    skip_on_missing_key: true

    # Stop trying providers after first success
    stop_on_first_success: true

    # Log when falling back to next tier
    log_fallback_events: true
```

### Fallback Flow

```
Request → Systran (Tier 1)
            ↓ (on failure)
          DeepL (Tier 2)
            ↓ (on failure)
          Google (Tier 3)
            ↓ (on failure)
          OpenAI (Tier 4)
            ↓ (on failure)
          ERROR
```

### Example Logs

```
INFO  FallbackRouter initialized with 3 providers: Systran (Tier 1), DeepL (Tier 2), Google (Tier 3)
INFO  Attempting translation with Systran (Tier 1)...
ERROR ✗ Systran (Tier 1) failed: API error 429: Rate limit exceeded
WARN  Retrying in 1024ms...
WARN  Used fallback provider DeepL (Tier 2) after 1 failure(s)
INFO  ✓ Translation successful with DeepL (Tier 2)
```

---

## Language Preferences

### Per-Provider Language Optimization

You can configure which providers are preferred for specific languages:

```yaml
translation:
  providers:
    systran:
      enabled: true
      priority: 1
      # Systran handles all languages well, so no preferences needed
      # preferred_languages: []

    deepl:
      enabled: true
      priority: 2
      # Optimize for European languages where DeepL excels
      preferred_languages:
        - bg # Bulgarian
        - cs # Czech
        - da # Danish
        - de # German
        - el # Greek
        - es # Spanish
        - et # Estonian
        - fi # Finnish
        - fr # French
        - hu # Hungarian
        - it # Italian
        - lt # Lithuanian
        - lv # Latvian
        - nb # Norwegian
        - nl # Dutch
        - pl # Polish
        - pt # Portuguese
        - ro # Romanian
        - ru # Russian
        - sk # Slovak
        - sl # Slovenian
        - sv # Swedish

    google:
      enabled: true
      priority: 3
      # Optimize for Asian and Middle Eastern languages
      preferred_languages:
        - ar # Arabic
        - th # Thai
        - vi # Vietnamese
        - hi # Hindi
        - zh # Chinese
        - ja # Japanese
        - ko # Korean
        - id # Indonesian
        - tr # Turkish
        - uk # Ukrainian

    openai:
      enabled: true
      priority: 4
      # No language preferences - use as universal fallback
      # preferred_languages: []
```

### How Language Preferences Work

1. **Request arrives** for target language (e.g., `fi` for Finnish)
2. **Check preferences**: Does any provider have `fi` in `preferred_languages`?
3. **Prioritize matches**: Providers with matching preferences go first
4. **Sort by tier**: Within each group, sort by priority (tier)
5. **Try providers**: Attempt in order, fallback on failure

**Example for Finnish (fi):**

```
Without preferences:
Systran (Tier 1) → DeepL (Tier 2) → Google (Tier 3) → OpenAI (Tier 4)

With preferences (DeepL prefers fi):
DeepL (Tier 2, matched) → Systran (Tier 1) → Google (Tier 3) → OpenAI (Tier 4)
```

---

## CLI Overrides

### Global Overrides

Override settings for all providers:

```bash
# Override timeout (all providers)
cargo i18n translate --lang fi --timeout 60

# Override batch size
cargo i18n translate --lang fi --batch-size 25

# Override retry attempts
cargo i18n translate --lang fi --max-retries 5
```

### Provider-Specific Overrides

Override settings for specific providers:

```bash
# Override Systran timeout
cargo i18n translate --lang fi --systran-timeout 60

# Override DeepL retries
cargo i18n translate --lang fi --deepl-retries 5

# Override Google batch size
cargo i18n translate --lang fi --google-batch-size 50

# Override OpenAI timeout
cargo i18n translate --lang fi --openai-timeout 120
```

### Disable Providers

```bash
# Disable specific providers
cargo i18n translate --lang fi --disable-provider openai

# Disable multiple providers
cargo i18n translate --lang fi \
  --disable-provider systran \
  --disable-provider openai
```

### Force Single Provider

```bash
# Use only DeepL (no fallback)
cargo i18n translate --lang fi --provider deepl --no-fallback

# Use only Google (no fallback)
cargo i18n translate --lang ar --provider google --no-fallback
```

---

## Common Scenarios

### Development Setup

**Goal**: Fast iteration with minimal cost

```yaml
translation:
  # Only use DeepL free tier
  deepl_api_key: '${DEEPL_API_KEY}'

  default_timeout_secs: 60 # Higher timeout for debugging
  default_batch_size: 10 # Smaller batches for testing

  providers:
    deepl:
      enabled: true
      priority: 1
      max_retries: 1 # Fast fail for development
```

### Staging Setup

**Goal**: Test all providers with production-like config

```yaml
translation:
  # All providers configured
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'

  default_timeout_secs: 45
  default_batch_size: 50

  providers:
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
      enabled: true
      priority: 4

  fallback:
    skip_on_missing_key: false # Fail if key missing (catch issues)
    stop_on_first_success: true
    log_fallback_events: true # Verbose logging
```

### Production Setup

**Goal**: Maximum reliability, cost optimization

```yaml
translation:
  # Use Systran + DeepL + Google, disable expensive OpenAI
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'

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
      batch_size: 50
      # Optimize for European languages
      preferred_languages:
        [
          'bg',
          'cs',
          'da',
          'de',
          'el',
          'es',
          'et',
          'fi',
          'fr',
          'hu',
          'it',
          'lt',
          'lv',
          'nb',
          'nl',
          'pl',
          'pt',
          'ro',
          'ru',
          'sk',
          'sl',
          'sv',
        ]

    google:
      enabled: true
      priority: 3
      timeout_secs: 30
      max_retries: 3
      batch_size: 100
      # Optimize for Asian/Middle Eastern languages
      preferred_languages: ['ar', 'th', 'vi', 'hi', 'zh', 'ja', 'ko', 'id', 'tr', 'uk']

    openai:
      enabled: false # Disabled to save costs

  fallback:
    skip_on_missing_key: true
    stop_on_first_success: true
    log_fallback_events: true
```

### Cost-Optimized Setup

**Goal**: Minimize costs while maintaining coverage

```yaml
translation:
  # Use DeepL free tier + Google
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'

  providers:
    deepl:
      enabled: true
      priority: 1
      # Use DeepL for supported languages
      preferred_languages:
        [
          'bg',
          'cs',
          'da',
          'de',
          'el',
          'es',
          'et',
          'fi',
          'fr',
          'hu',
          'it',
          'lt',
          'lv',
          'nb',
          'nl',
          'pl',
          'pt',
          'ro',
          'ru',
          'sk',
          'sl',
          'sv',
          'zh',
          'ja',
          'ko',
        ]

    google:
      enabled: true
      priority: 2
      # Use Google for languages DeepL doesn't support

  fallback:
    skip_on_missing_key: true
    stop_on_first_success: true
    log_fallback_events: true
```

---

## Troubleshooting

### No Providers Available Error

**Error**: `No translation providers available. Configure API keys.`

**Solution**: Set at least one API key

```bash
# Option 1: Environment variable
export DEEPL_API_KEY="your-key-here"

# Option 2: .env file
echo "DEEPL_API_KEY=your-key-here" >> .env

# Option 3: YAML config
# Edit .ampel-i18n.yaml:
translation:
  deepl_api_key: "your-key-here"
```

### Rate Limit Exceeded

**Error**: `API error 429: Rate limit exceeded`

**Solutions**:

1. **Reduce rate limit** in config:

```yaml
providers:
  deepl:
    rate_limit_per_sec: 5 # Lower than default 10
```

2. **Increase retry delay**:

```yaml
providers:
  deepl:
    retry_delay_ms: 2000 # Slower retries
    max_delay_ms: 60000
```

3. **Switch to provider with higher limit**:

```bash
cargo i18n translate --lang fi --disable-provider deepl
# Will fallback to Google (100 req/sec)
```

### Timeout Errors

**Error**: `Provider timeout after 30s`

**Solutions**:

1. **Increase timeout**:

```bash
cargo i18n translate --lang fi --timeout 60
```

2. **Reduce batch size** (smaller requests are faster):

```bash
cargo i18n translate --lang fi --batch-size 25
```

3. **Switch provider**:

```bash
cargo i18n translate --lang fi --disable-provider systran
# Will fallback to DeepL
```

### All Providers Failed

**Error**: `All translation providers failed or unavailable`

**Diagnosis**:

1. Check API keys are valid:

```bash
# Test each key
curl -H "Authorization: DeepL-Auth-Key ${DEEPL_API_KEY}" \
  https://api-free.deepl.com/v2/usage
```

2. Check network connectivity:

```bash
# Test external connectivity
curl https://api-free.deepl.com/v2/usage
```

3. Enable verbose logging:

```bash
RUST_LOG=debug cargo i18n translate --lang fi
```

### Fallback Not Working

**Issue**: Provider fails but doesn't fallback to next tier

**Check**:

1. **Is `stop_on_first_success` enabled?**

```yaml
fallback:
  stop_on_first_success: true # Should be true
```

2. **Are providers enabled?**

```yaml
providers:
  deepl:
    enabled: true # Must be true
```

3. **Check logs for fallback events**:

```bash
RUST_LOG=info cargo i18n translate --lang fi
# Look for: "Used fallback provider..."
```

---

## API Key Management

### Security Best Practices

1. **Never commit API keys** to version control
2. **Use environment variables** for secrets
3. **Add `.ampel-i18n.yaml` to `.gitignore`** if it contains keys
4. **Rotate keys regularly**
5. **Use separate keys** for dev/staging/prod

### Getting API Keys

- **Systran**: https://platform.systran.net/
- **DeepL**: https://www.deepl.com/pro-api
- **Google**: https://console.cloud.google.com/apis/credentials
- **OpenAI**: https://platform.openai.com/api-keys

---

## Performance Tips

### Optimize Translation Speed

1. **Use caching** (automatically enabled):
   - LRU cache reduces redundant API calls by 40-60%

2. **Batch translations**:
   - Default batch size (50) is optimal for most providers

3. **Use fastest provider for language**:
   - Configure `preferred_languages` for optimal routing

4. **Disable expensive providers**:
   - Disable OpenAI in production unless needed

### Optimize Costs

1. **Use DeepL free tier** for development:
   - 500,000 characters/month free

2. **Disable OpenAI** ($500/M chars):
   - Only use as emergency fallback

3. **Prefer Systran/Google** ($20/M chars):
   - More cost-effective than DeepL ($25/M chars)

4. **Cache aggressively**:
   - Cache reduces API calls significantly

---

## Related Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [4-TIER-PROVIDER-ARCHITECTURE.md](./4-TIER-PROVIDER-ARCHITECTURE.md) - Architecture design
- [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) - Quick start guide
- [README.md](../../crates/ampel-i18n-builder/README.md) - Main README

---

**Document Version**: 1.0
**Last Updated**: 2025-12-28
**Maintained By**: Ampel i18n Team
