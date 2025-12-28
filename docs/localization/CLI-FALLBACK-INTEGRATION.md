# CLI Fallback Router Integration

## Overview

The CLI has been updated to use the `FallbackTranslationRouter` for automatic provider fallback instead of the old `SmartTranslationRouter`. This enables resilient translation with automatic failover across the 4-tier provider hierarchy.

## What Changed

### 1. Default Behavior (Fallback Mode)

**Before:**

```bash
cargo i18n translate --lang fi --provider deepl
# Used only DeepL, failed if unavailable
```

**After:**

```bash
cargo i18n translate --lang fi
# Uses all available providers with automatic fallback
# Systran (Tier 1) → DeepL (Tier 2) → Google (Tier 3) → OpenAI (Tier 4)
```

### 2. Backward Compatibility (Single Provider Mode)

For users who want the old behavior:

```bash
cargo i18n translate --lang fi --provider deepl --no-fallback
# Uses only DeepL, fails if unavailable (old behavior)
```

### 3. New CLI Parameters

```bash
# Override global settings
cargo i18n translate --lang fi \
  --timeout 30 \
  --batch-size 100 \
  --max-retries 5

# Disable specific providers
cargo i18n translate --lang fi \
  --disable-provider systran \
  --disable-provider google

# Combine all
cargo i18n translate --lang fi \
  --timeout 60 \
  --batch-size 50 \
  --disable-provider openai
```

## Usage Examples

### Example 1: Default Fallback Translation

```bash
cargo i18n translate --lang fi
```

**Output:**

```
→ Fallback mode enabled: Translating to fi
✓ FallbackRouter initialized with 4 provider(s)
  ✓ Systran translator initialized (Tier 1)
  ✓ DeepL translator initialized (Tier 2)
  ✓ Google translator initialized (Tier 3)
  ✓ OpenAI translator initialized (Tier 4)
✓ Found 5 namespace(s): common, dashboard, errors, settings, validation
  → common - 12 missing key(s)
    ✓ Translation successful with Systran (Tier 1)
  → dashboard - 8 missing key(s)
    ✓ Translation successful with Systran (Tier 1)
✓ Translation complete!
```

### Example 2: Fallback in Action

If Systran fails, automatically tries next provider:

```bash
cargo i18n translate --lang fi
```

**Output:**

```
→ Fallback mode enabled: Translating to fi
✓ FallbackRouter initialized with 3 provider(s)
  ⊘ Systran skipped (no API key configured)
  ✓ DeepL translator initialized (Tier 2)
  ✓ Google translator initialized (Tier 3)
  ✓ OpenAI translator initialized (Tier 4)

  → common - 12 missing key(s)
    ✗ DeepL (Tier 2) failed: Rate limit exceeded
    ! Used fallback provider Google (Tier 3) after 1 failure(s)
    ✓ Translation successful with Google (Tier 3)
```

### Example 3: Single Provider Mode (Old Behavior)

```bash
cargo i18n translate --lang fi --provider deepl --no-fallback
```

**Output:**

```
→ Single provider mode: DeepL (no fallback)
✗ DeepL API key not found. Set DEEPL_API_KEY env var or config
```

### Example 4: CLI Overrides

```bash
cargo i18n translate --lang fi \
  --timeout 60 \
  --batch-size 50 \
  --max-retries 3 \
  --disable-provider openai
```

**Output:**

```
⚙ Override global timeout: 60s
⚙ Override batch size: 50
⚙ Override max retries: 3
! Disabled providers: openai
→ Fallback mode enabled: Translating to fi
✓ FallbackRouter initialized with 3 provider(s)
```

### Example 5: Sync Command (Uses Single Provider)

```bash
cargo i18n sync --provider deepl
```

The `sync` command uses `--no-fallback` internally to ensure consistent translation across all languages using the same provider.

## Configuration Priority

Settings are applied in this order (highest priority first):

