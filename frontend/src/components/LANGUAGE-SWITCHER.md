# Language Switcher Component

**Component:** `LanguageSwitcher.tsx`
**Status:** Implemented
**Total Languages Supported:** 27

---

## Overview

The LanguageSwitcher component provides a user interface for selecting the application language from 27 supported languages with 3 display variants, search functionality, favorites management, and full accessibility support.

---

## Supported Languages (27 Total)

### Simple Language Codes (21)

Languages using simple 2-letter ISO 639-1 codes:

| Code | Language   | Native Name | Script     | Direction |
| ---- | ---------- | ----------- | ---------- | --------- |
| `en` | English    | English     | Latin      | LTR       |
| `fr` | French     | Français    | Latin      | LTR       |
| `de` | German     | Deutsch     | Latin      | LTR       |
| `it` | Italian    | Italiano    | Latin      | LTR       |
| `ru` | Russian    | Русский     | Cyrillic   | LTR       |
| `ja` | Japanese   | 日本語      | Han/Kana   | LTR       |
| `ko` | Korean     | 한국어      | Hangul     | LTR       |
| `ar` | Arabic     | العربية     | Arabic     | **RTL**   |
| `he` | Hebrew     | עברית       | Hebrew     | **RTL**   |
| `hi` | Hindi      | हिन्दी      | Devanagari | LTR       |
| `nl` | Dutch      | Nederlands  | Latin      | LTR       |
| `pl` | Polish     | Polski      | Latin      | LTR       |
| `sr` | Serbian    | Српски      | Cyrillic   | LTR       |
| `th` | Thai       | ไทย         | Thai       | LTR       |
| `tr` | Turkish    | Türkçe      | Latin      | LTR       |
| `sv` | Swedish    | Svenska     | Latin      | LTR       |
| `da` | Danish     | Dansk       | Latin      | LTR       |
| `fi` | Finnish    | Suomi       | Latin      | LTR       |
| `vi` | Vietnamese | Tiếng Việt  | Latin      | LTR       |
| `no` | Norwegian  | Norsk       | Latin      | LTR       |
| `cs` | Czech      | Čeština     | Latin      | LTR       |

### Regional Variant Codes (6)

Languages requiring region-specific variants due to significant differences:

| Code    | Language              | Native Name        | Reason for Variant                                    |
| ------- | --------------------- | ------------------ | ----------------------------------------------------- |
| `en-GB` | English (UK)          | English (UK)       | British vs American spelling/vocabulary               |
| `pt-BR` | Portuguese (Brazil)   | Português (Brasil) | Brazilian vs European Portuguese differ significantly |
| `zh-CN` | Chinese (Simplified)  | 简体中文           | Simplified vs Traditional writing systems             |
| `zh-TW` | Chinese (Traditional) | 繁體中文           | Traditional characters used in Taiwan/Hong Kong       |
| `es-ES` | Spanish (Spain)       | Español (España)   | European Spanish vocabulary/grammar                   |
| `es-MX` | Spanish (Mexico)      | Español (México)   | Latin American Spanish vocabulary                     |

**Total: 21 simple + 6 regional = 27 languages**

---

## RTL (Right-to-Left) Support

**2 RTL Languages:**

- **Arabic (ar)** - Full RTL support with Arabic script
- **Hebrew (he)** - Full RTL support with Hebrew script

**RTL Features:**

- Automatic direction switching (`dir="rtl"` on `<html>`)
- CSS logical properties (margin-inline-start vs margin-left)
- Icon mirroring for directional icons
- Proper text alignment
- Bidirectional text support for mixed LTR/RTL content

---

## Component Variants

### 1. Dropdown Variant (Desktop Default)

```tsx
<LanguageSwitcher variant="dropdown" />
```

**Features:**

- Radix UI dropdown menu
- Grouped languages (Common, RTL, Others)
- Search/filter functionality
- Keyboard navigation
- Flag icons
- Current language indicator

**Best for:** Desktop, large screens

### 2. Select Variant (Mobile Optimized)

```tsx
<LanguageSwitcher variant="select" />
```

**Features:**

- Native select element for mobile
- Grouped optgroups
- Touch-optimized
- System-native UI

**Best for:** Mobile devices, touch screens

### 3. Inline Variant (Compact)

```tsx
<LanguageSwitcher variant="inline" />
```

**Features:**

- Compact button with flag only
- Tooltip on hover
- Minimal space usage

