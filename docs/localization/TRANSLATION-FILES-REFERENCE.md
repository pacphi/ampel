# Translation Files Reference

**Last Updated**: January 8, 2026
**Languages**: 27 total (en + 26 translations)

---

## Frontend Translation Files (JSON)

### File Structure

```
frontend/public/locales/
├── en/              # Source language (English)
│   ├── common.json      (120 keys, 69 lines)
│   ├── dashboard.json   (89 keys)
│   ├── settings.json    (67 keys)
│   ├── errors.json      (31 keys)
│   └── validation.json  (18 keys)
│
├── pt-BR/           # Portuguese (Brazil) - ✅ 100% Complete
│   ├── common.json      (120 lines, 3.4 KB)
│   ├── dashboard.json   (89 lines, 2.4 KB)
│   ├── settings.json    (111 lines, 4.5 KB)
│   ├── errors.json      (54 lines, 2.0 KB)
│   └── validation.json  (35 lines, 1.4 KB)
│
├── fr/              # French - ✅ 100% Complete
├── ar/              # Arabic - ⚠️ 63.7% Complete
├── es-ES/           # Spanish (Spain) - ⚠️ 64% Complete
├── es-MX/           # Spanish (Mexico) - ⚠️ 64% Complete
├── he/              # Hebrew - ⚠️ 64% Complete
├── zh-CN/           # Chinese (Simplified) - ⚠️ 64% Complete
├── en-GB/           # English (UK) - ⚠️ 39.7% Complete
├── de/              # German - ⚠️ 20.6% Complete
└── [... 18 more languages at 12-20% ...]
```

### Total Frontend Files

- **Namespaces**: 5 (common, dashboard, settings, errors, validation)
- **Languages**: 27 directories
- **Total Keys**: 325 per language
- **Total Files**: 135 JSON files (27 × 5)

---

## Backend Translation Files (YAML)

### File Structure

```
crates/ampel-api/locales/
├── en/              # Source language (English)
│   ├── common.yml       (157 keys)
│   ├── errors.yml       (45 keys)
│   ├── validation.yml   (25 keys)
│   └── providers.yml    (20 keys)
│
├── pt-BR/           # Portuguese (Brazil)
│   ├── common.yml       (9 lines, 269 bytes)   ⚠️ Partial
│   ├── errors.yml       (58 lines, 2.0 KB)     ✅ Complete
│   ├── providers.yml    (38 lines, 1.9 KB)     ✅ Complete
│   └── validation.yml   (11 lines, 235 bytes)  ⚠️ Partial
│
├── fr/              # French
├── ar/              # Arabic
├── es-ES/           # Spanish (Spain)
└── [... 24 more languages ...]
```

### Total Backend Files

- **Namespaces**: 4 (common, errors, validation, providers)
- **Languages**: 27 directories
- **Total Keys**: 90 per language
- **Total Files**: 108 YAML files (27 × 4)

---

## Example: pt-BR Translation Quality

### Frontend (dashboard.json)

**Nested Objects** - ✅ Preserved:

```json
{
  "blockers": {
    "awaitingReview": "Aguardando revisão",
    "changesRequested": "Alterações solicitadas",
    "ciFailed": "CI falhou"
  }
}
```

**Interpolation** - ✅ Placeholders Preserved:

```json
{
  "pr": {
    "number": "#{{number}}",
    "additions": "Adições"
  }
}
```

### Backend (errors.yml)

**Nested YAML** - ✅ Preserved:

```yaml
pt-BR:
  errors:
    auth:
      invalid_credentials: 'E-mail ou senha inválidos'
      user_not_found: 'Usuário não encontrado'
    repository:
      not_found: 'Repositório não encontrado'
      already_added: 'Repositório já adicionado'
```

**Variable Interpolation** - ✅ Preserved:

```yaml
pull_request:
  merge_failed: 'Merge falhou: %{reason}'
```

---

## Access Paths

### Frontend Files

```bash
# View English source
cat frontend/public/locales/en/common.json

# View Portuguese translation
cat frontend/public/locales/pt-BR/common.json

# View French translation
cat frontend/public/locales/fr/common.json

# List all frontend locales
ls -d frontend/public/locales/*/
```

### Backend Files

```bash
# View English source
cat crates/ampel-api/locales/en/errors.yml

# View Portuguese translation
cat crates/ampel-api/locales/pt-BR/errors.yml

# List all backend locales
ls -d crates/ampel-api/locales/*/
```

---

## Translation Features Supported

### ✅ Nested Objects

