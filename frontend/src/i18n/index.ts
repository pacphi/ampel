/**
 * i18n module - Internationalization support for Ampel
 *
 * @module i18n
 */

export { default as i18n } from './config';
export {
  SUPPORTED_LANGUAGES,
  NAMESPACES,
  getLanguageInfo,
  isRTL,
  type LanguageInfo,
  type Namespace,
} from './config';
export { useTranslation } from './hooks';
export type {
  SupportedLanguage,
  TranslationNamespace,
  AccountsTranslations,
  AnalyticsTranslations,
  BehaviorTranslations,
  CommonTranslations,
  DashboardTranslations,
  ErrorsTranslations,
  MergeTranslations,
  NotificationsTranslations,
  ProvidersTranslations,
  RepositoriesTranslations,
  SettingsTranslations,
  ValidationTranslations,
  Translations,
} from './types';
export { defaultLanguage, supportedLanguages } from './types';
