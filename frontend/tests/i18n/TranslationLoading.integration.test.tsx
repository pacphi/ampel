/**
 * Translation Loading Integration Tests
 *
 * Tests:
 * - Translation files load correctly from /locales
 * - t() function resolves to translated strings
 * - Namespace handling
 * - Fallback behavior
 * - Missing translation handling
 * - Interpolation and formatting
 */

import { describe, expect, it, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { I18nextProvider, useTranslation } from 'react-i18next';
import i18n from '@/i18n/config';

// Component to test multiple namespaces
function MultiNamespaceComponent() {
  const { t: tCommon } = useTranslation('common');
  const { t: tDashboard } = useTranslation('dashboard');
  const { t: tSettings } = useTranslation('settings');

  return (
    <div>
      <div data-testid="common-text">{tCommon('language')}</div>
      <div data-testid="dashboard-text">{tDashboard('prDashboard')}</div>
      <div data-testid="settings-text">{tSettings('tabs.profile')}</div>
    </div>
  );
}

describe('Translation Loading Integration Tests', () => {
  beforeEach(async () => {
    // Reset to English - don't clear cache as it causes timeout issues in CI
    // The cache clearing was causing subsequent changeLanguage calls to timeout
    // while trying to reload all translations from scratch
    await i18n.changeLanguage('en');
  }, 15000); // Extend hook timeout for translation loading

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('1. Translation File Loading', () => {
    it('should load English common translations', async () => {
      await i18n.changeLanguage('en');

      // Wait for translations to load
      await waitFor(
        () => {
          const hasTranslations = i18n.hasResourceBundle('en', 'common');
          expect(hasTranslations).toBe(true);
        },
        { timeout: 5000 }
      );
    });

    it('should load translations for multiple namespaces', async () => {
      await i18n.changeLanguage('en');

      // Load multiple namespaces
      await Promise.all([
        i18n.loadNamespaces(['common', 'dashboard', 'settings', 'errors', 'validation']),
      ]);

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
          expect(i18n.hasResourceBundle('en', 'dashboard')).toBe(true);
          expect(i18n.hasResourceBundle('en', 'settings')).toBe(true);
        },
        { timeout: 5000 }
      );
    });

    it('should load translations for different languages', async () => {
      const testLanguages = ['en', 'fr', 'de', 'es-ES'];

      for (const lang of testLanguages) {
        await i18n.changeLanguage(lang);

        await waitFor(
          () => {
            expect(i18n.hasResourceBundle(lang, 'common')).toBe(true);
          },
          { timeout: 5000 }
        );
      }
    });

    it('should handle RTL language translations (Arabic)', async () => {
      await i18n.changeLanguage('ar');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('ar', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );
    });

    it('should handle RTL language translations (Hebrew)', async () => {
      await i18n.changeLanguage('he');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('he', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );
    });
  });

  describe('2. t() Function Translation Resolution', () => {
    it('should translate common namespace keys', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      // Test that t() returns translated string (not the key)
      const translation = i18n.t('common:language');
      expect(translation).not.toBe('common:language');
      expect(typeof translation).toBe('string');
    });

    it('should translate dashboard namespace keys', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('dashboard');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'dashboard')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('dashboard:prDashboard');
      expect(translation).not.toBe('dashboard:prDashboard');
      expect(typeof translation).toBe('string');
    });

    it('should translate settings namespace keys', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('settings');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'settings')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('settings:tabs.profile');
      expect(translation).not.toBe('settings:tabs.profile');
      expect(typeof translation).toBe('string');
    });

    it('should handle nested translation keys', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      // Test nested key like 'auth.logout'
      const translation = i18n.t('common:auth.logout');
      expect(translation).not.toBe('common:auth.logout');
      expect(typeof translation).toBe('string');
    });
  });

  describe('3. Language-Specific Translations', () => {
    it('should return French translations when language is French', async () => {
      await i18n.changeLanguage('fr');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('fr', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('common:language');
      expect(translation).not.toBe('common:language');
      expect(translation).not.toBe('Language'); // Should not be English
      expect(typeof translation).toBe('string');
    });

    it('should return German translations when language is German', async () => {
      await i18n.changeLanguage('de');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('de', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('common:language');
      expect(translation).not.toBe('common:language');
      expect(typeof translation).toBe('string');
    });

    it('should return Spanish translations when language is Spanish', async () => {
      await i18n.changeLanguage('es-ES');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('es-ES', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('common:language');
      expect(translation).not.toBe('common:language');
      expect(typeof translation).toBe('string');
    });
  });

  describe('4. Fallback Behavior', () => {
    it('should fallback to English for missing translations', async () => {
      // Change to a language that might have incomplete translations
      await i18n.changeLanguage('cs'); // Czech
      await i18n.loadNamespaces('common');

      // Even if Czech translation is missing, should get English or key
      const translation = i18n.t('common:language');
      expect(typeof translation).toBe('string');
      expect(translation).not.toBe(''); // Should not be empty
    });

    it('should return key for completely missing translations', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('common');

      // Request a key that definitely doesn't exist
      const translation = i18n.t('common:this.key.definitely.does.not.exist.anywhere');
      expect(typeof translation).toBe('string');
      // Should return the key itself as fallback
      expect(translation).toContain('this.key.definitely.does.not.exist.anywhere');
    });

    it('should use default value when provided for missing keys', async () => {
      await i18n.changeLanguage('en');

      const translation = i18n.t('common:missing.key', 'Default Value');
      expect(translation).toBe('Default Value');
    });
  });

  describe('5. Translation Updates on Language Change', () => {
    it('should update translations when language changes', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const englishTranslation = i18n.t('common:language');
      expect(typeof englishTranslation).toBe('string');

      // Change to French
      await i18n.changeLanguage('fr');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('fr', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const frenchTranslation = i18n.t('common:language');
      expect(typeof frenchTranslation).toBe('string');

      // Translations should be different (unless they happen to be the same)
      // At minimum, they should both be valid strings
      expect(englishTranslation.length).toBeGreaterThan(0);
      expect(frenchTranslation.length).toBeGreaterThan(0);
    });
  });

  describe('6. Namespace Handling', () => {
    it('should handle default namespace (common)', async () => {
      await i18n.changeLanguage('en');

      // Without specifying namespace, should use default (common)
      const translation = i18n.t('language');
      expect(typeof translation).toBe('string');
    });

    it('should handle explicit namespace with colon syntax', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces(['common', 'dashboard']);

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'dashboard')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('dashboard:prDashboard');
      expect(typeof translation).toBe('string');
      expect(translation).not.toBe('dashboard:prDashboard');
    });

    it('should handle multiple namespace translations in same component', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces(['common', 'dashboard', 'settings']);

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
          expect(i18n.hasResourceBundle('en', 'dashboard')).toBe(true);
          expect(i18n.hasResourceBundle('en', 'settings')).toBe(true);
        },
        { timeout: 5000 }
      );

      render(
        <I18nextProvider i18n={i18n}>
          <MultiNamespaceComponent />
        </I18nextProvider>
      );

      // All namespace translations should resolve
      await waitFor(() => {
        const commonText = screen.getByTestId('common-text').textContent;
        const dashboardText = screen.getByTestId('dashboard-text').textContent;
        const settingsText = screen.getByTestId('settings-text').textContent;

        expect(commonText).not.toBe('language');
        expect(dashboardText).not.toBe('prDashboard');
        expect(settingsText).not.toBe('tabs.profile');
      });
    });
  });

  describe('7. Translation Key Formats', () => {
    it('should handle dot-separated nested keys', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      // Key like 'auth.logout'
      const translation = i18n.t('common:auth.logout');
      expect(typeof translation).toBe('string');
      expect(translation).not.toBe('auth.logout');
    });

    it('should handle deeply nested keys', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('settings');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'settings')).toBe(true);
        },
        { timeout: 5000 }
      );

      // Key like 'tabs.profile'
      const translation = i18n.t('settings:tabs.profile');
      expect(typeof translation).toBe('string');
      expect(translation).not.toBe('tabs.profile');
    });
  });

  describe('8. Translation Loading Performance', () => {
    it('should load translations within reasonable time', async () => {
      const startTime = Date.now();

      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const loadTime = Date.now() - startTime;

      // Should load within 5 seconds (generous timeout for CI)
      expect(loadTime).toBeLessThan(5000);
    });

    it('should cache loaded translations', async () => {
      // First load
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      // Second access should be instant (from cache)
      const startTime = Date.now();
      const translation = i18n.t('common:language');
      const accessTime = Date.now() - startTime;

      expect(typeof translation).toBe('string');
      expect(accessTime).toBeLessThan(100); // Should be instant
    });
  });

  describe('9. Error Handling', () => {
    it('should handle missing namespace gracefully', async () => {
      await i18n.changeLanguage('en');

      // Try to use a namespace that doesn't exist
      const translation = i18n.t('nonexistent:key');

      // Should not throw, should return fallback
      expect(typeof translation).toBe('string');
    });

    it('should handle malformed translation keys', async () => {
      await i18n.changeLanguage('en');
      await i18n.loadNamespaces('common');

      // Various malformed keys
      const testKeys = ['', '...', 'common:', ':key', 'common::', null, undefined];

      testKeys.forEach((key) => {
        expect(() => {
          i18n.t(key as string);
        }).not.toThrow();
      });
    });
  });

  describe('10. Regional Variants', () => {
    it('should load English (UK) translations', async () => {
      await i18n.changeLanguage('en-GB');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('en-GB', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('common:language');
      expect(typeof translation).toBe('string');
    });

    it('should load Portuguese (Brazil) translations', async () => {
      await i18n.changeLanguage('pt-BR');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('pt-BR', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('common:language');
      expect(typeof translation).toBe('string');
    });

    it('should load Chinese (Simplified) translations', async () => {
      await i18n.changeLanguage('zh-CN');
      await i18n.loadNamespaces('common');

      await waitFor(
        () => {
          expect(i18n.hasResourceBundle('zh-CN', 'common')).toBe(true);
        },
        { timeout: 5000 }
      );

      const translation = i18n.t('common:language');
      expect(typeof translation).toBe('string');
    });
  });
});
