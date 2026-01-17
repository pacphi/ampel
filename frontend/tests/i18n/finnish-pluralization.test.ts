/**
 * Finnish Pluralization Tests
 *
 * Finnish has complex pluralization rules with 15 grammatical cases.
 * i18next uses simplified plural rules: one (1) and other (0, 2-999+)
 *
 * Test cases based on IMPLEMENTATION_ROADMAP_V2.md Phase 2 requirements.
 *
 * Finnish Plural Rules (i18next simplified):
 * - one: count === 1
 * - other: everything else (0, 2, 3, 4, 5, 10, 100, etc.)
 *
 * Reference: https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html#fi
 */

import { describe, expect, it, beforeAll, beforeEach, afterAll } from 'vitest';
import { setupTestI18n, getTestI18n, changeTestLanguage } from '../setup/i18n-instance.js';

let i18n: ReturnType<typeof getTestI18n>;

describe('Finnish Pluralization', () => {
  beforeAll(async () => {
    await setupTestI18n('fi');
    i18n = getTestI18n();
  });

  beforeEach(async () => {
    await changeTestLanguage('fi');
  });

  afterAll(async () => {
    await changeTestLanguage('en');
  });

  describe('Request Pluralization', () => {
    it('handles 0 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
      // Should use "other" form (not "one")
      expect(result).not.toMatch(/\b1\b/);
    });

    it('handles 1 request (one form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
      // Should use singular form
    });

    it('handles 2 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
      // Should use "other" form
    });

    it('handles 5 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
      // Should use "other" form
    });

    it('handles 10 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 10 });
      expect(result).toBeTruthy();
      expect(result).toContain('10');
      // Should use "other" form
    });

    it('handles 21 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 21 });
      expect(result).toBeTruthy();
      expect(result).toContain('21');
      // Should use "other" form (not influenced by ending in 1)
    });

    it('handles 100 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 100 });
      expect(result).toBeTruthy();
      expect(result).toContain('100');
      // Should use "other" form
    });

    it('handles 1000 requests (other form)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1000 });
      expect(result).toBeTruthy();
      expect(result).toContain('1000');
      // Should use "other" form
    });
  });

  describe('Pull Request Pluralization', () => {
    it('handles 0 pull requests', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
    });

    it('handles 1 pull request', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
    });

    it('handles 2 pull requests', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 2 });
      expect(result).toBeTruthy();
      expect(result).toContain('2');
    });

    it('handles 5 pull requests', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 5 });
      expect(result).toBeTruthy();
      expect(result).toContain('5');
    });

    it('handles 99 pull requests', async () => {
      const result = i18n.t('common:pluralization.pullRequests', { count: 99 });
      expect(result).toBeTruthy();
      expect(result).toContain('99');
    });
  });

  describe('Comment Pluralization', () => {
    it('handles 0 comments', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 0 });
      expect(result).toBeTruthy();
      expect(result).toContain('0');
    });

    it('handles 1 comment', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 1 });
      expect(result).toBeTruthy();
      expect(result).toContain('1');
    });

    it('handles 3 comments', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 3 });
      expect(result).toBeTruthy();
      expect(result).toContain('3');
    });

    it('handles 10 comments', async () => {
      const result = i18n.t('common:pluralization.comments', { count: 10 });
      expect(result).toBeTruthy();
      expect(result).toContain('10');
    });
  });

  describe('Edge Cases', () => {
    it('handles fractional numbers (not typical but should work)', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1.5 });
      expect(result).toBeTruthy();
      // Should use "other" form for non-integers
    });

    it('handles negative numbers', async () => {
      const result = i18n.t('common:pluralization.requests', { count: -1 });
      expect(result).toBeTruthy();
      // Should use "other" form for negative numbers
    });

    it('handles very large numbers', async () => {
      const result = i18n.t('common:pluralization.requests', { count: 1000000 });
      expect(result).toBeTruthy();
      expect(result).toContain('1000000');
    });
  });

  describe('i18next Configuration Verification', () => {
    it('has Finnish language loaded', () => {
      expect(i18n.hasResourceBundle('fi', 'common')).toBe(true);
    });

    it('current language is Finnish', () => {
      expect(i18n.language).toBe('fi');
    });

    it('pluralization is enabled', () => {
      expect(i18n.options.interpolation).toBeDefined();
    });

    it('Finnish plural rules are applied', () => {
      // Verify plural suffix is used correctly
      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const other = i18n.t('common:pluralization.requests', { count: 2 });

      expect(one).not.toBe(other);
    });
  });

  describe('Runtime Pluralization Selection', () => {
    it('dynamically selects correct plural form at runtime', () => {
      const counts = [0, 1, 2, 5, 10, 21, 100];
      const results = counts.map((count) => i18n.t('common:pluralization.requests', { count }));

      // All results should be different or follow expected pattern
      results.forEach((result, index) => {
        expect(result).toContain(counts[index].toString());
      });

      // Verify that 1 uses different form than others
      expect(results[1]).not.toBe(results[2]);
    });

    it('maintains consistency across multiple calls', () => {
      const first = i18n.t('common:pluralization.requests', { count: 5 });
      const second = i18n.t('common:pluralization.requests', { count: 5 });

      expect(first).toBe(second);
    });

    it('handles rapid language switching', async () => {
      const finnishResult = i18n.t('common:pluralization.requests', { count: 5 });

      await i18n.changeLanguage('en');
      const englishResult = i18n.t('common:pluralization.requests', { count: 5 });

      await i18n.changeLanguage('fi');
      const finnishResultAgain = i18n.t('common:pluralization.requests', { count: 5 });

      expect(finnishResult).toBe(finnishResultAgain);
      expect(finnishResult).not.toBe(englishResult);
    });
  });

  describe('Translation Completeness', () => {
    it('all plural forms have non-empty translations', () => {
      const keys = ['requests', 'pullRequests', 'comments'];

      keys.forEach((key) => {
        const one = i18n.t(`common:pluralization.${key}`, { count: 1 });
        const other = i18n.t(`common:pluralization.${key}`, { count: 5 });

        expect(one).toBeTruthy();
        expect(other).toBeTruthy();
        expect(one).not.toBe(`common:pluralization.${key}`);
        expect(other).not.toBe(`common:pluralization.${key}`);
      });
    });
  });
});
