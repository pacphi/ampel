# Localization Enhancement Specification for Ampel

**Version:** 2.0
**Date:** 2025-12-27
**Status:** Specification
**Based On:** [LOCALIZATION_IMPLEMENTATION_PLAN.md](./LOCALIZATION_IMPLEMENTATION_PLAN.md)

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Extended Language Support Specification](#extended-language-support-specification)
3. [Translation Automation Crate Specification](#translation-automation-crate-specification)
4. [Enhanced Language Switcher Specification](#enhanced-language-switcher-specification)
5. [Integration Requirements](#integration-requirements)
6. [Quality Assurance Criteria](#quality-assurance-criteria)
7. [Acceptance Tests](#acceptance-tests)

---

## Executive Summary

This specification document outlines enhancements to the Ampel localization system, extending support from 13 to 20 languages, introducing automated translation workflows through a new `ampel-i18n-builder` crate, and implementing an enhanced language switcher with flag icons and localized tooltips.

### Enhancement Goals

1. **Extended Language Coverage**: Add 7 new languages (Finnish, Swedish, Norwegian, Thai, Arabic, Danish, Czech)
2. **Translation Automation**: Build Rust crate for API-driven translation bundle generation
3. **Enhanced UX**: Implement visual language switcher with flags, ISO codes, and tooltips

### Key Deliverables

- Support for 20 total languages (existing 13 + new 7)
- `ampel-i18n-builder` crate with translation API integrations
- Enhanced language switcher React component
- Comprehensive test coverage for new languages
- Updated CI/CD validation for 20 languages

---

## Extended Language Support Specification

### 1. New Languages Overview

| Language      | Code | Script | Direction | Pluralization      | Priority | Special Requirements     |
| ------------- | ---- | ------ | --------- | ------------------ | -------- | ------------------------ |
| **Finnish**   | fi   | Latin  | LTR       | Standard (2 forms) | Phase 2  | Complex compound words   |
| **Swedish**   | sv   | Latin  | LTR       | Standard (2 forms) | Phase 2  | Similar to Norwegian     |
| **Norwegian** | no   | Latin  | LTR       | Standard (2 forms) | Phase 2  | Bokmål variant (nb)      |
| **Thai**      | th   | Thai   | LTR       | None               | Phase 3  | Complex character set    |
| **Arabic**    | ar   | Arabic | **RTL**   | Complex (6 forms)  | Phase 3  | **RTL testing critical** |
| **Danish**    | da   | Latin  | LTR       | Standard (2 forms) | Phase 2  | Similar to Norwegian     |
| **Czech**     | cs   | Latin  | LTR       | Complex (3 forms)  | Phase 3  | Diacritics               |

### 2. Detailed Language Requirements

#### 2.1 Finnish (fi)

**Functional Requirements:**

- FR-FI-001: Support standard Finnish pluralization (one, other)
- FR-FI-002: Handle compound word translations without breaking semantics
- FR-FI-003: Support Finnish date format (DD.MM.YYYY)
- FR-FI-004: Support Finnish number format (space as thousands separator)

**Non-Functional Requirements:**

- NFR-FI-001: Translation files must use UTF-8 encoding for Finnish characters (ä, ö, å)
- NFR-FI-002: UI elements must accommodate 30% longer text than English (compound words)

**Pluralization Rules:**

```yaml
# locales/fi/common.yml
pull_requests:
  count:
    one: '%{count} pull request' # 1
    other: '%{count} pull requestia' # 0, 2-∞
```

**Character Set Considerations:**

- Finnish alphabet includes: å, ä, ö
- Case sensitivity: Å/å, Ä/ä, Ö/ö
- Collation: Special characters sort after z

---

#### 2.2 Swedish (sv)

**Functional Requirements:**

- FR-SV-001: Support standard Swedish pluralization (one, other)
- FR-SV-002: Support Swedish date format (YYYY-MM-DD)
- FR-SV-003: Support Swedish number format (space as thousands separator)

**Non-Functional Requirements:**

- NFR-SV-001: Translation files must use UTF-8 encoding for Swedish characters (å, ä, ö)
- NFR-SV-002: UI elements must accommodate 20% longer text than English

**Pluralization Rules:**

```yaml
# locales/sv/common.yml
pull_requests:
  count:
    one: '%{count} pull request' # 1
    other: '%{count} pull requests' # 0, 2-∞
```

**Character Set Considerations:**

- Swedish alphabet includes: å, ä, ö
- Similar to Finnish but different grammatical rules
- Collation: å, ä, ö sort after z

---

#### 2.3 Norwegian (no/nb)

**Functional Requirements:**

- FR-NO-001: Support Norwegian Bokmål (nb) as primary variant
- FR-NO-002: Support standard Norwegian pluralization (one, other)
- FR-NO-003: Support Norwegian date format (DD.MM.YYYY)
- FR-NO-004: Support Norwegian number format (space as thousands separator)

**Non-Functional Requirements:**

- NFR-NO-001: Use `nb` (Bokmål) code, with `no` as fallback alias
- NFR-NO-002: Translation files must use UTF-8 encoding for Norwegian characters (æ, ø, å)
- NFR-NO-003: UI elements must accommodate 20% longer text than English

**Pluralization Rules:**

```yaml
# locales/nb/common.yml
pull_requests:
  count:
    one: '%{count} pull request' # 1
    other: '%{count} pull requests' # 0, 2-∞
```

**Character Set Considerations:**

- Norwegian alphabet includes: æ, ø, å
- Bokmål vs Nynorsk: Support Bokmål only initially
- Collation: æ, ø, å sort after z

**Locale Code Normalization:**

```rust
// middleware/locale.rs
fn normalize_locale(locale: &str) -> Option<&str> {
    match locale.to_lowercase().as_str() {
        "no" | "nb" | "no-no" | "nb-no" => Some("nb"),
        // ... other languages
        _ => None,
    }
}
```

---

#### 2.4 Thai (th)

**Functional Requirements:**

- FR-TH-001: Support Thai script rendering (no pluralization needed)
- FR-TH-002: Support Thai Buddhist calendar (B.E.) in date formats
- FR-TH-003: Support Thai number format (comma as thousands separator)
- FR-TH-004: Handle Thai text wrapping (no spaces between words)

**Non-Functional Requirements:**

- NFR-TH-001: Translation files must use UTF-8 encoding for Thai characters
- NFR-TH-002: Fonts must support Thai Unicode range (U+0E00 to U+0E7F)
- NFR-TH-003: UI elements must accommodate 40% shorter text than English
- NFR-TH-004: Line breaking must use Thai dictionary-based algorithm

**Pluralization Rules:**

```yaml
# locales/th/common.yml
pull_requests:
  count: '%{count} pull request' # No plural forms in Thai
```

**Character Set Considerations:**

- Thai script: consonants (44), vowels (15), tone marks (4)
- No spaces between words (requires word segmentation)
- Complex character composition (base + above/below marks)
- Fonts required: Noto Sans Thai, Sarabun, Prompt

**Text Rendering Considerations:**

```typescript
// CSS for Thai text
.thai-text {
  font-family: 'Noto Sans Thai', 'Sarabun', sans-serif;
  word-break: keep-all;  // Prevent breaking mid-word
  overflow-wrap: break-word;
  line-height: 1.8;  // Accommodate tone marks
}
```

**Date Formatting:**

```typescript
// Thai Buddhist Era (B.E.) = Gregorian year + 543
const formatThaiDate = (date: Date): string => {
  const buddhistYear = date.getFullYear() + 543;
  return new Intl.DateTimeFormat('th-TH-u-ca-buddhist', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(date);
};
// Example output: "27 ธันวาคม พ.ศ. 2568"
```

---

#### 2.5 Arabic (ar)

**Functional Requirements:**

- FR-AR-001: Support full RTL (right-to-left) text direction
- FR-AR-002: Support Arabic pluralization (6 forms: zero, one, two, few, many, other)
- FR-AR-003: Support Arabic date format (DD/MM/YYYY)
- FR-AR-004: Support Arabic numeral rendering (Eastern Arabic: ٠١٢٣٤٥٦٧٨٩)
- FR-AR-005: Support bidirectional text (Bidi) for mixed LTR/RTL content

**Non-Functional Requirements:**

- NFR-AR-001: All UI layouts must mirror for RTL (existing Hebrew infrastructure)
- NFR-AR-002: Translation files must use UTF-8 encoding for Arabic characters
- NFR-AR-003: Fonts must support Arabic ligatures and contextual forms
- NFR-AR-004: Performance: RTL layout rendering must be <100ms
- NFR-AR-005: UI elements must accommodate 30% longer text than English

**Pluralization Rules:**

```yaml
# locales/ar/common.yml
pull_requests:
  count:
    zero: 'لا توجد pull requests' # 0
    one: 'pull request واحد' # 1
    two: 'pull requestان' # 2
    few: '%{count} pull requests' # 3-10
    many: '%{count} pull request' # 11-99
    other: '%{count} pull request' # 100+
```

**Frontend Pluralization (react-i18next):**

```json
{
  "pullRequests": {
    "count_zero": "لا توجد pull requests",
    "count_one": "{{count}} pull request واحد",
    "count_two": "{{count}} pull requestان",
    "count_few": "{{count}} pull requests",
    "count_many": "{{count}} pull request",
    "count_other": "{{count}} pull request"
  }
}
```

**RTL Implementation Requirements:**

1. **Document Direction:**

```typescript
// src/i18n/config.ts
export const SUPPORTED_LANGUAGES = {
  // ... existing languages
  ar: { name: 'العربية', dir: 'rtl', numeralSystem: 'arab' },
} as const;
```

2. **CSS Logical Properties:**

```css
/* Use logical properties for RTL support */
.container {
  margin-inline-start: 1rem; /* Not margin-left */
  padding-inline-end: 1rem; /* Not padding-right */
  border-inline-start: 1px solid; /* Not border-left */
}

/* RTL-specific overrides */
[dir='rtl'] .icon {
  transform: scaleX(-1); /* Flip directional icons */
}
```

3. **Bidirectional Text Handling:**

```typescript
// For mixed LTR/RTL content (e.g., code snippets in Arabic UI)
<span dir="ltr">{codeSnippet}</span>
<span dir="auto">{userInput}</span>  // Auto-detect direction
```

**Character Set Considerations:**

- Arabic alphabet: 28 letters with 4 forms each (isolated, initial, medial, final)
- Ligatures: Automatic connecting between letters
- Diacritics: Optional vowel marks (َ ِ ُ)
- Eastern Arabic numerals: ٠١٢٣٤٥٦٧٨٩ (vs Western: 0123456789)

**Font Requirements:**

```css
/* Arabic font stack */
.arabic-text {
  font-family: 'Noto Sans Arabic', 'Dubai', 'Tajawal', 'Amiri', sans-serif;
  font-feature-settings:
    'liga' 1,
    'calt' 1; /* Enable ligatures */
}
```

**Testing Requirements:**

- Visual regression tests for RTL layout
- Test Hebrew + Arabic together (both RTL)
- Test mixed LTR/RTL content (URLs, code, numbers)
- Test all dashboard views in Arabic
- Verify icon mirroring (arrows, chevrons)

---

#### 2.6 Danish (da)

**Functional Requirements:**

- FR-DA-001: Support standard Danish pluralization (one, other)
- FR-DA-002: Support Danish date format (DD-MM-YYYY)
- FR-DA-003: Support Danish number format (dot as thousands separator, comma as decimal)

**Non-Functional Requirements:**

- NFR-DA-001: Translation files must use UTF-8 encoding for Danish characters (æ, ø, å)
- NFR-DA-002: UI elements must accommodate 20% longer text than English

**Pluralization Rules:**

```yaml
# locales/da/common.yml
pull_requests:
  count:
    one: '%{count} pull request' # 1
    other: '%{count} pull requests' # 0, 2-∞
```

**Character Set Considerations:**

- Danish alphabet includes: æ, ø, å
- Similar to Norwegian but different word usage
- Collation: æ, ø, å sort after z

---

#### 2.7 Czech (cs)

**Functional Requirements:**

- FR-CS-001: Support Czech pluralization (3 forms: one, few, many)
- FR-CS-002: Support Czech date format (DD.MM.YYYY)
- FR-CS-003: Support Czech number format (space as thousands separator, comma as decimal)
- FR-CS-004: Handle Czech diacritics in search and sorting

**Non-Functional Requirements:**

- NFR-CS-001: Translation files must use UTF-8 encoding for Czech diacritics
- NFR-CS-002: UI elements must accommodate 25% longer text than English
- NFR-CS-003: Search must be diacritic-insensitive (e.g., "e" matches "é", "ě")

**Pluralization Rules:**

```yaml
# locales/cs/common.yml
pull_requests:
  count:
    one: '%{count} pull request' # 1
    few: '%{count} pull requesty' # 2-4
    many: '%{count} pull requestů' # 0, 5-∞
```

**Frontend Pluralization:**

```json
{
  "pullRequests": {
    "count_one": "{{count}} pull request",
    "count_few": "{{count}} pull requesty",
    "count_other": "{{count}} pull requestů"
  }
}
```

**Character Set Considerations:**

- Czech diacritics: á, č, ď, é, ě, í, ň, ó, ř, š, ť, ú, ů, ý, ž
- Uppercase variants: Á, Č, Ď, É, Ě, Í, Ň, Ó, Ř, Š, Ť, Ú, Ů, Ý, Ž
- Collation: Special characters have unique sort order (č after c, not after c)

---

### 3. Updated Language Support Matrix

| Language        | Code   | Script     | Direction | Pluralization | Phase | RTL |
| --------------- | ------ | ---------- | --------- | ------------- | ----- | --- |
| English         | en     | Latin      | LTR       | 2 forms       | 1     | ❌  |
| Portuguese (BR) | pt-BR  | Latin      | LTR       | 2 forms       | 1     | ❌  |
| Spanish (ES)    | es-ES  | Latin      | LTR       | 2 forms       | 1     | ❌  |
| Dutch           | nl     | Latin      | LTR       | 2 forms       | 1     | ❌  |
| German          | de     | Latin      | LTR       | 2 forms       | 1     | ❌  |
| Serbian         | sr     | Cyrillic   | LTR       | 2 forms       | 1     | ❌  |
| Russian         | ru     | Cyrillic   | LTR       | 3 forms       | 1     | ❌  |
| Hebrew          | he     | Hebrew     | RTL       | 2 forms       | 1     | ✅  |
| French          | fr     | Latin      | LTR       | 2 forms       | 1     | ❌  |
| Italian         | it     | Latin      | LTR       | 2 forms       | 1     | ❌  |
| Polish          | pl     | Latin      | LTR       | 3 forms       | 1     | ❌  |
| Chinese (CN)    | zh-CN  | Han        | LTR       | None          | 1     | ❌  |
| Japanese        | ja     | Han/Kana   | LTR       | None          | 1     | ❌  |
| **Finnish**     | **fi** | **Latin**  | **LTR**   | **2 forms**   | **2** | ❌  |
| **Swedish**     | **sv** | **Latin**  | **LTR**   | **2 forms**   | **2** | ❌  |
| **Norwegian**   | **nb** | **Latin**  | **LTR**   | **2 forms**   | **2** | ❌  |
| **Danish**      | **da** | **Latin**  | **LTR**   | **2 forms**   | **2** | ❌  |
| **Thai**        | **th** | **Thai**   | **LTR**   | **None**      | **3** | ❌  |
| **Arabic**      | **ar** | **Arabic** | **RTL**   | **6 forms**   | **3** | ✅  |
| **Czech**       | **cs** | **Latin**  | **LTR**   | **3 forms**   | **3** | ❌  |

**Total: 20 languages (13 existing + 7 new)**

---

## Translation Automation Crate Specification

### 1. Overview

The `ampel-i18n-builder` crate provides automated translation management through integration with translation APIs (Google Cloud Translation, DeepL, Amazon Translate).

### 2. Crate Structure

```
crates/ampel-i18n-builder/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API
│   ├── providers/
│   │   ├── mod.rs                # Provider trait
│   │   ├── google.rs             # Google Cloud Translation
│   │   ├── deepl.rs              # DeepL API
│   │   └── aws.rs                # Amazon Translate
│   ├── formats/
│   │   ├── mod.rs                # Format trait
│   │   ├── yaml.rs               # YAML parser/writer
│   │   └── json.rs               # JSON parser/writer
│   ├── config.rs                 # Configuration
│   ├── cache.rs                  # Translation cache
│   └── cli.rs                    # CLI interface
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/
│       └── sample_translations/
└── examples/
    └── translate_batch.rs
```

### 3. Functional Requirements

#### FR-I18N-001: Translation Provider Abstraction

**Description:** Support multiple translation providers through a trait-based abstraction.

**Interface:**

```rust
#[async_trait]
pub trait TranslationProvider: Send + Sync {
    /// Translate a single text from source to target language
    async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String, TranslationError>;

    /// Translate multiple texts in batch
    async fn translate_batch(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>, TranslationError>;

    /// Get supported languages
    async fn supported_languages(&self) -> Result<Vec<Language>, TranslationError>;
}

pub struct Language {
    pub code: String,
    pub name: String,
    pub native_name: String,
}
```

**Acceptance Criteria:**

- ✅ Implement trait for Google Cloud Translation API
- ✅ Implement trait for DeepL API
- ✅ Implement trait for Amazon Translate
- ✅ Support batch translation (100+ strings)
- ✅ Handle API rate limiting with exponential backoff
- ✅ Provide error context (which string failed, why)

---

#### FR-I18N-002: Format Handling

**Description:** Parse and write translation files in YAML (backend) and JSON (frontend).

**Interface:**

```rust
pub trait TranslationFormat {
    /// Parse translation file into key-value map
    fn parse(&self, content: &str) -> Result<TranslationMap, FormatError>;

    /// Write key-value map to translation file
    fn write(&self, map: &TranslationMap) -> Result<String, FormatError>;

    /// Validate format schema
    fn validate(&self, content: &str) -> Result<(), FormatError>;
}

pub type TranslationMap = BTreeMap<String, TranslationValue>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationValue {
    String(String),
    Plural(PluralForms),
    Nested(TranslationMap),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluralForms {
    pub zero: Option<String>,
    pub one: Option<String>,
    pub two: Option<String>,
    pub few: Option<String>,
    pub many: Option<String>,
    pub other: String,
}
```

**Acceptance Criteria:**

- ✅ Parse YAML translation files (rust-i18n format)
- ✅ Parse JSON translation files (react-i18next format)
- ✅ Preserve nested structure and pluralization rules
- ✅ Maintain key order (use BTreeMap)
- ✅ Validate against schema before writing

---

#### FR-I18N-003: Translation Cache

**Description:** Cache translations to avoid redundant API calls and reduce costs.

**Interface:**

```rust
pub struct TranslationCache {
    cache: Arc<Mutex<HashMap<CacheKey, String>>>,
    storage: CacheStorage,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct CacheKey {
    text: String,
    source_lang: String,
    target_lang: String,
    provider: String,
}

pub enum CacheStorage {
    Memory,
    File(PathBuf),
    Redis(String),  // Redis connection URL
}

impl TranslationCache {
    pub fn new(storage: CacheStorage) -> Self;
    pub async fn get(&self, key: &CacheKey) -> Option<String>;
    pub async fn set(&self, key: CacheKey, value: String);
    pub async fn clear(&self);
}
```

**Acceptance Criteria:**

- ✅ In-memory cache for runtime translations
- ✅ File-based cache for persistence (JSON)
- ✅ Optional Redis cache for production
- ✅ Cache hit rate >80% for repeated translations
- ✅ Automatic cache invalidation (TTL: 30 days)

---

#### FR-I18N-004: CLI Interface

**Description:** Provide CLI tool for translation automation workflows.

**Commands:**

```bash
# Translate all missing keys for a language
cargo i18n translate --lang fi --provider deepl

# Translate specific namespace
cargo i18n translate --lang fi --namespace dashboard --provider google

# Update all languages from English source
cargo i18n sync --source en --provider deepl

# Check translation coverage
cargo i18n coverage --lang fi

# Validate translation files
cargo i18n validate --all

# Export for external translation service
cargo i18n export --lang fi --format xliff --output translations.xliff

# Import from external translation service
cargo i18n import --lang fi --format xliff --input translations.xliff
```

**CLI Options:**

```rust
#[derive(Parser)]
#[command(name = "cargo-i18n")]
#[command(about = "Translation automation for Ampel")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Translate missing keys
    Translate {
        #[arg(short, long)]
        lang: String,

        #[arg(short, long)]
        provider: TranslationProvider,

        #[arg(short, long)]
        namespace: Option<String>,

        #[arg(long)]
        dry_run: bool,
    },

    /// Sync all languages from source
    Sync {
        #[arg(short, long, default_value = "en")]
        source: String,

        #[arg(short, long)]
        provider: TranslationProvider,
    },

    /// Check translation coverage
    Coverage {
        #[arg(short, long)]
        lang: Option<String>,

        #[arg(long)]
        min_coverage: Option<f32>,
    },

    /// Validate translation files
    Validate {
        #[arg(long)]
        all: bool,

        #[arg(short, long)]
        lang: Option<String>,
    },

    /// Export for external translation
    Export {
        #[arg(short, long)]
        lang: String,

        #[arg(short, long)]
        format: ExportFormat,

        #[arg(short, long)]
        output: PathBuf,
    },

    /// Import from external translation
    Import {
        #[arg(short, long)]
        lang: String,

        #[arg(short, long)]
        format: ExportFormat,

        #[arg(short, long)]
        input: PathBuf,
    },
}
```

**Acceptance Criteria:**

- ✅ Translate command completes in <5 minutes for 500 keys
- ✅ Sync command updates all 20 languages
- ✅ Coverage command shows percentage per language
- ✅ Validate command catches schema errors
- ✅ Export/Import support XLIFF format

---

#### FR-I18N-005: Configuration Management

**Description:** Configure translation providers and options via config file.

**Configuration File:**

```toml
# .i18n-config.toml
[translation]
default_provider = "deepl"
source_language = "en"
target_languages = ["fi", "sv", "nb", "th", "ar", "da", "cs"]

[cache]
enabled = true
storage = "file"
path = ".i18n-cache"
ttl_days = 30

[providers.deepl]
api_key_env = "DEEPL_API_KEY"
api_url = "https://api-free.deepl.com/v2/translate"
formality = "default"  # default | more | less

[providers.google]
api_key_env = "GOOGLE_TRANSLATE_API_KEY"
project_id_env = "GOOGLE_PROJECT_ID"

[providers.aws]
access_key_env = "AWS_ACCESS_KEY_ID"
secret_key_env = "AWS_SECRET_ACCESS_KEY"
region = "us-east-1"

[backend]
locales_dir = "crates/ampel-api/locales"
format = "yaml"

[frontend]
locales_dir = "frontend/public/locales"
format = "json"
namespaces = ["common", "dashboard", "settings", "errors", "validation"]
```

**Acceptance Criteria:**

- ✅ Load configuration from `.i18n-config.toml`
- ✅ Override config with environment variables
- ✅ Validate configuration on startup
- ✅ Support multiple provider credentials
- ✅ Error if API keys are missing

---

### 4. Non-Functional Requirements

#### NFR-I18N-001: Performance

- Translation batch processing: >100 strings per API call
- Cache hit rate: >80% for repeated translations
- CLI command execution: <5 minutes for 500 keys per language

#### NFR-I18N-002: Security

- Never log API keys or credentials
- Store credentials in environment variables only
- Encrypt cache files if using Redis

#### NFR-I18N-003: Reliability

- Retry failed API calls with exponential backoff (3 attempts)
- Handle rate limiting (429 errors) gracefully
- Provide detailed error messages with context

#### NFR-I18N-004: Maintainability

- 100% test coverage for core translation logic
- Integration tests for each provider
- Documentation for adding new providers

---

### 5. Dependencies

```toml
# crates/ampel-i18n-builder/Cargo.toml
[package]
name = "ampel-i18n-builder"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1.43", features = ["full"] }
async-trait = "0.1"

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# CLI
clap = { version = "4.5", features = ["derive"] }

# Configuration
config = "0.14"
toml = "0.8"

# Caching
redis = { version = "0.28", optional = true }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Utilities
chrono = "0.4"

[dev-dependencies]
tokio-test = "0.4"
mockito = "1.6"

[features]
default = []
redis-cache = ["redis"]
```

---

### 6. Testing Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_google_translate_single() {
        let provider = GoogleTranslateProvider::new("test-key");
        let result = provider.translate("Hello", "en", "fi").await;
        assert_eq!(result.unwrap(), "Hei");
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = TranslationCache::new(CacheStorage::Memory);
        let key = CacheKey { /* ... */ };
        cache.set(key.clone(), "cached value".to_string()).await;
        let result = cache.get(&key).await;
        assert_eq!(result, Some("cached value".to_string()));
    }

    #[test]
    fn test_yaml_parser() {
        let yaml = r#"
        hello: "Hei"
        pull_requests:
          count:
            one: "1 pull request"
            other: "%{count} pull requestia"
        "#;
        let format = YamlFormat;
        let map = format.parse(yaml).unwrap();
        assert_eq!(map.get("hello"), Some(&TranslationValue::String("Hei".to_string())));
    }
}
```

#### Integration Tests

```rust
#[tokio::test]
#[ignore]  // Requires API key
async fn test_deepl_batch_translation() {
    let api_key = std::env::var("DEEPL_API_KEY").expect("DEEPL_API_KEY not set");
    let provider = DeepLProvider::new(&api_key);

    let texts = vec![
        "Dashboard".to_string(),
        "Settings".to_string(),
        "Pull Requests".to_string(),
    ];

    let results = provider.translate_batch(&texts, "en", "fi").await.unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], "Kojelauta");
    assert_eq!(results[1], "Asetukset");
}
```

---

## Enhanced Language Switcher Specification

### 1. Overview

Enhanced language switcher component with flag icons, ISO-639 language codes, localized tooltips, and improved UX.

### 2. Functional Requirements

#### FR-LS-001: Visual Flag Icons

**Description:** Display country/region flags for visual language identification.

**Requirements:**

- Use emoji flags for lightweight implementation (no image assets)
- Map language codes to flag emojis
- Fallback to language code if flag unavailable

**Flag Mapping:**

```typescript
// src/i18n/flags.ts
export const LANGUAGE_FLAGS: Record<SupportedLanguage, string> = {
  en: '🇬🇧', // English - UK flag
  'pt-BR': '🇧🇷', // Portuguese - Brazil flag
  'es-ES': '🇪🇸', // Spanish - Spain flag
  nl: '🇳🇱', // Dutch - Netherlands flag
  de: '🇩🇪', // German - Germany flag
  sr: '🇷🇸', // Serbian - Serbia flag
  ru: '🇷🇺', // Russian - Russia flag
  he: '🇮🇱', // Hebrew - Israel flag
  fr: '🇫🇷', // French - France flag
  it: '🇮🇹', // Italian - Italy flag
  pl: '🇵🇱', // Polish - Poland flag
  'zh-CN': '🇨🇳', // Chinese - China flag
  ja: '🇯🇵', // Japanese - Japan flag
  fi: '🇫🇮', // Finnish - Finland flag
  sv: '🇸🇪', // Swedish - Sweden flag
  nb: '🇳🇴', // Norwegian - Norway flag
  th: '🇹🇭', // Thai - Thailand flag
  ar: '🇸🇦', // Arabic - Saudi Arabia flag
  da: '🇩🇰', // Danish - Denmark flag
  cs: '🇨🇿', // Czech - Czech Republic flag
};
```

**Acceptance Criteria:**

- ✅ Display flag emoji next to language name
- ✅ Flags render correctly across browsers
- ✅ Fallback to language code if emoji not supported

---

#### FR-LS-002: ISO-639 Language Codes

**Description:** Display ISO-639 language codes alongside language names.

**Requirements:**

```typescript
// src/i18n/config.ts
export const SUPPORTED_LANGUAGES = {
  en: {
    name: 'English',
    nativeName: 'English',
    isoCode: 'en',
    dir: 'ltr',
  },
  'pt-BR': {
    name: 'Portuguese (Brazil)',
    nativeName: 'Português (Brasil)',
    isoCode: 'pt-BR',
    dir: 'ltr',
  },
  fi: {
    name: 'Finnish',
    nativeName: 'Suomi',
    isoCode: 'fi',
    dir: 'ltr',
  },
  ar: {
    name: 'Arabic',
    nativeName: 'العربية',
    isoCode: 'ar',
    dir: 'rtl',
  },
  // ... other languages
} as const;
```

**Display Format:**

- Primary: `🇫🇮 Suomi (fi)`
- Compact: `🇫🇮 fi`
- Verbose: `🇫🇮 Finnish - Suomi (fi)`

**Acceptance Criteria:**

- ✅ Show ISO code in parentheses
- ✅ Support compact mode for mobile
- ✅ ISO codes follow ISO-639-1 standard

---

#### FR-LS-003: Localized Tooltips

**Description:** Show tooltip with language information in user's current language.

**Translation Keys:**

```json
// public/locales/en/common.json
{
  "languageSwitcher": {
    "title": "Change Language",
    "current": "Current language",
    "select": "Select a language",
    "languages": {
      "en": "English",
      "fi": "Finnish",
      "ar": "Arabic",
      "th": "Thai"
      // ... other languages
    },
    "tooltips": {
      "en": "Switch to English",
      "fi": "Switch to Finnish (Suomi)",
      "ar": "Switch to Arabic (العربية) - RTL",
      "th": "Switch to Thai (ไทย)"
      // ... other languages
    }
  }
}
```

**Acceptance Criteria:**

- ✅ Tooltip shows on hover (desktop) and long-press (mobile)
- ✅ Tooltip text translates with current language
- ✅ Tooltip indicates RTL languages

---

#### FR-LS-004: Enhanced UI/UX

**Description:** Improved visual design and interaction patterns.

**Requirements:**

- Dropdown menu with search/filter capability
- Keyboard navigation support (arrow keys, Enter, Escape)
- Active language indicator (checkmark or highlight)
- Grouped languages (Common, Regional, RTL)
- Responsive design (desktop, tablet, mobile)

**Component Structure:**

```typescript
// src/components/LanguageSwitcher.tsx
export function LanguageSwitcher() {
  const { i18n, t } = useTranslation();
  const [searchQuery, setSearchQuery] = useState('');

  const filteredLanguages = useMemo(() => {
    const query = searchQuery.toLowerCase();
    return Object.entries(SUPPORTED_LANGUAGES).filter(([code, info]) =>
      info.name.toLowerCase().includes(query) ||
      info.nativeName.toLowerCase().includes(query) ||
      code.toLowerCase().includes(query)
    );
  }, [searchQuery]);

  const groupedLanguages = useMemo(() => {
    const common = ['en', 'es-ES', 'fr', 'de', 'pt-BR'];
    const rtl = ['he', 'ar'];

    return {
      common: filteredLanguages.filter(([code]) => common.includes(code)),
      rtl: filteredLanguages.filter(([code]) => rtl.includes(code)),
      other: filteredLanguages.filter(([code]) =>
        !common.includes(code) && !rtl.includes(code)
      ),
    };
  }, [filteredLanguages]);

  return (
    <Select value={i18n.language} onValueChange={(lang) => i18n.changeLanguage(lang)}>
      <SelectTrigger className="w-[200px]">
        <SelectValue>
          <LanguageDisplay code={i18n.language} />
        </SelectValue>
      </SelectTrigger>
      <SelectContent>
        <div className="p-2">
          <Input
            placeholder={t('languageSwitcher.select')}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        {groupedLanguages.common.length > 0 && (
          <>
            <SelectLabel>{t('languageSwitcher.groups.common')}</SelectLabel>
            {groupedLanguages.common.map(([code, info]) => (
              <LanguageSelectItem key={code} code={code} info={info} />
            ))}
          </>
        )}

        {groupedLanguages.rtl.length > 0 && (
          <>
            <SelectSeparator />
            <SelectLabel>{t('languageSwitcher.groups.rtl')}</SelectLabel>
            {groupedLanguages.rtl.map(([code, info]) => (
              <LanguageSelectItem key={code} code={code} info={info} />
            ))}
          </>
        )}

        {groupedLanguages.other.length > 0 && (
          <>
            <SelectSeparator />
            <SelectLabel>{t('languageSwitcher.groups.other')}</SelectLabel>
            {groupedLanguages.other.map(([code, info]) => (
              <LanguageSelectItem key={code} code={code} info={info} />
            ))}
          </>
        )}
      </SelectContent>
    </Select>
  );
}

function LanguageDisplay({ code }: { code: string }) {
  const info = SUPPORTED_LANGUAGES[code as SupportedLanguage];
  const flag = LANGUAGE_FLAGS[code as SupportedLanguage];

  return (
    <div className="flex items-center gap-2">
      <span className="text-lg">{flag}</span>
      <span>{info.nativeName}</span>
      <span className="text-muted-foreground text-sm">({code})</span>
    </div>
  );
}

function LanguageSelectItem({ code, info }: { code: string; info: LanguageInfo }) {
  const { t } = useTranslation();
  const flag = LANGUAGE_FLAGS[code as SupportedLanguage];

  return (
    <SelectItem value={code}>
      <Tooltip>
        <TooltipTrigger asChild>
          <div className="flex items-center gap-2">
            <span className="text-lg">{flag}</span>
            <div className="flex flex-col">
              <span>{info.nativeName}</span>
              <span className="text-xs text-muted-foreground">
                {info.name} ({code})
                {info.dir === 'rtl' && ' • RTL'}
              </span>
            </div>
          </div>
        </TooltipTrigger>
        <TooltipContent>
          <p>{t(`languageSwitcher.tooltips.${code}`)}</p>
        </TooltipContent>
      </Tooltip>
    </SelectItem>
  );
}
```

**Acceptance Criteria:**

- ✅ Search filters languages by name, native name, or code
- ✅ Keyboard navigation works (Tab, Arrow keys, Enter, Escape)
- ✅ Current language shows checkmark indicator
- ✅ Languages grouped logically (Common, RTL, Other)
- ✅ Responsive on mobile (full-screen overlay)
- ✅ Accessible (ARIA labels, screen reader support)

---

### 3. Non-Functional Requirements

#### NFR-LS-001: Performance

- Language switcher renders in <50ms
- Search filtering updates in <16ms (60fps)
- No layout shift when switching languages

#### NFR-LS-002: Accessibility

- WCAG 2.1 AA compliant
- Keyboard navigable
- Screen reader compatible
- Focus indicators visible
- Color contrast ratio >4.5:1

#### NFR-LS-003: Browser Compatibility

- Emoji flags render in Chrome, Firefox, Safari, Edge
- Graceful degradation for older browsers
- Touch-friendly on mobile devices

---

## Integration Requirements

### 1. Backend Integration (rust-i18n)

#### Update Backend Configuration

```rust
// crates/ampel-api/src/main.rs
use rust_i18n::t;

// Initialize i18n with all 20 languages
rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    // Set available locales
    rust_i18n::set_locale("en");
}
```

#### Update Locale Middleware

```rust
// crates/ampel-api/src/middleware/locale.rs
fn normalize_locale(locale: &str) -> Option<&str> {
    match locale.to_lowercase().as_str() {
        // Existing languages
        "en" | "en-us" | "en-gb" => Some("en"),
        "pt" | "pt-br" => Some("pt-BR"),
        "es" | "es-es" => Some("es-ES"),
        "de" | "de-de" => Some("de"),
        "fr" | "fr-fr" => Some("fr"),
        "he" | "he-il" => Some("he"),
        "nl" | "nl-nl" => Some("nl"),
        "sr" | "sr-rs" => Some("sr"),
        "ru" | "ru-ru" => Some("ru"),
        "it" | "it-it" => Some("it"),
        "pl" | "pl-pl" => Some("pl"),
        "zh" | "zh-cn" => Some("zh-CN"),
        "ja" | "ja-jp" => Some("ja"),

        // New languages
        "fi" | "fi-fi" => Some("fi"),
        "sv" | "sv-se" => Some("sv"),
        "no" | "nb" | "no-no" | "nb-no" => Some("nb"),
        "th" | "th-th" => Some("th"),
        "ar" | "ar-sa" => Some("ar"),
        "da" | "da-dk" => Some("da"),
        "cs" | "cs-cz" => Some("cs"),

        _ => None,
    }
}
```

---

### 2. Frontend Integration (react-i18next)

#### Update i18n Configuration

```typescript
// frontend/src/i18n/config.ts
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import HttpBackend from 'i18next-http-backend';
import LanguageDetector from 'i18next-browser-languagedetector';

export const SUPPORTED_LANGUAGES = {
  // Existing languages
  en: { name: 'English', nativeName: 'English', isoCode: 'en', dir: 'ltr' },
  'pt-BR': {
    name: 'Portuguese (Brazil)',
    nativeName: 'Português (Brasil)',
    isoCode: 'pt-BR',
    dir: 'ltr',
  },
  'es-ES': {
    name: 'Spanish (Spain)',
    nativeName: 'Español (España)',
    isoCode: 'es-ES',
    dir: 'ltr',
  },
  de: { name: 'German', nativeName: 'Deutsch', isoCode: 'de', dir: 'ltr' },
  fr: { name: 'French', nativeName: 'Français', isoCode: 'fr', dir: 'ltr' },
  he: { name: 'Hebrew', nativeName: 'עברית', isoCode: 'he', dir: 'rtl' },
  nl: { name: 'Dutch', nativeName: 'Nederlands', isoCode: 'nl', dir: 'ltr' },
  sr: { name: 'Serbian', nativeName: 'Српски', isoCode: 'sr', dir: 'ltr' },
  ru: { name: 'Russian', nativeName: 'Русский', isoCode: 'ru', dir: 'ltr' },
  it: { name: 'Italian', nativeName: 'Italiano', isoCode: 'it', dir: 'ltr' },
  pl: { name: 'Polish', nativeName: 'Polski', isoCode: 'pl', dir: 'ltr' },
  'zh-CN': { name: 'Chinese (Simplified)', nativeName: '简体中文', isoCode: 'zh-CN', dir: 'ltr' },
  ja: { name: 'Japanese', nativeName: '日本語', isoCode: 'ja', dir: 'ltr' },

  // New languages
  fi: { name: 'Finnish', nativeName: 'Suomi', isoCode: 'fi', dir: 'ltr' },
  sv: { name: 'Swedish', nativeName: 'Svenska', isoCode: 'sv', dir: 'ltr' },
  nb: { name: 'Norwegian (Bokmål)', nativeName: 'Norsk (Bokmål)', isoCode: 'nb', dir: 'ltr' },
  th: { name: 'Thai', nativeName: 'ไทย', isoCode: 'th', dir: 'ltr' },
  ar: { name: 'Arabic', nativeName: 'العربية', isoCode: 'ar', dir: 'rtl' },
  da: { name: 'Danish', nativeName: 'Dansk', isoCode: 'da', dir: 'ltr' },
  cs: { name: 'Czech', nativeName: 'Čeština', isoCode: 'cs', dir: 'ltr' },
} as const;

i18n
  .use(HttpBackend)
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: 'en',
    supportedLngs: Object.keys(SUPPORTED_LANGUAGES),
    // ... rest of configuration
  });
