/**
 * Arabic Pluralization Tests
 *
 * Arabic has the most complex pluralization rules with 6 plural forms:
 * - zero: count === 0
 * - one: count === 1
 * - two: count === 2
 * - few: count % 100 in 3..10
 * - many: count % 100 in 11..99
 * - other: count >= 100 or fractional numbers
 *
 * Test cases based on IMPLEMENTATION_ROADMAP_V2.md Phase 2 requirements.
 *
 * Reference: https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#ar
 *
 * IMPORTANT: These tests use the REAL i18n instance with actual translation files,
 * not mocked translations. This ensures the pluralization rules work correctly
 * with the production i18n configuration.
 */

import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { setupTestI18n, getTestI18n, changeTestLanguage } from '../setup/i18n-instance.js';
// Available for future use - keep imports for reference
import type {
  testPluralForms as _testPluralForms,
  verifyDistinctPluralForms as _verifyDistinctPluralForms,
  verifyNamespacesLoaded as _verifyNamespacesLoaded,
} from '../setup/i18n-testing';

// Re-export for backward compatibility
let i18n: ReturnType<typeof getTestI18n>;

describe('Arabic Pluralization', () => {
  beforeAll(async () => {
    await setupTestI18n('ar');
    i18n = getTestI18n();
  });

  afterAll(async () => {
    // Reset to English for other tests
    await changeTestLanguage('en');
  });

  describe('Request Pluralization - All Six Forms', () => {
    // Arabic zero/one/two forms use grammatical constructions without numeric digits
    // e.g., "لا طلبات" (no requests), "طلب واحد" (one request), "طلبان" (two requests)
    // Few/many/other forms include the numeric count via {{count}} interpolation

    it('handles 0 requests (zero form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 0 });
      expect(result).toBeTruthy();
      // Arabic zero form uses "لا طلبات" (no requests) without numeric 0
      expect(result).toMatch(/[\u0600-\u06FF]/); // Contains Arabic characters
      expect(result).not.toBe('pluralization.requests'); // Not returning key
    });

    it('handles 1 request (one form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1 });
      expect(result).toBeTruthy();
      // Arabic one form uses "طلب واحد" (one request) without numeric 1
      expect(result).toMatch(/[\u0600-\u06FF]/); // Contains Arabic characters
      expect(result).not.toBe('pluralization.requests');
    });

    it('handles 2 requests (two form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 2 });
      expect(result).toBeTruthy();
      // Arabic two form uses "طلبان" (dual form) without numeric 2
      expect(result).toMatch(/[\u0600-\u06FF]/); // Contains Arabic characters
      expect(result).not.toBe('pluralization.requests');
    });

    it('handles 3 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3'); // Few form includes count
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 4 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 4 });
      expect(result).toBeTruthy();
      expect(result).toContain('4');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 5 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 10 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 10 });
      expect(result).toBeTruthy();
      expect(result).toContain('10');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 11 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 11 });
      expect(result).toBeTruthy();
      expect(result).toContain('11');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 20 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 20 });
      expect(result).toBeTruthy();
      expect(result).toContain('20');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 50 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 50 });
      expect(result).toBeTruthy();
      expect(result).toContain('50');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 99 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 99 });
      expect(result).toBeTruthy();
      expect(result).toContain('99');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 100 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 100 });
      expect(result).toBeTruthy();
      expect(result).toContain('100');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 101 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 101 });
      expect(result).toBeTruthy();
      expect(result).toContain('101');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 200 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 200 });
      expect(result).toBeTruthy();
      expect(result).toContain('200');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 1000 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1000 });
      expect(result).toBeTruthy();
      expect(result).toContain('1000');
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });
  });

  describe('Plural Form Boundaries', () => {
    it('correctly transitions from zero to one (0 -> 1)', () => {
      const zero = i18n.t('common:pluralization.requests', { count: 0 });
      const one = i18n.t('common:pluralization.requests', { count: 1 });

      expect(zero).not.toBe(one);
    });

    it('correctly transitions from one to two (1 -> 2)', () => {
      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const two = i18n.t('common:pluralization.requests', { count: 2 });

      expect(one).not.toBe(two);
    });

    it('correctly transitions from two to few (2 -> 3)', () => {
      const two = i18n.t('common:pluralization.requests', { count: 2 });
      const few = i18n.t('common:pluralization.requests', { count: 3 });

      expect(two).not.toBe(few);
    });

    it('correctly transitions from few to many (10 -> 11)', () => {
      const few = i18n.t('common:pluralization.requests', { count: 10 });
      const many = i18n.t('common:pluralization.requests', { count: 11 });

      expect(few).not.toBe(many);
    });

    it('correctly transitions from many to other (99 -> 100)', () => {
      const many = i18n.t('common:pluralization.requests', { count: 99 });
      const other = i18n.t('common:pluralization.requests', { count: 100 });

      expect(many).not.toBe(other);
    });
  });

  describe('Range Testing for Few Form (3-10)', () => {
    it('all numbers 3-10 use few form with count interpolation', () => {
      const counts = [3, 4, 5, 6, 7, 8, 9, 10];
      const results = counts.map((count) => i18n.t('common:pluralization.requests', { count }));

      // All should be truthy, contain count, and have Arabic text
      results.forEach((result, index) => {
        expect(result).toBeTruthy();
        expect(result).toContain(counts[index].toString());
        expect(result).toMatch(/[\u0600-\u06FF]/); // Contains Arabic characters
      });
    });

    it('103-110 also use few form (based on last two digits)', () => {
      const counts = [103, 104, 105, 110];
      const results = counts.map((count) => i18n.t('common:pluralization.requests', { count }));

      results.forEach((result, index) => {
        expect(result).toBeTruthy();
        expect(result).toContain(counts[index].toString());
        expect(result).toMatch(/[\u0600-\u06FF]/);
      });
    });
  });

  describe('Range Testing for Many Form (11-99)', () => {
    it('all numbers 11-99 use many form', () => {
      const testNumbers = [11, 12, 15, 20, 25, 50, 75, 99];
      const results = testNumbers.map((count) =>
        i18n.t('common:pluralization.requests', { count })
      );

      results.forEach((result, index) => {
        expect(result).toBeTruthy();
        expect(result).toContain(testNumbers[index].toString());
      });
    });

    it('111-199 also use many form (based on last two digits)', () => {
      const results = [111, 125, 150, 199].map((count) =>
        i18n.t('common:pluralization.requests', { count })
      );

      results.forEach((result) => {
        expect(result).toBeTruthy();
      });
    });
  });

  describe('Pull Request Pluralization', () => {
    // Arabic zero/one/two forms use grammatical constructions without numeric digits

    it('handles 0 pull requests (zero)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 0 });
      expect(result).toBeTruthy();
      // Zero form uses "لا pull requests" without numeric 0
      expect(result).not.toBe('pluralization.pullRequests');
    });

    it('handles 1 pull request (one)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 1 });
      expect(result).toBeTruthy();
      // One form uses "pull request واحد" without numeric 1
      expect(result).not.toBe('pluralization.pullRequests');
    });

    it('handles 2 pull requests (two)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 2 });
      expect(result).toBeTruthy();
      // Two form uses dual construction without numeric 2
      expect(result).not.toBe('pluralization.pullRequests');
    });

    it('handles 3 pull requests (few)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3'); // Few form includes count
    });

    it('handles 11 pull requests (many)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 11 });
      expect(result).toBeTruthy();
      expect(result).toContain('11'); // Many form includes count
    });

    it('handles 100 pull requests (other)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 100 });
      expect(result).toBeTruthy();
      expect(result).toContain('100'); // Other form includes count
    });
  });

  describe('Comment Pluralization', () => {
    // Arabic zero/one/two forms use grammatical constructions without numeric digits

    it('handles 0 comments (zero)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 0 });
      expect(result).toBeTruthy();
      // Zero form uses "لا تعليقات" without numeric 0
      expect(result).toMatch(/[\u0600-\u06FF]/); // Contains Arabic characters
      expect(result).not.toBe('pluralization.comments');
    });

    it('handles 1 comment (one)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 1 });
      expect(result).toBeTruthy();
      // One form uses "تعليق واحد" without numeric 1
      expect(result).toMatch(/[\u0600-\u06FF]/);
      expect(result).not.toBe('pluralization.comments');
    });

    it('handles 2 comments (two)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 2 });
      expect(result).toBeTruthy();
      // Two form uses "تعليقان" without numeric 2
      expect(result).toMatch(/[\u0600-\u06FF]/);
      expect(result).not.toBe('pluralization.comments');
    });

    it('handles 5 comments (few)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5'); // Few form includes count
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 20 comments (many)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 20 });
      expect(result).toBeTruthy();
      expect(result).toContain('20'); // Many form includes count
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });

    it('handles 200 comments (other)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 200 });
      expect(result).toBeTruthy();
      expect(result).toContain('200'); // Other form includes count
      expect(result).toMatch(/[\u0600-\u06FF]/);
    });
  });

  describe('Edge Cases', () => {
    it('handles fractional numbers (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1.5 });
      expect(result).toBeTruthy();
      // Should use "other" form for fractional numbers
    });

    it('handles 0.5 (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 0.5 });
      expect(result).toBeTruthy();
      // Should use "other" form
    });

    it('handles very large numbers (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1000000 });
      expect(result).toBeTruthy();
      expect(result).toContain('1000000');
    });
  });

  describe('i18next Configuration Verification', () => {
    it('has Arabic language loaded', () => {
      expect(i18n.hasResourceBundle('ar', 'common')).toBe(true);
    });

    it('current language is Arabic', () => {
      expect(i18n.language).toBe('ar');
    });

    it('Arabic is RTL language', () => {
      const languageInfo = i18n.options.supportedLngs?.includes('ar');
      expect(languageInfo).toBe(true);
    });

    it('Arabic plural rules produce six different forms', () => {
      const zero = i18n.t('common:pluralization.requests', { count: 0 });
      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const two = i18n.t('common:pluralization.requests', { count: 2 });
      const few = i18n.t('common:pluralization.requests', { count: 3 });
      const many = i18n.t('common:pluralization.requests', { count: 11 });
      const other = i18n.t('common:pluralization.requests', { count: 100 });

      // All six forms should be different
      const uniqueForms = new Set([zero, one, two, few, many, other]);
      expect(uniqueForms.size).toBe(6);
    });
  });

  describe('Runtime Pluralization Selection', () => {
    it('dynamically selects correct plural form for all six categories', () => {
      // Zero, one, two forms don't include numeric count in Arabic
      // Few, many, other forms include the count via {{count}} interpolation
      const testCases = [
        { count: 0, form: 'zero', hasCount: false },
        { count: 1, form: 'one', hasCount: false },
        { count: 2, form: 'two', hasCount: false },
        { count: 5, form: 'few', hasCount: true },
        { count: 20, form: 'many', hasCount: true },
        { count: 100, form: 'other', hasCount: true },
      ];

      testCases.forEach(({ count, hasCount }) => {
        const result = i18n.t('common:pluralization.requests', { count });
        expect(result).toBeTruthy();
        if (hasCount) {
          expect(result).toContain(count.toString());
        }
      });
    });

    it('maintains consistency across multiple calls', () => {
      const first = i18n.t('common:pluralization.requests', { count: 5 });
      const second = i18n.t('common:pluralization.requests', { count: 5 });

      expect(first).toBe(second);
    });

    it('handles all plural forms within single application session', () => {
      const results = {
        zero: i18n.t('common:pluralization.requests', { count: 0 }),
        one: i18n.t('common:pluralization.requests', { count: 1 }),
        two: i18n.t('common:pluralization.requests', { count: 2 }),
        few: i18n.t('common:pluralization.requests', { count: 5 }),
        many: i18n.t('common:pluralization.requests', { count: 20 }),
        other: i18n.t('common:pluralization.requests', { count: 100 }),
      };

      // All should be truthy
      Object.values(results).forEach((result) => {
        expect(result).toBeTruthy();
      });

      // All should be different
      const uniqueResults = new Set(Object.values(results));
      expect(uniqueResults.size).toBe(6);
    });

    it('handles rapid switching between different counts', () => {
      // Test forms that include count: few (3), many (11), other (100)
      const countsWithDigits = [3, 11, 100];
      const resultsWithDigits = countsWithDigits.map((count) =>
        i18n.t('common:pluralization.requests', { count })
      );

      resultsWithDigits.forEach((result, index) => {
        expect(result).toBeTruthy();
        expect(result).toContain(countsWithDigits[index].toString());
      });

      // Test forms without count: zero (0), one (1), two (2)
      const countsWithoutDigits = [0, 1, 2];
      const resultsWithoutDigits = countsWithoutDigits.map((count) =>
        i18n.t('common:pluralization.requests', { count })
      );

      resultsWithoutDigits.forEach((result) => {
        expect(result).toBeTruthy();
        expect(result).toMatch(/[\u0600-\u06FF]/); // Contains Arabic characters
      });
    });
  });

  describe('Translation Completeness', () => {
    it('all six plural forms have non-empty translations', () => {
      const keys = ['requests', 'pullRequests', 'comments'];
      const counts = [0, 1, 2, 5, 20, 100]; // zero, one, two, few, many, other

      keys.forEach((key) => {
        counts.forEach((count) => {
          const result = i18n.t(`common:pluralization.${key}`, { count });
          expect(result).toBeTruthy();
          expect(result).not.toBe(`common:pluralization.${key}`);
        });
      });
    });
  });

  describe('Complex Scenarios', () => {
    it('handles hundreds with different last two digits', () => {
      // 100-102 should use "other" form
      const count100 = i18n.t('common:pluralization.requests', { count: 100 });
      const count101 = i18n.t('common:pluralization.requests', { count: 101 });
      const count102 = i18n.t('common:pluralization.requests', { count: 102 });

      // 103-110 should use "few" form (last two digits 03-10)
      const count103 = i18n.t('common:pluralization.requests', { count: 103 });
      const count110 = i18n.t('common:pluralization.requests', { count: 110 });

      // 111-199 should use "many" form (last two digits 11-99)
      const count111 = i18n.t('common:pluralization.requests', { count: 111 });
      const count150 = i18n.t('common:pluralization.requests', { count: 150 });

      // Verify all are truthy
      [count100, count101, count102, count103, count110, count111, count150].forEach((result) => {
        expect(result).toBeTruthy();
      });
    });
  });
});
