/**
 * i18n Testing Utilities
 *
 * Provides proper i18n test patterns that verify:
 * 1. Correct translation keys are used (not hardcoded strings)
 * 2. Translation key existence in locale files
 * 3. Translation call tracking and verification
 * 4. Namespace loading validation
 *
 * Key principle: Test that components USE the right keys, not that
 * translations return specific hardcoded strings (which defeats the purpose).
 */

import { vi } from 'vitest';
import type { TFunction, i18n as I18nInstance } from 'i18next';

// ============================================================================
// Types
// ============================================================================

export interface TranslationCall {
  key: string;
  options?: Record<string, unknown>;
  namespace?: string;
  timestamp: number;
}

export interface TranslationSpy {
  t: TFunction;
  calls: TranslationCall[];
  getCallsForKey: (key: string) => TranslationCall[];
  getCallsWithCount: () => TranslationCall[];
  wasKeyCalled: (key: string) => boolean;
  reset: () => void;
}

export interface TranslationKeyResult {
  pass: boolean;
  message: () => string;
}

// ============================================================================
// Custom Vitest Matchers
// ============================================================================

/**
 * Custom matcher to verify translation key usage
 * Usage: expect(spy).toHaveUsedTranslationKey('common.app.title')
 */
export function toHaveUsedTranslationKey(
  received: TranslationSpy,
  expectedKey: string
): TranslationKeyResult {
  const pass = received.wasKeyCalled(expectedKey);

  return {
    pass,
    message: () =>
      pass
        ? `Expected translation key "${expectedKey}" NOT to have been used, but it was`
        : `Expected translation key "${expectedKey}" to have been used.\nKeys used: ${received.calls.map((c) => c.key).join(', ') || '(none)'}`,
  };
}

/**
 * Custom matcher to verify pluralization key usage
 * Usage: expect(spy).toHaveUsedPluralKey('common.pluralization.requests', 5)
 */
export function toHaveUsedPluralKey(
  received: TranslationSpy,
  expectedKey: string,
  expectedCount: number
): TranslationKeyResult {
  const matchingCalls = received.calls.filter(
    (call) =>
      call.key === expectedKey &&
      call.options &&
      (call.options as Record<string, unknown>).count === expectedCount
  );

  const pass = matchingCalls.length > 0;

  return {
    pass,
    message: () =>
      pass
        ? `Expected plural key "${expectedKey}" with count ${expectedCount} NOT to have been used`
        : `Expected plural key "${expectedKey}" with count ${expectedCount} to have been used.\n` +
          `Calls with count: ${JSON.stringify(received.getCallsWithCount())}`,
  };
}

/**
 * Custom matcher to verify all expected keys were used
 * Usage: expect(spy).toHaveUsedAllKeys(['key1', 'key2', 'key3'])
 */
export function toHaveUsedAllKeys(
  received: TranslationSpy,
  expectedKeys: string[]
): TranslationKeyResult {
  const usedKeys = new Set(received.calls.map((c) => c.key));
  const missingKeys = expectedKeys.filter((key) => !usedKeys.has(key));

  const pass = missingKeys.length === 0;

  return {
    pass,
    message: () =>
      pass
        ? `Expected NOT all keys to have been used, but they were`
        : `Expected all keys to have been used.\nMissing keys: ${missingKeys.join(', ')}`,
  };
}

/**
 * Custom matcher to verify namespace usage
 * Usage: expect(spy).toHaveUsedNamespace('dashboard')
 */
export function toHaveUsedNamespace(
  received: TranslationSpy,
  expectedNamespace: string
): TranslationKeyResult {
  const namespaceUsed = received.calls.some(
    (call) => call.key.startsWith(`${expectedNamespace}:`) || call.namespace === expectedNamespace
  );

  const pass = namespaceUsed;

  return {
    pass,
    message: () =>
      pass
        ? `Expected namespace "${expectedNamespace}" NOT to have been used`
        : `Expected namespace "${expectedNamespace}" to have been used.\n` +
          `Keys used: ${received.calls.map((c) => c.key).join(', ')}`,
  };
}

// ============================================================================
// Translation Spy Factory
// ============================================================================

