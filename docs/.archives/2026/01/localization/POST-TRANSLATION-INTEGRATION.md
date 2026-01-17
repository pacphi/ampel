# Post-Translation Integration Checklist

**Date**: January 8, 2026
**Status**: Translations Running, Integration Verification Needed

---

## Current Integration Status

### ✅ What's Already Working

**Backend (rust-i18n)**:

- ✅ `I18nextProvider` wraps entire app (`frontend/src/main.tsx:28`)
- ✅ `RTLProvider` handles Arabic/Hebrew layouts (`main.tsx:29`)
- ✅ i18n configuration loaded (`frontend/src/i18n/config.ts`)
- ✅ Locale middleware active (`crates/ampel-api/src/middleware/locale.rs`)
- ✅ Backend using `t!()` macro in error messages
- ✅ User language preference API (`GET/PUT /api/v1/user/preferences/language`)

**Frontend (react-i18next)**:

- ✅ Multiple components using `useTranslation()` hook:
  - `Dashboard.tsx` - `t('dashboard:prDashboard')`
  - `Header.tsx` - `t('common:auth.logout')`
  - `Settings.tsx` - `t('settings:tabs.profile')`
  - `Sidebar.tsx` - Using translations
  - `PRCard.tsx` - Using translations
  - `Login.tsx` - Using translations
  - `Register.tsx` - Using translations

---

## ⚠️ MISSING: Language Switcher UI Integration

### Issue

The `LanguageSwitcher` component exists (428 lines, 3 variants) but is **NOT rendered anywhere** in the app!

**Component Location**: `frontend/src/components/LanguageSwitcher.tsx`

**Status**: Created but not integrated into UI

### Where to Add LanguageSwitcher

**Option 1: Add to Header (Recommended)**

Add language selector next to theme toggle in the header:

```tsx
// frontend/src/components/layout/Header.tsx

import LanguageSwitcher from '@/components/LanguageSwitcher';

export default function Header() {
  const { t } = useTranslation(['dashboard', 'common']);
  // ... existing code ...

  return (
    <header className="flex h-16 items-center justify-between border-b bg-card px-6">
      <div className="flex items-center gap-4">
        <h1 className="text-lg font-semibold">{t('dashboard:prDashboard')}</h1>
      </div>
      <div className="flex items-center gap-4">
        {/* ADD THIS: Language Switcher */}
        <LanguageSwitcher variant="inline" size="sm" />

        {/* Existing theme toggle */}
        <Button variant="ghost" size="icon" onClick={toggleTheme}>
          {resolvedTheme === 'dark' ? <Sun /> : <Moon />}
        </Button>

        {/* ... rest of header ... */}
      </div>
    </header>
  );
}
```

**Option 2: Add to Settings Page**

Add full language selector in Settings > Profile:

```tsx
// frontend/src/pages/Settings.tsx (ProfileSettings component)

import LanguageSwitcher from '@/components/LanguageSwitcher';

function ProfileSettings() {
  // ... existing code ...

  return (
    <div className="space-y-6">
      {/* Existing profile fields */}

      {/* ADD THIS: Language Settings Section */}
      <div>
        <h3 className="text-lg font-medium">{t('settings:language.title')}</h3>
        <p className="text-sm text-muted-foreground mb-4">{t('settings:language.description')}</p>
        <LanguageSwitcher variant="dropdown" showSearch showFavorites />
      </div>
    </div>
  );
}
```

**Option 3: Both Locations**

- Header: Compact `inline` variant (flag icon only)
- Settings: Full `dropdown` variant with search and favorites

---

## Testing After Translation Completes

### 1. Verify Frontend Translations

**Start the development server**:

```bash
make dev-frontend
```

**Visit**: `http://localhost:5173`

**Test Steps**:

