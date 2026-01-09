/**
 * i18n Testing Utilities
 *
 * Provides utilities for testing internationalized components.
 * Supports key-based testing approach where tests verify translation keys
 * are used correctly rather than hardcoding translated text.
 *
 * @example
 * ```typescript
 * import { expectTranslationKey, getByTranslationKey } from '@/tests/setup/i18n-test-utils';
 *
 * // Verify a translation key is rendered
 * expectTranslationKey('dashboard:title').toBeRendered();
 *
 * // Query by translation key
 * const element = getByTranslationKey('common:auth.login');
 * expect(element).toBeInTheDocument();
 *
 * // Verify translation contains interpolated values
 * expectTranslationKey('common:time.minutesAgo', { count: 5 }).toContain('5');
 * ```
 */

import { screen, within } from '@testing-library/react';
import testI18n, { hasTranslation, getTranslation, changeTestLanguage } from './i18n-test-config';

// Re-export i18n utilities
export { testI18n, hasTranslation, getTranslation, changeTestLanguage };

// ============================================================================
// Translation Key Types
// ============================================================================

/**
 * Namespace prefixed key format (e.g., 'common:auth.login')
 */
export type TranslationKey = `${string}:${string}` | string;

/**
 * Options for translation key assertions
 */
export interface TranslationKeyOptions {
  /** Interpolation values for the translation */
  values?: Record<string, unknown>;
  /** Container to search within */
  container?: HTMLElement;
  /** Whether to use regex matching (default: false) */
  partial?: boolean;
}

// ============================================================================
// Query Functions
// ============================================================================

/**
 * Get element by its translation key.
 * Translates the key and finds the element containing that text.
 *
 * @param key - Translation key (e.g., 'common:auth.login' or 'dashboard:title')
 * @param options - Query options
 * @returns HTMLElement if found
 *
 * @example
 * ```typescript
 * const loginButton = getByTranslationKey('common:auth.login');
 * const dashboardTitle = getByTranslationKey('dashboard:title');
 * ```
 */
export function getByTranslationKey(
  key: TranslationKey,
  options: TranslationKeyOptions = {}
): HTMLElement {
  const { values, container, partial = false } = options;
  const translatedText = getTranslation(key, values);
  const searchContext = container ? within(container) : screen;

  if (partial) {
    return searchContext.getByText(new RegExp(translatedText, 'i'));
  }

  return searchContext.getByText(translatedText);
}

/**
 * Query element by translation key (returns null if not found).
 */
export function queryByTranslationKey(
  key: TranslationKey,
  options: TranslationKeyOptions = {}
): HTMLElement | null {
  const { values, container, partial = false } = options;
  const translatedText = getTranslation(key, values);
  const searchContext = container ? within(container) : screen;

  if (partial) {
    return searchContext.queryByText(new RegExp(translatedText, 'i'));
  }

  return searchContext.queryByText(translatedText);
}

/**
 * Find element by translation key (async, waits for element).
 */
export async function findByTranslationKey(
  key: TranslationKey,
  options: TranslationKeyOptions = {}
): Promise<HTMLElement> {
  const { values, container, partial = false } = options;
  const translatedText = getTranslation(key, values);
  const searchContext = container ? within(container) : screen;

  if (partial) {
    return searchContext.findByText(new RegExp(translatedText, 'i'));
  }

  return searchContext.findByText(translatedText);
}

/**
 * Get all elements by translation key.
 */
