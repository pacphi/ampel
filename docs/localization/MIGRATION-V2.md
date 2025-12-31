# Migration Guide: v1 → v2 (4-Tier Provider Architecture)

**Version**: 2.0.0
**Date**: December 2025
**Breaking Changes**: Yes (API and configuration)

---

## Table of Contents

1. [Overview](#overview)
2. [Breaking Changes Summary](#breaking-changes-summary)
3. [Configuration Migration](#configuration-migration)
4. [Code Changes](#code-changes)
5. [CLI Changes](#cli-changes)
6. [New Features](#new-features)
7. [Environment Variables](#environment-variables)
8. [Troubleshooting](#troubleshooting)
9. [Testing Your Migration](#testing-your-migration)

---

## Overview

### What Changed in v2

Version 2.0 introduces a **4-tier translation provider architecture** with intelligent automatic fallback, replacing the old `SmartTranslationRouter` with a robust `FallbackTranslationRouter`.

**Key Improvements**:

- **Reliability**: Automatic fallback through 4 provider tiers (Systran → DeepL → Google → OpenAI)
- **Configurability**: Per-provider configuration for timeout, batch size, retry logic, and language preferences
- **Observability**: Comprehensive logging of fallback events and provider failures
- **Flexibility**: Language-specific provider routing based on performance profiles
- **Environment Support**: First-class `.env` file integration with shell variable expansion

### Why the Change?

The v1 `SmartTranslationRouter` had hardcoded language preferences and no fallback mechanism:

- **Limited Reliability**: Single provider failure caused entire translation to fail
- **No Configuration**: Hardcoded timeouts, batch sizes, and language preferences
- **Poor Observability**: No logging of provider selection or failures
- **Limited Language Support**: Only 3 providers (no Systran)

The v2 `FallbackTranslationRouter` addresses all these issues with:

- **Automatic Fallback**: If Systran fails, try DeepL → Google → OpenAI
- **Full Configuration**: Per-provider settings for all aspects of translation
- **Rich Logging**: Track which provider was used and why
- **Language Optimization**: Configure preferred languages per provider

### Breaking Changes Summary

| Component                 | v1 Behavior                                         | v2 Behavior                              | Breaking?                   |
| ------------------------- | --------------------------------------------------- | ---------------------------------------- | --------------------------- |
| **Router Class**          | `SmartTranslationRouter`                            | `FallbackTranslationRouter`              | ❌ Yes                      |
| **API Initialization**    | `Translator::new(provider, config)`                 | `FallbackTranslationRouter::new(config)` | ❌ Yes                      |
| **Configuration**         | Flat structure (`timeout_secs`, `batch_size`)       | Nested structure (`providers.*`)         | ❌ Yes                      |
| **CLI Default Mode**      | Required `--provider` flag                          | Optional (fallback mode by default)      | ✅ No (backward compatible) |
| **Environment Variables** | `DEEPL_API_KEY`, `GOOGLE_API_KEY`, `OPENAI_API_KEY` | Added `SYSTRAN_API_KEY`                  | ✅ No (additive)            |
| **Provider Selection**    | Hardcoded by language                               | Configurable per provider                | ✅ No (configurable)        |

---

## Configuration Migration

### Step 1: Update .ampel-i18n.yaml

#### Before (v1)

```yaml
translation:
  # API Keys
  deepl_api_key: 'your-deepl-key'
  google_api_key: 'your-google-key'
  openai_api_key: 'your-openai-key'

  # Global settings
  timeout_secs: 30
  batch_size: 50
```

#### After (v2)

```yaml
translation:
  # API Keys (backward compatible - still supported)
  systran_api_key: '${SYSTRAN_API_KEY}' # NEW
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'

  # Global defaults (NEW - apply to all providers)
  default_timeout_secs: 30
  default_batch_size: 50
  default_max_retries: 3

  # Per-provider configuration (NEW)
  providers:
    systran:
      enabled: true
      priority: 1
      timeout_secs: 45
      max_retries: 3
      batch_size: 50
      rate_limit_per_sec: 100
      retry_delay_ms: 1000
      max_delay_ms: 30000
      backoff_multiplier: 2.0
      # Optional: Language-specific routing
      # preferred_languages: ["de", "fr", "fi", "sv"]

    deepl:
      enabled: true
      priority: 2
      timeout_secs: 30
      # ... (see .ampel-i18n.example.yaml for full config)

    google:
      enabled: true
      priority: 3
      # ...

    openai:
      enabled: true
      priority: 4
      # ...

  # Fallback behavior (NEW)
  fallback:
    skip_on_missing_key: true
    stop_on_first_success: true
    log_fallback_events: true
```

### Step 2: Migration Script

For quick migration from v1 to v2, use this script:

```bash
#!/bin/bash
# migrate-i18n-config.sh

# Backup existing config
if [ -f .ampel-i18n.yaml ]; then
  cp .ampel-i18n.yaml .ampel-i18n.yaml.v1.backup
  echo "✓ Backed up existing config to .ampel-i18n.yaml.v1.backup"
fi

# Copy example config as template
cp .ampel-i18n.example.yaml .ampel-i18n.yaml
echo "✓ Created new v2 config from template"

# Preserve API keys from v1 backup (if exists)
if [ -f .ampel-i18n.yaml.v1.backup ]; then
  echo "⚠ Please manually copy your API keys from .ampel-i18n.yaml.v1.backup"
  echo "  or set them as environment variables:"
  echo "    export SYSTRAN_API_KEY='your-key-here'"
  echo "    export DEEPL_API_KEY='your-key-here'"
  echo "    export GOOGLE_API_KEY='your-key-here'"
  echo "    export OPENAI_API_KEY='your-key-here'"
fi

echo "✓ Migration complete!"
```

Make executable and run:

```bash
chmod +x migrate-i18n-config.sh
./migrate-i18n-config.sh
```

### Step 3: Configuration Compatibility

**Good News**: v1 configuration still works in v2!

If you keep your v1 config unchanged:

- **API keys**: Still read from `deepl_api_key`, `google_api_key`, `openai_api_key`
- **Global settings**: `timeout_secs` and `batch_size` still apply
- **Default behavior**: All providers use default settings

However, you **won't get**:

- Systran provider (Tier 1) support
- Per-provider configuration
- Language-specific routing
- Configurable retry/backoff behavior
- Fallback logging

**Recommendation**: Migrate to v2 config format for full functionality.

---

## Code Changes

### API Changes

#### Before (v1)

```rust
use ampel_i18n_builder::cli::TranslationProvider;
use ampel_i18n_builder::config::Config;
use ampel_i18n_builder::translator::{SmartTranslationRouter, Translator};

// Old approach: Single provider, manual selection
let config = Config::load()?;
let translator = Translator::new(TranslationProvider::DeepL, &config)?;

let mut texts = HashMap::new();
texts.insert("greeting".to_string(), json!("Hello"));

let result = translator.translate_batch(&texts, "fi").await?;

// OR: Old SmartRouter (language-based selection)
let router = SmartTranslationRouter::new(&config)?;
let result = router.translate_batch(&texts, "fi").await?;
```

#### After (v2)

```rust
use ampel_i18n_builder::config::Config;
use ampel_i18n_builder::translator::fallback::FallbackTranslationRouter;

// New approach: Automatic fallback through 4 tiers
let config = Config::load()?;
let router = FallbackTranslationRouter::new(&config)?;

let mut texts = HashMap::new();
texts.insert("greeting".to_string(), json!("Hello"));

// Automatic fallback: Systran → DeepL → Google → OpenAI
let result = router.translate_batch(&texts, "fi").await?;

// Logs show fallback behavior:
// INFO: Attempting translation with Systran (Tier 1)... [1/4]
// ERROR: Systran (Tier 1) failed: API key not configured
// INFO: Attempting translation with DeepL (Tier 2)... [2/4]
// INFO: ✓ Translation successful with DeepL (Tier 2)
```

### Backward Compatibility Option

If you need the old single-provider behavior (no fallback):

```rust
use ampel_i18n_builder::translator::Translator;

// V1-style: Single provider (use --no-fallback in CLI)
let translator = Translator::new(TranslationProvider::DeepL, &config)?;
let result = translator.translate_batch(&texts, "fi").await?;
```

### Migration Example: Full Application

**Before (v1)**:

```rust
// src/translation/service.rs (v1)
use ampel_i18n_builder::translator::SmartTranslationRouter;

pub struct TranslationService {
    router: SmartTranslationRouter,
}

impl TranslationService {
    pub fn new(config: &Config) -> Result<Self> {
        let router = SmartTranslationRouter::new(config)?;
        Ok(Self { router })
    }

    pub async fn translate(&self, texts: &HashMap<String, Value>, lang: &str) -> Result<HashMap<String, Value>> {
        self.router.translate_batch(texts, lang).await
    }
}
```

**After (v2)**:

```rust
// src/translation/service.rs (v2)
use ampel_i18n_builder::translator::fallback::FallbackTranslationRouter;

pub struct TranslationService {
    router: FallbackTranslationRouter,
}

impl TranslationService {
    pub fn new(config: &Config) -> Result<Self> {
        let router = FallbackTranslationRouter::new(config)?;
        Ok(Self { router })
    }

    pub async fn translate(&self, texts: &HashMap<String, Value>, lang: &str) -> Result<HashMap<String, Value>> {
        // Same API, but now with automatic fallback
        self.router.translate_batch(texts, lang).await
    }
}
```

**Changes**:

1. Replace `SmartTranslationRouter` → `FallbackTranslationRouter`
2. Update import path: `translator::router` → `translator::fallback`
3. No other changes needed (API is compatible)

---

## CLI Changes

### Command Comparison

#### v1 CLI

```bash
# V1: Required --provider flag
cargo i18n translate --lang fi --provider deepl

# V1: No fallback support
cargo i18n translate --lang fi --provider google
# If Google fails → translation fails
```

#### v2 CLI

```bash
# V2: Default fallback mode (no --provider needed)
cargo i18n translate --lang fi
# Tries: Systran → DeepL → Google → OpenAI

# V2: Provider hint (prioritize but still fallback)
cargo i18n translate --lang fi --provider deepl
# Starts with DeepL, but falls back to Google/OpenAI if DeepL fails

# V2: Single provider mode (v1-compatible)
cargo i18n translate --lang fi --provider deepl --no-fallback
# Only uses DeepL (fails if DeepL fails)

# V2: Disable specific providers
cargo i18n translate --lang fi \
  --disable-provider systran \
  --disable-provider openai
# Only tries: DeepL → Google

# V2: Override global settings
cargo i18n translate --lang fi \
  --timeout 60 \
  --batch-size 100 \
  --max-retries 5
```

### New CLI Flags (v2)

| Flag                            | Description                        | Example                      | Default          |
| ------------------------------- | ---------------------------------- | ---------------------------- | ---------------- |
| `--timeout <SECS>`              | Override global timeout            | `--timeout 60`               | 30 seconds       |
| `--batch-size <N>`              | Override batch size                | `--batch-size 100`           | 50 texts         |
| `--max-retries <N>`             | Override retry attempts            | `--max-retries 5`            | 3 attempts       |
| `--disable-provider <PROVIDER>` | Disable specific provider          | `--disable-provider systran` | None             |
| `--no-fallback`                 | Single provider mode (v1 behavior) | `--no-fallback`              | Fallback enabled |

### Backward Compatibility

**All v1 commands work in v2**:

```bash
# V1 command (still works)
cargo i18n translate --lang fi --provider deepl

# V2 behavior:
# - Uses DeepL as primary
# - Falls back to Google/OpenAI if DeepL fails
# - To get exact v1 behavior, add --no-fallback
```

---

## New Features

### 1. 4-Tier Automatic Fallback

**Fallback Chain**: Systran → DeepL → Google → OpenAI

```yaml
providers:
  systran:
    enabled: true
    priority: 1 # Tried first
  deepl:
    enabled: true
    priority: 2 # Tried if Systran fails
  google:
    enabled: true
    priority: 3 # Tried if DeepL fails
  openai:
    enabled: true
    priority: 4 # Tried if Google fails
```

**Fallback Logging**:

```
INFO: Starting translation for fi with 4 provider(s) available
INFO: Attempting translation with Systran (Tier 1)... [1/4]
ERROR: ✗ Systran (Tier 1) failed: API key not configured
INFO: Attempting translation with DeepL (Tier 2)... [2/4]
INFO: ✓ Translation successful with DeepL (Tier 2)
WARN: Used fallback provider DeepL (Tier 2) after 1 failure(s)
```

### 2. Per-Provider Configuration

Configure each provider independently:

```yaml
providers:
  systran:
    enabled: true
    priority: 1
    timeout_secs: 45 # Longer timeout for enterprise service
    max_retries: 3
    batch_size: 50
    rate_limit_per_sec: 100
    retry_delay_ms: 1000
    max_delay_ms: 30000
    backoff_multiplier: 2.0

  openai:
    enabled: true
    priority: 4
    timeout_secs: 60 # Even longer for LLM processing
    max_retries: 2 # Fewer retries (high cost)
    batch_size: 0 # Unlimited batch size
    rate_limit_per_sec: 0 # No rate limiting
```

### 3. Language-Specific Routing

Optimize provider selection per language:

```yaml
providers:
  deepl:
    enabled: true
    priority: 2
    # DeepL excels at European languages
    preferred_languages: ['fi', 'sv', 'de', 'fr', 'pl', 'cs']

  google:
    enabled: true
    priority: 3
    # Google excels at Asian/Middle Eastern languages
    preferred_languages: ['ar', 'th', 'vi', 'hi', 'zh', 'ja']
```

**Behavior**:

- For Finnish (`fi`): DeepL is tried before Systran (language preference override)
- For Arabic (`ar`): Google is tried before DeepL (language preference override)
- For other languages: Standard priority order (Systran → DeepL → Google → OpenAI)

### 4. Enhanced Retry Logic

Configurable exponential backoff with jitter:

```yaml
providers:
  deepl:
    retry_delay_ms: 1000 # Initial delay: 1 second
    max_delay_ms: 30000 # Max delay: 30 seconds
    backoff_multiplier: 2.0 # Exponential backoff
    max_retries: 3 # Try 3 times
```

**Retry Schedule**:

- Attempt 1: Fail → wait 1000ms
- Attempt 2: Fail → wait 2000ms (1000 × 2.0)
- Attempt 3: Fail → wait 4000ms (2000 × 2.0)
- Attempt 4: Final fail (max retries reached)

### 5. .env File Support

Use environment variables with shell expansion:

**.env**:

```bash
# Translation API Keys
SYSTRAN_API_KEY=sk-systran-xxx
DEEPL_API_KEY=abcd1234-xxxx-yyyy-zzzz
GOOGLE_API_KEY=AIzaSyXXXXXXXXXXXX
OPENAI_API_KEY=sk-xxxxxxxxxxxx
```

**.ampel-i18n.yaml**:

```yaml
translation:
  # Shell variable expansion
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'
```

**Priority Order**:

1. Environment variables (highest priority)
2. `.ampel-i18n.yaml` literal values
3. `.ampel-i18n.yaml` shell expansion (`${VAR}`)
4. Default values (if missing)

### 6. Fallback Behavior Control

Fine-tune fallback behavior:

```yaml
fallback:
  # Skip providers without API keys (don't fail)
  skip_on_missing_key: true

  # Stop after first successful provider (don't try all)
  stop_on_first_success: true

  # Log fallback events (helpful for debugging)
  log_fallback_events: true
```

---

## Environment Variables

### Available Environment Variables

| Variable          | Description              | Required | Default | v1 Support   |
| ----------------- | ------------------------ | -------- | ------- | ------------ |
| `SYSTRAN_API_KEY` | Systran API key (Tier 1) | No       | None    | ❌ New in v2 |
| `DEEPL_API_KEY`   | DeepL API key (Tier 2)   | No       | None    | ✅ Yes       |
| `GOOGLE_API_KEY`  | Google API key (Tier 3)  | No       | None    | ✅ Yes       |
| `OPENAI_API_KEY`  | OpenAI API key (Tier 4)  | No       | None    | ✅ Yes       |

### Setting Environment Variables

**Linux/macOS**:

```bash
# Export for current session
export SYSTRAN_API_KEY="sk-systran-xxx"
export DEEPL_API_KEY="abcd1234-xxx"
export GOOGLE_API_KEY="AIzaSyXXX"
export OPENAI_API_KEY="sk-xxxx"

# Add to ~/.bashrc or ~/.zshrc for persistence
echo 'export SYSTRAN_API_KEY="sk-systran-xxx"' >> ~/.bashrc
```

**Windows (PowerShell)**:

```powershell
# Set for current session
$env:SYSTRAN_API_KEY="sk-systran-xxx"
$env:DEEPL_API_KEY="abcd1234-xxx"

# Set permanently (user scope)
[System.Environment]::SetEnvironmentVariable('SYSTRAN_API_KEY', 'sk-systran-xxx', 'User')
```

**Docker**:

```yaml
# docker-compose.yml
services:
  ampel-api:
    environment:
      - SYSTRAN_API_KEY=${SYSTRAN_API_KEY}
      - DEEPL_API_KEY=${DEEPL_API_KEY}
      - GOOGLE_API_KEY=${GOOGLE_API_KEY}
      - OPENAI_API_KEY=${OPENAI_API_KEY}
```

**GitHub Actions**:

```yaml
# .github/workflows/translate.yml
jobs:
  translate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Translate
        env:
          SYSTRAN_API_KEY: ${{ secrets.SYSTRAN_API_KEY }}
          DEEPL_API_KEY: ${{ secrets.DEEPL_API_KEY }}
          GOOGLE_API_KEY: ${{ secrets.GOOGLE_API_KEY }}
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
        run: |
          cargo i18n translate --lang fi
```

### .env File Integration

Create a `.env` file in your project root:

```bash
# .env (add to .gitignore!)
SYSTRAN_API_KEY=sk-systran-xxx
DEEPL_API_KEY=abcd1234-xxx
GOOGLE_API_KEY=AIzaSyXXX
OPENAI_API_KEY=sk-xxxx
```

Reference in `.ampel-i18n.yaml`:

```yaml
translation:
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'
```

**Important**: Add `.env` to `.gitignore`:

```bash
echo ".env" >> .gitignore
git add .gitignore
git commit -m "chore: ignore .env file"
```

---

## Troubleshooting

### Issue 1: "No translation providers available"

**Symptom**:

```
Error: Config("No translation providers available. Configure at least one API key.")
```

**Cause**: No API keys configured for any provider.

**Solution**:

1. Check environment variables:

   ```bash
   echo $SYSTRAN_API_KEY
   echo $DEEPL_API_KEY
   echo $GOOGLE_API_KEY
   echo $OPENAI_API_KEY
   ```

2. Check `.ampel-i18n.yaml`:

   ```yaml
   translation:
     systran_api_key: 'your-key-here' # Not empty?
     deepl_api_key: 'your-key-here' # Not empty?
   ```

3. Set at least one API key:
   ```bash
   export DEEPL_API_KEY="your-deepl-key"
   cargo i18n translate --lang fi
   ```

### Issue 2: "All translation providers failed"

**Symptom**:

```
Error: Translation("All 4 translation provider(s) failed. Last error: ...")
```

**Cause**: All enabled providers failed (network, API key, rate limit, etc.).

**Solution**:

1. Check logs for specific errors:

   ```
   ERROR: ✗ Systran (Tier 1) failed: API key invalid
   ERROR: ✗ DeepL (Tier 2) failed: Rate limit exceeded
   ERROR: ✗ Google (Tier 3) failed: Network timeout
   ERROR: ✗ OpenAI (Tier 4) failed: Insufficient credits
   ```

2. Fix the root cause:
   - **Invalid API key**: Update with valid key
   - **Rate limit**: Wait or increase `rate_limit_per_sec`
   - **Network timeout**: Increase `timeout_secs`
   - **Insufficient credits**: Add credits or disable provider

3. Test individual providers:

   ```bash
   # Test DeepL only (no fallback)
   cargo i18n translate --lang fi --provider deepl --no-fallback

   # Test Google only
   cargo i18n translate --lang fi --provider google --no-fallback
   ```

### Issue 3: Fallback not working as expected

**Symptom**: Provider failures don't trigger fallback.

**Cause**: Fallback disabled or misconfigured.

**Solution**:

1. Check `--no-fallback` flag:

   ```bash
   # ❌ Wrong: Fallback disabled
   cargo i18n translate --lang fi --no-fallback

   # ✅ Correct: Fallback enabled
   cargo i18n translate --lang fi
   ```

2. Check fallback configuration:

   ```yaml
   fallback:
     stop_on_first_success: true # Should be true
     skip_on_missing_key: true # Should be true
   ```

3. Enable fallback logging:

   ```yaml
   fallback:
     log_fallback_events: true
   ```

4. Verify provider priorities:
   ```yaml
   providers:
     systran:
       enabled: true
       priority: 1 # Lowest number = highest priority
     deepl:
       enabled: true
       priority: 2 # Should be > 1
   ```

### Issue 4: Configuration not loading

**Symptom**: Changes to `.ampel-i18n.yaml` don't take effect.

**Cause**: Environment variables override config file.

**Solution**:

1. Check for environment variable conflicts:

   ```bash
   # These override .ampel-i18n.yaml
   echo $DEEPL_API_KEY
   echo $GOOGLE_API_KEY
   ```

2. Unset conflicting environment variables:

   ```bash
   unset DEEPL_API_KEY
   unset GOOGLE_API_KEY
   ```

3. Use shell expansion in config:

   ```yaml
   translation:
     # Use env var if set, otherwise use literal value
     deepl_api_key: '${DEEPL_API_KEY:-your-default-key}'
   ```

4. Validate configuration:
   ```bash
   # Add validation command (if available)
   cargo i18n validate-config
   ```

### Issue 5: Providers tried in wrong order

**Symptom**: Lower priority provider used before higher priority.

**Cause**: Duplicate or incorrect priority values.

**Solution**:

1. Check priorities (must be unique):

   ```yaml
   providers:
     systran:
       priority: 1 # ✅ Unique
     deepl:
       priority: 2 # ✅ Unique
     google:
       priority: 2 # ❌ Duplicate! (warning in logs)
   ```

2. Check logs for warnings:

   ```
   WARN: Duplicate provider priorities detected. Providers with same priority will be ordered non-deterministically.
   ```

3. Fix priorities:
   ```yaml
   providers:
     systran:
       priority: 1
     deepl:
       priority: 2
     google:
       priority: 3 # ✅ Fixed
     openai:
       priority: 4
   ```

### Issue 6: Language-specific routing not working

**Symptom**: Provider with language preference not prioritized.

**Cause**: `preferred_languages` commented out or empty.

**Solution**:

1. Check `preferred_languages` configuration:

   ```yaml
   providers:
     deepl:
       # ❌ Wrong: Commented out
       # preferred_languages: ["fi", "sv"]

       # ✅ Correct: Uncommented and populated
       preferred_languages: ['fi', 'sv', 'de']
   ```

2. Verify language codes (ISO 639-1):

   ```yaml
   preferred_languages: ["fi"]  # ✅ Correct (2-letter code)
   preferred_languages: ["fin"] # ❌ Wrong (3-letter code)
   ```

3. Test with logging enabled:
   ```yaml
   fallback:
     log_fallback_events: true
   ```

---

## Testing Your Migration

### Step 1: Verify Configuration

**Test that configuration loads without errors**:

```bash
# Create minimal test config
cat > .ampel-i18n.yaml <<EOF
translation:
  deepl_api_key: "${DEEPL_API_KEY}"

  providers:
    deepl:
      enabled: true
      priority: 1
EOF

# Run validation (should not error)
cargo build --release
```

**Expected**: No configuration errors in build output.

### Step 2: Test Individual Providers

**Test each provider in isolation** (requires API keys):

```bash
# Test Systran only
export SYSTRAN_API_KEY="your-key"
cargo i18n translate --lang fi --provider systran --no-fallback --dry-run

# Test DeepL only
export DEEPL_API_KEY="your-key"
cargo i18n translate --lang fi --provider deepl --no-fallback --dry-run

# Test Google only
export GOOGLE_API_KEY="your-key"
cargo i18n translate --lang fi --provider google --no-fallback --dry-run

# Test OpenAI only
export OPENAI_API_KEY="your-key"
cargo i18n translate --lang fi --provider openai --no-fallback --dry-run
```

**Expected**: Each provider translates successfully (or reports specific errors).

### Step 3: Test Fallback Behavior

**Test automatic fallback with intentional failures**:

```bash
# Configure only DeepL and Google (disable others)
cat > .ampel-i18n.yaml <<EOF
translation:
  deepl_api_key: "${DEEPL_API_KEY}"
  google_api_key: "${GOOGLE_API_KEY}"

  providers:
    systran:
      enabled: false
    deepl:
      enabled: true
      priority: 1
    google:
      enabled: true
      priority: 2
    openai:
      enabled: false

  fallback:
    log_fallback_events: true
EOF

# Run translation (should try DeepL, fallback to Google if needed)
cargo i18n translate --lang fi --namespace common --dry-run
```

**Expected logs**:

```
INFO: Starting translation for fi with 2 provider(s) available
INFO: Attempting translation with DeepL (Tier 1)... [1/2]
INFO: ✓ Translation successful with DeepL (Tier 1)
```

**Test fallback on failure** (set invalid DeepL key):

```bash
export DEEPL_API_KEY="invalid-key"
cargo i18n translate --lang fi --namespace common --dry-run
```

**Expected logs**:

```
INFO: Starting translation for fi with 2 provider(s) available
INFO: Attempting translation with DeepL (Tier 1)... [1/2]
ERROR: ✗ DeepL (Tier 1) failed: Authentication failed
INFO: Attempting translation with Google (Tier 2)... [2/2]
INFO: ✓ Translation successful with Google (Tier 2)
WARN: Used fallback provider Google (Tier 2) after 1 failure(s)
```

### Step 4: Test Language-Specific Routing

**Configure language preferences**:

```yaml
translation:
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'

  providers:
    deepl:
      enabled: true
      priority: 2
      preferred_languages: ['fi', 'sv', 'de'] # European languages

    google:
      enabled: true
      priority: 3
      preferred_languages: ['ar', 'th', 'vi'] # Asian languages
```

**Test European language (should prefer DeepL)**:

```bash
cargo i18n translate --lang fi --dry-run
# Expected: DeepL tried first (language preference override)
```

**Test Asian language (should prefer Google)**:

```bash
cargo i18n translate --lang ar --dry-run
# Expected: Google tried first (language preference override)
```

### Step 5: Full Integration Test

**Test complete workflow**:

```bash
# 1. Clean slate
rm -rf frontend/public/locales/fi

# 2. Translate with fallback
cargo i18n translate --lang fi

# 3. Verify output
ls -la frontend/public/locales/fi/
cat frontend/public/locales/fi/common.json

# 4. Check all namespaces translated
cargo i18n coverage --lang fi
```

**Expected**:

- All namespaces translated
- Coverage >= 95%
- Valid JSON files
- No untranslated placeholders (`{{variable}}` preserved)

### Testing Checklist

Use this checklist to verify your migration:

- [ ] **Configuration loads without errors**

  ```bash
  cargo build --release
  ```

- [ ] **At least one provider works individually**

  ```bash
  cargo i18n translate --lang fi --provider deepl --no-fallback --dry-run
  ```

- [ ] **Fallback triggers on provider failure**

  ```bash
  # Set invalid key and verify fallback to next tier
  export DEEPL_API_KEY="invalid"
  cargo i18n translate --lang fi --dry-run
  ```

- [ ] **Language-specific routing works** (if configured)

  ```bash
  # Verify DeepL prioritized for Finnish
  cargo i18n translate --lang fi --dry-run
  ```

- [ ] **CLI flags override configuration**

  ```bash
  cargo i18n translate --lang fi --timeout 60 --batch-size 100
  ```

- [ ] **Dry run mode doesn't write files**

  ```bash
  cargo i18n translate --lang fi --dry-run
  # Verify no files created in frontend/public/locales/fi/
  ```

- [ ] **Full translation workflow completes**

  ```bash
  cargo i18n translate --lang fi
  cargo i18n coverage --lang fi
  # Expected: >= 95% coverage
  ```

- [ ] **Provider disabled flag works**

  ```bash
  cargo i18n translate --lang fi --disable-provider openai
  # Verify OpenAI not used
  ```

- [ ] **Fallback logging visible** (if enabled)

  ```yaml
  fallback:
    log_fallback_events: true
  ```

  ```bash
  cargo i18n translate --lang fi
  # Check logs for fallback events
  ```

- [ ] **Environment variables override config**
  ```bash
  export DEEPL_API_KEY="override-key"
  cargo i18n translate --lang fi --dry-run
  # Verify override key used (check logs)
  ```

---

## Migration Validation Script

Save this script as `validate-v2-migration.sh`:

```bash
#!/bin/bash
set -e

echo "========================================="
echo "   Ampel i18n v2 Migration Validator"
echo "========================================="
echo

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Configuration validation
echo "Test 1: Configuration validation"
if cargo build --release 2>&1 | grep -q "Error.*Config"; then
  echo -e "${RED}✗ FAIL${NC}: Configuration has errors"
  exit 1
else
  echo -e "${GREEN}✓ PASS${NC}: Configuration loads successfully"
fi
echo

# Test 2: Provider availability
echo "Test 2: Provider availability"
PROVIDERS_AVAILABLE=0

if [ -n "$SYSTRAN_API_KEY" ]; then
  echo "  ✓ Systran API key configured"
  ((PROVIDERS_AVAILABLE++))
fi

if [ -n "$DEEPL_API_KEY" ]; then
  echo "  ✓ DeepL API key configured"
  ((PROVIDERS_AVAILABLE++))
fi

if [ -n "$GOOGLE_API_KEY" ]; then
  echo "  ✓ Google API key configured"
  ((PROVIDERS_AVAILABLE++))
fi

if [ -n "$OPENAI_API_KEY" ]; then
  echo "  ✓ OpenAI API key configured"
  ((PROVIDERS_AVAILABLE++))
fi

if [ $PROVIDERS_AVAILABLE -eq 0 ]; then
  echo -e "${RED}✗ FAIL${NC}: No providers configured"
  exit 1
else
  echo -e "${GREEN}✓ PASS${NC}: $PROVIDERS_AVAILABLE provider(s) available"
fi
echo

# Test 3: Dry run test
echo "Test 3: Dry run translation"
if cargo i18n translate --lang fi --namespace common --dry-run 2>&1 | grep -q "Translation complete"; then
  echo -e "${GREEN}✓ PASS${NC}: Dry run successful"
else
  echo -e "${RED}✗ FAIL${NC}: Dry run failed"
  exit 1
fi
echo

# Test 4: Fallback configuration
echo "Test 4: Fallback configuration"
if grep -q "fallback:" .ampel-i18n.yaml; then
  echo -e "${GREEN}✓ PASS${NC}: Fallback configuration present"
else
  echo -e "${YELLOW}⚠ WARN${NC}: No fallback configuration (using defaults)"
fi
echo

# Test 5: Provider priorities
echo "Test 5: Provider priority uniqueness"
DUPLICATE_PRIORITIES=$(grep -A 1 "priority:" .ampel-i18n.yaml | grep -o "[0-9]" | sort | uniq -d | wc -l)
if [ "$DUPLICATE_PRIORITIES" -gt 0 ]; then
  echo -e "${YELLOW}⚠ WARN${NC}: Duplicate priorities detected (non-deterministic ordering)"
else
  echo -e "${GREEN}✓ PASS${NC}: All priorities unique"
fi
echo

echo "========================================="
echo "   Migration Validation Complete"
echo "========================================="
```

Run validator:

```bash
chmod +x validate-v2-migration.sh
./validate-v2-migration.sh
```

---

## Appendix: Quick Reference

### Configuration Template (Minimal)

```yaml
translation:
  deepl_api_key: '${DEEPL_API_KEY}'

  providers:
    deepl:
      enabled: true
      priority: 1
```

### Configuration Template (Full)

See `.ampel-i18n.example.yaml` for complete annotated configuration.

### Common Commands

```bash
# Translate with fallback (default)
cargo i18n translate --lang fi

# Translate without fallback (v1 behavior)
cargo i18n translate --lang fi --provider deepl --no-fallback

# Dry run (preview changes)
cargo i18n translate --lang fi --dry-run

# Override timeout and batch size
cargo i18n translate --lang fi --timeout 60 --batch-size 100

# Disable expensive providers
cargo i18n translate --lang fi --disable-provider openai
```

### Environment Variables

```bash
export SYSTRAN_API_KEY="sk-systran-xxx"
export DEEPL_API_KEY="abcd1234-xxx"
export GOOGLE_API_KEY="AIzaSyXXX"
export OPENAI_API_KEY="sk-xxxx"
```

### Rollback to v1 (if needed)

```bash
# Restore v1 config backup
cp .ampel-i18n.yaml.v1.backup .ampel-i18n.yaml

# Use v1-compatible mode
cargo i18n translate --lang fi --provider deepl --no-fallback
```

---

## Getting Help

### Documentation

- **Architecture Guide**: `docs/localization/4-TIER-PROVIDER-ARCHITECTURE.md`
- **Configuration Reference**: `.ampel-i18n.example.yaml`
- **Testing Guide**: `crates/ampel-i18n-builder/tests/`

### Reporting Issues

If you encounter issues during migration:

1. Check troubleshooting section above
2. Review logs with `log_fallback_events: true`
3. Test individual providers with `--no-fallback`
4. Open GitHub issue with:
   - Configuration (sanitized API keys)
   - Full error logs
   - Expected vs actual behavior

### Support

- **GitHub Issues**: https://github.com/pacphi/ampel/issues
- **Slack**: #ampel-i18n channel
- **Email**: support@ampel.dev

---

**Migration Guide Version**: 2.0.0
**Last Updated**: December 2025
**Compatibility**: Ampel v2.0.0+