```

---

### 3. File Structure Updates

```
# Backend
crates/ampel-api/locales/
├── en/
├── pt-BR/
├── ... (existing 13 languages)
├── fi/              # New
├── sv/              # New
├── nb/              # New
├── th/              # New
├── ar/              # New
├── da/              # New
└── cs/              # New

# Frontend
frontend/public/locales/
├── en/
├── pt-BR/
├── ... (existing 13 languages)
├── fi/              # New
├── sv/              # New
├── nb/              # New
├── th/              # New
├── ar/              # New
├── da/              # New
└── cs/              # New
```

---

## Quality Assurance Criteria

### 1. Translation Quality

#### QA-TRANS-001: Accuracy

- Professional translation service (DeepL, Google Translate, or human translators)
- Native speaker review for critical languages (fi, ar, th)
- Context provided for technical terms

#### QA-TRANS-002: Completeness

- 100% coverage for all keys across all 20 languages
- No missing translations in production
- Fallback to English for missing keys in development

#### QA-TRANS-003: Consistency

- Consistent terminology across all namespaces
- Glossary for technical terms (PR, repository, merge, etc.)
- Tone consistent with brand (professional, friendly)

---

### 2. Functional Quality

#### QA-FUNC-001: Pluralization

- All plural forms tested for each language
- Russian (3 forms), Polish (3 forms), Arabic (6 forms), Czech (3 forms) validated
- Thai and Japanese (no plurals) tested

#### QA-FUNC-002: RTL Support

- Arabic and Hebrew render correctly RTL
- Layouts mirror properly
- Mixed LTR/RTL content (URLs, code) handled
- Icons flip appropriately

#### QA-FUNC-003: Character Encoding

- UTF-8 encoding verified for all files
- Special characters render: Thai (ไทย), Arabic (العربية), Czech (čeština)
- No mojibake or encoding corruption

---

### 3. Performance Quality

#### QA-PERF-001: Load Time

- Initial language load: <200ms
- Language switch: <100ms
- Translation file size: <50KB per language

#### QA-PERF-002: Bundle Size

- Frontend i18n overhead: <35KB (gzipped)
- Lazy loading implemented for all languages
- No blocking on translation load

---

### 4. Accessibility Quality

#### QA-A11Y-001: WCAG Compliance

- Language switcher WCAG 2.1 AA compliant
- Screen reader announces language changes
- Keyboard navigation works for all interactions

#### QA-A11Y-002: Language Tags

- HTML `lang` attribute updated on language change
- `dir` attribute set correctly for RTL languages
- ARIA labels translated

---

## Acceptance Tests

### 1. Extended Language Support Tests

#### Test Suite: New Languages

```typescript
// frontend/tests/i18n/new-languages.test.ts
import { describe, it, expect } from 'vitest';
import i18n from '../../src/i18n/config';

