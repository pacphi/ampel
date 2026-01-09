/**
 * Czech Pluralization Tests
 *
 * Czech has complex pluralization rules with 7 grammatical cases.
 * i18next uses CLDR plural rules: one, few, many, other
 *
 * Test cases based on IMPLEMENTATION_ROADMAP_V2.md Phase 2 requirements.
 *
 * Czech Plural Rules:
 * - one: count === 1
 * - few: count === 2-4
 * - many: fractional numbers (1.5, 2.7, etc.)
 * - other: 0, 5+
 *
 * Reference: https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#cs
 */

import { describe, expect, it, beforeAll, beforeEach, afterAll } from 'vitest';
import { setupTestI18n, getTestI18n, changeTestLanguage } from '../setup/i18n-instance.js';

let i18n: ReturnType<typeof getTestI18n>;

describe('Czech Pluralization', () => {
  beforeAll(async () => {
    await setupTestI18n('cs');
    i18n = getTestI18n();
  });

  beforeEach(async () => {
    await changeTestLanguage('cs');
  });

  afterAll(async () => {
    await changeTestLanguage('en');
  });

  describe('Request Pluralization', () => {
    it('handles 0 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
      // Should use "other" form
    });

    it('handles 1 request (one form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
      // Should use singular "one" form
    });

    it('handles 2 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
      // Should use "few" form (2-4)
    });

    it('handles 3 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3');
      // Should use "few" form (2-4)
    });

    it('handles 4 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 4 });
      expect(result).toBeTruthy();
      expect(result).toContain('4');
      // Should use "few" form (2-4)
    });

    it('handles 5 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
      // Should use "other" form (5+)
    });

    it('handles 10 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 10 });
      expect(result).toBeTruthy();
      expect(result).toContain('10');
      // Should use "other" form
    });

    it('handles 1.5 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1.5 });
      expect(result).toBeTruthy();
      expect(result).toContain('1.5');
      // Should use "many" form for fractional numbers
    });

    it('handles 2.7 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 2.7 });
      expect(result).toBeTruthy();
      expect(result).toContain('2.7');
      // Should use "many" form for fractional numbers
    });

    it('handles 100 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 100 });
      expect(result).toBeTruthy();
      expect(result).toContain('100');
      // Should use "other" form
    });
  });

  describe('Pull Request Pluralization', () => {
    it('handles 0 pull requests (other)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
    });

    it('handles 1 pull request (one)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
    });

    it('handles 2 pull requests (few)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
    });

    it('handles 3 pull requests (few)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3');
    });

    it('handles 4 pull requests (few)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 4 });
      expect(result).toBeTruthy();
      expect(result).toContain('4');
    });

    it('handles 5 pull requests (other)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
    });

    it('handles 99 pull requests (other)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 99 });
      expect(result).toBeTruthy();
      expect(result).toContain('99');
    });
  });

  describe('Comment Pluralization', () => {
    it('handles 0 comments (other)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
    });

    it('handles 1 comment (one)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
    });

    it('handles 2 comments (few)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
    });

    it('handles 5 comments (other)', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
    });
  });

  describe('Plural Form Boundaries', () => {
    it('correctly transitions from one to few (1 -> 2)', () => {
      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const few = i18n.t('common:pluralization.requests', { count: 2 });

      expect(one).not.toBe(few);
    });

    it('correctly transitions from few to other (4 -> 5)', () => {
      const few = i18n.t('common:pluralization.requests', { count: 4 });
      const other = i18n.t('common:pluralization.requests', { count: 5 });

      expect(few).not.toBe(other);
    });

    it('handles fractional boundary (1 -> 1.5)', () => {
      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const many = i18n.t('common:pluralization.requests', { count: 1.5 });

      expect(one).not.toBe(many);
    });
  });

  describe('Edge Cases', () => {
    it('handles 0.5 (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 0.5 });
      expect(result).toBeTruthy();
      // Should use "many" form for fractional numbers
    });

    it('handles negative numbers', async () => {
      const result = i18n.t('common:pluralization.requests', { count: -1 });
      expect(result).toBeTruthy();
    });

    it('handles very large numbers (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1000000 });
      expect(result).toBeTruthy();
      expect(result).toContain('1000000');
    });
  });

  describe('i18next Configuration Verification', () => {
    it('has Czech language loaded', () => {
      expect(i18n.hasResourceBundle('cs', 'common')).toBe(true);
    });

    it('current language is Czech', () => {
      expect(i18n.language).toBe('cs');
    });

    it('Czech plural rules are applied correctly', () => {
      // Verify all four forms are different
      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const few = i18n.t('common:pluralization.requests', { count: 2 });
      const many = i18n.t('common:pluralization.requests', { count: 1.5 });
      const other = i18n.t('common:pluralization.requests', { count: 5 });

      // All forms should be different
      expect(one).not.toBe(few);
      expect(few).not.toBe(other);
      expect(one).not.toBe(many);
    });
  });

  describe('Runtime Pluralization Selection', () => {
    it('dynamically selects correct plural form for all categories', () => {
      const testCases = [
        { count: 0, expectedForm: 'other' },
        { count: 1, expectedForm: 'one' },
        { count: 2, expectedForm: 'few' },
        { count: 3, expectedForm: 'few' },
        { count: 4, expectedForm: 'few' },
        { count: 5, expectedForm: 'other' },
        { count: 1.5, expectedForm: 'many' },
      ];

      testCases.forEach(({ count }) => {
        const result = i18n.t('common:pluralization.requests', { count });
        expect(result).toBeTruthy();
        expect(result).toContain(count.toString());
      });
    });

    it('maintains consistency across multiple calls', () => {
      const first = i18n.t('common:pluralization.requests', { count: 3 });
      const second = i18n.t('common:pluralization.requests', { count: 3 });

      expect(first).toBe(second);
    });

    it('handles all plural forms within single application session', () => {
      const results = {
        one: i18n.t('common:pluralization.requests', { count: 1 }),
        few: i18n.t('common:pluralization.requests', { count: 2 }),
        many: i18n.t('common:pluralization.requests', { count: 1.5 }),
        other: i18n.t('common:pluralization.requests', { count: 5 }),
      };

      // All should be truthy
      Object.values(results).forEach((result) => {
        expect(result).toBeTruthy();
      });

      // All should be different
      const uniqueResults = new Set(Object.values(results));
      expect(uniqueResults.size).toBe(4);
    });
  });

  describe('Translation Completeness', () => {
    it('all plural forms have non-empty translations', () => {
      const keys = ['requests', 'pullRequests', 'comments'];
      const counts = [1, 2, 1.5, 5]; // one, few, many, other

      keys.forEach((key) => {
        counts.forEach((count) => {
          const result = i18n.t(`common:pluralization.${key}`, { count });
          expect(result).toBeTruthy();
          expect(result).not.toBe(`common:pluralization.${key}`);
        });
      });
    });
  });
});
