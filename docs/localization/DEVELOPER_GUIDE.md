# i18n Developer Guide

**Version:** 1.0
**Date:** 2025-12-27
**Quick Start:** 15 minutes

## Table of Contents

1. [Quick Start](#quick-start)
2. [First Translation Workflow](#first-translation-workflow)
3. [Adding New Languages](#adding-new-languages)
4. [Testing Translations Locally](#testing-translations-locally)
5. [Contributing Guidelines](#contributing-guidelines)

---

## Quick Start

### Prerequisites

- Rust 1.91+
- Node.js 18+ and pnpm 10.24+
- Redis 7+ (for caching, optional)
- DeepL API key (for translation)

### Installation

#### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/your-org/ampel.git
cd ampel

# Build the i18n-builder tool
cargo build --release --bin i18n-builder

# Install frontend dependencies
cd frontend && pnpm install && cd ..
```

#### 2. Configure API Keys

```bash
# Create configuration file
cat > .ampel-i18n.yaml <<EOF
translation_dir: "frontend/public/locales"

translation:
  timeout_secs: 30
  batch_size: 50
EOF

# Set DeepL API key
export DEEPL_API_KEY="your-deepl-api-key"

# Optional: Set Google Cloud API key (for Thai, Arabic)
export GOOGLE_API_KEY="your-google-api-key"
```

#### 3. Install Git Hooks

```bash
# Install pre-commit validation hooks
./scripts/install-git-hooks.sh

# Test hook installation
git add README.md
git commit -m "test: verify hooks"
```

### Project Structure

```
ampel/
├── crates/ampel-i18n-builder/      # Translation automation crate
│   ├── src/
│   │   ├── translator/             # DeepL, Google, OpenAI clients
│   │   ├── formats/                # YAML, JSON parsers
│   │   ├── validation/             # Coverage, missing keys
│   │   └── codegen/                # TypeScript, Rust generators
│   └── tests/                      # Integration tests
│
├── crates/ampel-api/locales/       # Backend translations (YAML)
│   ├── en/                         # English source
│   ├── es/                         # Spanish
│   ├── fi/                         # Finnish
│   └── ...
│
├── frontend/public/locales/        # Frontend translations (JSON)
│   ├── en/                         # English source
│   │   ├── common.json
│   │   ├── dashboard.json
│   │   └── settings.json
│   └── ...
│
└── docs/localization/              # Documentation
    ├── DEVELOPER_GUIDE.md          # This file
    ├── TRANSLATION_WORKFLOW.md     # Detailed workflow
    └── ARCHITECTURE.md             # System architecture
```

---

## First Translation Workflow

### Scenario: Add a New Dashboard Feature

#### Step 1: Add English Source Text

```bash
# Edit English source file
vim crates/ampel-api/locales/en/common.yml
```

```yaml
# Add new translation keys
dashboard:
  filters:
    status:
      label: "Filter by Status"
      options:
        all: "All Statuses"
        open: "Open"
        closed: "Closed"
        merged: "Merged"
```

#### Step 2: Translate to Target Languages

```bash
# Translate to Finnish using DeepL
cargo run --bin i18n-builder -- translate \
  --provider deepl \
  --input crates/ampel-api/locales/en/common.yml \
  --output crates/ampel-api/locales/fi/common.yml \
  --target-lang fi

# Output:
# ✓ Translated 4 keys in 1.2s
# ✓ Cache hit rate: 0%
# ✓ API usage: 85 characters
```

#### Step 3: Validate Coverage

```bash
# Check translation coverage
cargo run --bin i18n-builder -- validate \
  --input-dir crates/ampel-api/locales \
  --base-locale en \
  --min-coverage 95

# Output:
# ✓ Coverage: 96.5%
# ✓ No missing translations
# ✓ All variables match
```

#### Step 4: Use in Code

```rust
// crates/ampel-api/src/handlers/dashboard.rs
use rust_i18n::t;

pub async fn get_filters() -> Vec<FilterOption> {
    vec![
        FilterOption {
            value: "all",
            label: t!("dashboard.filters.status.options.all"),
        },
        FilterOption {
            value: "open",
            label: t!("dashboard.filters.status.options.open"),
        },
        // ...
    ]
}
```

#### Step 5: Commit Changes

```bash
# Add all translation files
git add crates/ampel-api/locales/

# Commit with conventional format
git commit -m "feat(i18n): add dashboard filter translations"

# Pre-commit hook runs:
# ✓ YAML syntax validation
# ✓ Coverage check (96.5% >= 95%)
# ✓ Variable consistency check
```

---

## Adding New Languages

### Supported Languages

The system currently supports 20 languages:

| Language | Code | Provider | RTL | Complex Plurals |
|----------|------|----------|-----|-----------------|
| English | en | - | No | No |
| Portuguese (Brazil) | pt-BR | DeepL | No | No |
| Spanish (Spain) | es-ES | DeepL | No | No |
| German | de | DeepL | No | No |
| French | fr | DeepL | No | No |
| Hebrew | he | DeepL | **Yes** | No |
| Dutch | nl | DeepL | No | No |
| Italian | it | DeepL | No | No |
| Polish | pl | DeepL | No | **Yes** |
| Russian | ru | DeepL | No | **Yes** |
| Serbian | sr | DeepL | No | Yes |
| Chinese (Simplified) | zh-CN | DeepL | No | No |
| Japanese | ja | DeepL | No | No |
| Finnish | fi | DeepL | No | **Yes** |
| Swedish | sv | DeepL | No | No |
| Norwegian | no | DeepL | No | No |
| Danish | da | DeepL | No | No |
| Czech | cs | DeepL | No | **Yes** |
| Thai | th | **Google** | No | No |
| Arabic | ar | **Google** | **Yes** | Yes |

### Adding a New Language

#### 1. Create Language Directory

```bash
# Backend
mkdir -p crates/ampel-api/locales/ko  # Korean

# Frontend
mkdir -p frontend/public/locales/ko
```

#### 2. Translate Initial Content

```bash
# Translate backend files
cargo run --bin i18n-builder -- translate \
  --provider deepl \
  --input crates/ampel-api/locales/en/common.yml \
  --output crates/ampel-api/locales/ko/common.yml \
  --target-lang ko

# Translate frontend files
for file in frontend/public/locales/en/*.json; do
  basename=$(basename "$file")
  cargo run --bin i18n-builder -- translate \
    --provider deepl \
    --input "$file" \
    --output "frontend/public/locales/ko/$basename" \
    --target-lang ko
done
```

#### 3. Configure Language Support

```typescript
// frontend/src/i18n/config.ts

export const SUPPORTED_LANGUAGES = {
  // ... existing languages

  ko: {
    name: 'Korean',
    nativeName: '한국어',
    dir: 'ltr',
    flag: '🇰🇷',
  },
} as const;
```

```rust
// crates/ampel-api/src/i18n.rs

rust_i18n::i18n!("locales", fallback = "en");

pub const SUPPORTED_LOCALES: &[&str] = &[
    "en", "pt-BR", "es-ES", "de", "fr", "he",
    "nl", "sr", "ru", "it", "pl", "zh-CN", "ja",
    "fi", "sv", "no", "da", "cs", "th", "ar",
    "ko",  // Add new language
];
```

#### 4. Test RTL Support (if applicable)

```bash
# For RTL languages (Hebrew, Arabic), test layout
cd frontend
REACT_APP_I18N_DEBUG=true pnpm dev

# Navigate to Settings > Language > Arabic
# Check:
# - Text alignment (right-to-left)
# - Icon direction (mirrored)
# - Modal positioning
```

#### 5. Validate and Commit

```bash
# Validate coverage
cargo run --bin i18n-builder -- validate \
  --input-dir crates/ampel-api/locales \
  --base-locale en \
  --min-coverage 95

# Commit
git add crates/ampel-api/locales/ko/ frontend/public/locales/ko/
git commit -m "feat(i18n): add Korean language support"
```

---

## Testing Translations Locally

### Backend Testing (Rust)

#### Unit Tests

```rust
// crates/ampel-api/tests/i18n_tests.rs

#[cfg(test)]
mod tests {
    use rust_i18n::t;

    #[test]
    fn test_finnish_pluralization() {
        rust_i18n::set_locale("fi");

        assert_eq!(t!("items.count", count = 1), "1 kohde");
        assert_eq!(t!("items.count", count = 2), "2 kohdetta");
        assert_eq!(t!("items.count", count = 5), "5 kohdetta");
    }

    #[test]
    fn test_variable_interpolation() {
        rust_i18n::set_locale("en");

        let name = "Alice";
        assert_eq!(
            t!("greeting", name = name),
            "Hello, Alice!"
        );
    }

    #[test]
    fn test_fallback_locale() {
        rust_i18n::set_locale("invalid");

        // Should fallback to English
        assert_eq!(t!("dashboard.title"), "Pull Request Dashboard");
    }
}
```

#### Run Backend Tests

```bash
# Run all i18n tests
cargo test --package ampel-api i18n

# Run with verbose output
cargo test --package ampel-api i18n -- --nocapture

# Test specific locale
RUST_I18N_LOCALE=fi cargo test i18n
```

### Frontend Testing (React)

#### Component Tests

```typescript
// frontend/src/components/__tests__/LanguageSwitcher.test.tsx

import { render, screen, fireEvent } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '../../i18n/config';
import { LanguageSwitcher } from '../LanguageSwitcher';

describe('LanguageSwitcher', () => {
  it('renders current language', () => {
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    expect(screen.getByText(/English/i)).toBeInTheDocument();
  });

  it('changes language on selection', async () => {
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    // Open dropdown
    fireEvent.click(screen.getByRole('button'));

    // Select Finnish
    fireEvent.click(screen.getByText(/Suomi/i));

    // Verify language changed
    expect(i18n.language).toBe('fi');
  });

  it('persists language preference', () => {
    localStorage.setItem('preferredLanguage', 'fi');

    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    expect(i18n.language).toBe('fi');
  });
});
```

#### Run Frontend Tests

```bash
cd frontend

# Run all i18n tests
pnpm test i18n

# Run with coverage
pnpm test:coverage i18n

# Test specific language
REACT_APP_I18N_LOCALE=fi pnpm test
```

### Integration Tests

#### Test Translation Workflow

```bash
# Run integration tests
cargo test --test integration_tests

# Test specific provider
cargo test --test integration_tests deepl

# Test with actual API (requires API key)
DEEPL_API_KEY=xxx cargo test --test integration_tests -- --ignored
```

### Manual Testing

#### Local Development Server

```bash
# Start backend with specific locale
RUST_I18N_LOCALE=fi make dev-api

# Start frontend with debug mode
cd frontend
REACT_APP_I18N_DEBUG=true pnpm dev
```

#### Test Different Languages

```bash
# Open browser console and run:
# Change language
window.localStorage.setItem('preferredLanguage', 'fi');
window.location.reload();

# Check current translations
i18next.t('dashboard.title');  // "Vetopyyntöjen hallintapaneeli"

# List all keys
Object.keys(i18next.store.data.fi.dashboard);
```

---

## Contributing Guidelines

### Translation Quality Standards

#### 1. Accuracy

- Use professional translation tools (DeepL preferred)
- Verify technical terms with native speakers
- Maintain context and tone of source text

#### 2. Consistency

- Use consistent terminology across all translations
- Follow platform-specific conventions (iOS, Android, Web)
- Preserve variable names and placeholders

#### 3. Completeness

- Maintain 95% minimum coverage
- Include all plural forms for complex languages
- Provide context comments for ambiguous strings

### Code Standards

#### Commit Message Format

```bash
# Format: <type>(i18n): <description>

feat(i18n): add Finnish language support
fix(i18n): correct Arabic RTL layout
chore(i18n): update translation coverage
docs(i18n): improve localization guide
test(i18n): add pluralization tests
```

#### Pull Request Checklist

- [ ] All translation keys are in English source files
- [ ] Coverage is ≥95% for all languages
- [ ] No hardcoded strings in code
- [ ] Variables match between source and translations
- [ ] RTL languages display correctly (if applicable)
- [ ] Tests pass locally
- [ ] Pre-commit hooks pass
- [ ] CI validation passes

### Review Process

1. **Automated Checks** (GitHub Actions)
   - YAML/JSON syntax validation
   - Coverage threshold verification
   - Variable consistency check
   - RTL visual regression tests

2. **Human Review**
   - Native speaker verification (critical languages)
   - Context and tone review
   - Technical term accuracy
   - User experience testing

3. **Approval Required**
   - At least one maintainer approval
   - All CI checks must pass
   - Coverage must be ≥95%

### Getting Help

#### Documentation

- **[TRANSLATION_WORKFLOW.md](TRANSLATION_WORKFLOW.md)** - Complete workflow guide
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture
- **[CI_CD_SETUP.md](CI_CD_SETUP.md)** - CI/CD automation

#### Community

- **GitHub Issues**: Technical questions and bug reports
- **Discussions**: Feature requests and general questions
- **Slack**: Real-time help (if available)

---

## Next Steps

1. ✅ Complete Quick Start
2. ✅ Run First Translation Workflow
3. 📖 Read [TRANSLATION_WORKFLOW.md](TRANSLATION_WORKFLOW.md) for advanced usage
4. 🏗️ Review [ARCHITECTURE.md](ARCHITECTURE.md) for system internals
5. 🚀 Start contributing!

---

**Last Updated:** 2025-12-27
**Maintained By:** Ampel Development Team
**Questions?** Open an issue with `[i18n]` prefix
