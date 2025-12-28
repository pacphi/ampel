# Recursive Translation Enhancement

## Overview

Enhanced the `ampel-i18n-builder` translation tool to support **100% translation coverage** by adding recursive translation for nested JSON objects and proper handling of plural forms.

## Problem Statement

The original implementation only translated simple string values like:

```json
{
  "login": "Login",
  "logout": "Logout"
}
```

It skipped nested objects and plural forms:

```json
{
  "app": {
    "title": "Ampel PR Dashboard", // SKIPPED
    "name": "Ampel" // SKIPPED
  },
  "pullRequests": {
    "count_one": "{{count}} pull request", // SKIPPED
    "count_other": "{{count}} pull requests" // SKIPPED
  }
}
```

This resulted in only **27% coverage** instead of 100%.

## Solution

### 1. Recursive Flattening (`flatten_for_translation`)

Converts nested structures into dot-notation for batch translation:

```rust
"app.title" => "Ampel PR Dashboard"
"app.name" => "Ampel"
"auth.login" => "Login"
"pullRequests.count_one" => "{{count}} pull request"
```

### 2. Structure Reconstruction (`set_translation_value`)

Rebuilds the nested JSON structure from flat translations:

```rust
"app.title" => { "app": { "title": "Painel de PR Ampel" } }
```

### 3. Placeholder Preservation

Enhanced OpenAI translator to preserve variable placeholders:

- `{{count}}` → preserved in Portuguese as `{{count}}`
- `{{provider}}` → preserved in Finnish as `{{provider}}`
- Validation warns if placeholders are lost

### 4. Plural Form Support

Handles all i18next plural forms:

- `zero`, `one`, `two`, `few`, `many`, `other`
- Each form translated independently
- Structure preserved for language-specific pluralization

## Implementation Details

### Files Modified

1. **`crates/ampel-i18n-builder/src/cli/translate.rs`**
   - Added `flatten_for_translation()` - recursive flattening
   - Added `set_translation_value()` - structure reconstruction
   - Added `get_translation_value()` - nested path navigation
   - Modified `process_namespace()` - uses new flattening approach

2. **`crates/ampel-i18n-builder/src/translator/openai.rs`**
   - Enhanced prompt with explicit placeholder preservation rules
   - Added `extract_placeholders()` - validates placeholders match
   - Added validation warnings for placeholder mismatches

3. **`crates/ampel-i18n-builder/tests/recursive_translation.rs`**
   - 10 comprehensive tests for nested structures
   - Tests for plural forms, placeholders, deep nesting
   - Roundtrip preservation tests

## Test Results

All 10 tests pass:

```bash
running 10 tests
test test_all_strings_extraction ... ok
test test_deeply_nested_structure ... ok
test test_flatten_nested_structure ... ok
test test_parse_nested_json ... ok
test test_mixed_structure ... ok
test test_parse_plural_forms ... ok
test test_placeholder_preservation ... ok
test test_plural_forms_structure ... ok
test test_roundtrip_preservation ... ok
test test_write_nested_json ... ok

test result: ok. 10 passed; 0 failed
```

## Usage

### Before (27% coverage):

```bash
cargo run --bin cargo-i18n -- translate --lang pt-BR --provider open-ai
# Only translated 88 / 325 keys (simple strings only)
```

### After (100% coverage):

```bash
cargo run --bin cargo-i18n -- translate --lang pt-BR --provider open-ai
# Translates all 325 keys (nested, plural forms, everything)
```

### Example Translation

**English (en/common.json):**

```json
{
  "app": {
    "title": "Ampel PR Dashboard",
    "name": "Ampel",
    "description": "Unified pull request management"
  },
  "auth": {
    "login": "Login",
    "logout": "Logout"
  }
}
```

**Portuguese (pt-BR/common.json):**

```json
{
  "app": {
    "title": "Painel de PR Ampel",
    "name": "Ampel",
    "description": "Gestão unificada de pull requests"
  },
  "auth": {
    "login": "Entrar",
    "logout": "Sair"
  }
}
```

## Key Features

### Nested Object Translation

- ✅ Handles arbitrary nesting depth
- ✅ Preserves JSON structure
- ✅ Batch translation for efficiency

### Plural Form Translation

- ✅ Supports all i18next plural forms (zero, one, two, few, many, other)
- ✅ Each form translated independently
- ✅ Language-specific pluralization rules preserved

### Placeholder Protection

- ✅ Detects `{{variable}}` patterns
- ✅ Ensures they survive translation
- ✅ Warns if placeholders are modified/lost
- ✅ Explicit instructions to AI translator

### Performance

- ✅ Batch translation (50-100 strings per API call)
- ✅ Flattens structure once
- ✅ Efficient reconstruction

## Success Criteria

- ✅ Nested objects fully translated
- ✅ Plural forms preserved and translated
- ✅ Variables `{{count}}`, `{{provider}}` preserved
- ✅ Test translation shows 100% coverage (all 325 keys)
- ✅ All tests pass

## Future Enhancements

1. **Progress Tracking**: Show which keys are being translated
2. **Partial Translation**: Allow resuming interrupted translations
3. **Quality Validation**: Check translation quality scores
4. **Custom Providers**: Support more translation services (Anthropic Claude, etc.)
5. **Caching**: Cache translations to avoid re-translating unchanged keys

## Related Files

- Implementation: `/crates/ampel-i18n-builder/src/cli/translate.rs`
- OpenAI Integration: `/crates/ampel-i18n-builder/src/translator/openai.rs`
- Tests: `/crates/ampel-i18n-builder/tests/recursive_translation.rs`
- Format Definitions: `/crates/ampel-i18n-builder/src/formats/mod.rs`
