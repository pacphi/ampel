# Language Standardization - Phase 1 Completion

## Overview

This document details the language standardization work completed for Phase 1 of the i18n implementation. All language codes have been aligned to match the user's requirements for exactly 20 supported languages.

## Standardized 20-Language List

The following 20 languages are now supported consistently across frontend and backend:

1. **en** - English
2. **es** - Spanish
3. **fr** - French
4. **de** - German
5. **it** - Italian
6. **pt-BR** - Portuguese (Brazil)
7. **ru** - Russian
8. **ja** - Japanese
9. **zh-CN** - Chinese (Simplified)
10. **ko** - Korean
11. **ar** - Arabic (RTL)
12. **hi** - Hindi
13. **nl** - Dutch
14. **pl** - Polish
15. **tr** - Turkish
16. **sv** - Swedish
17. **da** - Danish
18. **fi** - Finnish
19. **no** - Norwegian
20. **cs** - Czech

## Language Code Rationale

### Simple Codes (No Region)

Most languages use simple 2-letter ISO 639-1 codes for better developer experience and cleaner URLs:

- `en`, `es`, `fr`, `de`, `it`, `ru`, `ja`, `ko`, `ar`, `hi`, `nl`, `pl`, `tr`, `sv`, `da`, `fi`, `no`, `cs`

### Region-Specific Codes

Only used where absolutely necessary for disambiguation:

- **pt-BR**: Brazilian Portuguese (distinct from European Portuguese)
- **zh-CN**: Simplified Chinese (distinct from Traditional Chinese)

## Changes Made

### Frontend Configuration

#### 1. `/frontend/src/i18n/config.ts`

- Updated `SUPPORTED_LANGUAGES` array to exactly 20 languages
- Removed: `es-MX`, `zh-TW`, `he-IL` (Hebrew removed per user requirements)
- Added: `da` (Danish), `fi` (Finnish), `no` (Norwegian), `cs` (Czech), `hi` (Hindi)
- Standardized to simple codes where possible (e.g., `es` instead of `es-ES`)
- Maintained region codes only for `pt-BR` and `zh-CN`

#### 2. `/frontend/src/components/i18n/constants/languages.ts`

- Synchronized `SUPPORTED_LANGUAGES` with config.ts
- Removed: `es-MX`, `pt-PT`, `zh-TW`, `en-GB`, `he` (Hebrew)
- Added: `da`, `fi`, `no`, `cs`
- Updated `COMMON_LANGUAGES` to use simple codes
- Updated `RTL_LANGUAGES` to only include `ar` (Hebrew removed)

#### 3. Frontend Locale Directories

All 20 required locale directories created in `/frontend/public/locales/`:

```
en/  es/  fr/  de/  it/  pt-BR/  ru/  ja/  zh-CN/  ko/
ar/  hi/  nl/  pl/  tr/  sv/  da/  fi/  no/  cs/
```

Each contains `common.json` with base translations.

### Backend Configuration

#### 1. `/crates/ampel-api/src/middleware/locale.rs`

**Updated Constants:**

```rust
const SUPPORTED_LOCALES: &[&str] = &[
    "en", "es", "fr", "de", "it", "pt-BR", "ru", "ja", "zh-CN", "ko",
    "ar", "hi", "nl", "pl", "tr", "sv", "da", "fi", "no", "cs",
];
```

**Updated Normalization Logic:**

- Norwegian: `no` or `nb` → `no` (Norwegian Bokmål)
- Portuguese: `pt` → `pt-BR` (Brazilian Portuguese)
- Chinese: `zh` → `zh-CN` (Simplified Chinese)
- Removed Spanish regional fallback (now just `es`)
- Removed French-Canadian fallback (now just `fr`)
- Removed Hebrew support

**Updated Tests:**

- All 9 locale middleware tests pass
- Added test cases for new languages (da, fi, no, cs, hi)
- Updated normalization test expectations
- Added Norwegian Bokmål normalization test

#### 2. Backend Locale Directories

All 20 required locale directories created in `/crates/ampel-api/locales/`:

```
en/  es/  fr/  de/  it/  pt-BR/  ru/  ja/  zh-CN/  ko/
ar/  hi/  nl/  pl/  tr/  sv/  da/  fi/  no/  cs/
```

Each contains `common.yml` with Rust i18n translations.

## Verification Results

### Backend Compilation

✅ **All tests pass**: 9/9 locale middleware tests passing

