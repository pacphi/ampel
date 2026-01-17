/**
 * LanguageSwitcher Component - BDD Component Tests
 *
 * Tests the LanguageSwitcher component with REAL i18n integration.
 * NO mocking of useTranslation or i18n behavior.
 *
 * Test Infrastructure:
 * - Real i18n instance from tests/setup/i18n-instance.tsx
 * - RTL testing utilities from tests/setup/rtl-testing.ts
 * - RTLProvider to handle document direction changes
 */

import { describe, it, expect, beforeAll, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

// Real i18n testing infrastructure
import {
  setupTestI18n,
  getTestI18n,
  I18nTestProvider,
  changeTestLanguage,
  resetTestI18n,
} from '../setup/i18n-instance';

// RTL testing utilities
import { resetRTLState, getRTLState, expectLTRLayout, setRTLState } from '../setup/rtl-testing';

// Component under test
import { LanguageSwitcher } from '@/components/LanguageSwitcher';
import RTLProvider from '@/components/RTLProvider';

// Constants for verification
import { STORAGE_KEY_LANGUAGE, STORAGE_KEY_FAVORITES } from '@/components/i18n/constants/languages';

// ============================================================================
// Test Setup
// ============================================================================

/**
 * Wrapper that provides real i18n context AND RTLProvider for direction changes
 */
function TestWrapper({ children }: { children: React.ReactNode }) {
  return (
    <I18nTestProvider>
      <RTLProvider>{children}</RTLProvider>
    </I18nTestProvider>
  );
}

/**
 * Render helper with real i18n and RTL support
 */
function renderLanguageSwitcher(props?: React.ComponentProps<typeof LanguageSwitcher>) {
  return render(<LanguageSwitcher {...props} />, {
    wrapper: TestWrapper,
  });
}

// ============================================================================
// Test Suite
// ============================================================================

describe('LanguageSwitcher Component', () => {
  beforeAll(async () => {
    await setupTestI18n('en');
  });

  beforeEach(async () => {
    // Reset to English before each test
    await resetTestI18n();
    resetRTLState();
    localStorage.clear();
  });

  afterEach(() => {
    resetRTLState();
    vi.clearAllMocks();
  });

  // ==========================================================================
  // Feature: Language Display
  // ==========================================================================

  describe('Feature: Language Display', () => {
    describe('Scenario: Shows Current Language', () => {
      it('Given: User\'s language is English, When: LanguageSwitcher renders, Then: "English" is displayed', async () => {
        // Given: User's language is English (default from beforeEach)
        const i18n = getTestI18n();
        expect(i18n.language).toBe('en');

        // When: LanguageSwitcher renders
        renderLanguageSwitcher({ variant: 'dropdown' });

        // Then: "English" or related text is displayed
        const trigger = screen.getByRole('combobox');
        expect(trigger).toBeInTheDocument();
        expect(trigger).toHaveTextContent(/english/i);
      });

      it('Given: User\'s language is French, When: LanguageSwitcher renders, Then: "French" is displayed', async () => {
        // Given: User's language is French - set localStorage so component picks it up
        localStorage.setItem(STORAGE_KEY_LANGUAGE, 'fr');
        const i18n = getTestI18n();
        await i18n.changeLanguage('fr');

        // When: LanguageSwitcher renders
        renderLanguageSwitcher({ variant: 'dropdown' });

        // Wait for component to reflect the language change
        await waitFor(() => {
          const trigger = screen.getByRole('combobox');
          // Then: "French" is displayed (checking aria-label is more reliable)
          expect(trigger.getAttribute('aria-label')).toMatch(/french|fran[çc]ais/i);
        });
      });
    });

    describe('Scenario: Shows All Languages in Dropdown', () => {
      it('Given: LanguageSwitcher dropdown is open, When: User views options, Then: All 27 supported languages are listed', async () => {
        const user = userEvent.setup();

        // Given: LanguageSwitcher is rendered
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        // When: User opens the dropdown
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // Then: All 27 supported languages are listed
        // We check for a few key languages across different groups
        // Note: English (US) appears twice (trigger + dropdown), so we use getAllByText
        await waitFor(() => {
          // Common languages - English appears in both trigger and dropdown
          expect(screen.getAllByText('English (US)').length).toBeGreaterThanOrEqual(1);
          expect(screen.getByText('French')).toBeInTheDocument();
          expect(screen.getByText('German')).toBeInTheDocument();

          // RTL languages
          expect(screen.getByText('Arabic')).toBeInTheDocument();
          expect(screen.getByText('Hebrew')).toBeInTheDocument();

          // Asian languages
          expect(screen.getByText('Japanese')).toBeInTheDocument();
          expect(screen.getByText('Korean')).toBeInTheDocument();
          expect(screen.getByText('Chinese (Simplified)')).toBeInTheDocument();

          // European languages
          expect(screen.getByText('Polish')).toBeInTheDocument();
          expect(screen.getByText('Russian')).toBeInTheDocument();
        });
      });

      it('Given: Dropdown is open, Then: Languages are grouped by category', async () => {
        const user = userEvent.setup();

        // Given: LanguageSwitcher is rendered
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        // When: User opens the dropdown
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // Then: Categories are present
        await waitFor(() => {
          expect(screen.getByText('Common Languages')).toBeInTheDocument();
          expect(screen.getByText('Right-to-Left Languages')).toBeInTheDocument();
          expect(screen.getByText('Other Languages')).toBeInTheDocument();
        });
      });
    });

    describe('Scenario: Inline Variant Shows Language Code', () => {
      it('Given: Inline variant is used, Then: Shows language code in uppercase', async () => {
        // When: Inline variant is rendered
        renderLanguageSwitcher({ variant: 'inline' });

        // Then: Shows "EN" for English
        const button = screen.getByRole('button');
        expect(button).toHaveTextContent('EN');
      });

      it('Given: Current language is Arabic and inline variant, Then: Shows "AR"', async () => {
        // Given: Language is Arabic - set localStorage so component picks it up
        localStorage.setItem(STORAGE_KEY_LANGUAGE, 'ar');
        const i18n = getTestI18n();
        await i18n.changeLanguage('ar');

        // When: Inline variant is rendered
        renderLanguageSwitcher({ variant: 'inline' });

        // Then: Shows "AR"
        await waitFor(() => {
          const button = screen.getByRole('button');
          expect(button).toHaveTextContent('AR');
        });
      });
    });

    describe('Scenario: Select Variant (Mobile)', () => {
      it('Given: Select variant is used, Then: Renders as a select element', async () => {
        // When: Select variant is rendered
        renderLanguageSwitcher({ variant: 'select' });

        // Then: Has select trigger with language name
        const trigger = screen.getByRole('combobox');
        expect(trigger).toBeInTheDocument();
        expect(trigger).toHaveAttribute('aria-label', 'Select language');
      });
    });
  });

  // ==========================================================================
  // Feature: Language Switching
  // ==========================================================================

  describe('Feature: Language Switching', () => {
    describe('Scenario: Switch to French', () => {
      it('Given: Current language is English, When: User selects French, Then: i18n language changes to "fr"', async () => {
        const user = userEvent.setup();
        const onLanguageChange = vi.fn();

        // Given: Current language is English
        const i18n = getTestI18n();
        expect(i18n.language).toBe('en');

        // Render component
        // Note: French is in testResources (es-ES is not), so we test with a supported language
        renderLanguageSwitcher({ variant: 'dropdown', onLanguageChange, showSearch: false });

        // When: User opens dropdown and selects French
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // Wait for French to appear in dropdown
        await waitFor(() => {
          expect(screen.getByText('French')).toBeInTheDocument();
        });

        // Click French - French is in "Common Languages" group
        const frenchOption = screen.getByText('French');
        await user.click(frenchOption);

        // Then: i18n language changes to 'fr'
        await waitFor(() => {
          expect(i18n.language).toBe('fr');
        });

        // And: callback is called
        expect(onLanguageChange).toHaveBeenCalledWith('fr');
      });

      it('Given: Current language is English, When: User selects French, Then: localStorage is updated', async () => {
        const user = userEvent.setup();

        // Given: Current language is English, localStorage is empty
        expect(localStorage.getItem(STORAGE_KEY_LANGUAGE)).toBeNull();

        // Render component
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        // When: User opens dropdown and selects French
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        await waitFor(() => {
          expect(screen.getByText('French')).toBeInTheDocument();
        });

        const frenchOption = screen.getByText('French');
        await user.click(frenchOption);

        // Then: localStorage is updated
        await waitFor(() => {
          expect(localStorage.getItem(STORAGE_KEY_LANGUAGE)).toBe('fr');
        });
      });
    });

    describe('Scenario: Switch to Arabic (RTL)', () => {
      it('Given: Current language is English, When: User selects Arabic, Then: i18n language changes to "ar"', async () => {
        const user = userEvent.setup();

        // Given: Current language is English
        const i18n = getTestI18n();
        expect(i18n.language).toBe('en');

        // Render component
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        // When: User opens dropdown and selects Arabic
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        await waitFor(() => {
          expect(screen.getByText('Arabic')).toBeInTheDocument();
        });

        const arabicOption = screen.getByText('Arabic');
        await user.click(arabicOption);

        // Then: i18n language changes to 'ar'
        await waitFor(() => {
          expect(i18n.language).toBe('ar');
        });
      });

      it('Given: Current language is English, When: User selects Arabic, Then: Document direction becomes RTL', async () => {
        const user = userEvent.setup();

        // Given: Document is LTR
        expectLTRLayout();

        // Render component
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        // When: User opens dropdown and selects Arabic
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        await waitFor(() => {
          expect(screen.getByText('Arabic')).toBeInTheDocument();
        });

        const arabicOption = screen.getByText('Arabic');
        await user.click(arabicOption);

        // Then: Document direction becomes RTL
        await waitFor(() => {
          const state = getRTLState();
          expect(state.documentDir).toBe('rtl');
          expect(state.hasRTLClass).toBe(true);
          expect(state.documentLang).toBe('ar');
        });
      });
    });

    describe('Scenario: Switch to Hebrew (RTL)', () => {
      it('Given: Current language is English, When: User selects Hebrew, Then: Document direction becomes RTL', async () => {
        const user = userEvent.setup();

        // Given: Document is LTR
        expectLTRLayout();

        // Render component
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        // When: User opens dropdown and selects Hebrew
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        await waitFor(() => {
          expect(screen.getByText('Hebrew')).toBeInTheDocument();
        });

        const hebrewOption = screen.getByText('Hebrew');
        await user.click(hebrewOption);

        // Then: Document direction becomes RTL
        await waitFor(() => {
          const state = getRTLState();
          expect(state.documentDir).toBe('rtl');
          expect(state.hasRTLClass).toBe(true);
          expect(state.documentLang).toBe('he');
        });
      });
    });

    describe('Scenario: Switch back to LTR from RTL', () => {
      it('Given: Current language is Arabic (RTL), When: User selects English, Then: Document direction returns to LTR', async () => {
        const user = userEvent.setup();

        // Given: Current language is Arabic (RTL) - set localStorage so component picks it up
        localStorage.setItem(STORAGE_KEY_LANGUAGE, 'ar');
        const i18n = getTestI18n();
        await i18n.changeLanguage('ar');
        setRTLState('ar');

        const rtlState = getRTLState();
        expect(rtlState.documentDir).toBe('rtl');

        // Render component (will show Arabic as current)
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        // When: User opens dropdown and selects English
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // Wait for dropdown to open and find English in the listbox
        await waitFor(() => {
          expect(screen.getByRole('listbox')).toBeInTheDocument();
        });

        // Find English option in the dropdown (not the trigger area)
        const listbox = screen.getByRole('listbox');
        const englishOption = within(listbox).getByText('English (US)');
        await user.click(englishOption);

        // Then: Document direction returns to LTR
        await waitFor(() => {
          const state = getRTLState();
          expect(state.documentDir).toBe('ltr');
          expect(state.hasRTLClass).toBe(false);
          expect(state.documentLang).toBe('en');
        });
      });
    });
  });

  // ==========================================================================
  // Feature: Persistence
  // ==========================================================================

  describe('Feature: Persistence', () => {
    describe('Scenario: Language persists on reload', () => {
      it('Given: User selected French, When: Component remounts, Then: French is still selected', async () => {
        const user = userEvent.setup();

        // Given: User selects French
        const { unmount } = renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        await waitFor(() => {
          expect(screen.getByText('French')).toBeInTheDocument();
        });

        const frenchOption = screen.getByText('French');
        await user.click(frenchOption);

        // Verify French was selected and stored
        await waitFor(() => {
          expect(localStorage.getItem(STORAGE_KEY_LANGUAGE)).toBe('fr');
        });

        // Unmount and remount
        unmount();

        // When: Component remounts
        renderLanguageSwitcher({ variant: 'dropdown' });

        // Then: French is still selected (component reads from localStorage)
        await waitFor(() => {
          const newTrigger = screen.getByRole('combobox');
          expect(newTrigger).toHaveTextContent(/french|fran[çc]ais/i);
        });
      });
    });
  });

  // ==========================================================================
  // Feature: Search Functionality
  // ==========================================================================

  describe('Feature: Search Functionality', () => {
    describe('Scenario: Filter languages by search', () => {
      it('Given: Dropdown is open with search enabled, When: User types "Span", Then: Only Spanish languages are visible', async () => {
        const user = userEvent.setup();

        // Given: Dropdown with search enabled
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: true });

        // Open dropdown
        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // When: User types "Span" in search
        await waitFor(() => {
          expect(screen.getByLabelText('Search languages')).toBeInTheDocument();
        });

        const searchInput = screen.getByLabelText('Search languages');
        await user.type(searchInput, 'Span');

        // Then: Only Spanish languages are visible
        await waitFor(() => {
          expect(screen.getByText('Spanish (Spain)')).toBeInTheDocument();
          expect(screen.getByText('Spanish (Mexico)')).toBeInTheDocument();

          // Other languages should not be visible
          expect(screen.queryByText('French')).not.toBeInTheDocument();
          expect(screen.queryByText('German')).not.toBeInTheDocument();
        });
      });

      it('Given: Search is active, When: User clears search, Then: All languages are visible again', async () => {
        const user = userEvent.setup();

        // Given: Dropdown with search active
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: true });

        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        const searchInput = screen.getByLabelText('Search languages');
        await user.type(searchInput, 'Span');

        // Verify filtering works
        await waitFor(() => {
          expect(screen.queryByText('French')).not.toBeInTheDocument();
        });

        // When: User clears search by clicking clear button
        const clearButton = screen.getByLabelText('Clear search');
        await user.click(clearButton);

        // Then: All languages are visible again
        await waitFor(() => {
          expect(screen.getByText('French')).toBeInTheDocument();
          expect(screen.getByText('German')).toBeInTheDocument();
        });
      });

      it('Given: Search is active, When: User types non-matching text, Then: "No languages found" is shown', async () => {
        const user = userEvent.setup();

        // Given: Dropdown with search active
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: true });

        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        const searchInput = screen.getByLabelText('Search languages');

        // When: User types non-matching text
        await user.type(searchInput, 'xyznonexistent');

        // Then: "No languages found" is shown
        await waitFor(() => {
          expect(screen.getByText('No languages found')).toBeInTheDocument();
        });
      });

      it('Given: Search is active, When: User searches by native name, Then: Language is found', async () => {
        const user = userEvent.setup();

        // Given: Dropdown with search active
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: true });

        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        const searchInput = screen.getByLabelText('Search languages');

        // When: User types native name (Français for French)
        await user.type(searchInput, 'Français');

        // Then: French is found
        await waitFor(() => {
          expect(screen.getByText('French')).toBeInTheDocument();
        });
      });
    });
  });

  // ==========================================================================
  // Feature: Favorites Functionality
  // ==========================================================================

  describe('Feature: Favorites Functionality', () => {
    describe('Scenario: Add language to favorites', () => {
      it('Given: Dropdown is open with favorites enabled, When: User stars a language, Then: Language appears in Favorites section', async () => {
        const user = userEvent.setup();

        // Given: Dropdown with favorites enabled
        renderLanguageSwitcher({ variant: 'dropdown', showFavorites: true, showSearch: false });

        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // Initially no favorites
        await waitFor(() => {
          expect(screen.queryByText('Favorites')).not.toBeInTheDocument();
        });

        // When: User stars German (find the star button next to German)
        const germanRow = screen.getByText('German').closest('[role="option"]');
        expect(germanRow).toBeInTheDocument();

        const starButton = within(germanRow as HTMLElement).getByLabelText('Add to favorites');
        await user.click(starButton);

        // Then: Language appears in Favorites section
        await waitFor(() => {
          expect(screen.getByText('Favorites')).toBeInTheDocument();
        });

        // And: Stored in localStorage
        expect(localStorage.getItem(STORAGE_KEY_FAVORITES)).toContain('de');
      });
    });

    describe('Scenario: Remove language from favorites', () => {
      it('Given: A language is in favorites, When: User un-stars it, Then: Language is removed from Favorites', async () => {
        const user = userEvent.setup();

        // Given: German is already in favorites
        localStorage.setItem(STORAGE_KEY_FAVORITES, JSON.stringify(['de']));

        renderLanguageSwitcher({ variant: 'dropdown', showFavorites: true, showSearch: false });

        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // Verify Favorites section exists
        await waitFor(() => {
          expect(screen.getByText('Favorites')).toBeInTheDocument();
        });

        // When: User clicks remove from favorites
        const removeButton = screen.getByLabelText('Remove from favorites');
        await user.click(removeButton);

        // Then: Favorites section disappears
        await waitFor(() => {
          expect(screen.queryByText('Favorites')).not.toBeInTheDocument();
        });

        // And: localStorage is updated
        const favorites = JSON.parse(localStorage.getItem(STORAGE_KEY_FAVORITES) || '[]');
        expect(favorites).not.toContain('de');
      });
    });
  });

  // ==========================================================================
  // Feature: Keyboard Navigation
  // ==========================================================================

  describe('Feature: Keyboard Navigation', () => {
    describe('Scenario: Close dropdown with Escape key', () => {
      it('Given: Dropdown is open, When: User presses Escape, Then: Dropdown closes', async () => {
        const user = userEvent.setup();

        // Given: Dropdown is open
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // Verify dropdown is open (use getAllByText since English appears in both trigger and dropdown)
        await waitFor(() => {
          expect(screen.getByRole('listbox')).toBeInTheDocument();
        });

        // When: User presses Escape
        await user.keyboard('{Escape}');

        // Then: Dropdown closes (content is no longer visible)
        await waitFor(() => {
          // The dropdown content should be gone or hidden
          expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
        });
      });
    });
  });

  // ==========================================================================
  // Feature: Accessibility
  // ==========================================================================

  describe('Feature: Accessibility', () => {
    describe('Scenario: ARIA attributes are correct', () => {
      it('Given: LanguageSwitcher renders, Then: Trigger has correct ARIA attributes', async () => {
        // Given/When: Component renders
        renderLanguageSwitcher({ variant: 'dropdown' });

        // Then: Trigger has correct ARIA attributes
        const trigger = screen.getByRole('combobox');
        expect(trigger).toHaveAttribute('aria-haspopup', 'true');
        expect(trigger).toHaveAttribute('aria-expanded', 'false');
        expect(trigger).toHaveAttribute('aria-label');
        expect(trigger.getAttribute('aria-label')).toMatch(/current language/i);
      });

      it('Given: Dropdown is open, Then: Options have correct roles and states', async () => {
        const user = userEvent.setup();

        // Given: Dropdown is open
        renderLanguageSwitcher({ variant: 'dropdown', showSearch: false });

        const trigger = screen.getByRole('combobox');
        await user.click(trigger);

        // Then: Dropdown content has correct ARIA
        await waitFor(() => {
          const listbox = screen.getByRole('listbox');
          expect(listbox).toHaveAttribute('aria-label', 'Language selection menu');

          // Options have correct roles
          const options = screen.getAllByRole('option');
          expect(options.length).toBeGreaterThan(0);

          // Current language is marked as selected
          const currentOption = options.find((opt) => opt.getAttribute('aria-selected') === 'true');
          expect(currentOption).toBeInTheDocument();
        });
      });
    });
  });

  // ==========================================================================
  // Feature: Variant Props
  // ==========================================================================

  describe('Feature: Component Variants', () => {
    describe('Scenario: Size prop changes button size', () => {
      it('Given: size="sm" is passed, Then: Small button is rendered', () => {
        // When: size="sm" is passed
        renderLanguageSwitcher({ variant: 'dropdown', size: 'sm' });

        // Then: Button exists (size styling would be applied via className)
        const trigger = screen.getByRole('combobox');
        expect(trigger).toBeInTheDocument();
      });

      it('Given: size="lg" is passed, Then: Large button is rendered', () => {
        // When: size="lg" is passed
        renderLanguageSwitcher({ variant: 'dropdown', size: 'lg' });

        // Then: Button exists
        const trigger = screen.getByRole('combobox');
        expect(trigger).toBeInTheDocument();
      });
    });

    describe('Scenario: Custom className is applied', () => {
      it('Given: className prop is passed, Then: Class is applied to component', () => {
        // When: className is passed
        renderLanguageSwitcher({ variant: 'dropdown', className: 'custom-test-class' });

        // Then: Class is applied
        const trigger = screen.getByRole('combobox');
        expect(trigger).toHaveClass('custom-test-class');
      });
    });
  });

  // ==========================================================================
  // Feature: Integration with i18n
  // ==========================================================================

  describe('Feature: i18n Integration', () => {
    describe('Scenario: Component syncs with i18n state', () => {
      it('Given: i18n language changes externally, When: Component re-renders, Then: Display updates', async () => {
        // Given: Component is rendered showing English
        const { rerender } = renderLanguageSwitcher({ variant: 'dropdown' });

        const trigger = screen.getByRole('combobox');
        expect(trigger).toHaveTextContent(/english/i);

        // When: i18n language changes externally
        await changeTestLanguage('de');

        // Force re-render to pick up the change
        rerender(
          <TestWrapper>
            <LanguageSwitcher variant="dropdown" />
          </TestWrapper>
        );

        // Then: Display updates to German
        await waitFor(() => {
          expect(trigger).toHaveTextContent(/german|deutsch/i);
        });
      });
    });

    describe('Scenario: Initial language from localStorage', () => {
      it('Given: localStorage has saved language, When: Component mounts, Then: Saved language is used', async () => {
        // Given: localStorage has French saved
        localStorage.setItem(STORAGE_KEY_LANGUAGE, 'fr');

        // When: Component mounts
        renderLanguageSwitcher({ variant: 'dropdown' });

        // Then: Component triggers language change to French
        // The i18n instance should be updated
        await waitFor(() => {
          const i18n = getTestI18n();
          expect(i18n.language).toBe('fr');
        });
      });
    });
  });
});
