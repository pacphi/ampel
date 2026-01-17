import { useTranslation } from '@/i18n/hooks';
import { SUPPORTED_LANGUAGES } from '@/i18n/config';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

/**
 * Language Selector Component
 *
 * Dropdown to select the application language from 20 supported languages.
 * Supports both LTR and RTL languages.
 *
 * @example
 * ```tsx
 * <LanguageSelector />
 * ```
 */
export default function LanguageSelector() {
  const { i18n } = useTranslation();

  const handleLanguageChange = (languageCode: string) => {
    i18n.changeLanguage(languageCode);
  };

  const currentLanguage = SUPPORTED_LANGUAGES.find((lang) => lang.code === i18n.language);

  return (
    <Select value={i18n.language} onValueChange={handleLanguageChange}>
      <SelectTrigger className="w-[200px]">
        <SelectValue>
          {currentLanguage ? (
            <div className="flex items-center gap-2">
              <span>{currentLanguage.nativeName}</span>
              {currentLanguage.dir === 'rtl' && (
                <span className="text-xs text-muted-foreground">RTL</span>
              )}
            </div>
          ) : (
            'Select Language'
          )}
        </SelectValue>
      </SelectTrigger>
      <SelectContent>
        {SUPPORTED_LANGUAGES.map((lang) => (
          <SelectItem key={lang.code} value={lang.code}>
            <div className="flex items-center justify-between gap-3">
              <span>{lang.nativeName}</span>
              <span className="text-xs text-muted-foreground">{lang.name}</span>
              {lang.dir === 'rtl' && <span className="rounded bg-muted px-1 text-xs">RTL</span>}
            </div>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
