/**
 * MSW Handlers for i18n Locale Files
 *
 * Intercepts requests to /locales/{{lng}}/{{ns}}.json and returns
 * the actual locale files. This ensures i18n works correctly in tests
 * without network timeouts.
 */

import { http, HttpResponse } from 'msw';

// Import all English locale files (default language)
import commonEn from '../../../public/locales/en/common.json';
import dashboardEn from '../../../public/locales/en/dashboard.json';
import settingsEn from '../../../public/locales/en/settings.json';
import errorsEn from '../../../public/locales/en/errors.json';
import validationEn from '../../../public/locales/en/validation.json';

// Import French locale files (for language switching tests)
import commonFr from '../../../public/locales/fr/common.json';
import dashboardFr from '../../../public/locales/fr/dashboard.json';
import settingsFr from '../../../public/locales/fr/settings.json';
import errorsFr from '../../../public/locales/fr/errors.json';
import validationFr from '../../../public/locales/fr/validation.json';

// Import German locale files
import commonDe from '../../../public/locales/de/common.json';
import dashboardDe from '../../../public/locales/de/dashboard.json';
import settingsDe from '../../../public/locales/de/settings.json';
import errorsDe from '../../../public/locales/de/errors.json';
import validationDe from '../../../public/locales/de/validation.json';

// Import Arabic locale files (RTL testing)
import commonAr from '../../../public/locales/ar/common.json';
import dashboardAr from '../../../public/locales/ar/dashboard.json';
import settingsAr from '../../../public/locales/ar/settings.json';
import errorsAr from '../../../public/locales/ar/errors.json';
import validationAr from '../../../public/locales/ar/validation.json';

// Import Hebrew locale files (RTL testing)
import commonHe from '../../../public/locales/he/common.json';
import dashboardHe from '../../../public/locales/he/dashboard.json';
import settingsHe from '../../../public/locales/he/settings.json';
import errorsHe from '../../../public/locales/he/errors.json';
import validationHe from '../../../public/locales/he/validation.json';

// Import Spanish (Spain) locale files
import commonEsES from '../../../public/locales/es-ES/common.json';
import dashboardEsES from '../../../public/locales/es-ES/dashboard.json';
import settingsEsES from '../../../public/locales/es-ES/settings.json';
import errorsEsES from '../../../public/locales/es-ES/errors.json';
import validationEsES from '../../../public/locales/es-ES/validation.json';

// Import Italian locale files
import commonIt from '../../../public/locales/it/common.json';
import dashboardIt from '../../../public/locales/it/dashboard.json';
import settingsIt from '../../../public/locales/it/settings.json';
import errorsIt from '../../../public/locales/it/errors.json';
import validationIt from '../../../public/locales/it/validation.json';

// Import English GB locale files
import commonEnGB from '../../../public/locales/en-GB/common.json';
import dashboardEnGB from '../../../public/locales/en-GB/dashboard.json';
import settingsEnGB from '../../../public/locales/en-GB/settings.json';
import errorsEnGB from '../../../public/locales/en-GB/errors.json';
import validationEnGB from '../../../public/locales/en-GB/validation.json';

// Import Portuguese (Brazil) locale files
import commonPtBR from '../../../public/locales/pt-BR/common.json';
import dashboardPtBR from '../../../public/locales/pt-BR/dashboard.json';
import settingsPtBR from '../../../public/locales/pt-BR/settings.json';
import errorsPtBR from '../../../public/locales/pt-BR/errors.json';
import validationPtBR from '../../../public/locales/pt-BR/validation.json';

// Import Chinese (Simplified) locale files
import commonZhCN from '../../../public/locales/zh-CN/common.json';
import dashboardZhCN from '../../../public/locales/zh-CN/dashboard.json';
import settingsZhCN from '../../../public/locales/zh-CN/settings.json';
import errorsZhCN from '../../../public/locales/zh-CN/errors.json';
import validationZhCN from '../../../public/locales/zh-CN/validation.json';

