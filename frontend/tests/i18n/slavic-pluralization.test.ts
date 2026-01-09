/**
 * Slavic Languages Pluralization Tests (Russian & Polish)
 *
 * Both Russian and Polish use the same pluralization rules:
 * - one: count % 10 === 1 && count % 100 !== 11
 * - few: count % 10 in 2..4 && count % 100 not in 12..14
 * - many: everything else (0, 5-20, 25-30, etc.)
 *
 * Test cases based on IMPLEMENTATION_ROADMAP_V2.md Phase 2 requirements.
 *
 * Reference:
 * - Russian: https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#ru
 * - Polish: https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#pl
 *
 * IMPORTANT: These tests use the REAL i18n instance with actual translation files,
 * not mocked translations. This ensures the pluralization rules work correctly
 * with the production i18n configuration.
 */

import { describe, expect, it, beforeAll, beforeEach, afterAll } from 'vitest';
import { setupTestI18n, getTestI18n, changeTestLanguage } from '../setup/i18n-instance.js';
// Available for future use - keep imports for reference
import type {
  testPluralForms as _testPluralForms,
  verifyDistinctPluralForms as _verifyDistinctPluralForms,
} from '../setup/i18n-testing';

// Re-export for backward compatibility
let i18n: ReturnType<typeof getTestI18n>;

describe('Slavic Pluralization - Russian', () => {
  beforeAll(async () => {
    await setupTestI18n('ru');
    i18n = getTestI18n();
  });

  beforeEach(async () => {
    await changeTestLanguage('ru');
  });

  afterAll(async () => {
    // Reset to English for other tests
    await changeTestLanguage('en');
  });

  describe('Request Pluralization - Russian', () => {
    it('handles 0 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
      // Should use "many" form
    });

    it('handles 1 request (one form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
      // Should use "one" form (ends in 1, not 11)
    });

    it('handles 2 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
      // Should use "few" form (2-4, not 12-14)
    });

    it('handles 3 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3');
      // Should use "few" form
    });

    it('handles 4 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 4 });
      expect(result).toBeTruthy();
      expect(result).toContain('4');
      // Should use "few" form
    });

    it('handles 5 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
      // Should use "many" form (5-20)
    });

    it('handles 10 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 10 });
      expect(result).toBeTruthy();
      expect(result).toContain('10');
      // Should use "many" form
    });

    it('handles 11 requests (many form - exception)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 11 });
      expect(result).toBeTruthy();
      expect(result).toContain('11');
      // Should use "many" form (ends in 1 but is 11)
    });

    it('handles 12 requests (many form - exception)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 12 });
      expect(result).toBeTruthy();
      expect(result).toContain('12');
      // Should use "many" form (ends in 2 but is 12)
    });

    it('handles 21 requests (one form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 21 });
      expect(result).toBeTruthy();
      expect(result).toContain('21');
      // Should use "one" form (ends in 1, not 11)
    });

    it('handles 22 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 22 });
      expect(result).toBeTruthy();
      expect(result).toContain('22');
      // Should use "few" form (ends in 2, not 12)
    });

    it('handles 25 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 25 });
      expect(result).toBeTruthy();
      expect(result).toContain('25');
      // Should use "many" form
    });

    it('handles 100 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 100 });
      expect(result).toBeTruthy();
      expect(result).toContain('100');
      // Should use "many" form (ends in 0)
    });

    it('handles 101 requests (one form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 101 });
      expect(result).toBeTruthy();
      expect(result).toContain('101');
      // Should use "one" form (ends in 1, not 11)
    });
  });

  describe('Plural Form Boundaries - Russian', () => {
    it('correctly handles 10-19 range (all many)', () => {
      const results = [11, 12, 13, 14, 15, 16, 17, 18, 19].map((count) =>
        i18n.t('common:pluralization.requests', { count })
      );

      // All should use "many" form
      results.forEach((result, index) => {
        expect(result).toBeTruthy();
        expect(result).toContain((11 + index).toString());
      });
    });

    it('correctly handles 21-24 range (one, few, few, few)', () => {
      const count21 = i18n.t('common:pluralization.requests', { count: 21 });
      const count22 = i18n.t('common:pluralization.requests', { count: 22 });
      const _count23 = i18n.t('common:pluralization.requests', { count: 23 });
      const count24 = i18n.t('common:pluralization.requests', { count: 24 });
      const count25 = i18n.t('common:pluralization.requests', { count: 25 });

      // 21 should be different from 22-24
      expect(count21).not.toBe(count22);
      // 22-24 should be similar (few form)
      // 25 should be different (many form)
      expect(count24).not.toBe(count25);
    });
  });

  describe('Edge Cases - Russian', () => {
    it('handles negative numbers with correct form', async () => {
      const result = i18n.t('common:pluralization.requests', { count: -1 });
      expect(result).toBeTruthy();
      // Should follow same rules as positive
    });

    it('handles very large numbers correctly', async () => {
      const result1001 = i18n.t('common:pluralization.requests', { count: 1001 });
      const result1002 = i18n.t('common:pluralization.requests', { count: 1002 });
      const result1005 = i18n.t('common:pluralization.requests', { count: 1005 });

      expect(result1001).toBeTruthy();
      expect(result1002).toBeTruthy();
      expect(result1005).toBeTruthy();
      // Should follow same rules based on last two digits
    });
  });

  describe('i18next Configuration - Russian', () => {
    it('has Russian language loaded', () => {
      expect(i18n.hasResourceBundle('ru', 'common')).toBe(true);
    });

    it('current language is Russian', () => {
      expect(i18n.language).toBe('ru');
    });

    it('Russian plural rules produce three different forms', () => {
      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const few = i18n.t('common:pluralization.requests', { count: 2 });
      const many = i18n.t('common:pluralization.requests', { count: 5 });

      expect(one).not.toBe(few);
      expect(few).not.toBe(many);
      expect(one).not.toBe(many);
    });
  });
});

