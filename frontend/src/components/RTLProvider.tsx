import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { isRTL } from '@/i18n/config';

/**
 * RTL (Right-to-Left) Provider Component
 *
 * Monitors i18n language changes and updates:
 * - document.dir attribute (ltr/rtl)
 * - document.lang attribute
 * - 'rtl' class on html and body elements
 * - meta tags for language direction
 *
 * @example
 * ```tsx
 * <I18nextProvider i18n={i18n}>
 *   <RTLProvider>
 *     <App />
 *   </RTLProvider>
 * </I18nextProvider>
 * ```
 */
export default function RTLProvider({ children }: { children: React.ReactNode }) {
  const { i18n } = useTranslation();
  const currentLanguage = i18n.language;

  useEffect(() => {
    const isRtl = isRTL(currentLanguage);
    const dir = isRtl ? 'rtl' : 'ltr';

    // Update document attributes
    document.documentElement.dir = dir;
    document.documentElement.lang = currentLanguage;
    document.body.dir = dir;

    // Update CSS classes for RTL-specific styling
    if (isRtl) {
      document.documentElement.classList.add('rtl');
      document.body.classList.add('rtl');
    } else {
      document.documentElement.classList.remove('rtl');
      document.body.classList.remove('rtl');
    }

    // Update meta tags
    let metaDir = document.querySelector('meta[name="direction"]');
    if (!metaDir) {
      metaDir = document.createElement('meta');
      metaDir.setAttribute('name', 'direction');
      document.head.appendChild(metaDir);
    }
    metaDir.setAttribute('content', dir);

    let metaLang = document.querySelector('meta[http-equiv="content-language"]');
    if (!metaLang) {
      metaLang = document.createElement('meta');
      metaLang.setAttribute('http-equiv', 'content-language');
      document.head.appendChild(metaLang);
    }
    metaLang.setAttribute('content', currentLanguage);
  }, [currentLanguage]);

  return <>{children}</>;
}
