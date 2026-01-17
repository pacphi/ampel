/**
 * Real i18n Instance for Testing
 *
 * Instead of mocking useTranslation with hardcoded strings (which defeats
 * the purpose of i18n testing), this module provides a REAL i18n instance
 * configured for testing with:
 *
 * 1. In-memory resource loading (no HTTP)
 * 2. Missing key detection that throws errors
 * 3. Support for all languages including RTL
 * 4. Proper pluralization rules
 *
 * Usage:
 * ```typescript
 * import { setupTestI18n, getTestI18n } from '@/tests/setup/i18n-instance';
 *
 * beforeAll(async () => {
 *   await setupTestI18n('en');
 * });
 *
 * it('should use correct translation', () => {
 *   const i18n = getTestI18n();
 *   const result = i18n.t('common:app.title');
 *   expect(result).toBe('Ampel PR Dashboard');
 * });
 * ```
 */

import i18n, { i18n as I18nInstance } from 'i18next';
import { initReactI18next } from 'react-i18next';

// Import translation files directly for in-memory testing
// These are imported as JSON modules for vitest
import enCommon from '../../public/locales/en/common.json';
import enDashboard from '../../public/locales/en/dashboard.json';
import enSettings from '../../public/locales/en/settings.json';
import enErrors from '../../public/locales/en/errors.json';
import enValidation from '../../public/locales/en/validation.json';

import arCommon from '../../public/locales/ar/common.json';
import arDashboard from '../../public/locales/ar/dashboard.json';
import arSettings from '../../public/locales/ar/settings.json';
import arErrors from '../../public/locales/ar/errors.json';
import arValidation from '../../public/locales/ar/validation.json';

import heCommon from '../../public/locales/he/common.json';
import heDashboard from '../../public/locales/he/dashboard.json';
import heSettings from '../../public/locales/he/settings.json';
import heErrors from '../../public/locales/he/errors.json';
import heValidation from '../../public/locales/he/validation.json';

import ruCommon from '../../public/locales/ru/common.json';
import ruDashboard from '../../public/locales/ru/dashboard.json';
import ruSettings from '../../public/locales/ru/settings.json';
import ruErrors from '../../public/locales/ru/errors.json';
import ruValidation from '../../public/locales/ru/validation.json';

import plCommon from '../../public/locales/pl/common.json';
import plDashboard from '../../public/locales/pl/dashboard.json';
import plSettings from '../../public/locales/pl/settings.json';
import plErrors from '../../public/locales/pl/errors.json';
import plValidation from '../../public/locales/pl/validation.json';

import csCommon from '../../public/locales/cs/common.json';
import csDashboard from '../../public/locales/cs/dashboard.json';
import csSettings from '../../public/locales/cs/settings.json';
import csErrors from '../../public/locales/cs/errors.json';
import csValidation from '../../public/locales/cs/validation.json';

import fiCommon from '../../public/locales/fi/common.json';
import fiDashboard from '../../public/locales/fi/dashboard.json';
import fiSettings from '../../public/locales/fi/settings.json';
import fiErrors from '../../public/locales/fi/errors.json';
import fiValidation from '../../public/locales/fi/validation.json';

import frCommon from '../../public/locales/fr/common.json';
import frDashboard from '../../public/locales/fr/dashboard.json';
import frSettings from '../../public/locales/fr/settings.json';
import frErrors from '../../public/locales/fr/errors.json';
import frValidation from '../../public/locales/fr/validation.json';

import deCommon from '../../public/locales/de/common.json';
import deDashboard from '../../public/locales/de/dashboard.json';
import deSettings from '../../public/locales/de/settings.json';
import deErrors from '../../public/locales/de/errors.json';
import deValidation from '../../public/locales/de/validation.json';

// ============================================================================
// Translation Resources
// ============================================================================

/**
 * All translation resources bundled for testing
 * This avoids HTTP requests and ensures consistent test data
 */
