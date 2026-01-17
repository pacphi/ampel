# Pluralization Testing Documentation

## Overview

This document describes the comprehensive pluralization testing strategy for Phase 2 i18n implementation, covering the five most complex languages: Finnish, Czech, Russian, Polish, and Arabic.

## Test Files Created

### 1. Finnish Pluralization (`tests/i18n/finnish-pluralization.test.ts`)

**Plural Forms**: 2 (one, other)

**Rules**:

- `one`: count === 1
- `other`: everything else (0, 2-999+)

**Test Cases**:

- 0 requests (other form)
- 1 request (one form)
- 2 requests (other form)
- 5, 10, 21, 100, 1000 requests (all other form)
- Edge cases: fractional numbers, negative numbers, very large numbers
- Runtime pluralization selection and language switching

**Total Tests**: 28

### 2. Czech Pluralization (`tests/i18n/czech-pluralization.test.ts`)

**Plural Forms**: 4 (one, few, many, other)

**Rules**:

- `one`: count === 1
- `few`: count === 2-4
- `many`: fractional numbers (1.5, 2.7, etc.)
- `other`: 0, 5+

**Test Cases**:

- 0 requests (other)
- 1 request (one)
- 2, 3, 4 requests (few)
- 5, 10, 100 requests (other)
- 1.5, 2.7 requests (many)
- Plural form boundaries (1→2, 4→5, 1→1.5)
- Edge cases: 0.5, negative numbers, very large numbers

**Total Tests**: Comprehensive coverage of all 4 forms

### 3. Slavic Pluralization (`tests/i18n/slavic-pluralization.test.ts`)

**Languages**: Russian & Polish

**Plural Forms**: 3 (one, few, many)

**Rules** (same for both languages):

- `one`: count % 10 === 1 && count % 100 !== 11
- `few`: count % 10 in 2..4 && count % 100 not in 12..14
- `many`: everything else (0, 5-20, 25-30, etc.)

**Test Cases**:

**Russian**:

- 0, 1, 2, 3, 4, 5, 10, 11, 12, 21, 22, 25, 100, 101 requests
- 10-19 range (all many)
- 21-24 range (one, few, few, few)
- Edge cases: negative numbers, very large numbers

**Polish**:

- Same test cases as Russian
- Cross-language consistency verification

**Total Tests**: 45+ tests

### 4. Arabic Pluralization (`tests/i18n/arabic-pluralization.test.ts`)

**Plural Forms**: 6 (zero, one, two, few, many, other) - Most complex!

**Rules**:

- `zero`: count === 0
- `one`: count === 1
- `two`: count === 2
- `few`: count % 100 in 3..10
- `many`: count % 100 in 11..99
- `other`: count >= 100 or fractional numbers

**Test Cases**:

- All six forms: 0, 1, 2, 3-10, 11-99, 100+ requests
- Plural form boundaries (0→1, 1→2, 2→3, 10→11, 99→100)
- Range testing:
  - Few form (3-10): all numbers 3-10, also 103-110
  - Many form (11-99): 11, 12, 15, 20, 25, 50, 75, 99, also 111-199
- Edge cases: fractional numbers, very large numbers
- Complex scenarios: hundreds with different last two digits

**Total Tests**: 60+ tests covering all 6 forms

## Translation Keys Structure

All pluralization keys follow i18next v21+ format with suffixes:

```json
{
  "pluralization": {
    "requests_one": "{{count}} request",
    "requests_other": "{{count}} requests",
    "requests_few": "{{count}} requests", // Czech, Russian, Polish
    "requests_many": "{{count}} requests", // Czech, Russian, Polish, Arabic
    "requests_zero": "no requests", // Arabic only
    "requests_two": "two requests" // Arabic only
  }
}
```

## Test Coverage Goals

### ✅ Completed

- [x] Test suite structure for all 5 languages
- [x] Edge case testing (0, 1, 2, boundaries)
- [x] All plural forms tested per language
- [x] Translation keys added to all language files
- [x] i18next configuration verification tests
- [x] Runtime pluralization selection tests
- [x] Language switching tests
- [x] Translation completeness tests

### ⚠️ Pending

- [ ] Integration with actual i18next test environment
- [ ] Fix i18next pluralization initialization in tests
- [ ] Verify translations load correctly from JSON files
- [ ] Full test execution and pass rate

## Known Issues

1. **Test Environment Setup**: Tests currently fail because i18next isn't correctly loading/applying plural rules in the test environment.
2. **Translation Loading**: Need to verify JSON import and resource loading in vitest.
3. **Plural Suffix Resolution**: i18next may need additional configuration for proper plural suffix selection.

## Next Steps

1. Debug i18next test configuration to enable proper pluralization
2. Run all test suites and achieve 100% pass rate
3. Add visual regression tests for RTL (Arabic)
4. Integrate with CI/CD pipeline
5. Add performance benchmarks for plural form selection

## Reference Links

- [Finnish Plural Rules (CLDR)](https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#fi)
- [Czech Plural Rules (CLDR)](https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#cs)
- [Russian Plural Rules (CLDR)](https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#ru)
- [Polish Plural Rules (CLDR)](https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#pl)
- [Arabic Plural Rules (CLDR)](https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#ar)
- [i18next Pluralization Docs](https://www.i18next.com/translation-function/plurals)

## Examples from Tests

### Finnish (2 forms)

```typescript
// 0 requests → "0 pyyntöä" (other)
// 1 request → "1 pyyntö" (one)
// 2 requests → "2 pyyntöä" (other)
```

### Czech (4 forms)

```typescript
// 0 requests → "0 požadavků" (other)
// 1 request → "1 požadavek" (one)
// 2 requests → "2 požadavky" (few)
// 1.5 requests → "1.5 požadavku" (many)
// 5 requests → "5 požadavků" (other)
```

### Russian/Polish (3 forms)

```typescript
// 1 request → "1 запрос" (one)
// 2 requests → "2 запроса" (few)
// 5 requests → "5 запросов" (many)
// 11 requests → "11 запросов" (many - exception!)
// 21 requests → "21 запрос" (one - ends in 1)
```

### Arabic (6 forms)

```typescript
// 0 requests → "لا طلبات" (zero)
// 1 request → "طلب واحد" (one)
// 2 requests → "طلبان" (two)
// 5 requests → "5 طلبات" (few)
// 20 requests → "20 طلباً" (many)
// 100 requests → "100 طلب" (other)
```

## Acceptance Criteria Status

- ✅ All plural forms tested for each language
- ✅ Edge cases covered (0, 1, 2, boundaries)
- ⚠️ Tests pass with translated strings (pending i18next config fix)
- ✅ Documentation of pluralization rules

## Generated

- **Date**: 2025-12-27
- **Phase**: 2 - Complex Pluralization Testing
- **Status**: Test structure complete, execution pending i18next configuration
- **Languages**: Finnish, Czech, Russian, Polish, Arabic
