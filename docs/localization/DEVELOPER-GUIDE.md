# Ampel i18n Developer Guide

**Version**: 2.0
**Date**: January 8, 2026
**Status**: Active Development Phase 2

This guide provides comprehensive instructions for developers working with Ampel's internationalization (i18n) system.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Adding Translatable Strings](#adding-translatable-strings)
3. [Frontend: Using `t()` Hook](#frontend-using-t-hook)
4. [Backend: Using `t!()` Macro](#backend-using-t-macro)
5. [Adding New Languages](#adding-new-languages)
6. [Updating Existing Translations](#updating-existing-translations)
7. [Testing Translations Locally](#testing-translations-locally)
8. [Common Pitfalls & Solutions](#common-pitfalls--solutions)
9. [Translation Tool CLI Reference](#translation-tool-cli-reference)
10. [Debugging & Troubleshooting](#debugging--troubleshooting)

---

## Quick Start

### Prerequisites

```bash
# Ensure you have the development environment set up
make install      # Install all dependencies
make dev-api      # Start backend server
make dev-frontend # Start frontend dev server
```

### 30-Second Overview

1. **Add English string** to appropriate namespace JSON/YAML file
2. **Frontend**: Import `useTranslation()` and use `t('key')`
3. **Backend**: Use `t!("key")` macro in Rust code
4. **Translate**: Run `cargo i18n translate en/common.json --all-languages`
5. **Test**: Switch language in UI or use browser console

### Key Files to Know

| File                                        | Purpose                                 |
| ------------------------------------------- | --------------------------------------- |
| `frontend/public/locales/en/*.json`         | Frontend English strings (5 namespaces) |
| `crates/ampel-api/locales/en/*.yml`         | Backend English strings (4 namespaces)  |
| `frontend/src/i18n/config.ts`               | i18next configuration                   |
| `crates/ampel-api/src/middleware/locale.rs` | Locale detection middleware             |
| `crates/ampel-i18n-builder/`                | Translation automation tool             |

---

## Adding Translatable Strings

### Frontend: Adding Strings to JSON

**Location**: `frontend/public/locales/en/`

Supported namespaces:

- `common.json` - App-wide strings (auth, UI labels, etc.)
- `dashboard.json` - PR dashboard specific strings
- `settings.json` - Settings page strings
- `errors.json` - Error messages
- `validation.json` - Form validation messages

**Step 1: Identify the right namespace**

```typescript
// PRCard component - use 'dashboard' namespace
// Header component - use 'common' namespace
// Form validation - use 'validation' namespace
```

**Step 2: Add to English JSON file**

Use dot notation for nested keys:

```json
{
  "auth": {
    "form": {
      "username": "Username",
      "password": "Password",
      "rememberMe": "Remember me"
    },
    "error": {
      "invalidCredentials": "Invalid username or password",
      "sessionExpired": "Your session has expired"
    }
  }
}
```

**Step 3: Use dot notation in code**

```typescript
// ✅ CORRECT - Matches JSON structure
t('auth.form.username');
t('auth.error.invalidCredentials');

// ❌ WRONG - Won't find the key
t('username');
t('invalidCredentials');
```

**Guidelines for Frontend Strings**:

| Type                  | Example Key                 | Location          |
| --------------------- | --------------------------- | ----------------- |
| Page/Component Titles | `dashboard.prDashboard`     | `dashboard.json`  |
| Form Labels           | `auth.form.username`        | `common.json`     |
| Button Text           | `common.button.submit`      | `common.json`     |
| Error Messages        | `errors.auth.invalidEmail`  | `errors.json`     |
| Validation Messages   | `validation.email.required` | `validation.json` |
| Status Badges         | `common.status.open`        | `common.json`     |
| Menu Items            | `common.menu.settings`      | `common.json`     |

### Backend: Adding Strings to YAML

**Location**: `crates/ampel-api/locales/en/`

Supported namespaces:

- `common.yml` - General backend strings
- `errors.yml` - API error messages
- `validation.yml` - Field validation messages
- `providers.yml` - Git provider specific messages

**Step 1: Edit the English YAML file**

```yaml
errors:
  auth:
    invalid_credentials: 'Invalid username or password'
    token_expired: 'Authentication token has expired'
    unauthorized: 'You are not authorized to perform this action'

validation:
  email:
    required: 'Email address is required'
    invalid: 'Email address is invalid'
    already_exists: 'Email address is already registered'
```

**Step 2: Use in Rust code**

```rust
// Enable rust-i18n macro
use rust_i18n::t;

// Basic usage
return Err(AppError::Unauthorized(t!("errors.auth.unauthorized")));

// With interpolation
return Err(AppError::Validation(t!("validation.email.required")));

// With namespace
let message = t!("errors.auth.invalid_credentials", locale = "es");
```

**Guidelines for Backend Strings**:

| Type              | Example Key                       | Location         |
| ----------------- | --------------------------------- | ---------------- |
| API Errors        | `errors.auth.invalid_credentials` | `errors.yml`     |
| Validation Errors | `validation.email.required`       | `validation.yml` |
| Provider Errors   | `providers.github.auth_failed`    | `providers.yml`  |
| Success Messages  | `common.success.password_updated` | `common.yml`     |
| Log Messages      | `common.logs.user_login`          | `common.yml`     |

---

## Frontend: Using `t()` Hook

### Basic Setup

```typescript
import { useTranslation } from 'react-i18next';

export function MyComponent() {
  // Hook can specify one or multiple namespaces
  const { t } = useTranslation(['common', 'dashboard']);

  return (
    <div>
      <h1>{t('dashboard:prDashboard')}</h1>
      <p>{t('common:app.loading')}</p>
    </div>
  );
}
```

### Hook Parameters

```typescript
// Specify single namespace (uses default if not specified)
const { t } = useTranslation();

// Specify multiple namespaces
const { t } = useTranslation(['common', 'errors']);

// Get i18n instance for advanced usage
const { t, i18n } = useTranslation();
```

### Simple String Translation

```typescript
const { t } = useTranslation('common');

// Direct translation
<button>{t('button.submit')}</button>
<p>{t('auth.form.username')}</p>
<span>{t('status.open')}</span>
```

### String Interpolation

Use `{{variable}}` placeholder syntax:

```json
{
  "messages": {
    "welcome": "Welcome, {{name}}!",
    "prCount": "You have {{count}} pull requests",
    "lastUpdated": "Last updated: {{date}}"
  }
}
```

Usage in component:

```typescript
const { t } = useTranslation('common');

<h1>{t('messages.welcome', { name: 'John' })}</h1>
<p>{t('messages.prCount', { count: 5 })}</p>
<span>{t('messages.lastUpdated', { date: new Date().toLocaleDateString() })}</span>
```

### Pluralization

Define plural forms in JSON:

```json
{
  "pullRequests_one": "{{count}} pull request",
  "pullRequests_other": "{{count}} pull requests",

  "reviewsNeeded_zero": "No reviews needed",
  "reviewsNeeded_one": "{{count}} review needed",
  "reviewsNeeded_other": "{{count}} reviews needed"
}
```

Usage:

```typescript
const { t } = useTranslation('dashboard');

// i18next automatically selects correct plural form
<p>{t('pullRequests', { count: prList.length })}</p>
<p>{t('reviewsNeeded', { count: reviewCount })}</p>
```

### Default Fallback

```typescript
// If key not found, returns the key itself
<p>{t('unknown.key')}</p>  // Renders: "unknown.key"

// Provide custom fallback
<p>{t('unknown.key', { defaultValue: 'No data available' })}</p>
```

### Accessing i18n Instance

```typescript
const { t, i18n } = useTranslation();

// Get current language
console.log(i18n.language); // 'en', 'fr', 'de', etc.

// Change language programmatically
await i18n.changeLanguage('fr');

// Check if language is RTL
const isRTL = i18n.language === 'ar' || i18n.language === 'he';
```

### RTL Language Support

Components are automatically wrapped with `RTLProvider`:

```typescript
// No special code needed - RTLProvider handles it
// For languages 'ar' (Arabic) or 'he' (Hebrew):
// - document.dir === 'rtl'
// - document.lang updated
// - CSS classes applied

// Use logical CSS properties (not affected by RTL)
<div className="ps-4 me-2">  // padding-inline-start, margin-inline-end
  {t('common:app.title')}
</div>
```

---

## Backend: Using `t!()` Macro

### Setup

Enable rust-i18n in your crate:

```rust
// At top of main.rs or lib.rs
rust_i18n::i18n!("locales");
```

Directory structure:

```
crates/ampel-api/
├── locales/
│   ├── en/
│   │   ├── common.yml
│   │   ├── errors.yml
│   │   ├── validation.yml
│   │   └── providers.yml
│   ├── fr/
│   ├── de/
│   └── ... (25 more languages)
└── src/
```

### Basic Usage

```rust
use rust_i18n::t;

// Simple string
let msg = t!("errors.auth.unauthorized");

// With interpolation
let msg = t!("validation.email.already_exists", email = "user@example.com");

// With context locale override
let msg = t!("errors.auth.invalid_credentials", locale = "es");

// Construct error with translated message
return Err(AppError::Unauthorized(t!("errors.auth.unauthorized")));
```

### Locale Detection in Requests

Locale is automatically detected from requests via middleware:

```rust
// In handler function
use crate::middleware::locale::DetectedLocale;
use axum::extract::Extension;

async fn login(
    Extension(detected_locale): Extension<DetectedLocale>,
    // ... other parameters
) -> Result<impl IntoResponse> {
    // detected_locale.code contains detected language (e.g., "en", "fr", "de")

    // Translate error to detected locale
    if invalid_email(&email) {
        let error_msg = t!("validation.email.invalid", locale = detected_locale.code);
        return Err(AppError::Validation(error_msg));
    }

    Ok(Json(response))
}
```

### Interpolation with Variables

```yaml
# errors.yml
errors:
  provider:
    rate_limit: 'Rate limit exceeded. Please try again in {{minutes}} minutes'
    auth_failed: 'Authentication failed for {{provider}}: {{reason}}'
```

Usage:

```rust
let msg = t!("errors.provider.rate_limit", minutes = 5);
let msg = t!("errors.provider.auth_failed", provider = "GitHub", reason = "Invalid token");
```

### Pluralization (Advanced)

```yaml
# common.yml
pull_requests:
  one: '{{count}} pull request'
  other: '{{count}} pull requests'
```

```rust
// rust-i18n handles pluralization based on count
let msg = t!("pull_requests.one", count = 1);      // "1 pull request"
let msg = t!("pull_requests.other", count = 5);    // "5 pull requests"
```

### Error Handling Pattern

```rust
// ❌ Before: Hardcoded error messages
return Err(AppError::Unauthorized("Invalid credentials".into()));

// ✅ After: Localized error messages
return Err(AppError::Unauthorized(t!("errors.auth.invalid_credentials")));

// ✅ Best: With context
if !user_exists {
    return Err(AppError::NotFound(
        t!("errors.auth.user_not_found", locale = locale.code)
    ));
}
```

### Testing with Different Locales

```rust
#[tokio::test]
async fn test_error_message_in_finnish() {
    let response = client
        .post("/api/v1/auth/login?lang=fi")
        .json(&invalid_login)
        .send()
        .await;

    // Response will include Finnish error message
    assert!(response_body.contains("Virheellinen"));
}
```

---

## Adding New Languages

### Supported Languages

Ampel supports 27 languages:

**Simple Codes (21 languages)**:

```
en (English), fr (French), de (German), it (Italian), ru (Russian),
ja (Japanese), ko (Korean), ar (Arabic), he (Hebrew), hi (Hindi),
nl (Dutch), pl (Polish), sr (Serbian), th (Thai), tr (Turkish),
sv (Swedish), da (Danish), fi (Finnish), vi (Vietnamese),
no (Norwegian), cs (Czech)
```

**Regional Variants (6 languages)**:

```
en-GB (English UK), pt-BR (Portuguese Brazil), zh-CN (Chinese Simplified),
zh-TW (Chinese Traditional), es-ES (Spanish Spain), es-MX (Spanish Mexico)
```

### Adding Support for a New Language

**Step 1: Create locale directories**

```bash
# Frontend
mkdir -p frontend/public/locales/{language-code}

# Backend
mkdir -p crates/ampel-api/locales/{language-code}
```

**Step 2: Copy English template files**

```bash
# Frontend
cp frontend/public/locales/en/*.json frontend/public/locales/{language-code}/

# Backend
cp crates/ampel-api/locales/en/*.yml crates/ampel-api/locales/{language-code}/
```

**Step 3: Verify in configuration**

Check if language is in `SUPPORTED_LANGUAGES` array:

```typescript
// frontend/src/i18n/config.ts
export const SUPPORTED_LANGUAGES: LanguageInfo[] = [
  { code: 'xx', name: 'Language Name', nativeName: 'Native Name', dir: 'ltr', isoCode: 'xx-XX' },
];

// For RTL languages (Arabic, Hebrew)
dir: 'rtl';
```

Backend middleware already includes all 27 languages automatically.

**Step 4: Run translation CLI**

```bash
# Translate frontend files
cargo i18n translate frontend/public/locales/en/*.json \
  --target {language-code} \
  --all-languages

# Translate backend files (YAML to JSON → translate → back to YAML)
for namespace in common errors validation providers; do
  yq eval -o=json "crates/ampel-api/locales/en/${namespace}.yml" > "/tmp/${namespace}.json"
  cargo i18n translate "/tmp/${namespace}.json" --target {language-code}
  yq eval -P "/tmp/${namespace}.json" > "crates/ampel-api/locales/{language-code}/${namespace}.yml"
done
```

**Step 5: Validate translations**

```bash
# Check coverage
node validate-translations.js {language-code}

# Should show >90% coverage
```

**Step 6: Test locally**

```bash
# Frontend
make dev-frontend
# Change language to new language in browser

# Backend
make dev-api
curl http://localhost:8080/api/v1/auth/login?lang={language-code} \
  -H "Content-Type: application/json" \
  -d '{"email":"test","password":"test"}'
# Should return error in target language
```

---

## Updating Existing Translations

### Adding New Keys to Existing Language

**Frontend**:

1. Add key to `frontend/public/locales/en/{namespace}.json`
2. Translate to target languages:
   ```bash
   cargo i18n translate frontend/public/locales/en/{namespace}.json \
     --target {language-code}
   ```
3. Run validation:
   ```bash
   node validate-translations.js {language-code}
   ```

**Backend**:

1. Add key to `crates/ampel-api/locales/en/{namespace}.yml`
2. Convert, translate, and convert back:
   ```bash
   yq eval -o=json "crates/ampel-api/locales/en/{namespace}.yml" > "/tmp/{namespace}.json"
   cargo i18n translate "/tmp/{namespace}.json" --target {language-code}
   yq eval -P "/tmp/{namespace}.json" > "crates/ampel-api/locales/{language-code}/{namespace}.yml"
   ```

### Fixing Translation Quality Issues

**Option 1: Use alternative translation provider**

```bash
# Try different provider (Systran, DeepL, Google, OpenAI)
cargo i18n translate frontend/public/locales/en/common.json \
  --target fr \
  --provider deepl
```

**Option 2: Manual correction**

Edit the JSON/YAML file directly:

```json
{
  "messages": {
    "welcome": "Bienvenue, {{name}}!" // Manually corrected
  }
}
```

**Option 3: Get context for better translation**

Include context comments in source:

```json
{
  "status": {
    "merged": "Merged", // "Merged" as in PR merged, not "combined"
    "_comment_merged": "Context: Pull Request status, past tense"
  }
}
```

### Batch Update All Languages

```bash
# Translate all untranslated keys in all languages
cargo i18n translate frontend/public/locales/en/*.json \
  --all-languages \
  --parallel \
  --max-concurrent 3

# Check coverage report
node validate-translations.js --all
```

---

## Testing Translations Locally

### Frontend Testing

**Manual Testing**:

1. Start dev server:

   ```bash
   make dev-frontend
   ```

2. Open browser console:

   ```javascript
   // Change language
   localStorage.setItem('ampel-i18n-lng', 'fr');
   location.reload();

   // Check current language
   localStorage.getItem('ampel-i18n-lng');

   // Check favorites
   JSON.parse(localStorage.getItem('ampel-language-favorites'));
   ```

3. Verify UI changes to selected language

**Automated Testing**:

```bash
# Run frontend tests
make test-frontend

# Run specific language tests
npm test -- --testNamePattern="French"
```

**RTL Testing** (Arabic/Hebrew):

```javascript
// Browser console
localStorage.setItem('ampel-i18n-lng', 'ar');
location.reload();

// Verify RTL
console.log(document.dir); // Should be 'rtl'
console.log(document.lang); // Should be 'ar'
document.documentElement.classList.contains('rtl'); // Should be true
```

### Backend Testing

**Test with query parameter**:

```bash
# Start backend
make dev-api

# Finnish
curl http://localhost:8080/api/v1/auth/login?lang=fi \
  -H "Content-Type: application/json" \
  -d '{"email":"invalid","password":"wrong"}'

# German
curl http://localhost:8080/api/v1/auth/login?lang=de \
  -H "Content-Type: application/json" \
  -d '{"email":"invalid","password":"wrong"}'
```

**Test with Accept-Language header**:

```bash
curl http://localhost:8080/api/v1/auth/login \
  -H "Accept-Language: fr,en;q=0.9" \
  -H "Content-Type: application/json" \
  -d '{"email":"invalid","password":"wrong"}'
```

**Test with cookie**:

```bash
curl http://localhost:8080/api/v1/auth/login \
  -H "Cookie: lang=es-ES" \
  -H "Content-Type: application/json" \
  -d '{"email":"invalid","password":"wrong"}'
```

**Unit tests**:

```bash
# Run all locale middleware tests
cargo test --package ampel-api locale_detection

# Run specific test
cargo test --package ampel-api test_normalize_locale
```

### Translation Coverage Validation

```bash
# Validate single language
node validate-translations.js pt-BR

# Validate all languages
node validate-translations.js --all

# Example output:
# ✓ fr      ████████░░░░░░░░░░░░ 50.5% (164/325)
# ✗ de      ███░░░░░░░░░░░░░░░░░░ 16.6% (54/325)
```

---

## Common Pitfalls & Solutions

### Pitfall 1: Missing Translation Keys

**Problem**: Component shows raw key instead of translated text

```typescript
// Shows "common:app.unknownKey" in UI
{
  t('common:app.unknownKey');
}
```

**Solution**:

1. Check key exists in JSON:

   ```bash
   grep -r "unknownKey" frontend/public/locales/en/
   ```

2. Verify namespace and key path match:

   ```typescript
   // ✅ CORRECT
   const { t } = useTranslation('common');
   {
     t('app.unknownKey');
   }

   // ❌ WRONG (namespaced syntax)
   {
     t('common:app.unknownKey');
   } // Should omit namespace here
   ```

3. Add missing key to English file first:
   ```json
   {
     "app": {
       "unknownKey": "Some translation"
     }
   }
   ```

### Pitfall 2: Incorrect Namespace Usage

**Problem**: Translation not loading, wrong namespace specified

```typescript
// ❌ WRONG
const { t } = useTranslation('common');
{
  t('dashboard:prDashboard');
} // Can't access dashboard namespace

// ✅ CORRECT (Option 1: Load multiple namespaces)
const { t } = useTranslation(['common', 'dashboard']);
{
  t('dashboard:prDashboard');
}

// ✅ CORRECT (Option 2: Single namespace, proper usage)
const { t } = useTranslation('dashboard');
{
  t('prDashboard');
}
```

### Pitfall 3: Placeholder Variables Not Replaced

**Problem**: Interpolation shows `{{variable}}` literally

```typescript
// English JSON
"messages": {
  "welcome": "Welcome, {{name}}!"
}

// ❌ WRONG - Missing variable parameter
{t('messages.welcome')}  // Shows: "Welcome, {{name}}!"

// ✅ CORRECT
{t('messages.welcome', { name: 'Alice' })}  // Shows: "Welcome, Alice!"
```

**Solution**: Always pass variables as second parameter in object format.

### Pitfall 4: Pluralization Not Working

**Problem**: Plural forms not switching correctly

```json
// ❌ WRONG - Missing _one and _other suffixes
{
  "items": "Item|Items"
}

// ✅ CORRECT - Use i18next suffix convention
{
  "items_one": "{{count}} item",
  "items_other": "{{count}} items"
}
```

Usage:

```typescript
// Always use `count` parameter for pluralization
{
  t('items', { count: 1 });
} // "1 item"
{
  t('items', { count: 5 });
} // "5 items"
```

### Pitfall 5: RTL Layout Breaking

**Problem**: Layout doesn't flip for Arabic/Hebrew

**Solution**: Use logical CSS properties instead of physical:

```css
/* ❌ WRONG - Physical properties */
.sidebar {
  margin-left: 10px;
  padding-right: 15px;
  border-left: 1px solid #ccc;
}

/* ✅ CORRECT - Logical properties */
.sidebar {
  margin-inline-start: 10px;    /* Flips with text direction */
  padding-inline-end: 15px;
  border-inline-start: 1px solid #ccc;
}

/* Or use Tailwind logical utilities */
<div className="ps-4 me-2">...</div>
```

### Pitfall 6: Backend Locale Not Detected

**Problem**: Backend error messages always in English

**Causes and solutions**:

```rust
// ❌ WRONG - Not using detected locale
return Err(AppError::Unauthorized("Invalid credentials".into()));

// ✅ CORRECT - Use detected locale
use crate::middleware::locale::DetectedLocale;

async fn login(
    Extension(detected_locale): Extension<DetectedLocale>
) -> Result<()> {
    if invalid {
        let msg = t!("errors.auth.invalid", locale = detected_locale.code);
        return Err(AppError::Unauthorized(msg));
    }
}
```

Priority order for backend locale detection:

1. Query parameter: `?lang=fi`
2. Cookie: `lang=de`
3. Accept-Language header: `Accept-Language: pt-BR,pt;q=0.9`
4. Default: `en`

### Pitfall 7: Translation Files Out of Sync

**Problem**: Some languages missing translations others have

**Solution**:

```bash
# Validate all languages
node validate-translations.js --all

# Re-translate all untranslated keys
cargo i18n translate frontend/public/locales/en/*.json \
  --all-languages \
  --force  # Overwrite existing

# Check specific language coverage
node validate-translations.js de
# Output shows exactly what's missing
```

### Pitfall 8: Case Sensitivity Issues

**Problem**: Key lookups fail unexpectedly

```javascript
// ❌ WRONG - camelCase in JSON
{ "userName": "Username" }
t('userName')

// BUT in YAML, use snake_case
// errors.yml
errors:
  invalid_credentials: "Invalid credentials"
t!("errors.invalid_credentials")

// ✅ RULE: Follow source file conventions
// JSON files: camelCase
// YAML files: snake_case
```

---

## Translation Tool CLI Reference

### Installation

```bash
# Build ampel-i18n-builder crate
cd crates/ampel-i18n-builder
cargo build --release

# Or use via cargo
cargo i18n --help
```

### Available Commands

```bash
# Translate single file
cargo i18n translate frontend/public/locales/en/common.json --target fr

# Translate all namespaces to all languages
cargo i18n translate frontend/public/locales/en/*.json \
  --all-languages \
  --parallel \
  --max-concurrent 3

# Translate with specific provider
cargo i18n translate common.json --target de --provider deepl

# Translate with timeout override
cargo i18n translate settings.json --target ja --timeout 60

# Validate coverage
cargo i18n validate frontend/public/locales

# Generate coverage report
node validate-translations.js --all > coverage-report.txt
```

### Configuration

API keys in `.env`:

```bash
# Tier 1 - Primary provider
SYSTRAN_API_KEY="your_systran_key"

# Tier 2 - EU languages
DEEPL_API_KEY="your_deepl_key"

# Tier 3 - All languages
GOOGLE_API_KEY="your_google_key"

# Tier 4 - Fallback
OPENAI_API_KEY="your_openai_key"
```

---

## Debugging & Troubleshooting

### Enable Debug Logging

**Frontend**:

```typescript
import i18n from 'i18next';

// Enable debug mode
i18n.on('missingKey', (lng, ns, key) => {
  console.warn(`Missing translation: [${lng}][${ns}] ${key}`);
});

// Monitor language changes
i18n.on('languageChanged', (lng) => {
  console.log(`Language changed to: ${lng}`);
  console.log(`Direction: ${document.dir}`);
});
```

**Browser DevTools**:

```javascript
// Check i18next state
i18next.language;
i18next.languages;
i18next.ns;
i18next.backend;
i18next.t('key'); // Manually test translations
```

### Common Issues

**Issue**: Locale detection priority wrong

```bash
# Test detection order
curl "http://localhost:8080/api/test?lang=fi" \
  -H "Cookie: lang=de" \
  -H "Accept-Language: fr" \
  # Priority: fi (query) > de (cookie) > fr (header)
```

**Issue**: Translation file syntax error

```bash
# Validate JSON syntax
node -e "console.log(JSON.parse(require('fs').readFileSync('frontend/public/locales/en/common.json')))"

# Validate YAML syntax
yamllint crates/ampel-api/locales/en/errors.yml
```

**Issue**: Missing Backend Translation

```bash
# Check if translation macro is using correct key path
grep -r "t!(\"errors.auth" crates/ampel-api/src/

# Verify YAML file has the key
grep "invalid_credentials" crates/ampel-api/locales/en/errors.yml
```

### Performance Debugging

```typescript
// Measure language switch time
console.time('language-switch');
await i18n.changeLanguage('fr');
console.timeEnd('language-switch'); // Should be <100ms
```

```bash
# Measure backend response time with translation
time curl http://localhost:8080/api/v1/auth/login?lang=de \
  -H "Content-Type: application/json" \
  -d '{"email":"test","password":"test"}'
```

---

## Checklist: Adding New Feature with Translations

- [ ] Add English strings to frontend JSON (common/dashboard/settings/errors/validation)
- [ ] Add English strings to backend YAML (common/errors/validation/providers)
- [ ] Import `useTranslation()` in frontend component
- [ ] Use `t()` hook with correct namespace and key path
- [ ] Use `t!()` macro in backend error handlers
- [ ] Verify frontend builds without errors: `make build-frontend`
- [ ] Verify backend builds without errors: `make build-backend`
- [ ] Test in English: `make dev-frontend && make dev-api`
- [ ] Run translation tool: `cargo i18n translate en/*.json --all-languages`
- [ ] Validate coverage: `node validate-translations.js --all`
- [ ] Test in at least 3 languages (e.g., French, German, Arabic)
- [ ] Test RTL language if applicable (Arabic/Hebrew)
- [ ] Run test suite: `make test`
- [ ] Create PR with `[i18n]` prefix

---

## Additional Resources

- **[react-i18next Documentation](https://react.i18next.com/)** - Frontend hook usage
- **[rust-i18n Documentation](https://github.com/longbridge/rust-i18n)** - Backend macro usage
- **[i18next Pluralization](https://www.i18next.com/translation-function/plurals)** - Complex plural rules
- **[ICU MessageFormat](https://unicode-org.github.io/icu/userguide/format_parse/messages/)** - Advanced syntax
- **[CLDR Plural Rules](https://cldr.unicode.org/index/cldr-spec/plural-rules)** - Language-specific rules
- **[CSS Logical Properties](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Logical_Properties)** - RTL support

---

**Last Updated**: January 8, 2026
**Maintained By**: Ampel Development Team
**Status**: Active Development
