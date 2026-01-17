# Required Fixes for i18n Test Suite

**Date**: 2026-01-09
**Priority**: CRITICAL
**Blocking**: Merge to main branch

---

## Overview

The frontend test validation revealed **275 test failures** out of 795 tests (34.6% failure rate). This document outlines the specific fixes required to achieve 100% test pass rate.

---

## Critical Fix #1: i18next Test Configuration

### Problem

i18next is not properly loading translation resources in the test environment, causing:

- Translation keys returned instead of translated text
- TypeError: Cannot read 'hasLanguageSomeTranslations'
- Resource store undefined

### Evidence

```javascript
// Expected:
"1 запрос" (Russian translation)

// Actual:
"common.pluralization.requests" (untranslated key)
```

### Affected Tests

- 70+ pluralization tests (Russian, Polish, Arabic)
- 7 translation loading tests
- 4 translation coverage tests
- **Total: ~81 failures**

### Required Changes

#### File: `frontend/tests/helpers/i18n-test-helper.ts`

**Current Issue**: i18next not loading resources

**Fix Required**:

```typescript
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import Backend from 'i18next-http-backend';

// Import actual translation files for tests
import enCommon from '../../public/locales/en/common.json';
import enDashboard from '../../public/locales/en/dashboard.json';
import ruCommon from '../../public/locales/ru/common.json';
import plCommon from '../../public/locales/pl/common.json';
import arCommon from '../../public/locales/ar/common.json';
// ... import other required translations

export function createTestI18n(lng: string = 'en') {
  const instance = i18n.createInstance();

  instance.use(initReactI18next).init({
    lng,
    fallbackLng: 'en',
    debug: false,
    interpolation: {
      escapeValue: false,
    },
    // CRITICAL: Add resources directly
    resources: {
      en: {
        common: enCommon,
        dashboard: enDashboard,
      },
      ru: {
        common: ruCommon,
      },
      pl: {
        common: plCommon,
      },
      ar: {
        common: arCommon,
      },
      // ... add other languages
    },
    // Ensure proper namespace handling
    defaultNS: 'common',
    ns: ['common', 'dashboard', 'settings', 'errors', 'validation'],
  });

  return instance;
}
```

#### File: `frontend/tests/setup.ts`

**Add Global Test Setup**:

```typescript
import { beforeAll, afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';
import { createTestI18n } from './helpers/i18n-test-helper';

// Clean up after each test
afterEach(() => {
  cleanup();
});

// Initialize i18next before all tests
beforeAll(async () => {
  const i18n = createTestI18n();
  await i18n.changeLanguage('en');
});
```

---

## Critical Fix #2: Test Timeouts

### Problem

Integration tests timing out at 10s default, particularly:

- RTL integration tests
- LanguageSwitcher integration tests
- Translation loading tests

### Evidence

```
Hook timed out in 10000ms.
Test timed out in 10000ms.
```

### Affected Tests

- 23+ integration tests timing out
- Particularly in 3 integration test files

### Required Changes

#### File: `frontend/vitest.config.ts`

**Current**: Default 10s timeout
**Fix Required**:

```typescript
export default defineConfig({
  test: {
    // Increase timeout for i18n tests
    testTimeout: 30000, // 30 seconds
    hookTimeout: 30000, // 30 seconds for beforeEach/afterEach

    // Keep short timeout for unit tests
    slowTestThreshold: 1000,
  },
});
```

**Alternative**: Per-test timeout increase in integration tests:

```typescript
// In integration test files
it('should load translations', async () => {
  // ... test code
}, 30000); // 30 second timeout
```

---

## Critical Fix #3: React act() Warnings

### Problem

Async i18next operations not wrapped in act(), causing warnings:

```
A suspended resource finished loading inside a test, but the event was not wrapped in act(...).
```

### Affected Tests

- ~15 tests with act warnings
- LanguageSwitcher integration tests
- RTL integration tests

### Required Changes

#### Pattern to Fix:

```typescript
// ❌ WRONG:
await i18n.changeLanguage('ar');
const result = screen.getByText('Arabic Text');

// ✅ CORRECT:
import { act, waitFor } from '@testing-library/react';

await act(async () => {
  await i18n.changeLanguage('ar');
});

await waitFor(() => {
  expect(screen.getByText('Arabic Text')).toBeInTheDocument();
});
```

#### Files to Update:

1. `frontend/tests/i18n/LanguageSwitcher.integration.test.tsx`
2. `frontend/tests/i18n/RTL.integration.test.tsx`
3. `frontend/tests/i18n/TranslationLoading.integration.test.tsx`

---