// Import Japanese locale files
import commonJa from '../../../public/locales/ja/common.json';
import dashboardJa from '../../../public/locales/ja/dashboard.json';
import settingsJa from '../../../public/locales/ja/settings.json';
import errorsJa from '../../../public/locales/ja/errors.json';
import validationJa from '../../../public/locales/ja/validation.json';

// Import Czech locale files
import commonCs from '../../../public/locales/cs/common.json';
import dashboardCs from '../../../public/locales/cs/dashboard.json';
import settingsCs from '../../../public/locales/cs/settings.json';
import errorsCs from '../../../public/locales/cs/errors.json';
import validationCs from '../../../public/locales/cs/validation.json';

// Type for namespace
type Namespace = 'common' | 'dashboard' | 'settings' | 'errors' | 'validation';

// Type-safe locale resources
type LocaleResources = Record<Namespace, Record<string, unknown>>;

// Map of language code to locale resources
const localeResources: Record<string, LocaleResources> = {
  en: {
    common: commonEn,
    dashboard: dashboardEn,
    settings: settingsEn,
    errors: errorsEn,
    validation: validationEn,
  },
  fr: {
    common: commonFr,
    dashboard: dashboardFr,
    settings: settingsFr,
    errors: errorsFr,
    validation: validationFr,
  },
  de: {
    common: commonDe,
    dashboard: dashboardDe,
    settings: settingsDe,
    errors: errorsDe,
    validation: validationDe,
  },
  ar: {
    common: commonAr,
    dashboard: dashboardAr,
    settings: settingsAr,
    errors: errorsAr,
    validation: validationAr,
  },
  he: {
    common: commonHe,
    dashboard: dashboardHe,
    settings: settingsHe,
    errors: errorsHe,
    validation: validationHe,
  },
  'es-ES': {
    common: commonEsES,
    dashboard: dashboardEsES,
    settings: settingsEsES,
    errors: errorsEsES,
    validation: validationEsES,
  },
  it: {
    common: commonIt,
    dashboard: dashboardIt,
    settings: settingsIt,
    errors: errorsIt,
    validation: validationIt,
  },
  'en-GB': {
    common: commonEnGB,
    dashboard: dashboardEnGB,
    settings: settingsEnGB,
    errors: errorsEnGB,
    validation: validationEnGB,
  },
  'pt-BR': {
    common: commonPtBR,
    dashboard: dashboardPtBR,
    settings: settingsPtBR,
    errors: errorsPtBR,
    validation: validationPtBR,
  },
  'zh-CN': {
    common: commonZhCN,
    dashboard: dashboardZhCN,
    settings: settingsZhCN,
    errors: errorsZhCN,
    validation: validationZhCN,
  },
  ja: {
    common: commonJa,
    dashboard: dashboardJa,
    settings: settingsJa,
    errors: errorsJa,
    validation: validationJa,
  },
  cs: {
    common: commonCs,
    dashboard: dashboardCs,
    settings: settingsCs,
    errors: errorsCs,
    validation: validationCs,
  },
};

/**
 * Get locale resource for a given language and namespace.
 * Falls back to English if the language is not found.
 */
function getLocaleResource(language: string, namespace: string): Record<string, unknown> | null {
  const langResources = localeResources[language] || localeResources['en'];
  if (!langResources) {
    return null;
  }
  return langResources[namespace as Namespace] || null;
}

/**
 * MSW handlers for i18n locale files.
 * Intercepts requests to /locales/{{lng}}/{{ns}}.json
 */
export const i18nHandlers = [
  // Handler for locale JSON files
  http.get('/locales/:lng/:ns', ({ params }) => {
    const { lng, ns } = params;

    // Extract namespace name (remove .json extension if present)
    const namespace = (ns as string).replace('.json', '');
    const language = lng as string;

    const resource = getLocaleResource(language, namespace);

    if (resource) {
      return HttpResponse.json(resource);
    }

    // Return 404 for unknown locales/namespaces
    return new HttpResponse(null, { status: 404 });
  }),
];
