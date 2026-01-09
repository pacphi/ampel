# Phase 1 i18n Test Suite Documentation

## Overview

This document describes the comprehensive test suite created for Phase 1 of Ampel's internationalization (i18n) system. These tests follow Test-Driven Development (TDD) principles and define the expected behavior for all i18n components.

**Status**: Tests written, awaiting implementation
**Coverage Target**: 80%+
**Total Test Files**: 10
**Estimated Test Count**: 150+

## Test Organization

### Backend Tests (`crates/ampel-api/tests/i18n/`)

#### 1. `test_locale_middleware.rs` (18 tests)

Tests for locale detection and normalization middleware:

- **Locale Detection**:
  - Query parameter detection (`?lang=de`)
  - Cookie detection (`locale=fr`)
  - Accept-Language header parsing
  - Priority order: query > cookie > header > default

- **Locale Normalization**:
  - All 20 languages + variants (e.g., `en-US` → `en`)
  - Case-insensitive handling
  - Fallback to English for unsupported locales

**Run**: `cargo test --test test_locale_middleware`

#### 2. `test_user_preferences.rs` (15 tests)

Tests for user language preference API:

- **GET `/api/user/preferences`**:
  - Returns user's saved language
  - Requires authentication

- **PUT `/api/user/preferences`**:
  - Updates user's language preference
  - Validates language codes
  - Rejects invalid codes (400 error)

- **Database Integration**:
  - All 20 languages can be stored
  - Default is NULL for new users
  - Language normalization on update

**Run**: `cargo test --test test_user_preferences`

#### 3. `test_migration.rs` (12 tests)

Tests for database migration adding language column:

- **Migration Verification**:
  - Column exists after migration
  - Column is nullable
  - Accepts all 20 language codes

- **Data Integrity**:
  - Existing users have NULL language
  - Can query users by language
  - Can update language column

- **Idempotency**:
  - Migration can run multiple times safely

**Run**: `cargo test --test test_migration`

**Backend Total**: 45 tests

---

### Frontend Unit Tests (`frontend/src/components/i18n/__tests__/`)

#### 1. `RTLProvider.test.tsx` (15 tests)

Tests for RTL layout provider component:

- **Direction Attribute**:
  - Sets `dir="rtl"` for Arabic, Hebrew
  - Sets `dir="ltr"` for all other languages

- **RTL Class**:
  - Adds `rtl` class to `<html>` for RTL languages
  - Removes class when switching to LTR

- **Language Change Response**:
  - Updates direction on language change
  - Preserves children rendering

**Run**: `npm test RTLProvider.test.tsx`

#### 2. `i18nConfig.test.ts` (20 tests)

Tests for i18n configuration:

- **Supported Languages**:
  - All 20 languages configured
  - English as fallback

- **Language Resources**:
  - All languages load correctly
  - Fallback to English for missing keys

- **Features**:
  - RTL language detection (Arabic, Hebrew)
  - Browser language detector configured
  - localStorage caching
  - Lazy loading via HTTP backend
  - Interpolation and pluralization support

**Run**: `npm test i18nConfig.test.ts`

#### 3. `LanguageSwitcher.test.tsx` (35 tests)

Tests for language switcher dropdown component:

- **Rendering**:
  - Displays all 20 languages
  - Shows current language with flag
  - Highlights current selection

- **Search**:
  - Filters languages by name
  - Case-insensitive search
  - Shows "no results" message
  - Clears search on close

- **Selection**:
  - Changes language via i18n
  - Updates UI after selection
  - Closes dropdown after selection

- **Keyboard Navigation**:
  - Opens with Enter/Space
  - Arrow key navigation
  - Escape to close

- **Persistence**:
  - Saves to localStorage
  - Loads from localStorage on mount

- **Accessibility**:
  - ARIA labels
  - Screen reader announcements

**Run**: `npm test LanguageSwitcher.test.tsx`

#### 4. `FlagIcon.test.tsx` (25 tests)

Tests for flag icon component:

- **All 20 Flags**:
  - Renders correct flag for each language
  - Verifies all 20 flags render

- **Size Variants**:
  - Small (`sm`)
  - Medium (`md`, default)
  - Large (`lg`)

- **Accessibility**:
  - `role="img"`
  - Descriptive `aria-label` for each language

- **Fallback**:
  - Handles unsupported codes
  - Handles empty/undefined codes

- **Performance**:
  - Renders 20 flags in <100ms

**Run**: `npm test FlagIcon.test.tsx`

**Frontend Unit Total**: 95 tests

---

### Frontend Integration Tests (`frontend/tests/i18n/`)

#### 1. `languageSwitching.integration.test.tsx` (20 tests)

Integration tests for complete language switching flow:

- **Complete Flow**:
  - English → Spanish (LTR → LTR)
  - English → Arabic (LTR → RTL)
  - Arabic → English (RTL → LTR)
  - Hebrew RTL support

- **RTL Layout**:
  - All components update with RTL styles
  - Tailwind utilities work correctly

- **Persistence**:
  - Language saved to localStorage
  - Loaded on mount
  - RTL state persists

- **Lazy Loading**:
  - Loads translations on demand
  - Shows loading state
  - Falls back to English on error

- **Multiple Components**:
  - All components update simultaneously

- **Edge Cases**:
  - Rapid language switching
  - Missing translation keys

**Run**: `npm test languageSwitching.integration.test.tsx`

**Integration Total**: 20 tests

---

### E2E Tests (`frontend/tests/e2e/`)

#### 1. `language-switching.spec.ts` (30 tests)

Playwright E2E tests for full user workflow:

