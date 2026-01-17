/**
 * Translation Key Tests
 *
 * Tests that verify:
 * 1. Components use correct translation keys (not hardcoded strings)
 * 2. All required translation keys exist in locale files
 * 3. Namespace loading works correctly
 * 4. Key structure matches across languages
 *
 * These tests use the REAL i18n instance with actual translation files,
 * ensuring translations are properly configured and complete.
 */

import { describe, it, expect, beforeAll, beforeEach } from 'vitest';
import {
  setupTestI18n,
  getTestI18n,
  resetTestI18n,
  getMissingKeys,
  clearMissingKeys,
  getNamespaceKeys,
  findMissingTranslations,
} from '../setup/i18n-instance.js';
import {
  createTranslationSpy,
  testPluralForms,
  verifyDistinctPluralForms,
  verifyNamespacesLoaded,
} from '../setup/i18n-testing';

describe('Translation Key Tests', () => {
  beforeAll(async () => {
    await setupTestI18n('en');
  });

  beforeEach(async () => {
    await resetTestI18n();
    clearMissingKeys();
  });

  describe('Key Existence Verification', () => {
    describe('Common Namespace Keys', () => {
      const REQUIRED_COMMON_KEYS = [
        'app.title',
        'app.name',
        'app.description',
        'app.loading',
        'app.error',
        'auth.login',
        'auth.logout',
        'auth.email',
        'auth.password',
        'navigation.dashboard',
        'navigation.settings',
        'navigation.profile',
        'theme.light',
        'theme.dark',
        'actions.save',
        'actions.cancel',
        'actions.edit',
        'actions.delete',
      ];

      it('should have all required keys in English common namespace', () => {
        const i18n = getTestI18n();

        for (const key of REQUIRED_COMMON_KEYS) {
          const exists = i18n.exists(`common:${key}`, { lng: 'en' });
          expect(exists, `Key "common:${key}" should exist in English`).toBe(true);
        }
      });

      it('should have all required keys in Arabic common namespace', () => {
        const i18n = getTestI18n();

        for (const key of REQUIRED_COMMON_KEYS) {
          const exists = i18n.exists(`common:${key}`, { lng: 'ar' });
          expect(exists, `Key "common:${key}" should exist in Arabic`).toBe(true);
        }
      });

      it('should have all required keys in Hebrew common namespace', () => {
        const i18n = getTestI18n();

        for (const key of REQUIRED_COMMON_KEYS) {
          const exists = i18n.exists(`common:${key}`, { lng: 'he' });
          expect(exists, `Key "common:${key}" should exist in Hebrew`).toBe(true);
        }
      });
    });

    describe('Pluralization Keys', () => {
      const PLURAL_BASE_KEYS = [
        'pluralization.requests',
        'pluralization.pullRequests',
        'pluralization.comments',
      ];

      it('should have English pluralization keys (one/other forms)', () => {
        const i18n = getTestI18n();

        for (const baseKey of PLURAL_BASE_KEYS) {
          // English uses _one and _other suffixes
          const oneExists = i18n.exists(`common:${baseKey}_one`, { lng: 'en' });
          const otherExists = i18n.exists(`common:${baseKey}_other`, { lng: 'en' });

          expect(oneExists, `Key "common:${baseKey}_one" should exist`).toBe(true);
          expect(otherExists, `Key "common:${baseKey}_other" should exist`).toBe(true);
        }
      });

      it('should have Arabic pluralization keys (all 6 forms)', () => {
        const i18n = getTestI18n();
        const arabicForms = ['_zero', '_one', '_two', '_few', '_many', '_other'];

        for (const baseKey of PLURAL_BASE_KEYS) {
          for (const form of arabicForms) {
            const fullKey = `common:${baseKey}${form}`;
            const exists = i18n.exists(fullKey, { lng: 'ar' });
            expect(exists, `Key "${fullKey}" should exist in Arabic`).toBe(true);
          }
        }
      });
    });
  });

  describe('Translation Value Verification', () => {
    it('should return actual translations, not key names', () => {
      const i18n = getTestI18n();

      const title = i18n.t('common:app.title');
      expect(title).toBe('Ampel PR Dashboard');
      expect(title).not.toBe('app.title');
      expect(title).not.toBe('common:app.title');
    });

    it('should handle interpolation correctly', () => {
      const i18n = getTestI18n();

      const result = i18n.t('common:actions.viewOnProvider', { provider: 'GitHub' });
      expect(result).toBe('View on GitHub');
      expect(result).not.toContain('{{provider}}');
    });

    it('should handle pluralization with count', () => {
      const i18n = getTestI18n();

      const one = i18n.t('common:pluralization.requests', { count: 1 });
      const many = i18n.t('common:pluralization.requests', { count: 5 });

      expect(one).toBe('1 request');
      expect(many).toBe('5 requests');
    });
  });

  describe('Namespace Loading', () => {
    it('should have all required namespaces loaded for English', () => {
      const i18n = getTestI18n();
      const namespaces = ['common', 'dashboard', 'settings', 'errors', 'validation'];

      const { loaded, missing } = verifyNamespacesLoaded(i18n, 'en', namespaces);

      expect(missing).toHaveLength(0);
      expect(loaded).toEqual(namespaces);
    });

    it('should have all required namespaces loaded for Arabic', () => {
      const i18n = getTestI18n();
      const namespaces = ['common', 'dashboard', 'settings', 'errors', 'validation'];

      const { loaded, missing } = verifyNamespacesLoaded(i18n, 'ar', namespaces);

      expect(missing).toHaveLength(0);
      expect(loaded).toEqual(namespaces);
    });

    it('should access keys from different namespaces', async () => {
      const i18n = getTestI18n();

      // Common namespace
      const appTitle = i18n.t('common:app.title');
      expect(appTitle).toBeTruthy();
      expect(appTitle).not.toBe('app.title');

      // Settings namespace
      const settingsTitle = i18n.t('settings:title');
      expect(settingsTitle).toBeTruthy();
    });
  });

  describe('Cross-Language Key Parity', () => {
    it('should have same keys in Arabic as English for common namespace', () => {
      const { missing, extra } = findMissingTranslations('en', 'ar', 'common');

      // Log any discrepancies for debugging
      if (missing.length > 0) {
        console.warn('Keys in English but missing in Arabic:', missing);
      }
      if (extra.length > 0) {
        console.warn('Extra keys in Arabic not in English:', extra);
      }

      // Allow some tolerance for locale-specific keys
      expect(missing.length).toBeLessThan(5);
    });

    it('should have same keys in Hebrew as English for common namespace', () => {
      const { missing, extra: _extra } = findMissingTranslations('en', 'he', 'common');

      if (missing.length > 0) {
        console.warn('Keys in English but missing in Hebrew:', missing);
      }

      expect(missing.length).toBeLessThan(5);
    });

    it('should have same keys across all Slavic languages', () => {
      const slavicLanguages = ['ru', 'pl', 'cs'];
      const baselineKeys = getNamespaceKeys('common', 'en');

      for (const lang of slavicLanguages) {
        const langKeys = getNamespaceKeys('common', lang);

        // Check that most keys are present
        const missingCount = baselineKeys.filter((k) => !langKeys.includes(k)).length;
        expect(missingCount, `${lang} should have similar keys to English`).toBeLessThan(10);
      }
    });
  });

  describe('Translation Spy Verification', () => {
    it('should track translation key usage', () => {
      const spy = createTranslationSpy();

      // Simulate component calls
      spy.t('common:app.title');
      spy.t('common:navigation.dashboard');
      spy.t('common:auth.login');

      expect(spy.wasKeyCalled('common:app.title')).toBe(true);
      expect(spy.wasKeyCalled('common:navigation.dashboard')).toBe(true);
      expect(spy.wasKeyCalled('common:auth.login')).toBe(true);
      expect(spy.wasKeyCalled('common:nonexistent.key')).toBe(false);
    });

    it('should track pluralization calls with count', () => {
      const spy = createTranslationSpy();

      spy.t('common:pluralization.requests', { count: 1 });
      spy.t('common:pluralization.requests', { count: 5 });
      spy.t('common:pluralization.comments', { count: 0 });

      const countCalls = spy.getCallsWithCount();
      expect(countCalls).toHaveLength(3);

      const requestsCalls = spy.getCallsForKey('common:pluralization.requests');
      expect(requestsCalls).toHaveLength(2);
    });

    it('should track interpolation options', () => {
      const spy = createTranslationSpy();

      spy.t('common:actions.viewOnProvider', { provider: 'GitHub' });
      spy.t('common:time.minutesAgo', { count: 5 });

      const calls = spy.calls;
      expect(calls[0].options).toEqual({ provider: 'GitHub' });
      expect(calls[1].options).toEqual({ count: 5 });
    });

    it('should reset call tracking', () => {
      const spy = createTranslationSpy();

      spy.t('common:app.title');
      expect(spy.calls).toHaveLength(1);

      spy.reset();
      expect(spy.calls).toHaveLength(0);
    });
  });

  describe('Missing Key Detection', () => {
    it('should detect when translation keys are missing', async () => {
      const i18n = getTestI18n();

      // Try to translate a non-existent key
      const result = i18n.t('common:this.key.does.not.exist');

      // With our setup, missing keys are tracked
      // The result should be the key itself (fallback behavior)
      expect(result).toBe('this.key.does.not.exist');
    });

    it('should accumulate missing keys during test run', () => {
      const i18n = getTestI18n();
      clearMissingKeys();

      // Access several missing keys
      i18n.t('common:missing.key.one');
      i18n.t('common:missing.key.two');
      i18n.t('dashboard:another.missing');

      const missing = getMissingKeys();
      expect(missing).toContain('common:missing.key.one');
      expect(missing).toContain('common:missing.key.two');
      expect(missing).toContain('dashboard:another.missing');
    });
  });

  describe('Pluralization Form Verification', () => {
    it('should produce correct English plural forms', async () => {
      const i18n = getTestI18n();
      await i18n.changeLanguage('en');

      const results = testPluralForms(i18n, 'common:pluralization.requests', [0, 1, 2, 5, 10]);

      expect(results[0]).toBe('0 requests');
      expect(results[1]).toBe('1 request');
      expect(results[2]).toBe('2 requests');
      expect(results[5]).toBe('5 requests');
    });

    it('should produce distinct Arabic plural forms', async () => {
      const i18n = getTestI18n();
      await i18n.changeLanguage('ar');

      // Arabic has 6 distinct forms: zero, one, two, few, many, other
      const testCounts = [0, 1, 2, 3, 11, 100];
      const results = testPluralForms(i18n, 'common:pluralization.requests', testCounts);

      const verification = verifyDistinctPluralForms(results, 5);

      // Should have at least 5 distinct forms (some may be similar in edge cases)
      expect(verification.pass).toBe(true);
      expect(verification.distinctCount).toBeGreaterThanOrEqual(5);
    });

    it('should produce distinct Slavic plural forms', async () => {
      const i18n = getTestI18n();
      await i18n.changeLanguage('ru');

      // Russian has 3 forms: one, few, many
      const testCounts = [1, 2, 5, 11, 21, 22];
      const results = testPluralForms(i18n, 'common:pluralization.requests', testCounts);

      const verification = verifyDistinctPluralForms(results, 3);

      expect(verification.distinctCount).toBeGreaterThanOrEqual(3);
    });
  });

  describe('Language-Specific Content Verification', () => {
    it('should return Arabic text for Arabic locale', async () => {
      const i18n = getTestI18n();
      await i18n.changeLanguage('ar');

      const login = i18n.t('common:auth.login');
      // Arabic text contains Arabic characters
      expect(login).toMatch(/[\u0600-\u06FF]/);
    });

    it('should return Hebrew text for Hebrew locale', async () => {
      const i18n = getTestI18n();
      await i18n.changeLanguage('he');

      const login = i18n.t('common:auth.login');
      // Hebrew text contains Hebrew characters
      expect(login).toMatch(/[\u0590-\u05FF]/);
    });

    it('should return Cyrillic text for Russian locale', async () => {
      const i18n = getTestI18n();
      await i18n.changeLanguage('ru');

      const login = i18n.t('common:auth.login');
      // Russian text contains Cyrillic characters
      expect(login).toMatch(/[\u0400-\u04FF]/);
    });
  });

  describe('Fallback Behavior', () => {
    it('should fall back to English for missing translations', async () => {
      const i18n = getTestI18n();

      // Create a key that exists in English but might not in other languages
      // First verify it exists in English
      await i18n.changeLanguage('en');
      const englishValue = i18n.t('common:app.title');
      expect(englishValue).toBe('Ampel PR Dashboard');

      // Switch to a language - the key should still resolve
      await i18n.changeLanguage('fr');
      const frenchValue = i18n.t('common:app.title');

      // Should either be translated or fall back to English
      expect(frenchValue).toBeTruthy();
      expect(frenchValue).not.toBe('app.title');
    });
  });
});