1. ✅ Login to the application
2. ✅ Click language switcher (once integrated)
3. ✅ Select different language (e.g., French, German, Portuguese)
4. ✅ Verify UI text changes:
   - Header navigation
   - Dashboard labels
   - Button text
   - Form fields
   - Error messages
5. ✅ Test RTL languages (Arabic, Hebrew):
   - Layout should flip to right-to-left
   - Text should align right
   - Icons should flip direction
6. ✅ Refresh page - language should persist (localStorage)

### 2. Verify Backend Translations

**Start the API server**:

```bash
make dev-api
```

**Test Steps**:

**Test 1: Query Parameter**

```bash
# Request with Finnish locale
curl http://localhost:8080/api/v1/auth/login?lang=fi \
  -H "Content-Type: application/json" \
  -d '{"email":"invalid","password":"wrong"}'

# Expected: Finnish error message
# {"error":"Virheelliset kirjautumistiedot"}
```

**Test 2: Accept-Language Header**

```bash
# Request with German preference
curl http://localhost:8080/api/v1/auth/login \
  -H "Accept-Language: de,en;q=0.9" \
  -H "Content-Type: application/json" \
  -d '{"email":"invalid","password":"wrong"}'

# Expected: German error message
# {"error":"Ungültige Anmeldedaten"}
```

**Test 3: User Language Preference**

```bash
# Set user language preference to French
curl http://localhost:8080/api/v1/user/preferences/language \
  -X PUT \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"language":"fr"}'

# Future requests return French error messages
```

### 3. Verify Language Persistence

**Frontend localStorage**:

```javascript
// Open browser console
localStorage.getItem('ampel-i18n-lng'); // Should show selected language
localStorage.getItem('ampel-language-favorites'); // Should show favorites
```

**Backend database**:

```sql
-- Check user language preferences
SELECT id, email, language FROM users LIMIT 10;
```

### 4. Verify RTL Support

**Test with Arabic**:

1. Select Arabic (ar) from language switcher
2. Verify:
   - ✅ `document.dir === 'rtl'`
   - ✅ `document.lang === 'ar'`
   - ✅ `<html>` has class `rtl`
   - ✅ Layout flips right-to-left
   - ✅ Text aligns right
   - ✅ Margins/padding reversed

**Test with Hebrew**:

- Same as Arabic but with `lang='he'`

---

## Integration Requirements

### Required Changes for Full Functionality

**1. Add LanguageSwitcher to Header** ✅ REQUIRED

```tsx
// File: frontend/src/components/layout/Header.tsx
// Line ~38: Add between logo and theme toggle

<LanguageSwitcher variant="inline" size="sm" />
```

**2. Add LanguageSwitcher to Settings** ⚠️ OPTIONAL

```tsx
// File: frontend/src/pages/Settings.tsx
// In ProfileSettings component

<div className="space-y-4">
  <h3>{t('settings:language.title')}</h3>
  <LanguageSwitcher variant="dropdown" showSearch showFavorites />
</div>
```

**3. Add Language Settings Translations** ⚠️ OPTIONAL

```json
// frontend/public/locales/en/settings.json
{
  "language": {
    "title": "Language Preferences",
    "description": "Select your preferred language",
    "current": "Current Language",
    "favorites": "Favorite Languages"
  }
}
```

---

## Expected Behavior After Integration

### Scenario 1: First-Time User

1. User visits `http://localhost:5173`
2. Browser language detected (e.g., `fr-FR`)
3. App loads in French automatically
4. User can change via LanguageSwitcher
5. Preference saved to localStorage
6. After login, synced to database

### Scenario 2: Returning User

1. User visits app
2. Language loaded from localStorage (e.g., `de`)
3. App renders in German immediately
4. After login, database value checked
5. If different, user prompted to sync

### Scenario 3: Logged-In User

1. User already logged in
2. Language loaded from database (via API)
3. App renders in user's preferred language
4. Changes via LanguageSwitcher save to database
5. Persists across devices

### Scenario 4: RTL Language User

