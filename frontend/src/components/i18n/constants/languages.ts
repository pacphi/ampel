/**
 * Language metadata and constants for LanguageSwitcher component
 */

export interface Language {
  code: string;
  name: string;
  nativeName: string;
  dir?: 'ltr' | 'rtl';
  isRTL?: boolean;
}

/**
 * All 27 supported languages matching final hybrid strategy
 * Simple codes (21): en, fr, de, it, ru, ja, ko, ar, he, hi, nl, pl, sr, th, tr, sv, da, fi, vi, no, cs
 * Regional variants (6): en-GB, pt-BR, zh-CN, zh-TW, es-ES, es-MX
 */
export const SUPPORTED_LANGUAGES: Language[] = [
  { code: 'en', name: 'English (US)', nativeName: 'English (US)', dir: 'ltr' },
  { code: 'en-GB', name: 'English (UK)', nativeName: 'English (UK)', dir: 'ltr' },
  { code: 'fr', name: 'French', nativeName: 'Français', dir: 'ltr' },
  { code: 'de', name: 'German', nativeName: 'Deutsch', dir: 'ltr' },
  { code: 'it', name: 'Italian', nativeName: 'Italiano', dir: 'ltr' },
  { code: 'ru', name: 'Russian', nativeName: 'Русский', dir: 'ltr' },
  { code: 'ja', name: 'Japanese', nativeName: '日本語', dir: 'ltr' },
  { code: 'ko', name: 'Korean', nativeName: '한국어', dir: 'ltr' },
  { code: 'ar', name: 'Arabic', nativeName: 'العربية', dir: 'rtl', isRTL: true },
  { code: 'he', name: 'Hebrew', nativeName: 'עברית', dir: 'rtl', isRTL: true },
  { code: 'hi', name: 'Hindi', nativeName: 'हिन्दी', dir: 'ltr' },
  { code: 'nl', name: 'Dutch', nativeName: 'Nederlands', dir: 'ltr' },
  { code: 'pl', name: 'Polish', nativeName: 'Polski', dir: 'ltr' },
  { code: 'sr', name: 'Serbian', nativeName: 'Српски', dir: 'ltr' },
  { code: 'th', name: 'Thai', nativeName: 'ไทย', dir: 'ltr' },
  { code: 'tr', name: 'Turkish', nativeName: 'Türkçe', dir: 'ltr' },
  { code: 'sv', name: 'Swedish', nativeName: 'Svenska', dir: 'ltr' },
  { code: 'da', name: 'Danish', nativeName: 'Dansk', dir: 'ltr' },
  { code: 'fi', name: 'Finnish', nativeName: 'Suomi', dir: 'ltr' },
  { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt', dir: 'ltr' },
  { code: 'no', name: 'Norwegian', nativeName: 'Norsk', dir: 'ltr' },
  { code: 'cs', name: 'Czech', nativeName: 'Čeština', dir: 'ltr' },
  { code: 'pt-BR', name: 'Portuguese (Brazil)', nativeName: 'Português (Brasil)', dir: 'ltr' },
  { code: 'zh-CN', name: 'Chinese (Simplified)', nativeName: '简体中文', dir: 'ltr' },
  { code: 'zh-TW', name: 'Chinese (Traditional)', nativeName: '繁體中文', dir: 'ltr' },
  { code: 'es-ES', name: 'Spanish (Spain)', nativeName: 'Español (España)', dir: 'ltr' },
  { code: 'es-MX', name: 'Spanish (Mexico)', nativeName: 'Español (México)', dir: 'ltr' },
];

// Group languages for better UX
export const COMMON_LANGUAGES = ['en', 'en-GB', 'es-ES', 'fr', 'de', 'pt-BR'];
export const RTL_LANGUAGES = ['ar', 'he'];

export const groupLanguages = (languages: Language[], favorites: string[] = []) => {
  const favoriteList = languages.filter((lang) => favorites.includes(lang.code));
  const commonList = languages.filter(
    (lang) => COMMON_LANGUAGES.includes(lang.code) && !favorites.includes(lang.code)
  );
  const rtlList = languages.filter(
    (lang) => RTL_LANGUAGES.includes(lang.code) && !favorites.includes(lang.code)
  );
  const otherList = languages
    .filter(
      (lang) =>
        !COMMON_LANGUAGES.includes(lang.code) &&
        !RTL_LANGUAGES.includes(lang.code) &&
        !favorites.includes(lang.code)
    )
    .sort((a, b) => a.name.localeCompare(b.name));

  return { favoriteList, commonList, rtlList, otherList };
};

export const STORAGE_KEY_LANGUAGE = 'ampel-language';
export const STORAGE_KEY_FAVORITES = 'ampel-language-favorites';
