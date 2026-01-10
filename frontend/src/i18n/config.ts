import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import HttpBackend from 'i18next-http-backend';
import LanguageDetector from 'i18next-browser-languagedetector';

/**
 * Supported language metadata
 */
export interface LanguageInfo {
  /** ISO 639-1 language code */
  code: string;
  /** English name of the language */
  name: string;
  /** Native name of the language */
  nativeName: string;
  /** Text direction: 'ltr' (left-to-right) or 'rtl' (right-to-left) */
  dir: 'ltr' | 'rtl';
  /** Full ISO 639-1 code (e.g., 'en-US', 'pt-BR') */
  isoCode: string;
}

/**
 * All 26 supported languages with metadata
 * Final hybrid strategy: 19 simple codes + 7 regional variants
 * Simple codes: en, fr, de, it, ru, ja, ko, ar, he, hi, nl, pl, sr, th, tr, sv, da, fi, vi, no, cs
 * Regional variants: en-GB, pt-BR, zh-CN, es-ES, es-MX
 */
export const SUPPORTED_LANGUAGES: LanguageInfo[] = [
  { code: 'en', name: 'English (US)', nativeName: 'English (US)', dir: 'ltr', isoCode: 'en-US' },
  { code: 'en-GB', name: 'English (UK)', nativeName: 'English (UK)', dir: 'ltr', isoCode: 'en-GB' },
  { code: 'fr', name: 'French', nativeName: 'Français', dir: 'ltr', isoCode: 'fr-FR' },
  { code: 'de', name: 'German', nativeName: 'Deutsch', dir: 'ltr', isoCode: 'de-DE' },
  { code: 'it', name: 'Italian', nativeName: 'Italiano', dir: 'ltr', isoCode: 'it-IT' },
  { code: 'ru', name: 'Russian', nativeName: 'Русский', dir: 'ltr', isoCode: 'ru-RU' },
  { code: 'ja', name: 'Japanese', nativeName: '日本語', dir: 'ltr', isoCode: 'ja-JP' },
  { code: 'ko', name: 'Korean', nativeName: '한국어', dir: 'ltr', isoCode: 'ko-KR' },
  { code: 'ar', name: 'Arabic', nativeName: 'العربية', dir: 'rtl', isoCode: 'ar-SA' },
  { code: 'he', name: 'Hebrew', nativeName: 'עברית', dir: 'rtl', isoCode: 'he-IL' },
  { code: 'hi', name: 'Hindi', nativeName: 'हिन्दी', dir: 'ltr', isoCode: 'hi-IN' },
  { code: 'nl', name: 'Dutch', nativeName: 'Nederlands', dir: 'ltr', isoCode: 'nl-NL' },
  { code: 'pl', name: 'Polish', nativeName: 'Polski', dir: 'ltr', isoCode: 'pl-PL' },
  { code: 'sr', name: 'Serbian', nativeName: 'Српски', dir: 'ltr', isoCode: 'sr-RS' },
  { code: 'th', name: 'Thai', nativeName: 'ไทย', dir: 'ltr', isoCode: 'th-TH' },
  { code: 'tr', name: 'Turkish', nativeName: 'Türkçe', dir: 'ltr', isoCode: 'tr-TR' },
  { code: 'sv', name: 'Swedish', nativeName: 'Svenska', dir: 'ltr', isoCode: 'sv-SE' },
  { code: 'da', name: 'Danish', nativeName: 'Dansk', dir: 'ltr', isoCode: 'da-DK' },
  { code: 'fi', name: 'Finnish', nativeName: 'Suomi', dir: 'ltr', isoCode: 'fi-FI' },
  { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt', dir: 'ltr', isoCode: 'vi-VN' },
  { code: 'no', name: 'Norwegian', nativeName: 'Norsk', dir: 'ltr', isoCode: 'nb-NO' },
  { code: 'cs', name: 'Czech', nativeName: 'Čeština', dir: 'ltr', isoCode: 'cs-CZ' },
  {
    code: 'pt-BR',
    name: 'Portuguese (Brazil)',
    nativeName: 'Português (Brasil)',
    dir: 'ltr',
    isoCode: 'pt-BR',
  },
  {
    code: 'zh-CN',
    name: 'Chinese (Simplified)',
    nativeName: '简体中文',
    dir: 'ltr',
    isoCode: 'zh-CN',
  },
  {
    code: 'zh-TW',
    name: 'Chinese (Traditional)',
    nativeName: '繁體中文',
    dir: 'ltr',
    isoCode: 'zh-TW',
  },
  {
    code: 'es-ES',
    name: 'Spanish (Spain)',
    nativeName: 'Español (España)',
    dir: 'ltr',
    isoCode: 'es-ES',
  },
  {
    code: 'es-MX',
    name: 'Spanish (Mexico)',
    nativeName: 'Español (México)',
    dir: 'ltr',
    isoCode: 'es-MX',
  },
];

/**
 * Translation namespaces for code splitting
 */
export const NAMESPACES = ['common', 'dashboard', 'settings', 'errors', 'validation'] as const;
export type Namespace = (typeof NAMESPACES)[number];

/**
 * Get language info by code
 */
export function getLanguageInfo(code: string): LanguageInfo | undefined {
  return SUPPORTED_LANGUAGES.find((lang) => lang.code === code);
}

/**
 * Check if a language uses RTL text direction
 */
export function isRTL(languageCode: string): boolean {
  const lang = getLanguageInfo(languageCode);
  return lang?.dir === 'rtl';
}

/**
 * Initialize i18next with lazy loading and language detection
 */
i18n
  .use(HttpBackend) // Load translations via HTTP
  .use(LanguageDetector) // Detect user language
  .use(initReactI18next) // Pass i18n instance to react-i18next
  .init({
    // Default language
    fallbackLng: 'en',

    // Supported languages
    supportedLngs: SUPPORTED_LANGUAGES.map((lang) => lang.code),

    // Default namespace
    defaultNS: 'common',
    ns: NAMESPACES,

    // Lazy loading configuration
    backend: {
      loadPath: '/locales/{{lng}}/{{ns}}.json',
      // Cache translations in memory
      requestOptions: {
        cache: 'default',
      },
    },

    // Language detection configuration
    detection: {
      // Order of detection methods
      order: ['localStorage', 'navigator', 'htmlTag'],
      // Cache user language selection
      caches: ['localStorage'],
      // LocalStorage key
      lookupLocalStorage: 'ampel-i18n-lng',
      // Map detected languages to supported ones
      // e.g., 'en-US' → 'en', 'zh-Hans' → 'zh-CN'
      convertDetectedLanguage: (lng: string) => {
        const mapping: Record<string, string> = {
          'en-US': 'en',
          'zh-Hans': 'zh-CN',
          'zh-Hant': 'zh-TW',
          'pt': 'pt-BR',
          'es': 'es-ES',
          'nb': 'no',
          'nb-NO': 'no',
        };
        return mapping[lng] || lng;
      },
    },

    // Interpolation settings
    interpolation: {
      escapeValue: false, // React already escapes values
    },

    // React specific options
    react: {
      useSuspense: true, // Use React Suspense for loading states
    },

    // Don't load all namespaces at once
    load: 'currentOnly',

    // Performance optimization
    parseMissingKeyHandler: (key) => {
      // Return the key itself as fallback for missing translations
      return key;
    },
  });

export default i18n;