1. **CLI flags** (`--timeout`, `--batch-size`, etc.)
2. **Environment variables** (`SYSTRAN_API_KEY`, `DEEPL_API_KEY`, etc.)
3. **Config file** (`ampel.config.yml`)
4. **Defaults** (defined in `Config::default()`)

## Migration Guide

### If you used explicit `--provider`

**Before:**

```bash
cargo i18n translate --lang fi --provider deepl
```

**After (automatic fallback):**

```bash
cargo i18n translate --lang fi
# DeepL will be prioritized, but falls back to others if it fails
```

**After (same as before):**

```bash
cargo i18n translate --lang fi --provider deepl --no-fallback
```

### If you configured API keys

No changes needed. The router automatically uses all available providers based on configured API keys.

### If you used environment variables

No changes needed. Environment variables still work:

```bash
export SYSTRAN_API_KEY="your-key"
export DEEPL_API_KEY="your-key"
cargo i18n translate --lang fi
```

## Error Handling

### All Providers Fail

```bash
cargo i18n translate --lang fi
```

**Output:**

```
→ Fallback mode enabled: Translating to fi
✓ FallbackRouter initialized with 4 provider(s)
  → common - 12 missing key(s)
    ✗ Systran (Tier 1) failed: Connection timeout
    ✗ DeepL (Tier 2) failed: Rate limit exceeded
    ✗ Google (Tier 3) failed: API quota exceeded
    ✗ OpenAI (Tier 4) failed: Invalid API key
✗ Error: All 4 translation provider(s) failed. Last error: Invalid API key
```

### No Providers Available

```bash
cargo i18n translate --lang fi
# (no API keys configured)
```

**Output:**

```
✗ Error: No translation providers available. Configure at least one API key.

Hint: Set one of these environment variables:
  - SYSTRAN_API_KEY (Tier 1 - fastest, most cost-effective)
  - DEEPL_API_KEY (Tier 2 - high quality European languages)
  - GOOGLE_API_KEY (Tier 3 - broad language support)
  - OPENAI_API_KEY (Tier 4 - contextual translations)
```

## Implementation Details

### File Changes

1. **`crates/ampel-i18n-builder/src/cli/mod.rs`**
   - Added new CLI parameters (`--timeout`, `--batch-size`, `--max-retries`, `--disable-provider`, `--no-fallback`)
   - Made `--provider` optional (deprecated in favor of fallback mode)

2. **`crates/ampel-i18n-builder/src/cli/translate.rs`**
   - Replaced `Translator::new()` with `FallbackTranslationRouter::new()`
   - Added config override logic for CLI parameters
   - Added mode detection (fallback vs. single provider)
   - Updated to use `Box<dyn TranslationService>` for polymorphism

3. **`crates/ampel-i18n-builder/src/cli/sync.rs`**
   - Updated to pass new required fields to `TranslateArgs`
   - Uses `--no-fallback` internally for consistency

4. **`crates/ampel-i18n-builder/src/translator/mod.rs`**
   - Implemented `TranslationService` trait for `Translator`
   - Enables `Translator` to be used as `Box<dyn TranslationService>`

### Testing

Run integration tests:

```bash
# Test fallback router
cargo test --package ampel-i18n-builder --lib translator::fallback

# Test CLI integration
cargo test --package ampel-i18n-builder --lib cli::translate

# Test full workflow
cargo i18n translate --lang fi --dry-run
```

## Future Enhancements

### Phase 7: Provider-Specific Overrides

```bash
cargo i18n translate --lang fi \
  --systran-timeout 30 \
  --deepl-batch-size 100 \
  --google-retries 5
```

### Phase 8: Language-Specific Preferences

Automatically prioritize providers based on target language:

- Finnish → DeepL (optimized for European languages)
- Arabic → Google (better Middle Eastern support)
- Czech → Systran (better Slavic language support)

## Summary

- **Default behavior**: Automatic fallback across all available providers
- **Backward compatible**: Use `--no-fallback` for old behavior
- **Flexible**: CLI overrides for timeout, batch size, retries
- **Robust**: Clear error messages when providers fail
- **Documented**: Comprehensive help text and examples