**Best for:** Toolbars, headers with limited space

---

## Props API

```typescript
interface LanguageSwitcherProps {
  /** Display variant */
  variant?: 'dropdown' | 'select' | 'inline';

  /** Show search bar (dropdown variant only) */
  showSearch?: boolean;

  /** Show flag icons */
  showFlags?: boolean;

  /** Show ISO language codes */
  showCodes?: boolean;

  /** Custom CSS class */
  className?: string;

  /** Alignment for dropdown */
  align?: 'start' | 'center' | 'end';

  /** Callback when language changes */
  onLanguageChange?: (languageCode: string) => void;
}
```

---

## Features

### Language Groups

Languages are organized into logical groups for better UX:

1. **Favorites** - User-pinned languages (stored in localStorage)
2. **Common** - Frequently used: English, Spanish, French, German, Portuguese
3. **RTL** - Right-to-left languages: Arabic, Hebrew
4. **Others** - All remaining languages, alphabetically sorted

### Search & Filter

**Supports searching by:**

- Language name (e.g., "Finnish")
- Native name (e.g., "Suomi")
- ISO code (e.g., "fi")
- Case-insensitive
- Real-time filtering

**Example:**

- Search "fin" → Shows Finnish, Chinese
- Search "türk" → Shows Turkish
- Search "cs" → Shows Czech

### Favorites Management

**Features:**

- Pin/unpin languages with star icon
- Favorites stored in localStorage (`ampel-language-favorites`)
- Favorites appear first in dropdown
- Persists across sessions

**Usage:**

```typescript
// Favorites are automatically loaded from localStorage
// Users can click star icon to add/remove favorites
```

### Persistence

**Language selection persists via:**

1. **localStorage** - `ampel-i18n-lng` key
2. **Backend API** - User preferences endpoint (if authenticated)
3. **Browser detection** - Accept-Language header fallback

**Priority:**

1. Explicit user selection (localStorage)
2. Backend user preferences
3. Browser settings
4. Default to English

---

## Accessibility (WCAG 2.1 AA Compliant)

### Keyboard Navigation

**Supported keys:**

- **Tab** - Focus trigger button
- **Enter/Space** - Open dropdown
- **Arrow Down/Up** - Navigate options
- **Home/End** - Jump to first/last option
- **Escape** - Close dropdown
- **Type-ahead** - Jump to language by typing name

### Screen Reader Support

**ARIA attributes:**

```tsx
<button
  role="combobox"
  aria-expanded={isOpen}
  aria-haspopup="listbox"
  aria-label="Select language"
  aria-controls="language-listbox"
>
  Current: {currentLanguage}
</button>

<ul role="listbox" id="language-listbox">
  <li role="option" aria-selected={isSelected}>
    {language.name}
  </li>
</ul>
```

**Features:**

- Screen reader announces language changes
- Focus management (traps focus when open)
- Clear labels for all interactive elements
- Sufficient color contrast (4.5:1 minimum)

---

## Usage Examples

### Basic Usage

```tsx
import { LanguageSwitcher } from '@/components/LanguageSwitcher';

function Header() {
  return (
    <header>
      <LanguageSwitcher />
    </header>
  );
}
```

### With Callback

```tsx
<LanguageSwitcher
  onLanguageChange={(lang) => {
    // Track analytics, show notification, etc.
    analytics.track('Language Changed', { language: lang });
  }}
/>
```

### Compact in Toolbar

```tsx
<LanguageSwitcher variant="inline" showFlags={true} showCodes={false} className="ml-auto" />
```

### Mobile Optimized

```tsx
const isMobile = window.innerWidth < 768;

<LanguageSwitcher variant={isMobile ? 'select' : 'dropdown'} showSearch={!isMobile} />;
```

---

## Integration with i18next

The component is tightly integrated with react-i18next:

```tsx
import { useTranslation } from 'react-i18next';

// Inside LanguageSwitcher
const { i18n } = useTranslation();

// Change language
await i18n.changeLanguage('fi');

// Current language
const currentLang = i18n.language; // 'fi'

// Text direction
const dir = i18n.dir(); // 'ltr' or 'rtl'
```

**Automatic effects when language changes:**

1. RTLProvider updates `document.dir` attribute
2. All `t()` translation functions re-render
3. Lazy loading fetches new language bundle if needed
4. localStorage saves preference
5. Backend API updated (if authenticated)

---

## RTL (Right-to-Left) Behavior

### Automatic RTL Detection