describe('Extended Language Support', () => {
  const newLanguages = ['fi', 'sv', 'nb', 'th', 'ar', 'da', 'cs'];

  newLanguages.forEach((lang) => {
    describe(`Language: ${lang}`, () => {
      it('should load language bundle', async () => {
        await i18n.changeLanguage(lang);
        expect(i18n.language).toBe(lang);
      });

      it('should have all common keys translated', async () => {
        await i18n.changeLanguage(lang);
        const commonKeys = ['app.name', 'navigation.dashboard', 'status.green', 'actions.save'];

        commonKeys.forEach((key) => {
          const translation = i18n.t(`common:${key}`);
          expect(translation).not.toBe(key);
          expect(translation).not.toContain('Missing translation');
        });
      });

      it('should handle pluralization correctly', async () => {
        await i18n.changeLanguage(lang);

        if (lang === 'th' || lang === 'ja') {
          // No pluralization
          expect(i18n.t('common:pullRequests.count', { count: 0 })).toBeDefined();
          expect(i18n.t('common:pullRequests.count', { count: 5 })).toBeDefined();
        } else if (lang === 'ar') {
          // 6 forms
          expect(i18n.t('common:pullRequests.count', { count: 0 })).toContain('لا توجد');
          expect(i18n.t('common:pullRequests.count', { count: 1 })).toContain('واحد');
          expect(i18n.t('common:pullRequests.count', { count: 2 })).toContain('ان');
        } else if (lang === 'cs') {
          // 3 forms
          expect(i18n.t('common:pullRequests.count', { count: 1 })).toContain('request');
          expect(i18n.t('common:pullRequests.count', { count: 2 })).toContain('requesty');
          expect(i18n.t('common:pullRequests.count', { count: 5 })).toContain('requestů');
        } else {
          // 2 forms
          expect(i18n.t('common:pullRequests.count', { count: 1 })).toContain('1');
          expect(i18n.t('common:pullRequests.count', { count: 5 })).toContain('5');
        }
      });
    });
  });

  it('should set RTL direction for Arabic', async () => {
    await i18n.changeLanguage('ar');
    expect(i18n.dir()).toBe('rtl');
  });

  it('should set LTR direction for Finnish', async () => {
    await i18n.changeLanguage('fi');
    expect(i18n.dir()).toBe('ltr');
  });
});
```

---

### 2. Translation Automation Tests

#### Test Suite: ampel-i18n-builder

```rust
// crates/ampel-i18n-builder/tests/integration_tests.rs
use ampel_i18n_builder::*;

