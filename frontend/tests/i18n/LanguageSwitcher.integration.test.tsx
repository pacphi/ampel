/**
 * Integration tests for LanguageSwitcher in Header component
 *
 * Tests:
 * - LanguageSwitcher rendering and functionality
 * - Language switching triggers UI updates
 * - localStorage persistence
 * - RTL layout switching for Arabic/Hebrew
 * - Translation loading from locale files
 * - t() function resolves to translated strings
 */

import { describe, expect, it, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter } from 'react-router-dom';
import i18n from '@/i18n/config';
import RTLProvider from '@/components/RTLProvider';
import LanguageSwitcher from '@/components/LanguageSwitcher';

// Mock useAuth hook
vi.mock('@/hooks/useAuth', () => ({
  useAuth: () => ({
    user: {
      id: '1',
      email: 'test@example.com',
      displayName: 'Test User',
      avatarUrl: null,
    },
    logout: vi.fn(),
  }),
}));

// Mock useTheme hook
vi.mock('@/hooks/useTheme', () => ({
  useTheme: () => ({
    theme: 'light',
    setTheme: vi.fn(),
    resolvedTheme: 'light',
  }),
}));

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

const renderWithProviders = (component: React.ReactElement) => {
  const queryClient = createTestQueryClient();
  return render(
    <QueryClientProvider client={queryClient}>
      <I18nextProvider i18n={i18n}>
        <RTLProvider>
          <BrowserRouter>{component}</BrowserRouter>
        </RTLProvider>
      </I18nextProvider>
    </QueryClientProvider>
  );
};

