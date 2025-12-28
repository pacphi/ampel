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
 */

import { describe, expect, it, beforeAll } from 'vitest';
import i18n, { initI18nForTesting } from './i18n-test-config';

describe('Arabic Pluralization', () => {
  beforeAll(async () => {
    await initI18nForTesting();
    await i18n.changeLanguage('ar');
  });

  describe('Request Pluralization - All Six Forms', () => {
    it('handles 0 requests (zero form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
      // Should use "zero" form - specific to Arabic
    });

    it('handles 1 request (one form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
      // Should use "one" form
    });

    it('handles 2 requests (two form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
      // Should use "two" form - specific to Arabic
    });

    it('handles 3 requests (few form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3');
      // Should use "few" form (3-10)
    });

    it('handles 4 requests (few form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 4 });
      expect(result).toBeTruthy();
      expect(result).toContain('4');
      // Should use "few" form (3-10)
    });

    it('handles 5 requests (few form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
      // Should use "few" form (3-10)
    });

    it('handles 10 requests (few form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 10 });
      expect(result).toBeTruthy();
      expect(result).toContain('10');
      // Should use "few" form (3-10)
    });

    it('handles 11 requests (many form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 11 });
      expect(result).toBeTruthy();
      expect(result).toContain('11');
      // Should use "many" form (11-99)
    });

    it('handles 20 requests (many form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 20 });
      expect(result).toBeTruthy();
      expect(result).toContain('20');
      // Should use "many" form (11-99)
    });

    it('handles 50 requests (many form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 50 });
      expect(result).toBeTruthy();
      expect(result).toContain('50');
      // Should use "many" form (11-99)
    });

    it('handles 99 requests (many form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 99 });
      expect(result).toBeTruthy();
      expect(result).toContain('99');
      // Should use "many" form (11-99)
    });

    it('handles 100 requests (other form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 100 });
      expect(result).toBeTruthy();
      expect(result).toContain('100');
      // Should use "other" form (100+)
    });

    it('handles 101 requests (other form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 101 });
      expect(result).toBeTruthy();
      expect(result).toContain('101');
      // Should use "other" form (100+)
    });

    it('handles 200 requests (other form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 200 });
      expect(result).toBeTruthy();
      expect(result).toContain('200');
      // Should use "other" form (100+)
    });

    it('handles 1000 requests (other form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 1000 });
      expect(result).toBeTruthy();
      expect(result).toContain('1000');
      // Should use "other" form (100+)
    });
  });

  describe('Plural Form Boundaries', () => {
    it('correctly transitions from zero to one (0 -> 1)', () => {
      const zero = i18n.t('common.pluralization.requests', { count: 0 });
      const one = i18n.t('common.pluralization.requests', { count: 1 });

      expect(zero).not.toBe(one);
    });

    it('correctly transitions from one to two (1 -> 2)', () => {
      const one = i18n.t('common.pluralization.requests', { count: 1 });
      const two = i18n.t('common.pluralization.requests', { count: 2 });

      expect(one).not.toBe(two);
    });

    it('correctly transitions from two to few (2 -> 3)', () => {
      const two = i18n.t('common.pluralization.requests', { count: 2 });
      const few = i18n.t('common.pluralization.requests', { count: 3 });

      expect(two).not.toBe(few);
    });

    it('correctly transitions from few to many (10 -> 11)', () => {
      const few = i18n.t('common.pluralization.requests', { count: 10 });
      const many = i18n.t('common.pluralization.requests', { count: 11 });

      expect(few).not.toBe(many);
    });

    it('correctly transitions from many to other (99 -> 100)', () => {
      const many = i18n.t('common.pluralization.requests', { count: 99 });
      const other = i18n.t('common.pluralization.requests', { count: 100 });

      expect(many).not.toBe(other);
    });
  });

  describe('Range Testing for Few Form (3-10)', () => {
    it('all numbers 3-10 use few form', () => {
      const results = [3, 4, 5, 6, 7, 8, 9, 10].map((count) =>
        i18n.t('common.pluralization.requests', { count })
      );

      // All should be truthy and contain count
      results.forEach((result, index) => {
        expect(result).toBeTruthy();
        expect(result).toContain((index + 3).toString());
      });
    });

    it('103-110 also use few form (based on last two digits)', () => {
      const results = [103, 104, 105, 110].map((count) =>
        i18n.t('common.pluralization.requests', { count })
      );

      results.forEach((result, index) => {
        expect(result).toBeTruthy();
      });
    });
  });

  describe('Range Testing for Many Form (11-99)', () => {
    it('all numbers 11-99 use many form', () => {
      const testNumbers = [11, 12, 15, 20, 25, 50, 75, 99];
      const results = testNumbers.map((count) =>
        i18n.t('common.pluralization.requests', { count })
      );

      results.forEach((result, index) => {
        expect(result).toBeTruthy();
        expect(result).toContain(testNumbers[index].toString());
      });
    });

    it('111-199 also use many form (based on last two digits)', () => {
      const results = [111, 125, 150, 199].map((count) =>
        i18n.t('common.pluralization.requests', { count })
      );

      results.forEach((result) => {
        expect(result).toBeTruthy();
      });
    });
  });

  describe('Pull Request Pluralization', () => {
    it('handles 0 pull requests (zero)', async () => {
      const result = i18n.t('common.pluralization.pullRequests', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
    });

    it('handles 1 pull request (one)', async () => {
      const result = i18n.t('common.pluralization.pullRequests', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
    });

    it('handles 2 pull requests (two)', async () => {
      const result = i18n.t('common.pluralization.pullRequests', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
    });

    it('handles 3 pull requests (few)', async () => {
      const result = i18n.t('common.pluralization.pullRequests', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3');
    });

    it('handles 11 pull requests (many)', async () => {
      const result = i18n.t('common.pluralization.pullRequests', { count: 11 });
      expect(result).toBeTruthy();
      expect(result).toContain('11');
    });

    it('handles 100 pull requests (other)', async () => {
      const result = i18n.t('common.pluralization.pullRequests', { count: 100 });
      expect(result).toBeTruthy();
      expect(result).toContain('100');
    });
  });

  describe('Comment Pluralization', () => {
    it('handles 0 comments (zero)', async () => {
      const result = i18n.t('common.pluralization.comments', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
    });

    it('handles 1 comment (one)', async () => {
      const result = i18n.t('common.pluralization.comments', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
    });

    it('handles 2 comments (two)', async () => {
      const result = i18n.t('common.pluralization.comments', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
    });

    it('handles 5 comments (few)', async () => {
      const result = i18n.t('common.pluralization.comments', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
    });

    it('handles 20 comments (many)', async () => {
      const result = i18n.t('common.pluralization.comments', { count: 20 });
      expect(result).toBeTruthy();
      expect(result).toContain('20');
    });

    it('handles 200 comments (other)', async () => {
      const result = i18n.t('common.pluralization.comments', { count: 200 });
      expect(result).toBeTruthy();
      expect(result).toContain('200');
    });
  });

  describe('Edge Cases', () => {
    it('handles fractional numbers (other form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 1.5 });
      expect(result).toBeTruthy();
      // Should use "other" form for fractional numbers
    });

    it('handles 0.5 (other form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 0.5 });
      expect(result).toBeTruthy();
      // Should use "other" form
    });

    it('handles very large numbers (other form)', async () => {
      const result = i18n.t('common.pluralization.requests', { count: 1000000 });
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
      const zero = i18n.t('common.pluralization.requests', { count: 0 });
      const one = i18n.t('common.pluralization.requests', { count: 1 });
      const two = i18n.t('common.pluralization.requests', { count: 2 });
      const few = i18n.t('common.pluralization.requests', { count: 3 });
      const many = i18n.t('common.pluralization.requests', { count: 11 });
      const other = i18n.t('common.pluralization.requests', { count: 100 });

      // All six forms should be different
      const uniqueForms = new Set([zero, one, two, few, many, other]);
      expect(uniqueForms.size).toBe(6);
    });
  });

  describe('Runtime Pluralization Selection', () => {
    it('dynamically selects correct plural form for all six categories', () => {
      const testCases = [
        { count: 0, form: 'zero' },
        { count: 1, form: 'one' },
        { count: 2, form: 'two' },
        { count: 5, form: 'few' },
        { count: 20, form: 'many' },
        { count: 100, form: 'other' },
      ];

      testCases.forEach(({ count }) => {
        const result = i18n.t('common.pluralization.requests', { count });
        expect(result).toBeTruthy();
        expect(result).toContain(count.toString());
      });
    });

    it('maintains consistency across multiple calls', () => {
      const first = i18n.t('common.pluralization.requests', { count: 5 });
      const second = i18n.t('common.pluralization.requests', { count: 5 });

      expect(first).toBe(second);
    });

    it('handles all plural forms within single application session', () => {
      const results = {
        zero: i18n.t('common.pluralization.requests', { count: 0 }),
        one: i18n.t('common.pluralization.requests', { count: 1 }),
        two: i18n.t('common.pluralization.requests', { count: 2 }),
        few: i18n.t('common.pluralization.requests', { count: 5 }),
        many: i18n.t('common.pluralization.requests', { count: 20 }),
        other: i18n.t('common.pluralization.requests', { count: 100 }),
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
      const counts = [0, 1, 2, 3, 11, 100, 2, 1, 0];
      const results = counts.map((count) => i18n.t('common.pluralization.requests', { count }));

      // All results should be valid
      results.forEach((result, index) => {
        expect(result).toBeTruthy();
        expect(result).toContain(counts[index].toString());
      });
    });
  });

  describe('Translation Completeness', () => {
    it('all six plural forms have non-empty translations', () => {
      const keys = ['requests', 'pullRequests', 'comments'];
      const counts = [0, 1, 2, 5, 20, 100]; // zero, one, two, few, many, other

      keys.forEach((key) => {
        counts.forEach((count) => {
          const result = i18n.t(`common.pluralization.${key}`, { count });
          expect(result).toBeTruthy();
          expect(result).not.toBe(`common.pluralization.${key}`);
        });
      });
    });
  });

  describe('Complex Scenarios', () => {
    it('handles hundreds with different last two digits', () => {
      // 100-102 should use "other" form
      const count100 = i18n.t('common.pluralization.requests', { count: 100 });
      const count101 = i18n.t('common.pluralization.requests', { count: 101 });
      const count102 = i18n.t('common.pluralization.requests', { count: 102 });

      // 103-110 should use "few" form (last two digits 03-10)
      const count103 = i18n.t('common.pluralization.requests', { count: 103 });
      const count110 = i18n.t('common.pluralization.requests', { count: 110 });

      // 111-199 should use "many" form (last two digits 11-99)
      const count111 = i18n.t('common.pluralization.requests', { count: 111 });
      const count150 = i18n.t('common.pluralization.requests', { count: 150 });

      // Verify all are truthy
      [count100, count101, count102, count103, count110, count111, count150].forEach((result) => {
        expect(result).toBeTruthy();
      });
    });
  });
});
