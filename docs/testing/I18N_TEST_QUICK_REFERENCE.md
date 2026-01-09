# i18n Test Quick Reference

## Quick Command Reference

```bash
# Run all i18n integration tests
npm test -- tests/i18n/ --run

# Run specific test file
npm test -- tests/i18n/LanguageSwitcher.integration.test.tsx --run

# Run tests in watch mode (for development)
npm test -- tests/i18n/LanguageSwitcher.integration.test.tsx

# Run E2E tests
npm run test:e2e tests/e2e/language-switching.spec.ts
```

## Test File Organization

```
frontend/tests/i18n/
├── LanguageSwitcher.integration.test.tsx    # Language switcher component tests
├── TranslationLoading.integration.test.tsx  # Translation file loading tests
├── RTL.integration.test.tsx                 # RTL layout and direction tests
├── languageSwitching.integration.test.tsx   # (older tests - may be duplicate)
├── translationCoverage.test.ts              # Translation completeness tests
├── *-pluralization.test.ts                  # Language-specific pluralization
├── bidirectional-text.test.tsx              # RTL/LTR text mixing
├── rtl-layout.test.tsx                      # RTL layout components
└── css-logical-properties.test.ts           # CSS logical properties for RTL

frontend/tests/e2e/
└── language-switching.spec.ts               # End-to-end language switching tests
```

## Common Test Patterns

### 1. Testing Language Switching

```typescript
it('should switch language', async () => {
  const user = userEvent.setup();
  renderWithProviders(<LanguageSwitcher variant="dropdown" />);

  // Open dropdown
  await user.click(screen.getByRole('combobox'));

  await waitFor(
    () => {
      expect(screen.getByRole('listbox')).toBeInTheDocument();
    },
    { timeout: 3000 }
  );

  // Select language
  await user.click(screen.getByText('French'));

  // Verify change
  await waitFor(
    () => {
      expect(i18n.language).toBe('fr');
    },
    { timeout: 3000 }
  );
});
```

### 2. Testing RTL Layout

```typescript
it('should set RTL for Arabic', async () => {
  await i18n.changeLanguage('ar');

  render(
    <I18nextProvider i18n={i18n}>
      <RTLProvider>
        <Component />
      </RTLProvider>
    </I18nextProvider>
  );

  await waitFor(() => {
    expect(document.documentElement.getAttribute('dir')).toBe('rtl');
    expect(document.documentElement.classList.contains('rtl')).toBe(true);
  });
});
```

### 3. Testing Translation Loading

```typescript
it('should load translations', async () => {
  await i18n.changeLanguage('en');
  await i18n.loadNamespaces('common');

  await waitFor(
    () => {
      expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
    },
    { timeout: 5000 }
  );

  const translation = i18n.t('common:language');
  expect(translation).not.toBe('common:language'); // Should not return key
  expect(typeof translation).toBe('string');
});
```

## Common Issues and Solutions

### Issue 1: Test Timeout

**Problem:** Test times out waiting for dropdown/translations

```
Hook timed out in 10000ms
```

**Solution:** Increase timeout in `waitFor`:

```typescript
await waitFor(
  () => {
    expect(screen.getByRole('listbox')).toBeInTheDocument();
  },
  { timeout: 3000 } // or 5000 for slow operations
);
```

### Issue 2: Multiple Elements Found

**Problem:** `getByText` finds multiple elements with same text

```
TestingLibraryElementError: Found multiple elements with text: English (US)
```

**Solution:** Use more specific selectors:

```typescript
// Instead of:
const element = screen.getByText('English (US)');

// Use:
const elements = screen.getAllByText('English (US)');
const element = elements[0]; // or filter by other attributes

// Or use role with aria attributes:
const element = screen.getByRole('option', { selected: true });
```

### Issue 3: Unused Variables

**Problem:** ESLint warns about unused variables

```
'user' is defined but never used
```

**Solution:** Only destructure what you use:

```typescript
// If you don't use the variable, don't destructure it:
render(...); // instead of: const { rerender } = render(...);

// Or prefix with underscore if required by test framework:
const { rerender: _rerender } = render(...);
```

### Issue 4: React Suspense Warnings

**Problem:** Act warnings about suspended resources

```
A suspended resource finished loading inside a test, but the event was not wrapped in act(...)
```

**Solution:** Use `waitFor` for all async operations:

```typescript
await waitFor(() => {
  expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
});
```

## Debugging Tips

### 1. View Rendered Component

```typescript
import { screen, debug } from '@testing-library/react';

// Print entire DOM
screen.debug();

// Print specific element
const element = screen.getByRole('combobox');
screen.debug(element);
```

### 2. Check Available Queries

```typescript
// See all available text content
screen.getByText(/./); // Will fail and show all text

// See all available roles
screen.getByRole('nonexistent'); // Will fail and show all roles
```

### 3. Inspect i18n State

```typescript
console.log('Current language:', i18n.language);
console.log('Available languages:', i18n.languages);
console.log('Has bundle:', i18n.hasResourceBundle('en', 'common'));
console.log('Translation:', i18n.t('common:language'));
```

## Test Coverage Goals

### Current Coverage (Estimated)

- ✅ LanguageSwitcher component: ~85%
- ✅ Translation loading: ~90%
- ✅ RTL functionality: ~95%
- ⚠️ Search functionality: ~60%
- ⚠️ Favorites functionality: ~50%

### Priority Coverage Gaps

1. Language switcher search edge cases
2. Favorites persistence across sessions
3. Keyboard navigation complete flow
4. Mobile select variant
5. Translation interpolation with variables

## Best Practices

### DO ✅

- Use `waitFor` for all async operations
- Test actual behavior (translations, RTL), not implementation
- Use proper timeouts (3000-5000ms for complex operations)
- Clean up document state in `afterEach`
- Test with real i18n instance, not mocks

### DON'T ❌

- Don't use fixed delays (`setTimeout`)
- Don't test implementation details (internal state)
- Don't mock i18n unless absolutely necessary
- Don't forget to reset language in `beforeEach`
- Don't use synchronous tests for async operations

## CI/CD Considerations

### GitHub Actions

```yaml
- name: Run i18n tests
  run: |
    cd frontend
    npm test -- tests/i18n/ --run
  timeout-minutes: 10 # Allow extra time for i18n loading
```

### Performance

- Translation loading: ~500-2000ms
- Dropdown animations: ~200-500ms
- RTL layout switching: ~100-300ms
- Total test suite: ~2-5 minutes

## Further Reading

- [Testing Library Best Practices](https://testing-library.com/docs/guiding-principles/)
- [i18next Testing Guide](https://www.i18next.com/misc/testing)
- [React Testing Patterns](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)
- [Main Test Implementation Summary](./I18N_INTEGRATION_TEST_FIXES.md)