1. User selects Arabic or Hebrew
2. `RTLProvider` detects RTL language
3. Layout flips to right-to-left:
   - Navigation moves to right
   - Text aligns right
   - Icons flip direction
4. All CSS uses logical properties (ms-_, me-_, ps-_, pe-_)

---

## Validation Commands

### After Translation Completes

```bash
# 1. Validate all translations
node validate-translations.js --all

# 2. Check for 100% languages
node validate-translations.js --all | grep "100.0%"

# 3. Test specific language
node validate-translations.js fr

# 4. Count total translated keys
find frontend/public/locales -name "*.json" -exec jq -r 'recurse | strings | select(. != "")' {} \; | wc -l
```

### Manual Testing Checklist

**Frontend**:

- [ ] Start dev server: `make dev-frontend`
- [ ] Add LanguageSwitcher to Header
- [ ] Login to application
- [ ] Change language to French
- [ ] Verify all UI text changes to French
- [ ] Change to Arabic
- [ ] Verify layout flips to RTL
- [ ] Refresh page
- [ ] Verify language persists (localStorage)

**Backend**:

- [ ] Start API server: `make dev-api`
- [ ] Trigger validation error with `?lang=fi`
- [ ] Verify error message in Finnish
- [ ] Test Accept-Language header with German
- [ ] Verify error message in German
- [ ] Update user language preference via API
- [ ] Verify future errors in selected language

**Full Stack**:

- [ ] Login and set language to Spanish
- [ ] Verify frontend shows Spanish UI
- [ ] Trigger API error (e.g., invalid form)
- [ ] Verify error message from backend in Spanish
- [ ] Logout and login again
- [ ] Verify language persists from database

---

## Known Limitations (Current)

### 🚨 Language Switcher Not in UI

**Impact**: Users cannot change language (must edit localStorage manually)

**Temporary Workaround**:

```javascript
// Browser console
localStorage.setItem('ampel-i18n-lng', 'fr'); // French
location.reload(); // Reload to apply

localStorage.setItem('ampel-i18n-lng', 'ar'); // Arabic
location.reload();

localStorage.setItem('ampel-i18n-lng', 'de'); // German
location.reload();
```

**Permanent Fix**: Add LanguageSwitcher to Header (see above)

### ⚠️ Backend Locale Detection Order

**Current Priority**:

1. Query parameter (`?lang=fi`)
2. Cookie (`lang=fi`)
3. Accept-Language header
4. Default (`en`)

**Note**: User database preference is NOT checked automatically. Need to add middleware to check `user.language` column after authentication.

---

## Post-Translation Tasks

### Immediate (After Translation Completes)

- [ ] Validate all 26 languages reach 90%+ coverage
- [ ] Fix any translation quality issues
- [ ] Add LanguageSwitcher to Header
- [ ] Test language switching in browser
- [ ] Verify RTL layouts (ar, he)

### Short-Term

- [ ] Add backend middleware to check user.language from database
- [ ] Create language onboarding modal for first-time users
- [ ] Add language preview feature
- [ ] Native speaker review for top 5 languages

### Long-Term

- [ ] A/B test translation quality
- [ ] Collect user feedback on translations
- [ ] Implement translation correction workflow
- [ ] Add context-aware translations

---

## Success Criteria

**Frontend**: ✅ Working when LanguageSwitcher added

- Translations load from JSON files
- Language switching changes UI text
- RTL layouts work correctly
- Persistence via localStorage

**Backend**: ✅ Working now

- Translations load from YAML files
- Locale detection from headers/cookies
- Error messages in user's language
- API responds with correct locale

**Full Integration**: ⚠️ Needs LanguageSwitcher in UI

- User can select language from UI
- Preference saves to localStorage AND database
- Language persists across sessions
- RTL languages flip layout correctly

---

**Next Step**: Add LanguageSwitcher to Header.tsx after translations complete!
