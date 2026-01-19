# ⚡ Quick Reference Card

One-page cheatsheet for ampel-i18n-builder.

---

## 🔧 Installation (One-Time)

```bash
# 1. Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install ampel-i18n-builder
cargo install ampel-i18n-builder

# 3. Verify
ampel-i18n --version
```

**Don't want to type?** Tell Claude Code: `"Help me install ampel-i18n-builder"`

---

## 📋 Daily Commands

```bash
# Translate everything
ampel-i18n sync

# Check translation status
ampel-i18n coverage

# Find missing translations
ampel-i18n missing

# Generate TypeScript types
ampel-i18n generate-types --format typescript --output src/types/i18n.d.ts

# Generate full report
ampel-i18n report
```

---

## ⚙️ Minimal Config (.ampel-i18n.yaml)

```yaml
translation_dir: frontend/public/locales

translation:
  openai_api_key: '${OPENAI_API_KEY}'

  providers:
    openai:
      enabled: true
      model: gpt-5-mini
```

---

## 🔑 API Keys (.env)

```bash
# Pick ONE provider (or use multiple for fallback)
OPENAI_API_KEY=sk-...
DEEPL_API_KEY=...
GOOGLE_API_KEY=...
SYSTRAN_API_KEY=...
```

**Get API Keys:**

- OpenAI: https://platform.openai.com/api-keys (easiest)
- DeepL: https://www.deepl.com/pro-api (EU languages)
- Google: https://cloud.google.com/translate (broad coverage)
- Systran: https://platform.systran.net/ (enterprise)

---

## 🌍 Supported Languages

**21 Simple Codes:**
`en`, `fr`, `de`, `it`, `ru`, `ja`, `ko`, `ar`, `he`, `hi`, `nl`, `pl`, `sr`, `th`, `tr`, `sv`, `da`, `fi`, `vi`, `no`, `cs`

**6 Regional Variants:**
`en-GB`, `pt-BR`, `zh-CN`, `zh-TW`, `es-ES`, `es-MX`

---

## 🤖 Claude Code Prompts

### First Time Setup

```
/ampel-i18n:localize

I want to translate my [React/Vue/Rust] app.
I have [OpenAI/DeepL/Google] API access.
Walk me through the complete setup.
```

### Quick Translation

```
/ampel-i18n:localize

Translate my project using .ampel-i18n.yaml.
Show me coverage before and after.
```

### Add New Languages

```
/ampel-i18n:localize

Add support for Portuguese, Korean, and Arabic.
Update config and generate translations.
```

---

## 🆘 Troubleshooting

| Problem                         | Solution                                                |
| ------------------------------- | ------------------------------------------------------- |
| `cargo: command not found`      | Run: `source $HOME/.cargo/env` then restart terminal    |
| `ampel-i18n: command not found` | Verify install: `cargo install ampel-i18n-builder`      |
| `Permission denied`             | Run: `chmod +x ~/.cargo/bin/ampel-i18n`                 |
| `OpenAI API error 401`          | Check your API key in `.env`                            |
| Translations are wrong          | Use `--detect-untranslated` flag to force retranslation |

---

## 📚 Full Docs

- **Installation**: `install-guide.md`
- **Tutorial**: `getting-started.md`
- **Prompts**: `sample-prompts.md`
- **Config**: `config-template.yaml`

---

## 💡 Pro Tips

✅ **Start small**: 3-5 languages, expand later
✅ **Use namespaces**: Split by feature (auth, dashboard, etc.)
✅ **Generate types**: Catch typos at compile time
✅ **Run coverage often**: Find gaps early
✅ **One provider is enough**: OpenAI works for all languages

❌ **Don't commit API keys**: Use `.env` (not `.ampel-i18n.yaml`)
❌ **Don't translate placeholders**: Tool handles `{variables}` automatically
❌ **Don't edit generated files manually**: Re-run sync instead

---

_Confused? Ask Claude Code: `"Help me with ampel-i18n-builder"` and tell it what you're trying to do._