- **Complete Workflow**:
  - English → Spanish
  - English → Arabic (RTL)
  - English → Hebrew (RTL)
  - RTL → LTR switching

- **Persistent Preference**:
  - Survives page reload
  - RTL state persists

- **RTL Visual Regression**:
  - Arabic layout screenshot
  - Hebrew layout screenshot
  - LTR after RTL screenshot
  - Sidebar positioning verification

- **All 20 Languages**:
  - Individual test for each language
  - Verifies localStorage and translations

- **Accessibility**:
  - Keyboard navigation
  - ARIA labels
  - Screen reader announcements

- **Search**:
  - Filter languages by name

**Run**: `npx playwright test language-switching.spec.ts`

**E2E Total**: 30 tests

---

### Coverage Tests (`frontend/tests/i18n/`)

#### 1. `translationCoverage.test.ts` (40 tests)

Tests for translation completeness and type safety:

- **Required Keys** (60+ keys):
  - Common (welcome, loading, error, save, cancel, etc.)
  - Dashboard (title, pullRequests, filters, etc.)
  - Pull Requests (status, merge, author, etc.)
  - Settings (language, notifications, profile)
  - Languages (all 20 names)
  - Navigation, Auth, Errors

- **Translation Parity**:
  - All 20 languages have same keys as English
  - No missing translations
  - No empty values

- **Type Safety**:
  - TypeScript types match keys
  - Compile-time typo detection
  - Type-safe `useTranslation` hook

- **Pluralization**:
  - English plural forms (0, 1, many)
  - Complex rules (Polish, Russian, Arabic)

- **Interpolation**:
  - Variable substitution
  - Nested variables

- **Namespaces**:
  - Organized into logical groups
  - Lazy loading

- **Missing Key Detection**:
  - Reports missing keys
  - Falls back to English

- **Context & Variants**:
  - Contextual translations
  - Formal/informal variants

- **Performance**:
  - Loads in <200ms
  - Caches translations

**Run**: `npm test translationCoverage.test.ts`

**Coverage Total**: 40 tests

---

## Test Summary

| Category          | File                                     | Tests         | Status     |
| ----------------- | ---------------------------------------- | ------------- | ---------- |
| **Backend**       |                                          |               |            |
| Middleware        | `test_locale_middleware.rs`              | 18            | ✅ Written |
| API               | `test_user_preferences.rs`               | 15            | ✅ Written |
| Migration         | `test_migration.rs`                      | 12            | ✅ Written |
| **Frontend Unit** |                                          |               |            |
| RTL               | `RTLProvider.test.tsx`                   | 15            | ✅ Written |
| Config            | `i18nConfig.test.ts`                     | 20            | ✅ Written |
| Switcher          | `LanguageSwitcher.test.tsx`              | 35            | ✅ Written |
| Flags             | `FlagIcon.test.tsx`                      | 25            | ✅ Written |
| **Integration**   |                                          |               |            |
| Flow              | `languageSwitching.integration.test.tsx` | 20            | ✅ Written |
| **E2E**           |                                          |               |            |
| Playwright        | `language-switching.spec.ts`             | 30            | ✅ Written |
| **Coverage**      |                                          |               |            |
| Translation       | `translationCoverage.test.ts`            | 40            | ✅ Written |
| **TOTAL**         |                                          | **230 tests** | ✅         |

---

## Running Tests

### Backend

```bash
# All i18n tests
cargo test --test test_locale_middleware --test test_user_preferences --test test_migration

# Individual test files
cargo test --test test_locale_middleware
cargo test --test test_user_preferences
cargo test --test test_migration

# With PostgreSQL (recommended for full coverage)
export DATABASE_URL="postgres://ampel:ampel@localhost:5432/ampel_test"
export TEST_DATABASE_TYPE=postgres
cargo test --all-features
```

### Frontend

```bash
# All tests
npm test

# Unit tests only
npm test -- --run src/components/i18n/__tests__/

# Integration tests
npm test -- --run tests/i18n/

# E2E tests
npx playwright test

# Coverage report
npm test -- --coverage
```

---

## Coverage Goals

- **Backend**: 80%+ line coverage for i18n middleware and API
- **Frontend**: 80%+ line coverage for i18n components
- **E2E**: All 20 languages tested in real browser
- **Translation Files**: 100% key parity across all languages

---

## TDD Workflow

These tests were written **before** implementation following TDD:

1. **RED**: Tests fail (implementation doesn't exist yet)
2. **GREEN**: Implement features to make tests pass
3. **REFACTOR**: Improve code while keeping tests green

Current status: **RED phase** - awaiting implementation from other hivemind agents.

---

## Next Steps

1. **Frontend Developer**: Implement i18n config, RTLProvider, LanguageSwitcher, FlagIcon
2. **Backend Developer**: Implement locale middleware, user preferences API
3. **Database Developer**: Create migration for language column
4. **Integration**: Run tests and achieve 80%+ coverage
5. **Documentation**: Update with actual test results

---

## Coordination

Tests stored in hivemind coordination memory:

```bash
npx claude-flow@alpha hooks post-edit --file "crates/ampel-api/tests/i18n/test_locale_middleware.rs" --memory-key "hivemind/testing/backend-middleware"
npx claude-flow@alpha hooks post-edit --file "frontend/src/components/i18n/__tests__/RTLProvider.test.tsx" --memory-key "hivemind/testing/frontend-rtl"
```

---

**Generated by**: QE Test Engineer Agent
**Date**: 2025-12-27
**Hivemind Phase**: Phase 1 i18n Implementation
