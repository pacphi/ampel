import * as React from 'react';
import { Globe, Search, Star, X } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { FlagIcon } from '@/components/icons/FlagIcon';
import { cn } from '@/lib/utils';
import { useTranslation } from '@/i18n/hooks';
import {
  SUPPORTED_LANGUAGES,
  groupLanguages,
  STORAGE_KEY_LANGUAGE,
  STORAGE_KEY_FAVORITES,
  type Language,
} from '@/components/i18n/constants/languages';

export interface LanguageSwitcherProps {
  /** Display variant: dropdown (desktop), select (mobile), or inline (compact) */
  variant?: 'dropdown' | 'select' | 'inline';
  /** Show search input */
  showSearch?: boolean;
  /** Show favorite/pin functionality */
  showFavorites?: boolean;
  /** Trigger button size */
  size?: 'sm' | 'default' | 'lg';
  /** CSS class name */
  className?: string;
  /** Callback when language changes */
  onLanguageChange?: (languageCode: string) => void;
}

/**
 * LanguageSwitcher component for selecting application language
 * Supports multiple variants, search, favorites, and keyboard navigation
 * Persists selection to localStorage and syncs with i18next
 */
export const LanguageSwitcher: React.FC<LanguageSwitcherProps> = ({
  variant = 'dropdown',
  showSearch = true,
  showFavorites = true,
  size = 'default',
  className,
  onLanguageChange,
}) => {
  const { i18n } = useTranslation();
  const [searchQuery, setSearchQuery] = React.useState('');
  const [favorites, setFavorites] = React.useState<string[]>([]);
  const [isOpen, setIsOpen] = React.useState(false);
  const searchInputRef = React.useRef<HTMLInputElement>(null);

  // Load favorites from localStorage
  React.useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY_FAVORITES);
    if (stored) {
      try {
        setFavorites(JSON.parse(stored));
      } catch {
        // Failed to parse favorites, using empty array
      }
    }
  }, []);

  // Load language from localStorage on mount
  React.useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY_LANGUAGE);
    if (stored && i18n.language !== stored) {
      i18n.changeLanguage(stored);
    }
  }, [i18n]);

  const currentLanguage =
    SUPPORTED_LANGUAGES.find((lang) => lang.code === i18n.language) || SUPPORTED_LANGUAGES[0];

  const handleLanguageChange = React.useCallback(
    (languageCode: string) => {
      i18n.changeLanguage(languageCode);
      localStorage.setItem(STORAGE_KEY_LANGUAGE, languageCode);
      onLanguageChange?.(languageCode);
      setIsOpen(false);
    },
    [i18n, onLanguageChange]
  );

  const toggleFavorite = React.useCallback((languageCode: string) => {
    setFavorites((prev) => {
      const newFavorites = prev.includes(languageCode)
        ? prev.filter((code) => code !== languageCode)
        : [...prev, languageCode];
      localStorage.setItem(STORAGE_KEY_FAVORITES, JSON.stringify(newFavorites));
      return newFavorites;
    });
  }, []);

  // Filter languages based on search query
  const filteredLanguages = React.useMemo(() => {
    if (!searchQuery.trim()) return SUPPORTED_LANGUAGES;

    const query = searchQuery.toLowerCase();
    return SUPPORTED_LANGUAGES.filter(
      (lang) =>
        lang.name.toLowerCase().includes(query) ||
        lang.nativeName.toLowerCase().includes(query) ||
        lang.code.toLowerCase().includes(query)
    );
  }, [searchQuery]);

  const { favoriteList, commonList, rtlList, otherList } = React.useMemo(
    () => groupLanguages(filteredLanguages, favorites),
    [filteredLanguages, favorites]
  );

  // Focus search input when dropdown opens
  React.useEffect(() => {
    if (isOpen && showSearch && searchInputRef.current) {
      setTimeout(() => searchInputRef.current?.focus(), 100);
    }
  }, [isOpen, showSearch]);

  // Keyboard navigation helper
  const handleKeyDown = React.useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      setIsOpen(false);
    }
  }, []);

  // Render language item
  const renderLanguageItem = (
    lang: Language,
    isFavorite: boolean,
    context: 'dropdown' | 'select'
  ) => {
    const content = (
      <div className="flex items-center gap-2 w-full">
        <FlagIcon languageCode={lang.code} ariaLabel={`${lang.name} flag`} />
        <div className="flex-1 text-left">
          <div className="text-sm font-medium">{lang.name}</div>
          {lang.name !== lang.nativeName && (
            <div className="text-xs text-muted-foreground">{lang.nativeName}</div>
          )}
        </div>
        {showFavorites && context === 'dropdown' && (
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              toggleFavorite(lang.code);
            }}
            aria-label={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
          >
            <Star className={cn('h-3 w-3', isFavorite && 'fill-yellow-400 text-yellow-400')} />
          </Button>
        )}
      </div>
    );

    return content;
  };

  // Mobile/Select variant
  if (variant === 'select') {
    return (
      <Select value={i18n.language} onValueChange={handleLanguageChange}>
        <SelectTrigger className={cn('w-[180px]', className)} aria-label="Select language">
          <SelectValue>
            <div className="flex items-center gap-2">
              <FlagIcon languageCode={currentLanguage.code} />
              <span>{currentLanguage.name}</span>
            </div>
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          {SUPPORTED_LANGUAGES.map((lang) => (
            <SelectItem key={lang.code} value={lang.code}>
              <div className="flex items-center gap-2">
                <FlagIcon languageCode={lang.code} />
                <span>{lang.name}</span>
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    );
  }

  // Inline variant (compact, just flag + name)
  if (variant === 'inline') {
    return (
      <Button
        variant="ghost"
        size={size}
        className={cn('gap-2', className)}
        onClick={() => setIsOpen(true)}
        aria-label={`Current language: ${currentLanguage.name}`}
      >
        <FlagIcon languageCode={currentLanguage.code} />
        <span className="text-sm">{currentLanguage.code.toUpperCase()}</span>
      </Button>
    );
  }

  // Dropdown variant (desktop, full-featured)
  return (
    <TooltipProvider>
      <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
        <Tooltip>
          <TooltipTrigger asChild>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size={size}
                className={cn('gap-2', className)}
                aria-label={`Current language: ${currentLanguage.name}. Click to change language.`}
                aria-haspopup="true"
                aria-expanded={isOpen}
                role="combobox"
              >
                <Globe className="h-4 w-4" />
                <FlagIcon languageCode={currentLanguage.code} />
                <span className="hidden sm:inline">{currentLanguage.name}</span>
              </Button>
            </DropdownMenuTrigger>
          </TooltipTrigger>
          <TooltipContent>
            <p>Select language</p>
          </TooltipContent>
        </Tooltip>

        <DropdownMenuContent
          className="w-[320px] max-h-[500px] overflow-y-auto"
          align="end"
          onKeyDown={handleKeyDown}
          role="listbox"
          aria-label="Language selection menu"
        >
          {showSearch && (
            <div className="p-2 border-b">
              <div className="relative">
                <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  ref={searchInputRef}
                  placeholder="Search languages..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-8 pr-8"
                  aria-label="Search languages"
                />
                {searchQuery && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="absolute right-0 top-1/2 -translate-y-1/2 h-8 w-8 p-0"
                    onClick={() => setSearchQuery('')}
                    aria-label="Clear search"
                  >
                    <X className="h-4 w-4" />
                  </Button>
                )}
              </div>
            </div>
          )}

          {favoriteList.length > 0 && (
            <>
              <DropdownMenuLabel className="flex items-center gap-2">
                <Star className="h-3 w-3 fill-yellow-400 text-yellow-400" />
                Favorites
              </DropdownMenuLabel>
              {favoriteList.map((lang) => (
                <DropdownMenuItem
                  key={lang.code}
                  onClick={() => handleLanguageChange(lang.code)}
                  className={cn('cursor-pointer', lang.code === i18n.language && 'bg-accent')}
                  role="option"
                  aria-selected={lang.code === i18n.language}
                  aria-current={lang.code === i18n.language ? 'true' : undefined}
                >
                  {renderLanguageItem(lang, true, 'dropdown')}
                </DropdownMenuItem>
              ))}
              <DropdownMenuSeparator />
            </>
          )}

          {commonList.length > 0 && (
            <>
              <DropdownMenuLabel>Common Languages</DropdownMenuLabel>
              {commonList.map((lang) => (
                <DropdownMenuItem
                  key={lang.code}
                  onClick={() => handleLanguageChange(lang.code)}
                  className={cn('cursor-pointer', lang.code === i18n.language && 'bg-accent')}
                  role="option"
                  aria-selected={lang.code === i18n.language}
                  aria-current={lang.code === i18n.language ? 'true' : undefined}
                >
                  {renderLanguageItem(lang, favorites.includes(lang.code), 'dropdown')}
                </DropdownMenuItem>
              ))}
              <DropdownMenuSeparator />
            </>
          )}

          {rtlList.length > 0 && (
            <>
              <DropdownMenuLabel>Right-to-Left Languages</DropdownMenuLabel>
              {rtlList.map((lang) => (
                <DropdownMenuItem
                  key={lang.code}
                  onClick={() => handleLanguageChange(lang.code)}
                  className={cn('cursor-pointer', lang.code === i18n.language && 'bg-accent')}
                  role="option"
                  aria-selected={lang.code === i18n.language}
                  aria-current={lang.code === i18n.language ? 'true' : undefined}
                >
                  {renderLanguageItem(lang, favorites.includes(lang.code), 'dropdown')}
                </DropdownMenuItem>
              ))}
              <DropdownMenuSeparator />
            </>
          )}

          {otherList.length > 0 && (
            <>
              <DropdownMenuLabel>Other Languages</DropdownMenuLabel>
              {otherList.map((lang) => (
                <DropdownMenuItem
                  key={lang.code}
                  onClick={() => handleLanguageChange(lang.code)}
                  className={cn('cursor-pointer', lang.code === i18n.language && 'bg-accent')}
                  role="option"
                  aria-selected={lang.code === i18n.language}
                  aria-current={lang.code === i18n.language ? 'true' : undefined}
                >
                  {renderLanguageItem(lang, favorites.includes(lang.code), 'dropdown')}
                </DropdownMenuItem>
              ))}
            </>
          )}

          {filteredLanguages.length === 0 && (
            <div className="p-4 text-center text-sm text-muted-foreground">No languages found</div>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </TooltipProvider>
  );
};

export default LanguageSwitcher;