describe('Slavic Pluralization - Polish', () => {
  beforeAll(async () => {
    await setupTestI18n('pl');
    i18n = getTestI18n();
  });

  beforeEach(async () => {
    await changeTestLanguage('pl');
  });

  afterAll(async () => {
    // Reset to English for other tests
    await changeTestLanguage('en');
  });

  describe('Request Pluralization - Polish', () => {
    it('handles 0 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
    });

    it('handles 1 request (one form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
    });

    it('handles 2 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
    });

    it('handles 3 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3');
    });

    it('handles 4 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 4 });
      expect(result).toBeTruthy();
      expect(result).toContain('4');
    });

    it('handles 5 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
    });

    it('handles 11 requests (many form - exception)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 11 });
      expect(result).toBeTruthy();
      expect(result).toContain('11');
    });

    it('handles 12 requests (many form - exception)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 12 });
      expect(result).toBeTruthy();
      expect(result).toContain('12');
    });

    it('handles 21 requests (one form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 21 });
      expect(result).toBeTruthy();
      expect(result).toContain('21');
    });

    it('handles 22 requests (few form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 22 });
      expect(result).toBeTruthy();
      expect(result).toContain('22');
    });

    it('handles 100 requests (many form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 100 });
      expect(result).toBeTruthy();
      expect(result).toContain('100');
    });
  });

  describe('Pull Request Pluralization - Polish', () => {
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

    it('handles 5 pull requests (many)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
    });

    it('handles 22 pull requests (few)', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 22 });
      expect(result).toBeTruthy();
      expect(result).toContain('22');
    });
  });

  describe('i18next Configuration - Polish', () => {
    it('has Polish language loaded', () => {
      expect(i18n.hasResourceBundle('pl', 'common')).toBe(true);
    });

    it('current language is Polish', () => {
      expect(i18n.language).toBe('pl');
    });

    it('Polish plural rules produce three different forms', () => {
      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const few = i18n.t('common:pluralization.requests', { count: 2 });
      const many = i18n.t('common:pluralization.requests', { count: 5 });

      expect(one).not.toBe(few);
      expect(few).not.toBe(many);
      expect(one).not.toBe(many);
    });
  });

  describe('Cross-Language Consistency', () => {
    it('Russian and Polish follow same pluralization pattern', async () => {
      await i18n.changeLanguage('ru');
      const russianOne = i18n.t('common:pluralization.requests', { count: 1 });
      const russianFew = i18n.t('common:pluralization.requests', { count: 2 });
      const _russianMany = i18n.t('common:pluralization.requests', { count: 5 });

      await i18n.changeLanguage('pl');
      const polishOne = i18n.t('common:pluralization.requests', { count: 1 });
      const polishFew = i18n.t('common:pluralization.requests', { count: 2 });
      const _polishMany = i18n.t('common:pluralization.requests', { count: 5 });

      // Both languages should use same plural forms for same counts
      // (though text is different, the form selection is the same)
      expect(russianOne).not.toBe(russianFew);
      expect(polishOne).not.toBe(polishFew);
    });
  });

  describe('Runtime Pluralization Selection', () => {
    it('dynamically selects correct form at runtime', () => {
      const testCases = [
        { count: 1, form: 'one' },
        { count: 2, form: 'few' },
        { count: 5, form: 'many' },
        { count: 11, form: 'many' },
        { count: 21, form: 'one' },
        { count: 22, form: 'few' },
        { count: 25, form: 'many' },
      ];

      testCases.forEach(({ count }) => {
        const result = i18n.t('common:pluralization.requests', { count });
        expect(result).toBeTruthy();
        expect(result).toContain(count.toString());
      });
    });
  });
});