#[tokio::test]
async fn test_translate_to_finnish() {
    let provider = DeepLProvider::new(&get_api_key());
    let result = provider.translate("Dashboard", "en", "fi").await.unwrap();
    assert_eq!(result, "Kojelauta");
}

#[tokio::test]
async fn test_translate_batch_to_arabic() {
    let provider = GoogleTranslateProvider::new(&get_api_key());
    let texts = vec!["Dashboard".to_string(), "Settings".to_string()];
    let results = provider.translate_batch(&texts, "en", "ar").await.unwrap();

    assert_eq!(results.len(), 2);
    assert!(results[0].contains("لوحة"));
    assert!(results[1].contains("إعدادات"));
}

#[tokio::test]
async fn test_cache_reduces_api_calls() {
    let cache = TranslationCache::new(CacheStorage::Memory);
    let provider = CachedTranslationProvider::new(
        DeepLProvider::new(&get_api_key()),
        cache,
    );

    // First call - hits API
    let result1 = provider.translate("Hello", "en", "fi").await.unwrap();

    // Second call - hits cache
    let result2 = provider.translate("Hello", "en", "fi").await.unwrap();

    assert_eq!(result1, result2);
    // Verify only 1 API call was made (would need mock to verify)
}

#[test]
fn test_yaml_format_preserves_plurals() {
    let yaml = r#"
    pull_requests:
      count:
        one: "1 pull request"
        other: "%{count} pull requests"
    "#;

    let format = YamlFormat;
    let map = format.parse(yaml).unwrap();
    let yaml_out = format.write(&map).unwrap();

    assert!(yaml_out.contains("one:"));
    assert!(yaml_out.contains("other:"));
}
```

---

### 3. Enhanced Language Switcher Tests

#### Test Suite: Language Switcher Component

```typescript
// frontend/src/components/LanguageSwitcher.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import i18n from '../i18n/config';
import { LanguageSwitcher } from './LanguageSwitcher';

