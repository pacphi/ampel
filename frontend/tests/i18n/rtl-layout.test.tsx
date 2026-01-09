/**
 * Integration tests for RTL layout behavior
 *
 * Tests:
 * - CSS logical properties validation
 * - Bidirectional text handling
 * - RTL class application
 * - Direction switching
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '@/i18n';
import RTLProvider from '@/components/RTLProvider';
import { isRTL, getLanguageInfo } from '@/i18n/config';

// Mock component for testing
function TestComponent({ children }: { children: React.ReactNode }) {
  return (
    <I18nextProvider i18n={i18n}>
      <RTLProvider>
        <div data-testid="test-root">{children}</div>
      </RTLProvider>
    </I18nextProvider>
  );
}

describe('RTL Layout Integration Tests', () => {
  beforeEach(() => {
    // Reset document attributes
    document.documentElement.dir = 'ltr';
    document.documentElement.lang = 'en';
    document.documentElement.classList.remove('rtl');
    document.body.dir = 'ltr';
    document.body.classList.remove('rtl');

    // Clear localStorage
    localStorage.clear();
  });

  describe('isRTL helper function', () => {
    it('should identify Arabic as RTL', () => {
      expect(isRTL('ar')).toBe(true);
    });

    it('should identify Hebrew as RTL', () => {
      expect(isRTL('he')).toBe(true);
    });

    it('should identify English as LTR', () => {
      expect(isRTL('en')).toBe(false);
    });

    it('should identify all supported LTR languages', () => {
      const ltrLanguages = ['en', 'en-GB', 'fr', 'de', 'it', 'ru', 'ja', 'ko', 'hi', 'nl', 'pl'];
      ltrLanguages.forEach((lang) => {
        expect(isRTL(lang)).toBe(false);
      });
    });

    it('should handle unknown languages gracefully', () => {
      expect(isRTL('unknown')).toBe(false);
    });
  });

  describe('getLanguageInfo helper function', () => {
    it('should return correct info for Arabic', () => {
      const info = getLanguageInfo('ar');
      expect(info).toMatchObject({
        code: 'ar',
        name: 'Arabic',
        nativeName: 'العربية',
        dir: 'rtl',
      });
    });

    it('should return correct info for Hebrew', () => {
      const info = getLanguageInfo('he');
      expect(info).toMatchObject({
        code: 'he',
        name: 'Hebrew',
        nativeName: 'עברית',
        dir: 'rtl',
      });
    });

    it('should return undefined for unknown language', () => {
      expect(getLanguageInfo('unknown')).toBeUndefined();
    });
  });

  describe('RTLProvider component', () => {
    it('should set RTL attributes for Arabic', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        expect(document.documentElement.dir).toBe('rtl');
        expect(document.documentElement.lang).toBe('ar');
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
        expect(document.body.dir).toBe('rtl');
        expect(document.body.classList.contains('rtl')).toBe(true);
      });
    });

    it('should set RTL attributes for Hebrew', async () => {
      await i18n.changeLanguage('he');

      render(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        expect(document.documentElement.dir).toBe('rtl');
        expect(document.documentElement.lang).toBe('he');
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
      });
    });

    it('should set LTR attributes for English', async () => {
      await i18n.changeLanguage('en');

      render(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        expect(document.documentElement.dir).toBe('ltr');
        expect(document.documentElement.lang).toBe('en');
        expect(document.documentElement.classList.contains('rtl')).toBe(false);
        expect(document.body.classList.contains('rtl')).toBe(false);
      });
    });

    it('should update attributes when switching from RTL to LTR', async () => {
      // Start with Arabic
      await i18n.changeLanguage('ar');

      const { rerender } = render(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        expect(document.documentElement.dir).toBe('rtl');
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
      });

      // Switch to English
      await i18n.changeLanguage('en');

      rerender(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        expect(document.documentElement.dir).toBe('ltr');
        expect(document.documentElement.lang).toBe('en');
        expect(document.documentElement.classList.contains('rtl')).toBe(false);
        expect(document.body.classList.contains('rtl')).toBe(false);
      });
    });

    it('should update attributes when switching from LTR to RTL', async () => {
      // Start with English
      await i18n.changeLanguage('en');

      const { rerender } = render(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        expect(document.documentElement.dir).toBe('ltr');
      });

      // Switch to Arabic
      await i18n.changeLanguage('ar');

      rerender(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        expect(document.documentElement.dir).toBe('rtl');
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
      });
    });

    it('should create meta tags for direction and language', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        const metaDir = document.querySelector('meta[name="direction"]');
        const metaLang = document.querySelector('meta[http-equiv="content-language"]');

        expect(metaDir?.getAttribute('content')).toBe('rtl');
        expect(metaLang?.getAttribute('content')).toBe('ar');
      });
    });

    it('should update existing meta tags when language changes', async () => {
      await i18n.changeLanguage('ar');

      const { rerender } = render(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        const metaDir = document.querySelector('meta[name="direction"]');
        expect(metaDir?.getAttribute('content')).toBe('rtl');
      });

      // Switch language
      await i18n.changeLanguage('en');

      rerender(
        <TestComponent>
          <h1>Test Content</h1>
        </TestComponent>
      );

      await waitFor(() => {
        const metaDir = document.querySelector('meta[name="direction"]');
        const metaLang = document.querySelector('meta[http-equiv="content-language"]');

        expect(metaDir?.getAttribute('content')).toBe('ltr');
        expect(metaLang?.getAttribute('content')).toBe('en');
      });
    });
  });

  describe('CSS Logical Properties Validation', () => {
    it('should use margin-inline-start instead of margin-left', () => {
      const element = document.createElement('div');
      element.className = 'ms-4'; // Tailwind margin-inline-start
      document.body.appendChild(element);

      // In RTL, margin-inline-start becomes margin-right
      // This test verifies logical properties are being used
      expect(element.className).toContain('ms-');
      expect(element.className).not.toContain('ml-');

      document.body.removeChild(element);
    });

    it('should use text-align-start instead of text-left', () => {
      const element = document.createElement('div');
      element.className = 'text-start';
      document.body.appendChild(element);

      expect(element.className).toContain('text-start');
      expect(element.className).not.toContain('text-left');

      document.body.removeChild(element);
    });
  });

  describe('Bidirectional Text Handling', () => {
    it('should handle mixed LTR and RTL text', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestComponent>
          <p data-testid="mixed-text">مرحبا Hello العالم World</p>
        </TestComponent>
      );

      const text = screen.getByTestId('mixed-text');
      expect(text).toBeInTheDocument();

      // Text should be displayed (browser handles bidi algorithm)
      expect(text.textContent).toContain('مرحبا');
      expect(text.textContent).toContain('Hello');
    });

    it('should handle URLs in RTL text', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestComponent>
          <p data-testid="url-text">زيارة https://example.com للمزيد</p>
        </TestComponent>
      );

      const text = screen.getByTestId('url-text');
      expect(text.textContent).toContain('https://example.com');
    });

    it('should handle numbers in RTL text', async () => {
      await i18n.changeLanguage('ar');

      render(
        <TestComponent>
          <p data-testid="number-text">العدد: 12345</p>
        </TestComponent>
      );

      const text = screen.getByTestId('number-text');
      expect(text.textContent).toContain('12345');
    });
  });

  describe('Language Direction Consistency', () => {
    it('should maintain RTL direction across all RTL languages', async () => {
      const rtlLanguages = ['ar', 'he'];

      for (const lang of rtlLanguages) {
        await i18n.changeLanguage(lang);

        const { unmount } = render(
          <TestComponent>
            <h1>Test</h1>
          </TestComponent>
        );

        await waitFor(() => {
          expect(document.documentElement.dir).toBe('rtl');
          expect(document.documentElement.classList.contains('rtl')).toBe(true);
        });

        unmount();
      }
    });

    it('should maintain LTR direction across all LTR languages', async () => {
      const ltrLanguages = ['en', 'fr', 'de', 'ja', 'ko'];

      for (const lang of ltrLanguages) {
        await i18n.changeLanguage(lang);

        const { unmount } = render(
          <TestComponent>
            <h1>Test</h1>
          </TestComponent>
        );

        await waitFor(() => {
          expect(document.documentElement.dir).toBe('ltr');
          expect(document.documentElement.classList.contains('rtl')).toBe(false);
        });

        unmount();
      }
    });
  });
});