/**
 * Creates a translation spy that tracks all t() calls
 * while returning the translation key for verification.
 *
 * This pattern allows tests to verify:
 * - Correct keys are used
 * - Correct count values for pluralization
 * - Correct interpolation options are passed
 *
 * Example:
 * ```typescript
 * const spy = createTranslationSpy();
 * render(<MyComponent />, { wrapper: createI18nWrapper(spy.t) });
 * expect(spy).toHaveUsedTranslationKey('common.app.title');
 * ```
 */
export function createTranslationSpy(): TranslationSpy {
  const calls: TranslationCall[] = [];

  const t = vi.fn((key: string, options?: Record<string, unknown>) => {
    // Extract namespace from key if present (e.g., 'common:app.title')
    const colonIndex = key.indexOf(':');
    const namespace = colonIndex > 0 ? key.substring(0, colonIndex) : undefined;
    const _actualKey = colonIndex > 0 ? key.substring(colonIndex + 1) : key;

    calls.push({
      key,
      options,
      namespace,
      timestamp: Date.now(),
    });

    // Return a string that includes key and count for easy debugging
    if (options?.count !== undefined) {
      return `[${key}|count:${options.count}]`;
    }

    // Handle interpolation for testing
    if (options && typeof options === 'object') {
      const interpolated = Object.entries(options)
        .filter(([k]) => k !== 'defaultValue' && k !== 'count')
        .map(([k, v]) => `${k}:${v}`)
        .join(',');
      if (interpolated) {
        return `[${key}|${interpolated}]`;
      }
    }

    return `[${key}]`;
  }) as TFunction;

  return {
    t,
    calls,
    getCallsForKey: (key: string) => calls.filter((c) => c.key === key),
    getCallsWithCount: () => calls.filter((c) => c.options?.count !== undefined),
    wasKeyCalled: (key: string) => calls.some((c) => c.key === key),
    reset: () => {
      calls.length = 0;
      vi.clearAllMocks();
    },
  };
}

// ============================================================================
// Translation Key Verification
// ============================================================================

/**
 * Verifies that a translation key exists in the provided translation resources
 *
 * @param key - The translation key (e.g., 'common.app.title' or 'common:app.title')
 * @param resources - The translation resource object
 * @param language - The language to check (default: 'en')
 */
export function verifyKeyExists(
  key: string,
  resources: Record<string, Record<string, unknown>>,
  language = 'en'
): boolean {
  // Handle namespaced keys (e.g., 'common:app.title')
  let namespace = 'common';
  let keyPath = key;

  const colonIndex = key.indexOf(':');
  if (colonIndex > 0) {
    namespace = key.substring(0, colonIndex);
    keyPath = key.substring(colonIndex + 1);
  }

  // Get the namespace translations
  const nsTranslations = resources[language]?.[namespace];
  if (!nsTranslations || typeof nsTranslations !== 'object') {
    return false;
  }

  // Navigate the key path
  const parts = keyPath.split('.');
  let current: unknown = nsTranslations;

  for (const part of parts) {
    if (current === null || typeof current !== 'object') {
      return false;
    }
    current = (current as Record<string, unknown>)[part];
  }

  return current !== undefined;
}

/**
 * Collects all translation keys used by a component
 *
 * @param renderFn - Function that renders the component
 * @returns Array of translation keys used
 */
export async function collectUsedKeys(
  renderFn: (spy: TranslationSpy) => void | Promise<void>
): Promise<string[]> {
  const spy = createTranslationSpy();
  await renderFn(spy);
  return [...new Set(spy.calls.map((c) => c.key))];
}

// ============================================================================
// Mock useTranslation Hook Factory
// ============================================================================

/**
 * Creates a mock useTranslation hook that uses the translation spy
 *
 * Example:
 * ```typescript
 * const spy = createTranslationSpy();
 * vi.mock('react-i18next', () => ({
 *   useTranslation: createMockUseTranslationWithSpy(spy),
 * }));
 * ```
 */
