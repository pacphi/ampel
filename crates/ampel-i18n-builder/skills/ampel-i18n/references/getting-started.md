# Getting Started with ampel-i18n-builder

_Translate your app into 27 languages in under 10 minutes._

---

## ⚠️ First Time Here?

**If you haven't installed the tool yet**, read `install-guide.md` first. It walks you through:

- Installing Rust/Cargo (if needed)
- Installing ampel-i18n-builder
- Verifying everything works

**Already installed?** Continue below! ⬇️

---

## What You'll Need

- ✅ **ampel-i18n-builder installed** (verify with `ampel-i18n --version`)
- **Your project**: Any web or mobile app with text to translate
- **One API key**: From OpenAI, DeepL, Google, or Systran (pick one)
- **5-10 minutes**: Seriously, that's it

---

## Step 1: Create Your Config File

In your project's root folder, create a file called `.ampel-i18n.yaml`:

```yaml
# The language your app is currently written in
source_locale: en

# Where your translation files will live
locales_dir: src/locales

# File format (json or yaml)
file_format: json

# Languages you want to support
target_locales:
  - es # Spanish
  - fr # French
  - de # German
  - ja # Japanese
  - zh-CN # Chinese (Simplified)
```

> **Tip**: Start with 3-5 languages. You can add more anytime.

---

## Step 2: Add Your API Key

Create a `.env` file in your project root:

```bash
# Use whichever provider you have access to
OPENAI_API_KEY=sk-your-key-here
```

> **Don't have an API key?** OpenAI is the easiest to set up: https://platform.openai.com/api-keys

---

## Step 3: Run It!

```bash
ampel-i18n sync
# OR
ampel-i18n sync
```

> **Note**: Both commands work the same — `ampel-i18n` and `ampel-i18n` are aliases.

That's it! The tool will:

1. ✅ Scan your project for translatable text
2. ✅ Create translation files for each language
3. ✅ Translate everything automatically
4. ✅ Preserve any placeholders like `{userName}`

---

## 🤖 Even Easier: Use Claude Code

If you're using Claude Code, just type:

```
/ampel-i18n:localize
```

Then say something like:

> "Please internationalize my project. My config is in .ampel-i18n.yaml and credentials are in .env."

Claude will:

1. Check if the tool is installed (and help install if needed)
2. Validate your configuration
3. Run commands for you
4. Explain the results

**Complete beginner?** Try this prompt:

> "I just discovered ampel-i18n-builder and want to translate my [React/Vue/Rust] app. I have [OpenAI/DeepL/Google] API access. Walk me through setup from the very beginning."

---

## What Happens Next?

After running `sync`, you'll have:

```
src/locales/
├── en/
│   └── common.json     # Your original English
├── es/
│   └── common.json     # Spanish translations
├── fr/
│   └── common.json     # French translations
├── de/
│   └── common.json     # German translations
├── ja/
│   └── common.json     # Japanese translations
└── zh-CN/
    └── common.json     # Chinese translations
```

---

## Checking Your Progress

See how complete your translations are:

```bash
ampel-i18n coverage
```

Output:

```
Translation Coverage Report
===========================
en:    100% ████████████████████ (base)
es:     98% ███████████████████░
fr:     95% ███████████████████░
de:     97% ███████████████████░
ja:     92% ██████████████████░░
zh-CN:  94% ██████████████████░░
```

---

## Finding Missing Translations

```bash
ampel-i18n missing
```

Shows exactly which keys need attention:

```
Missing translations:
- ja: dashboard.newFeatureTitle
- ja: dashboard.newFeatureDescription
- zh-CN: settings.advancedOptions
```

---

## Supported Languages

| Flag | Language              | Code  |
| ---- | --------------------- | ----- |
| 🇺🇸   | English               | en    |
| 🇬🇧   | British English       | en-GB |
| 🇪🇸   | Spanish (Spain)       | es-ES |
| 🇲🇽   | Spanish (Mexico)      | es-MX |
| 🇫🇷   | French                | fr    |
| 🇩🇪   | German                | de    |
| 🇮🇹   | Italian               | it    |
| 🇧🇷   | Portuguese (Brazil)   | pt-BR |
| 🇷🇺   | Russian               | ru    |
| 🇯🇵   | Japanese              | ja    |
| 🇰🇷   | Korean                | ko    |
| 🇨🇳   | Chinese (Simplified)  | zh-CN |
| 🇹🇼   | Chinese (Traditional) | zh-TW |
| 🇸🇦   | Arabic (RTL)          | ar    |
| 🇮🇱   | Hebrew (RTL)          | he    |
| 🇮🇳   | Hindi                 | hi    |
| 🇳🇱   | Dutch                 | nl    |
| 🇵🇱   | Polish                | pl    |
| 🇷🇸   | Serbian               | sr    |
| 🇹🇭   | Thai                  | th    |
| 🇹🇷   | Turkish               | tr    |
| 🇸🇪   | Swedish               | sv    |
| 🇩🇰   | Danish                | da    |
| 🇫🇮   | Finnish               | fi    |
| 🇻🇳   | Vietnamese            | vi    |
| 🇳🇴   | Norwegian             | no    |
| 🇨🇿   | Czech                 | cs    |

---

## FAQ

**Q: How much does this cost?**
The tool is free. You only pay for the translation API you choose (OpenAI, DeepL, etc.). For a typical app, expect $5-20 for initial translation.

**Q: Will it mess up my code?**
No. The tool only creates/updates translation JSON files. It never touches your source code.

**Q: What about placeholders like `{userName}`?**
They're automatically protected. `Hello, {userName}` becomes `Hola, {userName}` — not `Hola, {nombreDeUsuario}`.

**Q: Can I edit the translations manually?**
Absolutely. The generated files are standard JSON/YAML. Edit them however you want.

**Q: What if a translation is wrong?**
Fix it in the file, and the tool will preserve your edit on future syncs.

---

## Advanced Configuration

### OpenAI Model Selection

You can customize which OpenAI model to use in your `.ampel-i18n.yaml`:

```yaml
translation:
  openai_api_key: '${OPENAI_API_KEY}'
  providers:
    openai:
      enabled: true
      model: gpt-5-mini # or gpt-4o, gpt-4o-mini, gpt-4-turbo, etc.
```

**Available models:**

- `gpt-5-mini` (default) - Fast, cost-efficient, 400k context ($0.25/1M input, $2/1M output)
- `gpt-5-mini-2025-08-07` - Latest snapshot
- `gpt-4o` - Advanced reasoning, multimodal
- `gpt-4o-mini` - Smaller, faster GPT-4
- `gpt-4-turbo` - Previous generation

See: https://platform.openai.com/docs/models

---

## Need Help?

- 📖 **Full config reference**: See `config-template.yaml`
- 🔑 **API key setup**: See `env-template.txt`
- 💬 **Sample prompts**: See `sample-prompts.md`

---

_Happy translating!_ 🌍