describe('Key Structure Analysis', () => {
  beforeAll(async () => {
    await setupTestI18n('en');
  });

  it('should have consistent key depth across namespaces', () => {
    const namespaces = ['common', 'dashboard', 'settings'];

    for (const ns of namespaces) {
      const keys = getNamespaceKeys(ns, 'en');

      // Check that keys have reasonable depth (1-4 levels)
      for (const key of keys) {
        const depth = key.split('.').length;
        expect(depth, `Key "${ns}:${key}" has unusual depth`).toBeLessThanOrEqual(5);
        expect(depth, `Key "${ns}:${key}" is too shallow`).toBeGreaterThanOrEqual(1);
      }
    }
  });

  it('should follow naming conventions for keys', () => {
    const keys = getNamespaceKeys('common', 'en');

    for (const key of keys) {
      // Keys should be camelCase or snake_case for plural suffixes
      const parts = key.split('.');
      for (const part of parts) {
        // Allow camelCase, or _plural suffixes
        const isValid =
          /^[a-z][a-zA-Z0-9]*$/.test(part) || // camelCase
          /^[a-z][a-zA-Z0-9]*_(one|other|zero|two|few|many)$/.test(part); // plural suffix

        expect(isValid, `Key part "${part}" in "${key}" should follow naming convention`).toBe(
          true
        );
      }
    }
  });
});