export const testResources = {
  en: {
    common: enCommon,
    dashboard: enDashboard,
    settings: enSettings,
    errors: enErrors,
    validation: enValidation,
  },
  ar: {
    common: arCommon,
    dashboard: arDashboard,
    settings: arSettings,
    errors: arErrors,
    validation: arValidation,
  },
  he: {
    common: heCommon,
    dashboard: heDashboard,
    settings: heSettings,
    errors: heErrors,
    validation: heValidation,
  },
  ru: {
    common: ruCommon,
    dashboard: ruDashboard,
    settings: ruSettings,
    errors: ruErrors,
    validation: ruValidation,
  },
  pl: {
    common: plCommon,
    dashboard: plDashboard,
    settings: plSettings,
    errors: plErrors,
    validation: plValidation,
  },
  cs: {
    common: csCommon,
    dashboard: csDashboard,
    settings: csSettings,
    errors: csErrors,
    validation: csValidation,
  },
  fi: {
    common: fiCommon,
    dashboard: fiDashboard,
    settings: fiSettings,
    errors: fiErrors,
    validation: fiValidation,
  },
  fr: {
    common: frCommon,
    dashboard: frDashboard,
    settings: frSettings,
    errors: frErrors,
    validation: frValidation,
  },
  de: {
    common: deCommon,
    dashboard: deDashboard,
    settings: deSettings,
    errors: deErrors,
    validation: deValidation,
  },
};

// ============================================================================
// Language Metadata
// ============================================================================

export const RTL_LANGUAGES = ['ar', 'he'] as const;
export const LTR_LANGUAGES = ['en', 'fr', 'de', 'ru', 'pl', 'cs', 'fi'] as const;

export function isRTLLanguage(lang: string): boolean {
  return RTL_LANGUAGES.includes(lang as (typeof RTL_LANGUAGES)[number]);
}

// ============================================================================
// Test i18n Instance Management
// ============================================================================

let testI18nInstance: I18nInstance | null = null;
let missingKeys: string[] = [];

/**
 * Get the list of missing keys encountered during testing
 */
export function getMissingKeys(): string[] {
  return [...missingKeys];
}

/**
 * Clear the list of missing keys
 */
export function clearMissingKeys(): void {
  missingKeys = [];
}

/**
 * Setup i18n for testing with real translations loaded in memory
 *
 * @param language - Initial language (default: 'en')
 * @param options - Additional configuration options
 * @returns Configured i18n instance
 *
 * Features:
 * - In-memory resources (no HTTP)
 * - Missing key detection
 * - Proper pluralization
 * - RTL support
 */
export async function setupTestI18n(
  language = 'en',
  options?: {
    /** Throw error on missing keys (default: false, just collects them) */
    throwOnMissingKey?: boolean;
    /** Additional resources to merge */
    additionalResources?: Record<string, Record<string, unknown>>;
  }
): Promise<I18nInstance> {
  // Reset missing keys
  missingKeys = [];

  // Merge additional resources if provided
  const resources = options?.additionalResources
    ? { ...testResources, ...options.additionalResources }
    : testResources;

  // Create or reset instance
  if (testI18nInstance) {
    // Reset existing instance
    await testI18nInstance.changeLanguage(language);
    return testI18nInstance;
  }

  // Initialize new instance
  await i18n.use(initReactI18next).init({
    lng: language,
    fallbackLng: 'en',
    supportedLngs: Object.keys(resources),

    // Use in-memory resources
    resources,

    // Namespace configuration
    defaultNS: 'common',
    ns: ['common', 'dashboard', 'settings', 'errors', 'validation'],

    // Interpolation settings
    interpolation: {
      escapeValue: false,
    },

    // React specific options
    react: {
      useSuspense: false, // Disable suspense for testing
    },

    // Enable i18next v4 plural format
    compatibilityJSON: 'v4',

    // Don't use key as fallback - makes it easy to detect missing keys
    returnEmptyString: false,
    returnNull: false,

    // Track missing keys
    saveMissing: true,
    missingKeyHandler: (lngs, ns, key, fallbackValue) => {
      const fullKey = `${ns}:${key}`;
      missingKeys.push(fullKey);

      if (options?.throwOnMissingKey) {
        throw new Error(
          `Missing translation key: ${fullKey}\n` +
            `Languages: ${lngs.join(', ')}\n` +
            `Fallback: ${fallbackValue}`
        );
      }
    },

    // Performance optimization for tests
    initImmediate: true,

    // Debug mode (can be enabled for troubleshooting)
    debug: false,
  });

  testI18nInstance = i18n;
  return i18n;
}