- Frontend: JSON nested structures
- Backend: YAML nested structures
- Example: `auth.login`, `errors.repository.not_found`

### ✅ Plural Forms

- i18next format: `_one`, `_other` suffixes
- Example: `pullRequests_one`, `pullRequests_other`
- Language-specific rules (Arabic has 6 forms!)

### ✅ Interpolation

- Frontend: `{{variable}}` format
- Backend: `%{variable}` format
- Placeholders preserved during translation

### ✅ Arrays

- Lists of strings
- Lists of objects

### ✅ RTL Languages

- Arabic (`ar`): Right-to-left text
- Hebrew (`he`): Right-to-left text
- Layout automatically flips

---

## File Sizes

### Frontend (per language when complete)

| Namespace  | Keys | File Size  | Compressed  |
| ---------- | ---- | ---------- | ----------- |
| common     | 120  | ~3.4 KB    | ~1.2 KB     |
| dashboard  | 89   | ~2.4 KB    | ~0.8 KB     |
| settings   | 67   | ~4.5 KB    | ~1.5 KB     |
| errors     | 31   | ~2.0 KB    | ~0.7 KB     |
| validation | 18   | ~1.4 KB    | ~0.5 KB     |
| **TOTAL**  | 325  | **~14 KB** | **~4.7 KB** |

### Backend (per language when complete)

| Namespace  | Keys | File Size   | Compressed  |
| ---------- | ---- | ----------- | ----------- |
| common     | 157  | ~4 KB       | ~1.5 KB     |
| errors     | 45   | ~2 KB       | ~0.7 KB     |
| validation | 25   | ~1.5 KB     | ~0.5 KB     |
| providers  | 20   | ~1 KB       | ~0.4 KB     |
| **TOTAL**  | 247  | **~8.5 KB** | **~3.1 KB** |

---

## Translation Commands

### Translate Single Language

```bash
# Frontend + Backend for German
cargo run -p ampel-i18n-builder --release -- translate --lang de

# Only frontend for French
cargo run -p ampel-i18n-builder --release -- translate --lang fr --translation-dir frontend/public/locales

# Only specific namespace
cargo run -p ampel-i18n-builder --release -- translate --lang ja --namespace settings
```

### Translate All Languages

```bash
# Dry run (preview only)
./translate-all-languages.sh --dry-run

# Actual translation
./translate-all-languages.sh
```

### Check Coverage

```bash
# Count keys in all languages
for lang in frontend/public/locales/*/; do
  echo "$lang: $(find "$lang" -name "*.json" -exec jq -r 'keys | length' {} \; | paste -sd+ | bc) keys"
done
```

---

## Translation Status Summary

### Fully Complete (100%)

- ✅ **pt-BR** (Portuguese Brazil) - 325/325 frontend keys
- ✅ **fr** (French) - 325/325 frontend keys

### High Coverage (60%+)

- ⚠️ **ar** (Arabic) - 207/325 keys (63.7%)
- ⚠️ **es-ES** (Spanish Spain) - 208/325 keys (64%)
- ⚠️ **es-MX** (Spanish Mexico) - 208/325 keys (64%)
- ⚠️ **he** (Hebrew) - 208/325 keys (64%)
- ⚠️ **zh-CN** (Chinese Simplified) - 208/325 keys (64%)

### Needs Translation (12-40%)

- 20 languages at 12-40% completion

---

## Quality Verification

### Check Translation Quality

```bash
# Compare English and Portuguese side-by-side
diff -y <(jq -S . frontend/public/locales/en/common.json) \
        <(jq -S . frontend/public/locales/pt-BR/common.json) | less

# Check for missing keys
cargo run -p ampel-i18n-builder --release -- validate --lang pt-BR

# Check coverage statistics
cargo run -p ampel-i18n-builder --release -- coverage
```

### Verify Placeholders

```bash
# Ensure {{placeholders}} are preserved
grep -r "{{" frontend/public/locales/pt-BR/

# Check backend %{variables}
grep -r "%{" crates/ampel-api/locales/pt-BR/
```

---

## Next Steps

1. **Complete remaining languages**: Run `./translate-all-languages.sh`
2. **Backend YAML translations**: Convert and translate backend files
3. **Quality review**: Native speaker review for top 5 languages
4. **Test RTL layouts**: Run visual regression tests for ar/he
5. **Test pluralization**: Verify plural forms work correctly

---

**See Also:**

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [TRANSLATION_WORKFLOW.md](./TRANSLATION_WORKFLOW.md) - Translation workflow guide
- [DEVELOPER-GUIDE.md](./DEVELOPER-GUIDE.md) - Developer quick start