When Arabic or Hebrew is selected:

```tsx
// RTLProvider automatically:
1. Sets document.documentElement.dir = 'rtl'
2. Adds 'rtl' class to <html> and <body>
3. Updates <meta name="direction" content="rtl">
4. Triggers CSS layout changes
```

### CSS Logical Properties

All styles use logical properties for automatic RTL mirroring:

```css
/* ❌ Old way (broken in RTL) */
margin-left: 1rem;
padding-right: 2rem;
border-left: 1px solid;

/* ✅ New way (works in RTL) */
margin-inline-start: 1rem;
padding-inline-end: 2rem;
border-inline-start: 1px solid;
```

### Icon Mirroring

Directional icons automatically flip in RTL:

```tsx
<ChevronRight className="rtl:rotate-180" />
<ArrowLeft className="icon-directional" />
```

---

## Performance

### Bundle Size

- **Component code:** ~8 KB (minified + gzipped)
- **Dependencies:** Included in existing Radix UI bundle
- **Translation files:** ~5 KB per language per namespace (lazy loaded)
- **Initial load:** Only current language + common namespace (~10 KB)

### Loading Strategy

**Lazy loading with caching:**

```typescript
// User switches to Finnish
1. Check if fi/common.json already loaded → Use cache
2. If not loaded → Fetch /locales/fi/common.json
3. Parse and cache in memory
4. Render with Finnish translations
5. Load other namespaces on demand (dashboard, settings, etc.)
```

**Performance metrics:**

- Language switch: <100ms (cache hit)
- Language switch: <500ms (cache miss, network load)
- Search filtering: <16ms (60fps)
- No layout shift on language change

---

## Testing

### Unit Tests

**File:** `frontend/src/components/i18n/__tests__/LanguageSwitcher.test.tsx`

**Coverage (35 tests):**

- ✅ Renders all 27 languages with flags
- ✅ Search filters languages correctly
- ✅ Language selection updates i18n
- ✅ Keyboard navigation works
- ✅ localStorage persistence
- ✅ Favorites management
- ✅ RTL language detection
- ✅ All variants render correctly
- ✅ Accessibility attributes present

### E2E Tests

**File:** `frontend/tests/e2e/language-switching.spec.ts`

**Scenarios tested:**

- Complete language switching workflow for all 27 languages
- RTL layout changes (Arabic, Hebrew)
- Persistent language preferences
- Search functionality
- Keyboard-only navigation
- Mobile responsiveness

---

## Browser Compatibility

**Tested on:**

- ✅ Chrome 120+
- ✅ Firefox 121+
- ✅ Safari 17+
- ✅ Edge 120+

**Mobile:**

- ✅ iOS Safari 17+
- ✅ Chrome Android 120+

**Flag Emoji Support:**

- All browsers support Unicode flag emojis (U+1F1E6 - U+1F1FF range)
- Fallback to country code text if emoji not supported

---

## Troubleshooting

### Language not changing

**Check:**

1. Is language code in SUPPORTED_LANGUAGES? (must be one of 27)
2. Do translation files exist? (`/locales/{lang}/common.json`)
3. Check browser console for loading errors
4. Verify i18next initialized properly

### RTL layout broken

**Check:**

1. Is language in RTL_LANGUAGES? (ar, he)
2. Is RTLProvider component mounted?
3. Are CSS logical properties used?
4. Check `document.dir` attribute value

### Search not working

**Check:**

1. Is variant="dropdown"? (search only available in dropdown)
2. Check browser console for React errors
3. Verify SUPPORTED_LANGUAGES array is accessible

### Flags not rendering

**Check:**

1. Browser supports emoji flags
2. FlagIcon component mapping includes language code
3. Font supports regional indicator symbols

---

## Future Enhancements

**Phase 2+:**

- [ ] Language auto-detection from IP geolocation
- [ ] Translation preview before switching
- [ ] Suggested languages based on user activity
- [ ] Translation completeness indicator per language
- [ ] Community translation contributions

---

## Related Documentation

- [Translation Workflow](../../../docs/localization/TRANSLATION-WORKFLOW.md)
- [i18n Configuration](../i18n/config.ts)
- [RTL Provider](./RTLProvider.tsx)
- [Language Strategy](../../../docs/localization/FINAL-LANGUAGE-STRATEGY.md)

---

**Component Version:** 1.0.0
**Last Updated:** 2025-12-27
**Maintained By:** Ampel Frontend Team
