# Ampel i18n User Guide

**Version**: 2.0
**Date**: January 8, 2026
**Status**: Active

This guide explains how to use Ampel in your preferred language.

---

## Table of Contents

1. [Changing Language](#changing-language)
2. [Supported Languages](#supported-languages)
3. [RTL Language Support](#rtl-language-support)
4. [Language Persistence](#language-persistence)
5. [Troubleshooting](#troubleshooting)
6. [Accessibility](#accessibility)

---

## Changing Language

### Using the Language Switcher

Ampel provides an easy way to change your language:

**Location**: Language switcher is available in the header (top-right area)

**Steps**:

1. Click the language icon or current language name in the header
2. Search for or select your preferred language
3. UI automatically updates to your selected language
4. Your preference is saved automatically

### Available Language Switcher Options

The language selector includes:

- **Language Name** (in English) - e.g., "French", "German"
- **Native Name** - e.g., "Français", "Deutsch"
- **Language Code** - ISO 639-1 code for reference
- **Flag Icon** - Visual indicator of the language/country

### Keyboard Navigation

- **Tab**: Move between languages
- **Enter/Space**: Select highlighted language
- **Escape**: Close selector
- **Type to search**: Start typing language name to filter options

---

## Supported Languages

Ampel supports **27 languages** across the globe:

### Popular Languages (6 most commonly used)

| Language               | Code  | Native Name        | Direction     |
| ---------------------- | ----- | ------------------ | ------------- |
| 🇬🇧 English             | en    | English (US)       | Left-to-Right |
| 🇫🇷 French              | fr    | Français           | Left-to-Right |
| 🇩🇪 German              | de    | Deutsch            | Left-to-Right |
| 🇪🇸 Spanish (Spain)     | es-ES | Español (España)   | Left-to-Right |
| 🇧🇷 Portuguese (Brazil) | pt-BR | Português (Brasil) | Left-to-Right |
| 🇮🇹 Italian             | it    | Italiano           | Left-to-Right |

### European Languages (10 languages)

| Language        | Code  | Direction | Notes            |
| --------------- | ----- | --------- | ---------------- |
| 🇬🇧 English (UK) | en-GB | LTR       | British spelling |
| 🇸🇪 Swedish      | sv    | LTR       | Nordic languages |
| 🇳🇴 Norwegian    | no    | LTR       | Nordic languages |
| 🇩🇰 Danish       | da    | LTR       | Nordic languages |
| 🇫🇮 Finnish      | fi    | LTR       | Nordic languages |
| 🇳🇱 Dutch        | nl    | LTR       | Western Europe   |
| 🇵🇱 Polish       | pl    | LTR       | Central Europe   |
| 🇨🇿 Czech        | cs    | LTR       | Central Europe   |
| 🇷🇸 Serbian      | sr    | LTR       | Eastern Europe   |
| 🇷🇺 Russian      | ru    | LTR       | Cyrillic script  |

### Asian Languages (5 languages)

| Language                 | Code  | Direction | Script         |
| ------------------------ | ----- | --------- | -------------- |
| 🇨🇳 Chinese (Simplified)  | zh-CN | LTR       | Han characters |
| 🇹🇼 Chinese (Traditional) | zh-TW | LTR       | Han characters |
| 🇯🇵 Japanese              | ja    | LTR       | Kanji/Kana     |
| 🇰🇷 Korean                | ko    | LTR       | Hangul         |
| 🇹🇭 Thai                  | th    | LTR       | Thai script    |

### Middle Eastern & South Asian Languages (5 languages)

| Language      | Code | Direction         | Script                |
| ------------- | ---- | ----------------- | --------------------- |
| 🇸🇦 Arabic     | ar   | **Right-to-Left** | Arabic script         |
| 🇮🇱 Hebrew     | he   | **Right-to-Left** | Hebrew script         |
| 🇮🇳 Hindi      | hi   | LTR               | Devanagari script     |
| 🇹🇷 Turkish    | tr   | LTR               | Latin with diacritics |
| 🇻🇳 Vietnamese | vi   | LTR               | Latin with diacritics |

**Note**: RTL languages (Arabic, Hebrew) flip the entire layout for proper reading experience.

---

## RTL Language Support

### What is RTL?

**RTL** stands for "Right-to-Left". Some languages (Arabic, Hebrew) are read from right to left instead of left to right.

When you select an RTL language in Ampel:

1. **Layout flips**: Navigation moves to the right side
2. **Text aligns right**: All text aligns to the right edge
3. **Icons flip**: Icons that indicate direction are mirrored
4. **Margins/padding reverse**: Spacing adjusts accordingly

### How Ampel Handles RTL

Ampel automatically:

- Detects RTL languages (Arabic, Hebrew)
- Flips the entire layout for proper readability
- Uses logical CSS properties (work in both directions)
- Updates HTML direction attribute (`dir="rtl"`)
- Mirrors directional icons

### Selecting an RTL Language

**Step 1**: Click language selector

**Step 2**: Search for "Arabic" or "Hebrew"

**Step 3**: Select from results

**Result**:

- Layout automatically flips right-to-left
- Navigation moves to right side
- All text aligns right
- Icons flip direction

### RTL Language Examples

**Arabic (العربية)**:

- Layout direction: Right-to-Left
- Text direction: Right-to-Left
- Uses Arabic script

**Hebrew (עברית)**:

- Layout direction: Right-to-Left
- Text direction: Right-to-Left
- Uses Hebrew script

### Switching Back to LTR

Simply select any non-RTL language:

**Step 1**: Click language selector

**Step 2**: Select English, French, or another LTR language

**Result**: Layout returns to normal left-to-right

---

## Language Persistence

### How Your Language Choice is Saved

Your language preference is saved in **two places**:

#### 1. Browser Storage (Immediate)

Your preference is saved locally on your device in browser storage:

- **Location**: Browser's local storage
- **Key**: `ampel-i18n-lng`
- **Persistence**: Survives browser restart
- **Scope**: Works even when offline

**How to check** (Browser Console):

```javascript
// Open DevTools (F12) > Console tab
localStorage.getItem('ampel-i18n-lng');
// Result: 'fr', 'de', 'ar', etc.
```

#### 2. Server Storage (After Login)

When you're logged in, your preference is also saved to your account:

- **Location**: Ampel database (your user profile)
- **Sync time**: Automatic when you log in
- **Persistence**: Across all devices
- **Scope**: Works across different computers

### What Gets Persisted

**Saved**:

- Selected language (e.g., French, Arabic)
- Whether you manually changed it

**Not Saved**:

- Temporary translations in your browser cache
- Translation history

### Behavior by Scenario

#### First-Time User (Not Logged In)

1. Visit Ampel
2. Browser language detected (e.g., French)
3. App loads in French automatically
4. Language saved to browser storage
5. On next visit, French loads automatically

**How to test**: Open browser DevTools > Application > Local Storage

#### Returning User (Not Logged In)

1. Visit Ampel
2. App loads language from browser storage (your previous choice)
3. UI displays in selected language immediately
4. No need to select language again

#### Switching Languages While Using

1. Click language selector
2. Choose new language
3. UI updates immediately
4. New language saved to browser storage
5. API requests start using new language

**Impact**: Error messages from server appear in newly selected language

#### Login After Changing Language

1. You changed language (e.g., German)
2. You log in
3. App checks your account preference
4. If different, account preference loaded
5. UI might prompt to sync preferences

#### Logged-In User

1. Sign in
2. App loads language from your account
3. UI displays in your preferred language
4. Language choice persists across devices
5. Mobile app, tablet, desktop all show same language

#### Logout and Login Again

1. You logout
2. Return later (different session)
3. Sign in again
4. Your account language loads automatically
5. No need to re-select

### Manual Control (Advanced)

If you're comfortable with browser console, you can manually control language storage:

```javascript
// Set language to French
localStorage.setItem('ampel-i18n-lng', 'fr');
location.reload();

// Set to Arabic
localStorage.setItem('ampel-i18n-lng', 'ar');
location.reload();

// Clear and use browser default
localStorage.removeItem('ampel-i18n-lng');
location.reload();

// View saved favorites
JSON.parse(localStorage.getItem('ampel-language-favorites'));
```

---

## Troubleshooting

### Issue: Language Doesn't Change

**Symptom**: Click language, UI stays in English

**Solutions**:

**Solution 1: Refresh the page**

- Press `F5` or `Ctrl+R` to refresh
- If language persists after refresh, try Solution 2

**Solution 2: Clear browser cache**

1. Open DevTools (`F12`)
2. Go to "Application" tab
3. Click "Local Storage"
4. Click your Ampel domain
5. Delete all entries
6. Refresh page
7. Select language again

**Solution 3: Check if JavaScript is enabled**

- Ampel requires JavaScript to work
- Go to browser settings > JavaScript
- Ensure it's enabled for ampel.example.com

**Solution 4: Try different language**

- Try selecting a completely different language (e.g., Arabic)
- If that works, your previous language may have cache issues
- Clear cache (Solution 2) and try again

### Issue: Language Reverts to English

**Symptom**: Language changes, but reverts to English on page refresh

**Causes and Solutions**:

**Cause 1: Browser cache corrupted**

Solution: Clear local storage

```javascript
// Browser console
localStorage.clear();
location.reload();
```

**Cause 2: Not logged in, new browser session**

Solution: Log in to sync your account preference

**Cause 3: Browser doesn't support local storage**

Solution: Enable local storage in browser settings

### Issue: RTL Language Shows Wrong Direction

**Symptom**: Arabic/Hebrew content shows left-to-right

**Solutions**:

**Solution 1: Check browser direction**

```javascript
// Browser console
document.dir; // Should be 'rtl' for Arabic/Hebrew
document.lang; // Should be 'ar' or 'he'
```

**Solution 2: Check browser zoom**

- Sometimes zoom levels cause layout issues
- Press `Ctrl+0` to reset zoom to 100%

**Solution 3: Check viewport**

- Resize browser window
- Some layouts need minimum width
- Try full-screen (F11)

**Solution 4: Try another RTL language**

- If Arabic doesn't work, try Hebrew
- If both don't work, issue may be browser-specific

### Issue: Error Messages in Wrong Language

**Symptom**: UI is in French but error messages appear in English

**Causes and Solutions**:

**Cause 1: API doesn't know about language change**

Solution: Refresh page after changing language

**Cause 2: Error from browser, not API**

Solution: Check browser console for JavaScript errors

**Cause 3: Older cached API response**

Solution: Clear browser cache (Ctrl+Shift+Delete)

### Issue: Special Characters Display Incorrectly

**Symptom**: Arabic, Chinese, or Thai text shows garbled

**Solutions**:

**Solution 1: Check page encoding**

1. Open browser DevTools (`F12`)
2. Go to Network tab
3. Check response headers
4. Look for `Content-Type: charset=utf-8`

**Solution 2: Check system font support**

- Some languages require specific fonts
- Install language support in your OS:
  - **Windows**: Settings > Language > Add language
  - **Mac**: System Preferences > Language & Region
  - **Linux**: Install language packages

**Solution 3: Try different browser**

- Try Chrome, Firefox, Safari, or Edge
- Some browsers have better language support

### Issue: Language Selection Dropdownn is Broken

**Symptom**: Can't open or interact with language selector

**Solutions**:

**Solution 1: Hard refresh**

- `Ctrl+Shift+R` (Windows) or `Cmd+Shift+R` (Mac)
- Clears cache and reloads everything

**Solution 2: Check keyboard**

- Try clicking instead of keyboard
- Some keyboard shortcuts may interfere

**Solution 3: Clear browser extensions**

- Some extensions interfere with dropdowns
- Temporarily disable extensions
- Try again

**Solution 4: Report issue**

If none of above work, report issue with:

- Browser name and version
- Operating system
- Screenshot of problem

---

## Accessibility

### Language Support for Accessibility

Ampel properly sets language attributes for accessibility:

- **Screen readers**: Announce current language
- **Spell check**: Uses language-specific dictionary
- **Text size**: Adjusts for scripts with varying sizes (CJK, Arabic)

### Using with Screen Readers

When you change language:

- Screen reader announces new language
- Content reads in correct language/accent (if available)
- Text direction (RTL) is announced

**Popular Screen Readers**:

- NVDA (Windows)
- JAWS (Windows)
- VoiceOver (Mac/iOS)
- TalkBack (Android)

### Using with Browser Extensions

Some helpful extensions for multi-language users:

- **Google Translate**: Auto-translate any page
- **Language Switcher**: Quick keyboard shortcuts
- **Spell Checker**: Support for multiple languages

### Keyboard Navigation

- **Tab**: Move between language options
- **Arrow Keys**: Navigate up/down in list
- **Enter**: Select highlighted language
- **Escape**: Close language selector

---

## Getting Help

### Where to Report Issues

Found a translation problem?

1. **Note the details**:
   - Language you were using
   - Which text was wrong
   - What was shown vs. what should show
   - Page/feature where issue occurred

2. **Report via**:
   - GitHub Issues: Include `[i18n]` in title
   - Contact support: Include screenshot

### Translation Improvements

Help improve translations:

1. Note the issue
2. Suggest better translation
3. Include context if possible
4. Report via GitHub Issues or support

---

## Quick Reference

### Language Codes Quick Look

| Popular    | Code  | Less Common           | Code  |
| ---------- | ----- | --------------------- | ----- |
| English    | en    | English (UK)          | en-GB |
| French     | fr    | Portuguese (Brazil)   | pt-BR |
| German     | de    | Spanish (Mexico)      | es-MX |
| Spanish    | es-ES | Chinese (Simple)      | zh-CN |
| Italian    | it    | Chinese (Traditional) | zh-TW |
| Portuguese | pt-BR | -                     | -     |

### Browser Compatibility

| Browser           | Status     | Notes                                    |
| ----------------- | ---------- | ---------------------------------------- |
| Chrome            | ✅ Full    | Latest versions supported                |
| Firefox           | ✅ Full    | Latest versions supported                |
| Safari            | ✅ Full    | Latest versions supported                |
| Edge              | ✅ Full    | Latest versions supported                |
| Internet Explorer | ⚠️ Limited | Some languages may not display correctly |

---

## Tips & Tricks

### Tip 1: Set Browser Language as Default

Let Ampel detect your browser's language automatically:

1. Set your browser's language
   - **Chrome**: Settings > Language
   - **Firefox**: Settings > Language
   - **Safari**: System Preferences > Language & Region
2. Clear Ampel language selection
   ```javascript
   localStorage.removeItem('ampel-i18n-lng');
   ```
3. Reload Ampel
4. It will use your browser's language

### Tip 2: Quick Language Switching

Create bookmarks for quick language access:

```javascript
// Bookmark this for French
javascript: localStorage.setItem('ampel-i18n-lng', 'fr');
location.reload();

// Or for Arabic
javascript: localStorage.setItem('ampel-i18n-lng', 'ar');
location.reload();
```

Click bookmark to instantly switch language.

### Tip 3: Share Language-Specific Links

Include language in URL to share specific language:

```
https://ampel.example.com/?lang=de  # German
https://ampel.example.com/?lang=ja  # Japanese
https://ampel.example.com/?lang=ar  # Arabic
```

When recipient opens link, they get that language.

### Tip 4: Multi-Language Team

If your team uses multiple languages:

- Each person sets their preferred language
- Preference saved to their account
- Works across all devices
- No need to configure repeatedly

---

## Supported Locales by Region

### Americas

- 🇺🇸 English (en)
- 🇧🇷 Portuguese - Brazil (pt-BR)
- 🇲🇽 Spanish - Mexico (es-MX)

### Europe

- 🇬🇧 English - UK (en-GB)
- 🇫🇷 French (fr)
- 🇩🇪 German (de)
- 🇮🇹 Italian (it)
- 🇪🇸 Spanish - Spain (es-ES)
- 🇳🇱 Dutch (nl)
- 🇸🇪 Swedish (sv)
- 🇳🇴 Norwegian (no)
- 🇩🇰 Danish (da)
- 🇫🇮 Finnish (fi)
- 🇵🇱 Polish (pl)
- 🇨🇿 Czech (cs)
- 🇷🇸 Serbian (sr)

### Asia-Pacific

- 🇨🇳 Chinese - Simplified (zh-CN)
- 🇹🇼 Chinese - Traditional (zh-TW)
- 🇯🇵 Japanese (ja)
- 🇰🇷 Korean (ko)
- 🇹🇭 Thai (th)
- 🇻🇳 Vietnamese (vi)
- 🇮🇳 Hindi (hi)

### Middle East & Africa

- 🇸🇦 Arabic (ar)
- 🇮🇱 Hebrew (he)

### Other

- 🇷🇺 Russian (ru)
- 🇹🇷 Turkish (tr)

---

## Contact & Support

**Have questions?**

- Check this guide again (likely answer here)
- Visit [Ampel GitHub](https://github.com/pacphi/ampel)
- Open an issue with `[i18n]` tag
- Contact support team

**Found a translation error?**

- Report with context
- Include which language and what was wrong
- Suggest correction if you know it

---

**Last Updated**: January 8, 2026
**Language Support**: 27 languages
**Version**: 2.0
