/**
 * i18n Test Configuration
 *
 * Configures i18next for synchronous operation in tests.
 * Loads translations directly without HTTP backend for instant availability.
 */

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

// Import translation files directly for synchronous loading
// These paths are relative to the frontend directory
import commonEn from '../../public/locales/en/common.json';
import dashboardEn from '../../public/locales/en/dashboard.json';
import settingsEn from '../../public/locales/en/settings.json';
import errorsEn from '../../public/locales/en/errors.json';
import validationEn from '../../public/locales/en/validation.json';

/**
 * Test i18n instance with synchronous loading.
 * All translations are bundled directly for instant availability.
 */
const testI18n = i18n.createInstance();

testI18n.use(initReactI18next).init({
  // Disable language detection in tests
  lng: 'en',
  fallbackLng: 'en',

  // Load translations synchronously
  resources: {
    en: {
      common: commonEn,
      dashboard: dashboardEn,
      settings: settingsEn,
      errors: errorsEn,
      validation: validationEn,
    },
  },

  // Default namespace
  defaultNS: 'common',
  ns: ['common', 'dashboard', 'settings', 'errors', 'validation'],

  // Interpolation settings
  interpolation: {
    escapeValue: false,
  },

  // React specific options - disable suspense for synchronous operation
  react: {
    useSuspense: false,
  },

  // Return key as fallback for missing translations (helps identify untranslated keys)
  returnEmptyString: false,
  parseMissingKeyHandler: (key) => `[MISSING: ${key}]`,

  // Debug mode for tests (can be enabled for troubleshooting)
  debug: false,
});

export default testI18n;

/**
 * Change language in tests
 */
export const changeTestLanguage = async (language: string): Promise<void> => {
  await testI18n.changeLanguage(language);
};

/**
 * Get current test language
 */
export const getTestLanguage = (): string => {
  return testI18n.language;
};

/**
 * Check if a translation key exists
 */
export const hasTranslation = (key: string, ns?: string): boolean => {
  return testI18n.exists(key, { ns });
};

/**
 * Get translation for a key (for assertions)
 */
export const getTranslation = (key: string, options?: Record<string, unknown>): string => {
  return testI18n.t(key, options);
};