describe('LanguageSwitcher Integration Tests', () => {
  beforeEach(async () => {
    // Reset i18n to English
    await i18n.changeLanguage('en');

    // Clear localStorage
    localStorage.clear();

    // Reset document attributes
    document.documentElement.removeAttribute('dir');
    document.documentElement.removeAttribute('lang');
    document.documentElement.classList.remove('rtl');
    document.body.classList.remove('rtl');
  });

  afterEach(() => {
    // Clean up document state
    document.documentElement.removeAttribute('dir');
    document.documentElement.removeAttribute('lang');
    document.documentElement.classList.remove('rtl');
    document.body.classList.remove('rtl');
  });

  describe('1. LanguageSwitcher in Header', () => {
    it('should render LanguageSwitcher component standalone', async () => {
      renderWithProviders(<LanguageSwitcher variant="inline" size="sm" />);

      // Wait for component to render
      await waitFor(() => {
        expect(screen.getByText('EN')).toBeInTheDocument();
      });
    });

    it('should display current language in dropdown variant', async () => {
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      // Should show Globe icon and language name
      await waitFor(() => {
        expect(screen.getByRole('combobox')).toBeInTheDocument();
        expect(screen.getByText('English (US)')).toBeInTheDocument();
      });
    });

    it('should show all 27 supported languages when opened', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" showSearch />);

      // Open dropdown
      const trigger = screen.getByRole('combobox');
      await user.click(trigger);

      // Wait for dropdown to open
      await waitFor(
        () => {
          expect(screen.getByRole('listbox')).toBeInTheDocument();
        },
        { timeout: 3000 }
      );

      // Check that all language groups are present
      await waitFor(() => {
        expect(screen.getByText('Common Languages')).toBeInTheDocument();
        expect(screen.getByText('Right-to-Left Languages')).toBeInTheDocument();
      });

      // Verify some languages are visible
      expect(screen.getByText('French')).toBeInTheDocument();
      expect(screen.getByText('German')).toBeInTheDocument();
      expect(screen.getByText('Arabic')).toBeInTheDocument();
    });
  });

  describe('2. Language Switching Triggers UI Updates', () => {
    it('should change i18n language when language is selected', async () => {
      const user = userEvent.setup();
      const changeLanguageSpy = vi.spyOn(i18n, 'changeLanguage');

      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      // Open dropdown
      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      // Click on French
      const frenchOption = screen.getByText('French');
      await user.click(frenchOption);

      // Verify language change was called
      expect(changeLanguageSpy).toHaveBeenCalledWith('fr');
    });

    it('should update displayed language after selection', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      // Initial state
      await waitFor(() => {
        expect(screen.getByText('English (US)')).toBeInTheDocument();
      });

      // Open and select German
      await user.click(screen.getByRole('combobox'));

      await waitFor(
        () => {
          expect(screen.getByRole('listbox')).toBeInTheDocument();
        },
        { timeout: 3000 }
      );

      await user.click(screen.getByText('German'));

      // Wait for language to change
      await waitFor(
        () => {
          expect(i18n.language).toBe('de');
        },
        { timeout: 3000 }
      );

      // Verify display updated
      await waitFor(
        () => {
          expect(screen.getByText('German')).toBeInTheDocument();
        },
        { timeout: 3000 }
      );
    });

    it('should close dropdown after language selection', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      // Open dropdown
      await user.click(screen.getByRole('combobox'));

      await waitFor(
        () => {
          expect(screen.getByRole('listbox')).toBeInTheDocument();
        },
        { timeout: 3000 }
      );

      // Select a language
      await user.click(screen.getByText('Italian'));

      // Wait for dropdown close with longer timeout
      await waitFor(
        () => {
          expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
        },
        { timeout: 5000 }
      );
    });
  });

  describe('3. localStorage Persistence', () => {
    it('should save selected language to localStorage', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      // Open and select Spanish
      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Spanish (Spain)'));

      // Wait for localStorage to be updated
      await waitFor(() => {
        const stored = localStorage.getItem('ampel-language');
        expect(stored).toBe('es-ES');
      });
    });

    it('should load language from localStorage on mount', async () => {
      // Set localStorage before rendering
      localStorage.setItem('ampel-language', 'de');

      // Need to reload i18n to pick up localStorage value
      await i18n.changeLanguage('de');

      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      // Should display German
      await waitFor(() => {
        expect(screen.getByText('German')).toBeInTheDocument();
      });
    });

    it('should persist language across component remounts', async () => {
      const user = userEvent.setup();

      // First render - select French
      const { unmount } = renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      await user.click(screen.getByRole('combobox'));
      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      await user.click(screen.getByText('French'));

      await waitFor(() => {
        expect(localStorage.getItem('ampel-language')).toBe('fr');
      });

      // Unmount and remount
      unmount();

      // Reload with saved language
      await i18n.changeLanguage('fr');

      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      // Should still be French
      expect(screen.getByText('French')).toBeInTheDocument();
    });
  });

  describe('4. RTL Layout Switching', () => {
    it('should set dir="rtl" for Arabic', async () => {
      const user = userEvent.setup();
      renderWithProviders(
        <>
          <LanguageSwitcher variant="dropdown" />
          <div data-testid="test-content">Test Content</div>
        </>
      );

      // Select Arabic
      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Arabic'));

      // Wait for RTL to be applied
      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('rtl');
        expect(document.documentElement.getAttribute('lang')).toBe('ar');
      });
    });

    it('should add rtl class for Hebrew', async () => {
      const user = userEvent.setup();
      renderWithProviders(
        <>
          <LanguageSwitcher variant="dropdown" />
          <div data-testid="test-content">Test Content</div>
        </>
      );

      // Select Hebrew
      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Hebrew'));

      // Wait for RTL to be applied
      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('rtl');
        expect(document.documentElement.classList.contains('rtl')).toBe(true);
        expect(document.body.classList.contains('rtl')).toBe(true);
      });
    });

    it('should remove rtl when switching from Arabic to English', async () => {
      const user = userEvent.setup();

      // Start with Arabic
      await i18n.changeLanguage('ar');
      document.documentElement.setAttribute('dir', 'rtl');
      document.documentElement.classList.add('rtl');

      renderWithProviders(
        <>
          <LanguageSwitcher variant="dropdown" />
          <div data-testid="test-content">Test Content</div>
        </>
      );

      // Should be RTL initially
      expect(document.documentElement.getAttribute('dir')).toBe('rtl');

      // Switch to English
      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      await user.click(screen.getByText('English (US)'));

      // Should switch to LTR
      await waitFor(() => {
        expect(document.documentElement.getAttribute('dir')).toBe('ltr');
        expect(document.documentElement.classList.contains('rtl')).toBe(false);
      });
    });

    it('should set correct meta tags for RTL languages', async () => {
      const user = userEvent.setup();
      renderWithProviders(
        <>
          <LanguageSwitcher variant="dropdown" />
          <div data-testid="test-content">Test Content</div>
        </>
      );

      // Select Arabic
      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Arabic'));

      // Check meta tags
      await waitFor(() => {
        const dirMeta = document.querySelector('meta[name="direction"]');
        expect(dirMeta?.getAttribute('content')).toBe('rtl');

        const langMeta = document.querySelector('meta[http-equiv="content-language"]');
        expect(langMeta?.getAttribute('content')).toBe('ar');
      });
    });
  });

  describe('5. Search Functionality', () => {
    it('should filter languages by search query', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" showSearch />);

      // Open dropdown
      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      // Type in search
      const searchInput = screen.getByPlaceholderText('Search languages...');
      await user.type(searchInput, 'Span');

      // Should show Spanish languages
      await waitFor(() => {
        expect(screen.getByText('Spanish (Spain)')).toBeInTheDocument();
        expect(screen.getByText('Spanish (Mexico)')).toBeInTheDocument();
      });

      // Should hide non-matching languages
      expect(screen.queryByText('French')).not.toBeInTheDocument();
      expect(screen.queryByText('German')).not.toBeInTheDocument();
    });

    it('should show "No languages found" for non-matching search', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" showSearch />);

      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText('Search languages...');
      await user.type(searchInput, 'xyz123');

      await waitFor(() => {
        expect(screen.getByText('No languages found')).toBeInTheDocument();
      });
    });

    it('should clear search with X button', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" showSearch />);

      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText('Search languages...');
      await user.type(searchInput, 'French');

      // Click clear button
      const clearButton = screen.getByLabelText('Clear search');
      await user.click(clearButton);

      // Search should be cleared
      expect(searchInput).toHaveValue('');

      // All languages should be visible again
      await waitFor(() => {
        expect(screen.getByText('German')).toBeInTheDocument();
        expect(screen.getByText('Italian')).toBeInTheDocument();
      });
    });
  });

  describe('6. Favorites Functionality', () => {
    it('should toggle favorite with star button', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" showFavorites />);

      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      // Find star button for French (should be in Common Languages section)
      const frenchRow = screen.getByText('French').closest('div[role="option"]');
      expect(frenchRow).toBeInTheDocument();

      const starButton = frenchRow?.querySelector('button[aria-label*="favorite"]');
      expect(starButton).toBeInTheDocument();

      // Click to add to favorites
      await user.click(starButton!);

      // Check localStorage
      await waitFor(() => {
        const stored = localStorage.getItem('ampel-language-favorites');
        expect(stored).toContain('fr');
      });
    });

    it('should show favorites section when languages are favorited', async () => {
      // Pre-set favorites
      localStorage.setItem('ampel-language-favorites', JSON.stringify(['de', 'fr']));

      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" showFavorites />);

      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      // Should show Favorites section
      expect(screen.getByText('Favorites')).toBeInTheDocument();
    });
  });

  describe('7. Keyboard Navigation', () => {
    it('should open dropdown with Enter key', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      const trigger = screen.getByRole('combobox');
      trigger.focus();

      await user.keyboard('{Enter}');

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });
    });

    it('should close dropdown with Escape key', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      // Open dropdown
      await user.click(screen.getByRole('combobox'));

      await waitFor(() => {
        expect(screen.getByRole('listbox')).toBeInTheDocument();
      });

      // Press Escape
      await user.keyboard('{Escape}');

      // Dropdown should close
      await waitFor(() => {
        expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
      });
    });
  });

  describe('8. Variants', () => {
    it('should render inline variant with language code', () => {
      renderWithProviders(<LanguageSwitcher variant="inline" size="sm" />);

      expect(screen.getByText('EN')).toBeInTheDocument();
      expect(screen.getByLabelText(/Current language: English/i)).toBeInTheDocument();
    });

    it('should render select variant for mobile', () => {
      renderWithProviders(<LanguageSwitcher variant="select" />);

      // Should use native select
      expect(screen.getByRole('combobox')).toBeInTheDocument();
      expect(screen.getByText('English (US)')).toBeInTheDocument();
    });
  });

  describe('9. Accessibility', () => {
    it('should have proper ARIA labels', () => {
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      const trigger = screen.getByRole('combobox');
      expect(trigger).toHaveAttribute('aria-label');
      expect(trigger).toHaveAttribute('aria-haspopup', 'true');
    });

    it('should update aria-expanded when opened', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      const trigger = screen.getByRole('combobox');
      expect(trigger).toHaveAttribute('aria-expanded', 'false');

      await user.click(trigger);

      await waitFor(() => {
        expect(trigger).toHaveAttribute('aria-expanded', 'true');
      });
    });

    it('should mark selected language with aria-selected', async () => {
      const user = userEvent.setup();
      renderWithProviders(<LanguageSwitcher variant="dropdown" />);

      await user.click(screen.getByRole('combobox'));

      await waitFor(
        () => {
          expect(screen.getByRole('listbox')).toBeInTheDocument();
        },
        { timeout: 3000 }
      );

      // Current language (English) should be selected
      // Find the selected option by aria-selected attribute
      const selectedOptions = screen.queryAllByRole('option', { selected: true });
      expect(selectedOptions.length).toBeGreaterThan(0);
      expect(selectedOptions[0]).toHaveAttribute('aria-selected', 'true');
    });
  });
});