describe('LanguageSwitcher', () => {
  it('should render with current language', async () => {
    await i18n.changeLanguage('en');
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    expect(screen.getByText(/English/i)).toBeInTheDocument();
    expect(screen.getByText('🇬🇧')).toBeInTheDocument();
  });

  it('should show all 20 languages in dropdown', () => {
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    fireEvent.click(screen.getByRole('combobox'));

    // Check new languages are present
    expect(screen.getByText(/Suomi/i)).toBeInTheDocument();  // Finnish
    expect(screen.getByText(/العربية/i)).toBeInTheDocument(); // Arabic
    expect(screen.getByText(/ไทย/i)).toBeInTheDocument();    // Thai
    expect(screen.getByText(/Čeština/i)).toBeInTheDocument(); // Czech
  });

  it('should filter languages on search', () => {
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    fireEvent.click(screen.getByRole('combobox'));

    const searchInput = screen.getByPlaceholderText(/select/i);
    fireEvent.change(searchInput, { target: { value: 'fin' } });

    expect(screen.getByText(/Finnish/i)).toBeInTheDocument();
    expect(screen.queryByText(/German/i)).not.toBeInTheDocument();
  });

  it('should change language on selection', async () => {
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    fireEvent.click(screen.getByRole('combobox'));
    fireEvent.click(screen.getByText(/Suomi/i));

    expect(i18n.language).toBe('fi');
  });

  it('should show RTL indicator for Arabic', () => {
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    fireEvent.click(screen.getByRole('combobox'));

    const arabicItem = screen.getByText(/العربية/i).closest('[role="option"]');
    expect(arabicItem).toHaveTextContent('RTL');
  });

  it('should be keyboard navigable', () => {
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    const combobox = screen.getByRole('combobox');

    // Open with Enter
    fireEvent.keyDown(combobox, { key: 'Enter' });
    expect(screen.getByRole('listbox')).toBeInTheDocument();

    // Navigate with Arrow Down
    fireEvent.keyDown(combobox, { key: 'ArrowDown' });

    // Close with Escape
    fireEvent.keyDown(combobox, { key: 'Escape' });
    expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
  });

  it('should show tooltips on hover', async () => {
    render(
      <I18nextProvider i18n={i18n}>
        <LanguageSwitcher />
      </I18nextProvider>
    );

    fireEvent.click(screen.getByRole('combobox'));

    const finnishItem = screen.getByText(/Suomi/i);
    fireEvent.mouseEnter(finnishItem);

    // Wait for tooltip
    await screen.findByText(/Switch to Finnish/i);
  });
});
```

---

### 4. Visual Regression Tests

#### Test Suite: RTL Layout

```typescript
// frontend/tests/visual/rtl-new-languages.spec.ts
import { test, expect } from '@playwright/test';

