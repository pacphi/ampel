# RTL Visual Regression Testing Guide

This directory contains comprehensive visual regression tests for RTL (Right-to-Left) language support in Ampel.

## Overview

The test suite validates:

- ✅ Arabic (ar) and Hebrew (he) layout rendering
- ✅ CSS logical properties usage
- ✅ Bidirectional text handling
- ✅ Icon directional flipping
- ✅ Form controls in RTL
- ✅ Responsive RTL layouts
- ✅ RTL to LTR switching

## Test Files

### Visual Regression Tests (Playwright)

- **rtl-dashboard.spec.ts** - Dashboard page visual tests
  - Full page screenshots (ar, he)
  - Sidebar positioning
  - Summary tiles RTL alignment
  - PR cards layout
  - Navigation menu
  - Responsive viewports (desktop, laptop, tablet, mobile)
  - Icon directional flipping
  - RTL ↔ LTR switching

- **rtl-settings.spec.ts** - Settings page visual tests
  - Profile settings RTL layout
  - Form input alignment
  - Button positioning
  - Checkboxes and form controls
  - Dropdown menu alignment
  - All settings sections (accounts, filters, notifications, behavior)
  - Mobile responsive layouts

### Integration Tests (Vitest)

- **rtl-layout.test.tsx** - RTL provider and direction switching
  - RTLProvider component behavior
  - Document attributes (dir, lang, class)
  - Meta tag management
  - Language switching (RTL ↔ LTR)

- **css-logical-properties.test.ts** - CSS validation
  - Scans codebase for hardcoded directional CSS
  - Validates use of logical properties
  - Provides migration recommendations
  - Calculates RTL compatibility score

- **bidirectional-text.test.tsx** - BiDi text handling
  - Mixed LTR/RTL content
  - Numbers in RTL text
  - URLs and code snippets in RTL
  - Punctuation handling
  - Unicode BiDi control characters
  - Form inputs with mixed content

## Running Tests

### Visual Regression Tests

```bash
# Install Playwright browsers (first time only)
npx playwright install

# Run all visual tests
pnpm exec playwright test

# Run specific test file
pnpm exec playwright test rtl-dashboard

# Run tests for specific browser
pnpm exec playwright test --project=chromium-rtl-arabic

# Run tests in headed mode (see browser)
pnpm exec playwright test --headed

# Update baseline screenshots
pnpm exec playwright test --update-snapshots

# View HTML report
pnpm exec playwright show-report
```

### Integration Tests

```bash
# Run all integration tests
pnpm test tests/i18n/

# Run specific test file
pnpm test rtl-layout.test.tsx

# Run with coverage
pnpm test --coverage tests/i18n/

# Run in watch mode
pnpm test --watch rtl-layout
```

## Browser Coverage

Visual tests run on multiple browser engines:

- **chromium-ltr** - Baseline LTR (English)
- **chromium-rtl-arabic** - Chromium with Arabic locale
- **chromium-rtl-hebrew** - Chromium with Hebrew locale
- **firefox-rtl-arabic** - Firefox with Arabic locale
- **webkit-rtl-arabic** - Safari with Arabic locale

## Viewport Coverage

Tests validate responsive RTL across:

- Desktop: 1920x1080
- Laptop: 1366x768
- Tablet: 768x1024
- Mobile: 375x667

## Screenshot Organization

Baseline screenshots are stored in:

```
tests/visual/rtl-dashboard.spec.ts-snapshots/
tests/visual/rtl-settings.spec.ts-snapshots/
```

Each screenshot is named by test, browser, and platform:

```
dashboard-arabic-full-chromium-rtl-arabic-linux.png
settings-hebrew-full-chromium-rtl-hebrew-linux.png
```

## CSS Logical Properties

### Approved Properties

✅ **Use these:**

- `ms-*` (margin-inline-start) instead of `ml-*`
- `me-*` (margin-inline-end) instead of `mr-*`
- `ps-*` (padding-inline-start) instead of `pl-*`
- `pe-*` (padding-inline-end) instead of `pr-*`
- `text-start` instead of `text-left`
- `text-end` instead of `text-right`
- `start-*` (position) instead of `left-*`
- `end-*` (position) instead of `right-*`

❌ **Avoid these:**

- `ml-*`, `mr-*`, `pl-*`, `pr-*`
- `text-left`, `text-right`
- `left-*`, `right-*`
- `float: left/right`

### Migration Example

```tsx
// ❌ Bad (hardcoded direction)
<div className="ml-4 text-left">Content</div>

// ✅ Good (logical properties)
<div className="ms-4 text-start">Content</div>
```

## Acceptance Criteria

For Phase 2 RTL implementation to be complete:

### Dashboard

- [ ] All pages render correctly in Arabic
- [ ] All pages render correctly in Hebrew
- [ ] Sidebar appears on right in RTL
- [ ] Summary tiles aligned properly
- [ ] PR cards mirror correctly
- [ ] Navigation menu on correct side
- [ ] Icons flip appropriately
- [ ] No layout breaks or overlaps

### Settings

- [ ] All settings pages render in Arabic
- [ ] All settings pages render in Hebrew
- [ ] Forms align correctly
- [ ] Input fields have proper text direction
- [ ] Buttons positioned correctly
- [ ] Dropdowns align properly
- [ ] Mobile layouts work in RTL

### Code Quality

- [ ] No hardcoded `ml-*`, `mr-*` classes
- [ ] No hardcoded `pl-*`, `pr-*` classes
- [ ] No `text-left`, `text-right` usage
- [ ] All components use logical properties
- [ ] RTL compatibility score > 90%

### Functionality

- [ ] Language switching works (RTL ↔ LTR)
- [ ] RTL state persists across reloads
- [ ] Bidirectional text renders correctly
- [ ] Numbers, URLs display properly in RTL
- [ ] Form inputs handle mixed content

## Debugging Tips

### Visual Differences

If visual tests fail due to legitimate differences:

1. Review the diff image in `test-results/`
2. If change is intentional, update snapshots:
   ```bash
   pnpm exec playwright test --update-snapshots
   ```

### RTL Not Applied

If RTL isn't working in tests:

1. Check localStorage is set: `ampel-i18n-lng: 'ar'`
2. Verify i18next initialized: `window.i18next.isInitialized`
3. Check document.dir: should be `'rtl'`
4. Check html class: should have `'rtl'` class

### CSS Violations

If logical property violations are found:

1. Review console warnings in test output
2. Check file paths for violations
3. Apply migration guide recommendations
4. Re-run tests to verify fixes

## CI/CD Integration

Visual tests run automatically on:

- Pull requests to `main`
- Commits to feature branches
- Before deployments

Test results are published as GitHub Actions artifacts.

## Reporting Issues

When reporting RTL visual bugs, include:

1. Language (ar or he)
2. Browser and version
3. Screenshot showing issue
4. Expected vs actual behavior
5. Steps to reproduce

## Additional Resources

- [W3C BiDi Text](https://www.w3.org/International/articles/inline-bidi-markup/)
- [CSS Logical Properties](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Logical_Properties)
- [Tailwind RTL Plugin](https://tailwindcss.com/docs/text-align#using-logical-properties)
- [Playwright Testing](https://playwright.dev/docs/intro)

---

**Last Updated**: 2024-12-27
**Test Coverage**: Dashboard, Settings, RTL Layouts, BiDi Text
**Browser Coverage**: Chromium, Firefox, WebKit
**Languages**: Arabic (ar), Hebrew (he)
