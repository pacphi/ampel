import { useTranslation as useTranslationOriginal } from 'react-i18next';
import type { Namespace } from './config';

/**
 * Type-safe wrapper around react-i18next's useTranslation hook
 *
 * @example
 * ```tsx
 * const { t } = useTranslation('common');
 * return <p>{t('app.title')}</p>;
 * ```
 */
export function useTranslation(ns?: Namespace | Namespace[]) {
  return useTranslationOriginal(ns);
}
