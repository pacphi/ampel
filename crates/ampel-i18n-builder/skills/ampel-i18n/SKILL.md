---
name: ampel-i18n
description: Internationalize and localize applications using ampel-i18n-builder. Use when the user wants to (1) extract translatable strings from source code, (2) add multi-language support to a project, (3) translate an app into multiple languages, (4) generate i18n translation files, (5) check translation coverage or find missing translations, (6) sync translations across locales, or when user types /ampel-i18n:localize. Supports React/TypeScript (JSON), Rust (YAML), and Java/Spring (.properties).
---

# ampel-i18n

Automate internationalization (i18n) and localization (l10n) for any application using `ampel-i18n-builder`.

## ⚠️ PRE-FLIGHT CHECK (CRITICAL)

**Before doing ANYTHING else**, verify the tool is installed:

```bash
ampel-i18n --version
# OR
ampel-i18n --version
```

### If Command Not Found:

**STOP and redirect the user:**

> "You need to install ampel-i18n-builder first. I've created a step-by-step installation guide at `references/install-guide.md`.
>
> Would you like me to help you install it now? It takes about 5 minutes and I can guide you through each step."

**Then offer to help install:**

1. Check if Rust/Cargo is installed (`cargo --version`)
2. If missing, guide Rust installation (OS-specific commands)
3. Run `cargo install ampel-i18n-builder`
4. Verify with `ampel-i18n --version`

**DO NOT proceed with commands if tool is not installed.**

---

## Quick Start (Tool Already Installed)

1. Check if configuration exists:
   - Look for `.ampel-i18n.yaml` in project root
   - Look for `.env` file with translation provider credentials

2. If no config exists, create one using the template in `references/config-template.yaml`

3. Run the appropriate command based on user needs

**Essential Documentation:**

- **First-time install**: `references/install-guide.md` ⬅️ Start here if new
- **Quick start**: `references/getting-started.md`
- **Sample prompts**: `references/sample-prompts.md`
- **Full config reference**: `references/config-template.yaml`

## Commands

| Command                     | Purpose                                                   |
| --------------------------- | --------------------------------------------------------- |
| `ampel-i18n init`           | Interactive setup wizard for first-time users             |
| `ampel-i18n doctor`         | Health check - validate configuration and diagnose issues |
| `ampel-i18n extract`        | **NEW:** Extract translatable strings from source code    |
| `ampel-i18n sync`           | Generate/update translations for all configured languages |
| `ampel-i18n coverage`       | Show translation completion percentages per language      |
| `ampel-i18n missing`        | List all untranslated keys                                |
| `ampel-i18n report`         | Generate comprehensive translation status report          |
| `ampel-i18n generate-types` | Create TypeScript/Rust types from translation files       |

## Workflow

### For New Projects (With String Extraction)

**Recommended automated workflow:**

```bash
# 1. Initialize configuration
ampel-i18n init

# 2. Extract translatable strings from your code
ampel-i18n extract \
  --source frontend/src \
  --patterns "*.tsx" "*.ts" \
  --format json \
  --output frontend/public/locales/en/extracted.json \
  --merge

# 3. Translate to all languages
ampel-i18n sync
```

Or manual setup (without extraction):

1. Identify the i18n framework in use (i18next, react-intl, rust-i18n, etc.)
2. Locate existing translation files or create initial structure
3. Create `.ampel-i18n.yaml` configuration (see `references/config-template.yaml`)
4. Set up provider credentials in `.env` (see `references/env-template.txt`)
5. Run `ampel-i18n sync` to generate translations

### For Existing Projects

1. **Health check:** Run `ampel-i18n doctor` to validate setup
2. Run `ampel-i18n coverage` to assess current state
3. Run `ampel-i18n missing` to identify gaps
4. Run `ampel-i18n sync` to fill in missing translations

## Provider Setup

The tool uses a 4-tier fallback system. Configure at least one provider in `.env`:

| Tier | Provider         | Strength                  |
| ---- | ---------------- | ------------------------- |
| 1    | Systran          | Enterprise neural MT      |
| 2    | DeepL            | High-quality EU languages |
| 3    | Google Translate | Broad coverage            |
| 4    | OpenAI           | Complex content fallback  |

Only one provider is required. The system falls through tiers as needed.

## Supported Languages (27 total)

**Simple codes (21):** en, fr, de, it, ru, ja, ko, ar, he, hi, nl, pl, sr, th, tr, sv, da, fi, vi, no, cs

**Regional variants (6):** en-GB, pt-BR, zh-CN, zh-TW, es-ES, es-MX

**RTL support:** Arabic (ar), Hebrew (he) — handled automatically

## Common Patterns

**React/i18next project:**

```yaml
source_locale: en
locales_dir: src/locales
file_format: json
namespaces: [common, auth, dashboard]
```

**Rust project with rust-i18n:**

```yaml
source_locale: en
locales_dir: locales
file_format: yaml
namespaces: [errors, validation, providers]
```

**Vue/vue-i18n project:**

```yaml
source_locale: en
locales_dir: src/i18n/locales
file_format: json
```

## Type Generation

After syncing translations, generate type definitions for compile-time safety:

```bash
# TypeScript
ampel-i18n generate-types --lang typescript --output src/types/i18n.d.ts

# Rust
ampel-i18n generate-types --lang rust --output src/i18n_types.rs
```
