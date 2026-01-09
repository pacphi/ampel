# Dashboard Component i18n Test Fixes

## Overview

This document summarizes the fixes applied to dashboard component tests to work with the i18n (internationalization) integration. The components now use translation keys instead of hardcoded strings, which required updating all test files to properly mock the `react-i18next` library.

## Problem

After implementing i18n in dashboard components, tests were failing because:

1. Components now call `useTranslation()` hook instead of using hardcoded strings
2. Tests were asserting against hardcoded English strings like "Draft", "Conflicts", "CI failed"
3. No i18n mocking was in place for the test environment

## Solution

### 1. Created i18n Test Helper (`tests/helpers/i18n-test-helper.ts`)

A reusable test utility that provides:

- **`createMockT()`**: Returns a mock translation function that returns keys
- **`createMockUseTranslation()`**: Mocks the useTranslation hook with key-based returns
- **`createMockTWithTranslations()`**: Returns actual translated strings for assertion tests
- **`mockTranslations`**: Mapping of translation keys to English strings

This helper can be used across all test files that need i18n support.

### 2. Updated Test Files

#### PRCard.test.tsx

**Changes:**

- Added `vi.mock('react-i18next')` at the top level (before component import)
- Mocked `useTranslation` to return actual translated strings
- Translation keys mapped:
  - `dashboard:blockers.draft` → "Draft"
  - `dashboard:blockers.conflicts` → "Conflicts"
  - `dashboard:blockers.ciFailed` → "CI failed"
  - `dashboard:blockers.ciPending` → "CI pending"
  - `dashboard:blockers.changesRequested` → "Changes requested"
  - `dashboard:blockers.awaitingReview` → "Awaiting review"
  - `dashboard:blockers.needsReview` → "Needs review"
  - `dashboard:actions.merge` → "Merge"

**Test Coverage:**

- ✅ 25 tests passing
- Basic display tests
- Status badge rendering
- Merge button logic
- Blocker display logic
- External links

#### ListView.test.tsx

**Changes:**

- Added i18n mock (component imports StatusBadge which uses translations)
- Mocked `useTranslation` hook at module level
- Tests validate list rendering and sorting functionality

**Test Coverage:**

- ✅ 8 tests passing
- Empty state display
- Repository list rendering
- Table headers and sorting
- Status badges
- External links

#### GridView.test.tsx

**Changes:**

- Added i18n mock (component imports RepoCard which may use translations)
- Mocked `useTranslation` hook at module level
- Tests validate grid layout and repository cards

**Test Coverage:**

- ✅ 4 tests passing
- Empty state display
- Grid layout rendering
- Repository card rendering

## Key Patterns

### Mock Placement (Critical!)

The `vi.mock()` call **must** be placed:

1. **After** vitest imports
2. **Before** the component import
3. At the **top level** of the test file (not inside describe blocks)

```typescript
import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

// ✅ CORRECT: Mock before component import
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key, // or return actual translations
    i18n: { language: 'en', changeLanguage: vi.fn() },
    ready: true,
  }),
}));

import MyComponent from './MyComponent';
```

### Translation Strategy

**Option 1: Return Keys (for most tests)**

```typescript
t: (key: string) => key;
```

Good when tests don't need to validate exact strings.

**Option 2: Return Translations (for assertion tests)**

```typescript
t: (key: string) => {
  const translations = {
    'dashboard:blockers.draft': 'Draft',
    'dashboard:actions.merge': 'Merge',
  };
  return translations[key] || key;
};
```

Use when tests assert against actual English strings.

## Test Results

All dashboard component tests now pass:

```
Test Files  8 passed (8)
Tests      129 passed (129)
Duration   9.32s
```

### Passing Test Suites:

- ✅ PRCard.test.tsx (25 tests)
- ✅ ListView.test.tsx (8 tests)
- ✅ GridView.test.tsx (4 tests)
- ✅ PRListView.test.tsx (11 tests)
- ✅ StatusBadge.test.tsx (15 tests)
- ✅ BreakdownTile.test.tsx (42 tests)
- ✅ SummaryBreakdownTile.test.tsx (20 tests)
- ✅ RepositoryStatusIcons.test.tsx (4 tests)

## Translation Keys Used

### Dashboard Namespace (`dashboard:`)

- `blockers.draft` - "Draft"
- `blockers.conflicts` - "Conflicts"
- `blockers.ciFailed` - "CI failed"
- `blockers.ciPending` - "CI pending"
- `blockers.changesRequested` - "Changes requested"
- `blockers.awaitingReview` - "Awaiting review"
- `blockers.needsReview` - "Needs review"
- `actions.merge` - "Merge"
- `empty.title` - "No repositories found"
- `empty.description` - "Add repositories from the Repositories page to get started"

### Common Namespace (`common:`)

- `visibility.public` - "Public"
- `visibility.private` - "Private"
- `visibility.archived` - "Archived"

## Best Practices

1. **Always mock i18n for component tests** - Even if the component doesn't directly use translations, child components might
2. **Use the helper utility** - The `i18n-test-helper.ts` provides consistent mocking patterns
3. **Keep translations up to date** - When adding new translation keys, update test mocks
4. **Test i18n integration separately** - Have dedicated integration tests for language switching
5. **Mock at module level** - Ensures all component instances use the mock

## Future Improvements

1. **Centralized Translation Mock**: Create a single test setup file that automatically mocks i18n for all tests
2. **Translation Key Validation**: Add tests that verify all translation keys exist in locale files
3. **Language Switching Tests**: Add tests that verify components re-render correctly when language changes
4. **RTL Language Tests**: Add tests for Right-to-Left language support (Arabic, Hebrew)

## Related Documentation

- [i18n Implementation Guide](../localization/DEVELOPER-GUIDE.md)
- [Translation Files Reference](../localization/TRANSLATION-FILES-REFERENCE.md)
- [Testing Guide](./I18N_TEST_SUMMARY.md)

## Troubleshooting

### Issue: "Cannot access '**vi_import_X**' before initialization"

**Solution:** Move `vi.mock()` call before component import

### Issue: Tests expecting English strings but getting keys

**Solution:** Use translation mapping in mock (see Option 2 above)

### Issue: "useTranslation: You will need to pass in an i18next instance"

**Solution:** Mock includes `i18n` object with required properties

---

**Last Updated:** 2026-01-09
**Status:** ✅ All dashboard tests passing
