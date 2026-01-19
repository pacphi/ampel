# Sample Prompts for ampel-i18n-builder

Use these prompts with Claude Code to quickly internationalize your project.

---

## 🚀 Quick Start Prompt

Copy and paste this into Claude Code:

```
/ampel-i18n:localize

Please internationalize my project. My configuration is in .ampel-i18n.yaml
and my translation provider credentials are in .env.

Target repo: [YOUR PROJECT PATH]

Start by checking the current coverage, then sync any missing translations.
```

---

## 📋 First-Time Setup Prompt

If you don't have config files yet:

```
/ampel-i18n:localize

I want to add multi-language support to my project at [YOUR PROJECT PATH].

This is a [React/Vue/Rust/etc.] project using [i18next/vue-i18n/rust-i18n/etc.].

Please:
1. Create an .ampel-i18n.yaml config file
2. Create an .env template for translation provider credentials
3. Set up the initial translation file structure
4. Run a coverage report to show me the current state

I want to support: English, Spanish, French, German, Japanese, and Chinese.
```

---

## 🔍 Extract Strings from Code (NEW!)

**For projects with hardcoded strings:**

```
/ampel-i18n:localize

Extract all translatable strings from my React app.

Source directory: frontend/src
File patterns: *.tsx, *.ts
Output: frontend/public/locales/en/extracted.json
Merge with existing translations: yes

Generate semantic keys like "button.save" and "error.invalidEmail".
```

**Extract from specific components:**

```
/ampel-i18n:localize

I just built a new user profile page with hardcoded strings.

Extract translatable text from:
- frontend/src/pages/Profile.tsx
- frontend/src/components/ProfileCard.tsx

Merge into frontend/public/locales/en/profile.json with semantic keys.
```

**Extract from Rust backend:**

```
/ampel-i18n:localize

Extract error messages from my Rust API:
- Source: crates/ampel-api/src
- Patterns: *.rs
- Format: yaml
- Output: crates/ampel-api/locales/en/errors.yaml

Find strings in anyhow!, bail!, and #[error(...)] macros.
```

---

## 🔧 Refactor Code to Use i18n (NEW!)

**After extracting strings, automatically refactor your code:**

```
/ampel-i18n:localize

I've extracted strings to frontend/public/locales/en/extracted.json.

Now refactor my React components to replace hardcoded strings with t() calls.

Target directory: frontend/src
Namespace: common
Create backups before modifying files.
```

**Refactor specific components:**

```
/ampel-i18n:localize

Refactor these files to use i18n:
- frontend/src/pages/Dashboard.tsx
- frontend/src/components/Sidebar.tsx

Use the mapping from frontend/public/locales/en/dashboard.json
Namespace: dashboard
```

**Preview changes before applying:**

```
/ampel-i18n:localize

Show me what changes would be made if I refactor my code to use i18n.

Target: frontend/src/components/Button.tsx
Mapping: frontend/public/locales/en/common.json

Run in dry-run mode so I can review before applying.
```

**Refactor Rust backend code:**

```
/ampel-i18n:localize

Refactor my Rust error messages to use the t! macro.

Target: crates/ampel-api/src
Mapping: crates/ampel-api/locales/en/errors.yaml
Patterns: *.rs

Replace anyhow! and #[error] strings with t! macro calls.
```

---

## 🔄 Sync Translations Prompt

For ongoing translation updates:

```
/ampel-i18n:localize

Please sync my translations using the config at .ampel-i18n.yaml.

Show me:
1. Current coverage per language
2. Any missing translations that were filled in
3. A summary of what changed
```

---

## 📊 Coverage Report Prompt

To check translation status:

```
/ampel-i18n:localize

Generate a translation coverage report for my project.

Show me:
- Overall coverage percentage
- Per-language breakdown
- List of any missing keys
- Recommendations for improvement
```

---

## 🏗️ Type Generation Prompt

For type-safe translations:

```
/ampel-i18n:localize

Generate TypeScript types from my translation files so I get
compile-time checking of translation keys.

Output to: src/types/i18n.d.ts
```

---

## 🆘 Troubleshooting Prompt

If something isn't working:

```
/ampel-i18n:localize

I'm having issues with my i18n setup. Please:
1. Validate my .ampel-i18n.yaml configuration
2. Check that my .env has valid provider credentials
3. Run a diagnostic to identify any problems
4. Suggest fixes
```

---

## 🌍 Add New Languages Prompt

To expand language support:

```
/ampel-i18n:localize

I want to add support for these new languages:
- Portuguese (Brazilian)
- Korean
- Arabic

Please:
1. Update my .ampel-i18n.yaml config
2. Generate the new translation files
3. Show me the updated coverage report
```

---

## 🔍 Find Untranslated Text Prompt

To audit your codebase:

```
/ampel-i18n:localize

Scan my codebase for any hardcoded strings that should be
internationalized but aren't yet.

Look in:
- src/components/
- src/pages/

Report what you find and suggest translation keys for each.
```

---

## 🎯 Complete Workflow (Extract → Refactor → Translate)

**For projects migrating to i18n from scratch:**

```
/ampel-i18n:localize

I want to internationalize my React app from scratch.

Step 1: Extract all hardcoded strings from frontend/src
Step 2: Refactor my code to replace strings with t() calls
Step 3: Translate everything to Spanish, French, and German
Step 4: Show me a coverage report

Use semantic keys like "button.save" and "error.invalidInput".
```

---

## 💡 Tips

- **Start small**: Begin with 3-5 languages, then expand
- **Use namespaces**: Split translations by feature (auth, dashboard, etc.)
- **Run coverage often**: Catch missing translations early
- **Generate types**: Prevent typos in translation keys
- **One provider is enough**: OpenAI works great if you already have an API key
- **Always preview refactoring**: Use `--dry-run` to see changes before applying
- **Backups are automatic**: Refactored files are backed up to `.ampel-i18n-backups/`