```
test middleware::locale::tests::test_normalize_locale ... ok
test middleware::locale::tests::test_is_supported_locale ... ok
test middleware::locale::tests::test_parse_accept_language ... ok
test middleware::locale::tests::test_locale_detection_query_param ... ok
test middleware::locale::tests::test_locale_detection_cookie ... ok
test middleware::locale::tests::test_locale_detection_accept_language ... ok
test middleware::locale::tests::test_locale_detection_priority ... ok
test middleware::locale::tests::test_locale_detection_fallback ... ok
test middleware::locale::tests::test_extract_query_param ... ok
```

### Frontend Verification

✅ **All 20 locale directories present**
✅ **Configuration files synchronized**
✅ **Type definitions updated**

## Extra Locale Directories

### Frontend

The frontend has additional locale directories beyond the required 20 for backwards compatibility and future expansion:

- Regional variants: `es-ES`, `es-MX`, `fr-FR`, `de-DE`, etc.
- Other languages: `ar-SA`, `he-IL`, `zh-TW`, etc.

These extra directories do not interfere with the standardized 20-language configuration.

### Backend

The backend has a few extra locale directories:

- `es-ES`, `es-MX` (Spanish regional variants)
- `fr-CA` (French Canadian)
- `sr`, `th`, `vi` (Serbian, Thai, Vietnamese)
- `he`, `zh-TW` (Hebrew, Traditional Chinese)

These can be kept for future expansion or removed if desired.

## Language Detection Priority

Both frontend and backend follow this detection order:

1. **Query parameter**: `?lang=fi`
2. **Cookie**: `lang=fi`
3. **Accept-Language header**: `fi,en;q=0.9`
4. **Fallback**: `en` (English)

## RTL Language Support

Only **Arabic (ar)** is configured as RTL in this 20-language set.

Previously supported RTL languages removed:

- Hebrew (`he`, `he-IL`) - Not in user's required 20 languages

## Next Steps

1. ✅ All configuration files updated and synchronized
2. ✅ All 20 locale directories created
3. ✅ Backend tests passing
4. ✅ Language codes standardized
5. 🔄 Translation files need to be populated with actual translations
6. 🔄 Frontend UI components need translation key updates
7. 🔄 Backend API responses need translation key updates

## Consistency Checklist

- ✅ Frontend config.ts has exactly 20 languages
- ✅ Frontend constants/languages.ts has exactly 20 languages
- ✅ Backend locale.rs SUPPORTED_LOCALES has exactly 20 languages
- ✅ All 20 frontend locale directories exist
- ✅ All 20 backend locale directories exist
- ✅ Language codes match between frontend and backend
- ✅ RTL configuration consistent (ar only)
- ✅ Backend compilation successful
- ✅ Backend tests passing (9/9)

## Language Mapping Reference

| Language             | Code  | Frontend | Backend | ISO Code | Direction |
| -------------------- | ----- | -------- | ------- | -------- | --------- |
| English              | en    | ✓        | ✓       | en-US    | LTR       |
| Spanish              | es    | ✓        | ✓       | es-ES    | LTR       |
| French               | fr    | ✓        | ✓       | fr-FR    | LTR       |
| German               | de    | ✓        | ✓       | de-DE    | LTR       |
| Italian              | it    | ✓        | ✓       | it-IT    | LTR       |
| Portuguese (Brazil)  | pt-BR | ✓        | ✓       | pt-BR    | LTR       |
| Russian              | ru    | ✓        | ✓       | ru-RU    | LTR       |
| Japanese             | ja    | ✓        | ✓       | ja-JP    | LTR       |
| Chinese (Simplified) | zh-CN | ✓        | ✓       | zh-CN    | LTR       |
| Korean               | ko    | ✓        | ✓       | ko-KR    | LTR       |
| Arabic               | ar    | ✓        | ✓       | ar-SA    | RTL       |
| Hindi                | hi    | ✓        | ✓       | hi-IN    | LTR       |
| Dutch                | nl    | ✓        | ✓       | nl-NL    | LTR       |
| Polish               | pl    | ✓        | ✓       | pl-PL    | LTR       |
| Turkish              | tr    | ✓        | ✓       | tr-TR    | LTR       |
| Swedish              | sv    | ✓        | ✓       | sv-SE    | LTR       |
| Danish               | da    | ✓        | ✓       | da-DK    | LTR       |
| Finnish              | fi    | ✓        | ✓       | fi-FI    | LTR       |
| Norwegian            | no    | ✓        | ✓       | nb-NO    | LTR       |
| Czech                | cs    | ✓        | ✓       | cs-CZ    | LTR       |

---

**Status**: ✅ COMPLETE - All 20 languages standardized and verified
**Date**: 2025-12-27
**Phase**: 1 - Configuration & Infrastructure