/**
 * Get the current test i18n instance
 * Throws if setupTestI18n hasn't been called
 */
export function getTestI18n(): I18nInstance {
  if (!testI18nInstance) {
    throw new Error(
      'Test i18n instance not initialized. Call setupTestI18n() in beforeAll/beforeEach.'
    );
  }
  return testI18nInstance;
}

/**
 * Reset the test i18n instance (useful for isolated tests)
 */
export async function resetTestI18n(): Promise<void> {
  if (testI18nInstance) {
    await testI18nInstance.changeLanguage('en');
    clearMissingKeys();
  }
}

/**
 * Change language and verify it was set correctly
 */
export async function changeTestLanguage(language: string): Promise<void> {
  const i18n = getTestI18n();
  await i18n.changeLanguage(language);

  if (i18n.language !== language) {
    throw new Error(`Failed to change language to ${language}. Current: ${i18n.language}`);
  }
}

// ============================================================================
// Translation Verification Utilities
// ============================================================================

/**
 * Verify that a key exists in the translation resources
 */
export function verifyKeyExists(key: string, language = 'en', namespace = 'common'): boolean {
  const i18n = getTestI18n();
  const exists = i18n.exists(key, { lng: language, ns: namespace });
  return exists;
}

/**
 * Get all keys from a namespace for a language
 */
export function getNamespaceKeys(namespace: string, language = 'en'): string[] {
  const i18n = getTestI18n();
  const bundle = i18n.getResourceBundle(language, namespace);

  if (!bundle) return [];

  const keys: string[] = [];

  function collectKeys(obj: Record<string, unknown>, prefix = ''): void {
    for (const [key, value] of Object.entries(obj)) {
      const fullKey = prefix ? `${prefix}.${key}` : key;
      if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
        collectKeys(value as Record<string, unknown>, fullKey);
      } else {
        keys.push(fullKey);
      }
    }
  }

  collectKeys(bundle);
  return keys;
}

/**
 * Compare keys between two languages to find missing translations
 */
export function findMissingTranslations(
  sourceLanguage: string,
  targetLanguage: string,
  namespace = 'common'
): { missing: string[]; extra: string[] } {
  const sourceKeys = new Set(getNamespaceKeys(namespace, sourceLanguage));
  const targetKeys = new Set(getNamespaceKeys(namespace, targetLanguage));

  const missing = [...sourceKeys].filter((key) => !targetKeys.has(key));
  const extra = [...targetKeys].filter((key) => !sourceKeys.has(key));

  return { missing, extra };
}

// ============================================================================
// Test Wrapper Components
// ============================================================================

import type { ReactNode } from 'react';
import { I18nextProvider } from 'react-i18next';

/**
 * Props for I18nTestProvider
 */
export interface I18nTestProviderProps {
  children: ReactNode;
  language?: string;
}

/**
 * React context provider for tests using real i18n
 *
 * Usage:
 * ```tsx
 * render(
 *   <I18nTestProvider language="ar">
 *     <MyComponent />
 *   </I18nTestProvider>
 * );
 * ```
 */
export function I18nTestProvider({ children, language = 'en' }: I18nTestProviderProps) {
  const i18n = getTestI18n();

  // Change language if different from current
  if (i18n.language !== language) {
    i18n.changeLanguage(language);
  }

  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>;
}

/**
 * Create a render wrapper with i18n support for testing-library
 *
 * Usage:
 * ```tsx
 * const { wrapper } = createI18nTestWrapper('ar');
 * render(<MyComponent />, { wrapper });
 * ```
 */
export function createI18nTestWrapper(language = 'en') {
  return {
    wrapper: ({ children }: { children: ReactNode }) => (
      <I18nTestProvider language={language}>{children}</I18nTestProvider>
    ),
  };
}

// Export default i18n for backward compatibility
export default i18n;
