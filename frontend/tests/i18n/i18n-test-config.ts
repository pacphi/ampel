/**
 * i18next configuration for testing
 *
 * This configuration uses in-memory resources instead of HTTP backend
 * to avoid timeouts and ensure fast test execution.
 */

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

// Import translation resources for testing
import enCommon from '../../public/locales/en/common.json';
import fiCommon from '../../public/locales/fi/common.json';
import csCommon from '../../public/locales/cs/common.json';
import ruCommon from '../../public/locales/ru/common.json';
import plCommon from '../../public/locales/pl/common.json';
import arCommon from '../../public/locales/ar/common.json';

// Initialize i18next for testing with in-memory resources
export function initI18nForTesting() {
  return i18n.use(initReactI18next).init({
    lng: 'en',
    fallbackLng: 'en',

    // Use in-memory resources instead of HTTP backend
    resources: {
      en: {
        common: enCommon,
      },
      fi: {
        common: fiCommon,
      },
      cs: {
        common: csCommon,
      },
      ru: {
        common: ruCommon,
      },
      pl: {
        common: plCommon,
      },
      ar: {
        common: arCommon,
      },
    },

    // Namespace configuration
    defaultNS: 'common',
    ns: ['common'],

    // Interpolation settings
    interpolation: {
      escapeValue: false,
    },

    // React specific options
    react: {
      useSuspense: false, // Disable suspense for testing
    },

    // Enable pluralization
    compatibilityJSON: 'v4', // Use i18next v21+ plural suffix format

    // Don't use key as fallback
    returnEmptyString: false,
    returnNull: false,

    // Performance optimization for tests
    initImmediate: false,

    // Debug mode for development
    debug: false,
  });
}

export default i18n;