## Critical Fix #4: Linting Warnings (61 total)

### Problem

ESLint configured with `--max-warnings 0`, CI failing due to 61 warnings.

### Affected Files

- 14 test files with unused variables
- 2 test helper files with `any` types

### Required Changes

#### 1. Remove Unused Variables

**Pattern**:

```typescript
// ❌ WRONG:
const user = userEvent.setup();
// ... user never used

// ✅ CORRECT (if not needed):
// Remove the line entirely

// ✅ CORRECT (if intentionally unused):
const _user = userEvent.setup();
```

#### 2. Fix TypeScript any Types

**File**: `frontend/tests/helpers/i18n-test-helper.ts`

```typescript
// ❌ WRONG:
export function setupI18n(config: any) {
  // ...
}

// ✅ CORRECT:
import type { InitOptions } from 'i18next';

export function setupI18n(config: InitOptions) {
  // ...
}
```

#### 3. Remove Unused Imports

**Pattern**:

```typescript
// ❌ WRONG:
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
// Only using screen

// ✅ CORRECT:
import { screen } from '@testing-library/react';
```

---

## Fix Priority Order

### Phase 1: Critical Fixes (Blocking)

1. **Fix i18next test configuration** → Fixes 81 tests
2. **Increase test timeouts** → Fixes 23 tests
3. **Wrap async in act()** → Fixes 15 tests
4. **Fix linting warnings** → Unblocks CI

**Estimated Impact**: ~119 test fixes + CI unblock

### Phase 2: Remaining Failures

5. Investigate remaining ~176 test failures
6. Fix specific component integration issues
7. Address edge cases

---

## Validation Checklist

After implementing fixes, verify:

- [ ] Run: `cd frontend && pnpm test`
  - [ ] Expected: 795/795 tests passing (100%)
  - [ ] Expected: 0 test failures
  - [ ] Expected: Duration < 5 minutes

- [ ] Run: `cd frontend && pnpm run lint`
  - [ ] Expected: 0 errors, 0 warnings
  - [ ] Expected: Exit code 0

- [ ] Run: `cd frontend && pnpm run build`
  - [ ] Expected: Successful build
  - [ ] Expected: No TypeScript errors

- [ ] Integration test categories:
  - [ ] RTL integration tests: 11/11 passing
  - [ ] LanguageSwitcher tests: 14/14 passing
  - [ ] Translation loading tests: 8/8 passing
  - [ ] Pluralization tests: 70+ passing
  - [ ] Coverage tests: 4/4 passing

---

## Specific File Changes Required

### High Priority (Must Fix)

1. ✅ `frontend/tests/helpers/i18n-test-helper.ts` - Add resource loading
2. ✅ `frontend/tests/setup.ts` - Add global i18n initialization
3. ✅ `frontend/vitest.config.ts` - Increase timeouts
4. ✅ `frontend/tests/i18n/LanguageSwitcher.integration.test.tsx` - Wrap in act()
5. ✅ `frontend/tests/i18n/RTL.integration.test.tsx` - Wrap in act()
6. ✅ `frontend/tests/i18n/TranslationLoading.integration.test.tsx` - Wrap in act()

### Medium Priority (Cleanup)

7. Remove unused variables in 14 test files
8. Fix TypeScript `any` types in 2 files
9. Remove unused imports in 7 files

---

## Recommended Approach

### Step 1: Fix i18next Configuration

- Update `i18n-test-helper.ts` with resource imports
- Add resources to i18next init
- Test: Run pluralization tests to verify

### Step 2: Fix Timeouts

- Update `vitest.config.ts`
- Test: Run integration tests to verify

### Step 3: Fix act() Warnings

- Wrap all `changeLanguage()` calls in act()
- Add `waitFor()` for DOM updates
- Test: Run integration tests, verify no warnings

### Step 4: Fix Linting

- Run `pnpm run lint --fix` for auto-fixes
- Manually fix remaining warnings
- Test: Run `pnpm run lint`, expect 0 warnings

### Step 5: Final Validation

- Run full test suite
- Verify all 795 tests pass
- Verify build succeeds
- Verify no linting errors

---

## Expected Outcomes

After all fixes:

```
Test Results:
  Test Files  49 passed (49)
  Tests       795 passed (795)
  Duration    ~3-4 minutes

Linting:
  0 errors, 0 warnings

Build:
  ✓ TypeScript compilation successful
  ✓ Vite build successful
```

---

## Assistance Required

**Coder Agents**: Please implement the fixes in the priority order specified above.

**Tester Agent**: Will re-validate after fixes are complete.

**Timeline**: Estimated 4-6 hours for complete fix implementation and validation.