test.describe('RTL Visual Regression - Arabic', () => {
  test('dashboard renders correctly in Arabic', async ({ page }) => {
    await page.goto('/dashboard?lang=ar');
    await page.waitForSelector('[data-testid="dashboard-title"]');

    // Verify RTL direction
    const html = await page.locator('html');
    await expect(html).toHaveAttribute('dir', 'rtl');

    // Take screenshot
    await expect(page).toHaveScreenshot('dashboard-arabic-rtl.png');
  });

  test('language switcher renders correctly in Arabic', async ({ page }) => {
    await page.goto('/dashboard?lang=ar');

    await page.click('[data-testid="language-switcher"]');
    await expect(page).toHaveScreenshot('language-switcher-arabic.png');
  });
});

test.describe('Character Rendering - Thai', () => {
  test('Thai text renders with proper fonts', async ({ page }) => {
    await page.goto('/dashboard?lang=th');
    await page.waitForSelector('[data-testid="dashboard-title"]');

    // Verify Thai characters render
    const title = await page.textContent('[data-testid="dashboard-title"]');
    expect(title).toMatch(/[\u0E00-\u0E7F]/); // Thai Unicode range

    await expect(page).toHaveScreenshot('dashboard-thai.png');
  });
});
```

---

### 5. CI/CD Validation Tests

#### Test Suite: Translation Coverage

```bash
# .github/workflows/i18n-validation.yml
- name: Check coverage for all 20 languages
  run: |
    cd frontend
    pnpm test tests/i18n/coverage.test.ts --reporter=verbose

    # Ensure 100% coverage for all languages
    COVERAGE_REPORT=$(node scripts/i18n-coverage-report.js --json)

    for lang in en pt-BR es-ES nl de sr ru he fr it pl zh-CN ja fi sv nb th ar da cs; do
      COVERAGE=$(echo $COVERAGE_REPORT | jq -r ".${lang}.common")
      if [ "$COVERAGE" != "100" ]; then
        echo "ERROR: Language ${lang} has incomplete coverage: ${COVERAGE}%"
        exit 1
      fi
    done
```

---

## Summary

This specification document provides comprehensive requirements for:

1. **Extended Language Support**: 7 new languages with detailed pluralization, RTL, and character set requirements
2. **Translation Automation Crate**: `ampel-i18n-builder` with provider abstraction, caching, and CLI
3. **Enhanced Language Switcher**: Visual flags, ISO codes, tooltips, and improved UX
4. **Integration Requirements**: Backend and frontend configuration updates
5. **Quality Assurance**: Translation quality, functional quality, performance, and accessibility criteria
6. **Acceptance Tests**: Comprehensive test suites for all new features

**Total Estimated Effort**: 80-100 hours
**Timeline**: 4-5 weeks
**Priority**: Medium (Phase 2-3 rollout)

**Next Steps:**

1. Review specification with stakeholders
2. Prioritize Phase 2 languages (fi, sv, nb, da) vs Phase 3 (th, ar, cs)
3. Set up translation service accounts (DeepL, Google Translate)
4. Begin implementation of `ampel-i18n-builder` crate
5. Create initial translation bundles for Phase 2 languages