export function getAllByTranslationKey(
  key: TranslationKey,
  options: TranslationKeyOptions = {}
): HTMLElement[] {
  const { values, container, partial = false } = options;
  const translatedText = getTranslation(key, values);
  const searchContext = container ? within(container) : screen;

  if (partial) {
    return searchContext.getAllByText(new RegExp(translatedText, 'i'));
  }

  return searchContext.getAllByText(translatedText);
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/**
 * Assertion builder for translation keys.
 * Provides a fluent API for testing translation key usage.
 *
 * @example
 * ```typescript
 * // Assert key is rendered
 * expectTranslationKey('dashboard:title').toBeRendered();
 *
 * // Assert key is NOT rendered
 * expectTranslationKey('errors:notFound').notToBeRendered();
 *
 * // Assert with interpolation
 * expectTranslationKey('common:time.minutesAgo', { count: 5 }).toBeRendered();
 *
 * // Assert translated text contains substring
 * expectTranslationKey('dashboard:stats.total').toContain('Total');
 * ```
 */
export function expectTranslationKey(key: TranslationKey, values?: Record<string, unknown>) {
  const translatedText = getTranslation(key, values);

  return {
    /**
     * Assert that the translation key is rendered in the document
     */
    toBeRendered(options?: { container?: HTMLElement; partial?: boolean }) {
      const element = queryByTranslationKey(key, { values, ...options });
      expect(element).toBeInTheDocument();
      return element;
    },

    /**
     * Assert that the translation key is NOT rendered
     */
    notToBeRendered(options?: { container?: HTMLElement; partial?: boolean }) {
      const element = queryByTranslationKey(key, { values, ...options });
      expect(element).not.toBeInTheDocument();
    },

    /**
     * Assert the translated text contains a substring
     */
    toContain(substring: string) {
      expect(translatedText).toContain(substring);
    },

    /**
     * Assert the translated text matches a pattern
     */
    toMatch(pattern: RegExp) {
      expect(translatedText).toMatch(pattern);
    },

    /**
     * Assert the translated text equals exactly
     */
    toEqual(expected: string) {
      expect(translatedText).toBe(expected);
    },

    /**
     * Get the translated text for further assertions
     */
    getTranslatedText() {
      return translatedText;
    },
  };
}

// ============================================================================
// Translation Coverage Helpers
// ============================================================================

/**
 * Verify all keys in a namespace are present
 */
export function verifyNamespaceKeys(namespace: string, expectedKeys: string[]): void {
  const missingKeys: string[] = [];
  const _extraKeys: string[] = [];

  expectedKeys.forEach((key) => {
    const fullKey = `${namespace}:${key}`;
    if (!hasTranslation(fullKey)) {
      missingKeys.push(key);
    }
  });

  if (missingKeys.length > 0) {
    throw new Error(
      `Missing translation keys in namespace '${namespace}': ${missingKeys.join(', ')}`
    );
  }
}

/**
 * Get all translation keys used in a component (for coverage tracking).
 * This requires instrumenting the component with a custom hook.
 */
export class TranslationTracker {
  private usedKeys: Set<string> = new Set();

  /**
   * Track a translation key usage
   */
  track(key: string): void {
    this.usedKeys.add(key);
  }

  /**
   * Get all tracked keys
   */
  getUsedKeys(): string[] {
    return Array.from(this.usedKeys);
  }

  /**
   * Clear tracked keys
   */
  clear(): void {
    this.usedKeys.clear();
  }

  /**
   * Check if a key was used
   */
  wasUsed(key: string): boolean {
    return this.usedKeys.has(key);
  }
}

// Global tracker instance
export const translationTracker = new TranslationTracker();

// ============================================================================
// RTL Testing Helpers
// ============================================================================

/**
 * Test RTL (Right-to-Left) layout support
 */
export async function testRTLLayout(callback: () => Promise<void>): Promise<void> {
  const originalLang = testI18n.language;

  // Switch to RTL language
  await changeTestLanguage('ar');

  try {
    await callback();
  } finally {
    // Restore original language
    await changeTestLanguage(originalLang);
  }
}

/**
 * Assert element has RTL direction
 */
export function expectRTLDirection(element: HTMLElement): void {
  const computedStyle = getComputedStyle(element);
  expect(computedStyle.direction).toBe('rtl');
}

/**
 * Assert element has LTR direction
 */
export function expectLTRDirection(element: HTMLElement): void {
  const computedStyle = getComputedStyle(element);
  expect(computedStyle.direction).toBe('ltr');
}

// ============================================================================
// Pluralization Testing
// ============================================================================

/**
 * Test pluralization for a key with different counts
 */
export function testPluralization(
  key: TranslationKey,
  testCases: { count: number; expected: string | RegExp }[]
): void {
  testCases.forEach(({ count, expected }) => {
    const translated = getTranslation(key, { count });

    if (typeof expected === 'string') {
      expect(translated).toBe(expected);
    } else {
      expect(translated).toMatch(expected);
    }
  });
}

/**
 * Common pluralization test cases for count-based translations
 */
export const pluralizationTestCases = {
  /** Standard 0, 1, many pattern */
  standard: [
    { count: 0, expected: /0/ },
    { count: 1, expected: /1/ },
    { count: 2, expected: /2/ },
    { count: 5, expected: /5/ },
  ],
};