export function createMockUseTranslationWithSpy(spy: TranslationSpy) {
  return vi.fn((_ns?: string | string[]) => ({
    t: spy.t,
    i18n: {
      language: 'en',
      changeLanguage: vi.fn(),
      isInitialized: true,
      options: {
        supportedLngs: ['en', 'ar', 'he', 'fr', 'de', 'ru', 'pl', 'cs'],
      },
    },
    ready: true,
  }));
}

// ============================================================================
// Test Utilities for Key Validation
// ============================================================================

/**
 * Creates a test case that verifies a component uses expected translation keys
 *
 * @param componentName - Name of the component for test description
 * @param expectedKeys - Array of expected translation keys
 * @param renderFn - Function that renders the component with the spy
 */
export function createKeyVerificationTest(
  componentName: string,
  expectedKeys: string[],
  renderFn: (spy: TranslationSpy) => void | Promise<void>
) {
  return async () => {
    const spy = createTranslationSpy();
    await renderFn(spy);

    const usedKeys = new Set(spy.calls.map((c) => c.key));
    const missingKeys = expectedKeys.filter((key) => !usedKeys.has(key));

    if (missingKeys.length > 0) {
      throw new Error(
        `${componentName} did not use expected keys:\n` +
          `Missing: ${missingKeys.join(', ')}\n` +
          `Used: ${[...usedKeys].join(', ')}`
      );
    }
  };
}

// ============================================================================
// Pluralization Testing Utilities
// ============================================================================

/**
 * Test all plural forms for a given key pattern
 *
 * @param i18n - The i18n instance
 * @param baseKey - The base pluralization key (e.g., 'common.pluralization.requests')
 * @param testCounts - Array of counts to test
 * @returns Object with results for each count
 */
export function testPluralForms(
  i18n: I18nInstance,
  baseKey: string,
  testCounts: number[]
): Record<number, string> {
  const results: Record<number, string> = {};

  for (const count of testCounts) {
    results[count] = i18n.t(baseKey, { count });
  }

  return results;
}

/**
 * Verifies that plural forms are distinct (not all the same)
 */
export function verifyDistinctPluralForms(
  forms: Record<number, string>,
  expectedDistinctCount: number
): { pass: boolean; distinctCount: number; forms: Record<number, string> } {
  const uniqueForms = new Set(Object.values(forms));
  const distinctCount = uniqueForms.size;

  return {
    pass: distinctCount >= expectedDistinctCount,
    distinctCount,
    forms,
  };
}

// ============================================================================
// Namespace Loading Verification
// ============================================================================

/**
 * Verifies that required namespaces are loaded
 */
export function verifyNamespacesLoaded(
  i18n: I18nInstance,
  language: string,
  namespaces: string[]
): { loaded: string[]; missing: string[] } {
  const loaded: string[] = [];
  const missing: string[] = [];

  for (const ns of namespaces) {
    if (i18n.hasResourceBundle(language, ns)) {
      loaded.push(ns);
    } else {
      missing.push(ns);
    }
  }

  return { loaded, missing };
}

// ============================================================================
// Extend Vitest Matchers
// ============================================================================

// Declare custom matchers for TypeScript
declare module 'vitest' {
  interface Assertion<T = unknown> {
    toHaveUsedTranslationKey(expectedKey: string): T;
    toHaveUsedPluralKey(expectedKey: string, expectedCount: number): T;
    toHaveUsedAllKeys(expectedKeys: string[]): T;
    toHaveUsedNamespace(expectedNamespace: string): T;
  }
  interface AsymmetricMatchersContaining {
    toHaveUsedTranslationKey(expectedKey: string): unknown;
    toHaveUsedPluralKey(expectedKey: string, expectedCount: number): unknown;
    toHaveUsedAllKeys(expectedKeys: string[]): unknown;
    toHaveUsedNamespace(expectedNamespace: string): unknown;
  }
}

/**
 * Register custom matchers with Vitest
 * Call this in your test setup file
 */
export function registerI18nMatchers() {
  // Note: expect.extend is called at module load time in setup files
  // This function provides a way to explicitly register if needed
}

export const i18nMatchers = {
  toHaveUsedTranslationKey,
  toHaveUsedPluralKey,
  toHaveUsedAllKeys,
  toHaveUsedNamespace,
};
