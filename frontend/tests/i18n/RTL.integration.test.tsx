/**
 * RTL Provider Integration Tests
 *
 * Tests:
 * - RTL detection and layout switching
 * - Document attribute updates
 * - Meta tag management
 * - CSS class application
 * - Language direction changes
 */

import { describe, expect, it, beforeEach, afterEach } from 'vitest';
import { render, waitFor } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '@/i18n/config';
import RTLProvider from '@/components/RTLProvider';
import { isRTL } from '@/i18n/config';

describe('RTL Provider Integration Tests', () => {
  beforeEach(async () => {
    // Reset to English
    await i18n.changeLanguage('en');

    // Clear document state
    document.documentElement.removeAttribute('dir');
    document.documentElement.removeAttribute('lang');
    document.documentElement.classList.remove('rtl');
    document.body.removeAttribute('dir');
    document.body.classList.remove('rtl');

    // Remove meta tags
    document
      .querySelectorAll('meta[name="direction"], meta[http-equiv="content-language"]')
      .forEach((el) => {
        el.remove();
      });
  });

  afterEach(() => {
    // Clean up document state
    document.documentElement.removeAttribute('dir');
    document.documentElement.removeAttribute('lang');
    document.documentElement.classList.remove('rtl');
    document.body.removeAttribute('dir');
    document.body.classList.remove('rtl');
  });

  describe('1. RTL Detection', () => {
    it('should identify Arabic as RTL', () => {
      expect(isRTL('ar')).toBe(true);
    });

    it('should identify Hebrew as RTL', () => {
      expect(isRTL('he')).toBe(true);
    });

    it('should identify English as LTR', () => {
      expect(isRTL('en')).toBe(false);
    });

    it('should identify all non-RTL languages as LTR', () => {
      const ltrLanguages = [
        'en',
        'en-GB',
        'fr',
        'de',
        'it',
        'ru',
        'ja',
        'ko',
        'hi',
        'nl',
        'pl',
        'sr',
        'th',
        'tr',
        'sv',
        'da',
        'fi',
        'vi',
        'no',
        'cs',
        'pt-BR',
        'zh-CN',
        'zh-TW',
        'es-ES',
        'es-MX',
      ];

      ltrLanguages.forEach((lang) => {
        expect(isRTL(lang)).toBe(false);
      });
    });
  });

  describe('2. Document Direction Attribute', () => {
    it('should set dir="ltr" for English', async () => {
      await i18n.changeLanguage('en');

      render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('ltr');
        expect(document.body.getAttribute('dir')).toBe('ltr');
      });
    });

    it('should set dir="rtl" for Arabic', async () => {
      await i18n.changeLanguage('ar');

      render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('rtl');
        expect(document.body.getAttribute('dir')).toBe('rtl');
      });
    });

    it('should set dir="rtl" for Hebrew', async () => {
      await i18n.changeLanguage('he');

      render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('rtl');
        expect(document.body.getAttribute('dir')).toBe('rtl');
      });
    });
  });

  describe('3. Language Attribute', () => {
    it('should set lang attribute for each language', async () => {
      const testLanguages = ['en', 'fr', 'de', 'ar', 'he', 'ja', 'zh-CN'];

      for (const lang of testLanguages) {
        await i18n.changeLanguage(lang);

        const { unmount } = render(
          <I18nextProvider i18n={i18n}>
            <RTLProvider>
              <div>Test Content</div>
            </RTLProvider>
          </I18nextProvider>
        );

        await waitFor(() => {
          expect(document.documentElement.getAttribute('lang')).toBe(lang);
        });

        unmount();
      }
    });
  });

  describe('4. RTL CSS Class', () => {
    it('should add rtl class for Arabic', async () => {
      await i18n.changeLanguage('ar');

      render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
        expect(document.body.classList.contains('rtl')).toBe(true);
      });
    });

    it('should add rtl class for Hebrew', async () => {
      await i18n.changeLanguage('he');

      render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
        expect(document.body.classList.contains('rtl')).toBe(true);
      });
    });

    it('should not add rtl class for LTR languages', async () => {
      const ltrLanguages = ['en', 'fr', 'de', 'ja'];

      for (const lang of ltrLanguages) {
        await i18n.changeLanguage(lang);

        const { unmount } = render(
          <I18nextProvider i18n={i18n}>
            <RTLProvider>
              <div>Test Content</div>
            </RTLProvider>
          </I18nextProvider>
        );

        await waitFor(() => {
          expect(document.documentElement.classList.contains('rtl')).toBe(false);
          expect(document.body.classList.contains('rtl')).toBe(false);
        });

        unmount();
      }
    });

    it('should remove rtl class when switching from RTL to LTR', async () => {
      // Start with Arabic
      await i18n.changeLanguage('ar');

      const { rerender } = render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      // Verify RTL is applied
      await waitFor(() => {
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
      });

      // Switch to English
      await i18n.changeLanguage('en');

      rerender(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      // Verify RTL is removed
      await waitFor(() => {
        expect(document.documentElement.classList.contains('rtl')).toBe(false);
        expect(document.body.classList.contains('rtl')).toBe(false);
        expect(document.documentElement.getAttribute('dir')).toBe('ltr');
      });
    });
  });

  describe('5. Meta Tags', () => {
    it('should create and set direction meta tag', async () => {
      await i18n.changeLanguage('ar');

      render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        const metaDir = document.querySelector('meta[name="direction"]');
        expect(metaDir).not.toBeNull();
        expect(metaDir?.getAttribute('content')).toBe('rtl');
      });
    });

    it('should create and set content-language meta tag', async () => {
      await i18n.changeLanguage('fr');

      render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        const metaLang = document.querySelector('meta[http-equiv="content-language"]');
        expect(metaLang).not.toBeNull();
        expect(metaLang?.getAttribute('content')).toBe('fr');
      });
    });

    it('should update meta tags when language changes', async () => {
      await i18n.changeLanguage('en');

      const { rerender } = render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        const metaDir = document.querySelector('meta[name="direction"]');
        expect(metaDir?.getAttribute('content')).toBe('ltr');
      });

      // Change to Arabic
      await i18n.changeLanguage('ar');

      rerender(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div>Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      await waitFor(() => {
        const metaDir = document.querySelector('meta[name="direction"]');
        expect(metaDir?.getAttribute('content')).toBe('rtl');

        const metaLang = document.querySelector('meta[http-equiv="content-language"]');
        expect(metaLang?.getAttribute('content')).toBe('ar');
      });
    });
  });

  describe('6. Language Transitions', () => {
    it('should handle LTR to RTL transition', async () => {
      await i18n.changeLanguage('en');

      const { rerender, getByTestId } = render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div data-testid="content">Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      // Verify content is rendered and initial LTR state
      expect(getByTestId('content')).toHaveTextContent('Test Content');
      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('ltr');
        expect(document.documentElement.classList.contains('rtl')).toBe(false);
      });

      // Switch to Hebrew
      await i18n.changeLanguage('he');

      rerender(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div data-testid="content">Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      // Verify RTL state
      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('rtl');
        expect(document.documentElement.getAttribute('lang')).toBe('he');
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
      });
    });

    it('should handle RTL to RTL transition (Arabic to Hebrew)', async () => {
      await i18n.changeLanguage('ar');

      const { rerender, getByTestId } = render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div data-testid="content">Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      // Verify content and initial Arabic RTL state
      expect(getByTestId('content')).toHaveTextContent('Test Content');
      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('rtl');
        expect(document.documentElement.getAttribute('lang')).toBe('ar');
      });

      // Switch to Hebrew (still RTL)
      await i18n.changeLanguage('he');

      rerender(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div data-testid="content">Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      // Should still be RTL but with different language
      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('rtl');
        expect(document.documentElement.getAttribute('lang')).toBe('he');
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
      });
    });

    it('should handle multiple rapid language changes', async () => {
      const { rerender, getByTestId } = render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div data-testid="content">Test Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      // Verify initial content
      expect(getByTestId('content')).toHaveTextContent('Test Content');

      // Rapid language changes
      const languages = ['en', 'ar', 'fr', 'he', 'de', 'ar', 'en'];

      for (const lang of languages) {
        await i18n.changeLanguage(lang);

        rerender(
          <I18nextProvider i18n={i18n}>
            <RTLProvider>
              <div data-testid="content">Test Content</div>
            </RTLProvider>
          </I18nextProvider>
        );

        await waitFor(() => {
          const expectedDir = isRTL(lang) ? 'rtl' : 'ltr';
          expect(document.documentElement.getAttribute('dir')).toBe(expectedDir);
          expect(document.documentElement.getAttribute('lang')).toBe(lang);
        });
      }
    });
  });

  describe('7. Children Rendering', () => {
    it('should render children without modification', async () => {
      await i18n.changeLanguage('en');

      const { getByTestId } = render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div data-testid="child-1">Child 1</div>
            <div data-testid="child-2">Child 2</div>
          </RTLProvider>
        </I18nextProvider>
      );

      expect(getByTestId('child-1')).toHaveTextContent('Child 1');
      expect(getByTestId('child-2')).toHaveTextContent('Child 2');
    });

    it('should preserve children during language changes', async () => {
      await i18n.changeLanguage('en');

      const { getByTestId, rerender } = render(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div data-testid="persistent-child">Persistent Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      expect(getByTestId('persistent-child')).toHaveTextContent('Persistent Content');

      // Change to Arabic
      await i18n.changeLanguage('ar');

      rerender(
        <I18nextProvider i18n={i18n}>
          <RTLProvider>
            <div data-testid="persistent-child">Persistent Content</div>
          </RTLProvider>
        </I18nextProvider>
      );

      // Children should still be there
      expect(getByTestId('persistent-child')).toHaveTextContent('Persistent Content');
    });
  });
});
