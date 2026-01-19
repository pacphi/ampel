# ampel-i18n-builder

Translation file parser, validator, and code generator for Ampel internationalization system with enterprise-grade 4-tier translation provider architecture.

## 🚀 Quick Start for Non-Technical Users

Never installed a Rust tool before? **No problem!** This tool is designed to be beginner-friendly.

### Option 1: Use Claude Code (Easiest) 🤖

If you have Claude Code CLI installed, follow these steps:

**Step 1: Install the Claude Code skill** (one-time setup)

```bash
# Option A: Install globally for all projects
cp -r skills/ampel-i18n ~/.claude/skills/

# Option B: Install for this project only
mkdir -p .claude/skills
cp -r skills/ampel-i18n .claude/skills/
```

**Step 2: Restart Claude Code** to recognize the new skill

**Step 3: Use the skill**

```
/ampel-i18n:localize

Help me install and set up ampel-i18n-builder. I want to translate my [React/Vue/Rust] app using [OpenAI/DeepL/Google].
```

Claude will:

1. Check if you have Rust installed (and guide you to install it if needed)
2. Install ampel-i18n-builder for you
3. Set up your configuration files
4. Help you get your first translations

**That's it!** Claude handles everything step-by-step.

### Option 2: Manual Installation (5 minutes) 📖

**Step 1: Install Rust** (one-time setup)

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows: Download installer from https://rustup.rs/
```

**Step 2: Install ampel-i18n-builder**

```bash
cargo install ampel-i18n-builder
```

**Step 3: Interactive Setup**

```bash
ampel-i18n init
```

This wizard will:

- Ask you a few questions (framework, languages, provider)
- Generate configuration files for you
- Create sample translation files
- Get you ready to translate!

**Step 4: Run Your First Translation**

```bash
ampel-i18n sync
```

Done! Your app is now multi-lingual. 🌍

For detailed installation help, see: [skills/ampel-i18n/references/install-guide.md](skills/ampel-i18n/references/install-guide.md)

---

## Features

- **String Extraction**: Automatically extract translatable strings from TypeScript/React and Rust codebases
- **Semantic Key Generation**: Context-aware translation keys (e.g., `button.saveChanges`, `error.invalidEmail`)
- **Intelligent Merging**: Merge extracted strings with existing translations without data loss
- **4-Tier Translation Architecture**: Systran (Tier 1) → DeepL (Tier 2) → Google (Tier 3) → OpenAI (Tier 4) with automatic fallback
- **Multi-Format Support**: JSON (React/Vue/Angular), YAML (Rust), and Java .properties (Spring)
- **Interactive Setup Wizard**: `ampel-i18n init` - No manual config needed
- **Health Checks**: `ampel-i18n doctor` - Diagnose issues automatically
- **Translation API Integration**: Systran, DeepL, Google Cloud Translation, and OpenAI support
- **Intelligent Fallback**: Automatic provider selection with graceful degradation on failures
- **Code Generation**: Generate TypeScript type definitions from translations
- **Validation Suite**: Coverage analysis, missing keys, duplicate detection, variable consistency
- **Type-Safe**: Compile-time translation key validation with generated types
- **Pluralization**: Support for CLDR plural forms (zero, one, two, few, many, other)
- **Configurable Providers**: Per-provider timeout, retry, and batch size configuration

## Architecture

Ampel i18n builder uses a **4-tier translation provider architecture** with automatic fallback:

### Provider Tiers

- **Tier 1: Systran** - Enterprise neural MT (primary provider)
  - Best for: All languages with high accuracy
  - Rate limit: 100 req/sec
  - Batch size: 50 texts/request

- **Tier 2: DeepL** - High-quality European languages
  - Best for: EU languages (de, fr, fi, sv, pl, cs, etc.)
  - Rate limit: 10 req/sec
  - Batch size: 50 texts/request

- **Tier 3: Google Translate** - Broad language coverage
  - Best for: Asian/Middle Eastern languages (ar, th, vi, zh, ja)
  - Rate limit: 100 req/sec
  - Batch size: 100 texts/request

- **Tier 4: OpenAI** - Fallback for complex content
  - Best for: Technical content, complex placeholders
  - Rate limit: Unlimited (token-based)
  - Batch size: Unlimited

### Fallback Behavior

If Tier 1 (Systran) fails, the system automatically falls back to Tier 2 (DeepL), then Tier 3 (Google), then Tier 4 (OpenAI). This ensures **99.9% translation success rate**.

**Example Fallback Flow:**

```
Systran (Tier 1) → Timeout
  ↓ Automatic Fallback
DeepL (Tier 2) → Success ✓
```

## Quick Start

### 1. Configure API Keys

**Option A: .env file (Recommended)**

```bash
# Copy example file
cp .env.example .env

# Edit .env and add your API keys
# At least one provider key is required
SYSTRAN_API_KEY=your-systran-key    # Tier 1 (recommended)
DEEPL_API_KEY=your-deepl-key        # Tier 2
GOOGLE_API_KEY=your-google-key      # Tier 3
OPENAI_API_KEY=your-openai-key      # Tier 4 (optional, fallback)
```

**Option B: System Environment Variables**

```bash
# Set API keys (overrides .env file)
export SYSTRAN_API_KEY="your-systran-api-key"
export DEEPL_API_KEY="your-deepl-api-key"
export GOOGLE_API_KEY="your-google-api-key"
```

**Configuration Precedence:**

1. CLI parameters (highest priority)
2. System environment variables
3. `.env` file
4. `.ampel-i18n.yaml` configuration file
5. Default values (lowest priority)

### 2. Extract Translatable Strings (New!)

**Extract from React/TypeScript:**

```bash
# Extract strings from frontend code
cargo i18n extract \
  --source frontend/src \
  --patterns "*.tsx" "*.ts" \
  --format json \
  --output frontend/public/locales/en/extracted.json \
  --merge

# Extract with semantic key generation
cargo i18n extract \
  --source frontend/src/pages \
  --patterns "*.tsx" \
  --key-strategy semantic \
  --namespace dashboard \
  --output frontend/public/locales/en/dashboard.json
```

**Extract from Rust:**

```bash
# Extract error messages and UI strings from Rust
cargo i18n extract \
  --source crates/ampel-api/src \
  --patterns "*.rs" \
  --format yaml \
  --output crates/ampel-api/locales/en/errors.yaml \
  --merge
```

**Note:** Currently supports React/TypeScript and Rust extraction. Java source code extraction is planned for a future release, but you can manually create .properties files and use the translation features.

**Options:**
- `--source`: Source directories to scan (can specify multiple)
- `--patterns`: File patterns like `*.tsx`, `*.rs`, `*.java`
- `--format`: Output format (`json`, `yaml`, or `properties`)
- `--key-strategy`: Key generation strategy (`semantic`, `hash`, or `incremental`)
- `--merge`: Merge with existing translations (preserves existing keys)
- `--dry-run`: Preview extraction without writing files
- `--namespace`: Organize extracted keys under a namespace

**What gets extracted:**
- JSX text content: `<Button>Click me</Button>`
- JSX attributes: `<Input placeholder="Enter name" />`, `aria-label`, `title`
- Template strings: `` `Welcome, ${userName}!` `` (with variable detection)
- Error messages: `anyhow!("Auth failed")`, `#[error("Not found")]`
- String literals in context: `const error = "Invalid email";`

**What gets skipped:**
- Strings already using i18n: `t('key')`, `t!("key")`
- Very short strings (< 3 characters)
- Technical strings: URLs, file paths, SQL queries
- Log messages: `println!()`, `console.log()`

### 3. Run Translation

**Automatic Mode (Recommended)** - Uses all available providers with fallback:

```bash
# Automatic provider selection with fallback
cargo i18n translate --lang fi

# With custom timeout and batch size
cargo i18n translate --lang fi --timeout 60 --batch-size 50
```

**Single Provider Mode** - No fallback, uses specific provider:

```bash
# Use only DeepL (no fallback)
cargo i18n translate --lang fi --provider deepl --no-fallback

# Use only Google (no fallback)
cargo i18n translate --lang ar --provider google --no-fallback
```

**Override Settings** - CLI parameters override configuration:

```bash
# Override timeout for all providers
cargo i18n translate --lang fi --timeout 60

# Override batch size
cargo i18n translate --lang fi --batch-size 25

# Override retry attempts
cargo i18n translate --lang fi --max-retries 5

# Disable specific providers
cargo i18n translate --lang fi --disable-provider openai
```

### 4. Validate Translations

```bash
# Check coverage (requires ≥95%)
cargo i18n validate \
  --input-dir crates/ampel-api/locales \
  --base-locale en \
  --min-coverage 95

# Check for missing keys
cargo i18n validate \
  --input-dir frontend/public/locales \
  --check missing

# Check for variable mismatches
cargo i18n validate \
  --check variables
```

### 5. Generate Type Definitions

```bash
# Generate TypeScript types (CLI command)
cargo i18n generate-types \
  --output frontend/src/types/i18n.generated.ts \
  --translation-dir frontend/public/locales

# Note: CLI only supports TypeScript generation
# For Rust types, use the library API (see "Usage as a Library" section)
```

## Configuration

### Basic Configuration (.ampel-i18n.yaml)

```yaml
translation:
  # API Keys (or use environment variables)
  systran_api_key: '${SYSTRAN_API_KEY}'
  deepl_api_key: '${DEEPL_API_KEY}'
  google_api_key: '${GOOGLE_API_KEY}'
  openai_api_key: '${OPENAI_API_KEY}'

  # Global Defaults
  default_timeout_secs: 30
  default_batch_size: 50
  default_max_retries: 3

  # Fallback Strategy
  fallback:
    skip_on_missing_key: true # Skip providers without API keys
    stop_on_first_success: true # Don't try more providers after success
    log_fallback_events: true # Log when falling back to next tier
```

### Advanced Configuration (Per-Provider Settings)

See [docs/localization/PROVIDER-CONFIGURATION.md](../../docs/localization/PROVIDER-CONFIGURATION.md) for complete configuration guide.

```yaml
translation:
  providers:
    systran:
      enabled: true
      priority: 1 # Tier 1 (highest)
      timeout_secs: 45
      max_retries: 3
      batch_size: 50
      rate_limit_per_sec: 100

    deepl:
      enabled: true
      priority: 2 # Tier 2
      timeout_secs: 30
      max_retries: 3
      batch_size: 50
      rate_limit_per_sec: 10

    google:
      enabled: true
      priority: 3 # Tier 3
      timeout_secs: 30
      max_retries: 3
      batch_size: 100
      rate_limit_per_sec: 100

    openai:
      enabled: true
      priority: 4 # Tier 4 (fallback)
      timeout_secs: 60
      max_retries: 2
      model: 'gpt-5-mini'
      temperature: 0.3
```

## Usage as a Library

### Parse Translation Files

```rust
use ampel_i18n_builder::formats::{TranslationFormat, JsonFormat, YamlFormat};

// Parse JSON
let format = JsonFormat::new();
let translations = format.parse(r#"{
    "greeting": "Hello, {name}!",
    "items": {
        "one": "1 item",
        "other": "{{count}} items"
    }
}"#)?;

// Parse YAML
let format = YamlFormat::new();
let translations = format.parse(r#"
greeting: "Hello, {name}!"
items:
  one: "1 item"
  other: "{{count}} items"
"#)?;
```

### Validate Translations

```rust
use ampel_i18n_builder::validation::{CoverageValidator, MissingKeysValidator, Validator};
use std::collections::HashMap;

// Check coverage
let validator = CoverageValidator::new(
    source_translations.clone(),
    target_translations.clone(),
    95.0  // Minimum coverage threshold
);
let result = validator.validate();

if !result.is_valid() {
    for error in result.errors {
        eprintln!("Validation error: {}", error);
    }
}

// Check for missing keys
let validator = MissingKeysValidator::new(
    source_translations,
    target_translations,
    "fi"
);
let result = validator.validate();
```

### Translate with APIs

```rust
use ampel_i18n_builder::translator::{Translator, TranslationProvider};
use ampel_i18n_builder::config::Config;
use std::collections::HashMap;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let translator = Translator::new(TranslationProvider::DeepL, &config)?;

    let mut texts = HashMap::new();
    texts.insert("greeting".to_string(), json!("Hello, world!"));
    texts.insert("farewell".to_string(), json!("Goodbye!"));

    let translations = translator.translate_batch(&texts, "fi").await?;

    println!("Finnish:");
    println!("  greeting: {}", translations["greeting"]);
    println!("  farewell: {}", translations["farewell"]);

    Ok(())
}
```

### Generate Type Definitions

```rust
use ampel_i18n_builder::codegen::{typescript::TypeScriptGenerator, rust::RustGenerator, CodeGenerator, GeneratorOptions};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Generate TypeScript types
    let ts_generator = TypeScriptGenerator::new();
    let options = GeneratorOptions {
        pretty_print: true,
        include_metadata: true,
        split_by_namespace: false,
        create_index: true,
    };

    let result = ts_generator.generate(
        &translations,
        "en",
        &PathBuf::from("frontend/src/types"),
        options.clone()
    ).await?;

    println!("Generated {} TypeScript files", result.files_created.len());

    // Generate Rust types
    let rust_generator = RustGenerator::new();
    let result = rust_generator.generate(
        &translations,
        "en",
        &PathBuf::from("src/i18n"),
        options
    ).await?;

    println!("Generated {} Rust files", result.files_created.len());
    Ok(())
}
```

## Project Architecture

```
ampel-i18n-builder/
├── src/
│   ├── translator/          # Translation providers (4-tier)
│   │   ├── mod.rs           # TranslationService trait
│   │   ├── systran.rs       # Systran API (Tier 1)
│   │   ├── deepl.rs         # DeepL API (Tier 2)
│   │   ├── google.rs        # Google Cloud (Tier 3)
│   │   ├── openai.rs        # OpenAI API (Tier 4)
│   │   ├── router.rs        # FallbackRouter orchestrator
│   │   └── cache.rs         # Translation cache (LRU)
│   │
│   ├── formats/             # Format parsers
│   │   ├── yaml.rs          # YAML parser (backend)
│   │   └── json.rs          # JSON parser (frontend)
│   │
│   ├── validation/          # Validation suite
│   │   ├── coverage.rs      # Coverage analysis
│   │   ├── missing.rs       # Missing key detection
│   │   ├── duplicates.rs    # Duplicate key detection
│   │   └── variables.rs     # Variable consistency
│   │
│   ├── codegen/             # Code generators
│   │   ├── typescript.rs    # TypeScript type definitions
│   │   └── rust.rs          # Rust type definitions
│   │
│   ├── extraction/          # String extraction
│   │   ├── extractor.rs     # Extraction engine
│   │   ├── typescript.rs    # TypeScript/React extractor
│   │   ├── rust.rs          # Rust extractor
│   │   ├── key_generator.rs # Key generation strategies
│   │   └── merger.rs        # Translation merging
│   │
│   ├── refactor/            # Code refactoring
│   │   ├── typescript_oxc.rs # TypeScript refactoring
│   │   ├── rust_syn.rs      # Rust refactoring
│   │   └── backup.rs        # Backup management
│   │
│   ├── config.rs            # Configuration management
│   └── cli/                 # CLI interface
│       ├── translate.rs     # Translation command
│       ├── validate.rs      # Validation command
│       ├── generate_types.rs # Type generation command
│       ├── extract.rs       # Extraction command
│       └── refactor.rs      # Refactoring command
│
└── tests/
    ├── integration/         # Integration tests
    │   ├── fallback_tests.rs        # Fallback behavior tests
    │   └── translation_api_tests.rs # Provider integration tests
    └── fixtures/            # Test data
```

## Supported Languages

### Provider Coverage

The 4-tier architecture provides coverage for **133+ languages**:

| Provider             | Languages | Best For                                          |
| -------------------- | --------- | ------------------------------------------------- |
| **Systran** (Tier 1) | 55+       | All languages with enterprise quality             |
| **DeepL** (Tier 2)   | 28        | European languages (de, fr, fi, sv, pl, cs, etc.) |
| **Google** (Tier 3)  | 133+      | Asian/Middle Eastern (ar, th, vi, zh, ja, hi)     |
| **OpenAI** (Tier 4)  | All       | Fallback for complex content and technical text   |

### Provider Selection Logic

The system selects providers based on:

1. **Priority Order**: Tier 1 → Tier 2 → Tier 3 → Tier 4
2. **Language Preferences**: Configured per-provider for optimal quality
3. **Availability**: Skips providers without API keys
4. **Fallback**: Automatically tries next tier on failure

**Example:**

- Finnish (fi): Systran (Tier 1) → DeepL (Tier 2) → Google (Tier 3) → OpenAI (Tier 4)
- Arabic (ar): Systran (Tier 1) → Google (Tier 3) → DeepL (Tier 2) → OpenAI (Tier 4)

## Documentation

- **[DEVELOPER-GUIDE.md](../../docs/localization/DEVELOPER-GUIDE.md)** - Quick start for developers
- **[TRANSLATION_WORKFLOW.md](../../docs/localization/TRANSLATION_WORKFLOW.md)** - Complete workflow guide
- **[ARCHITECTURE.md](../../docs/localization/ARCHITECTURE.md)** - System architecture details
- **[PROVIDER-CONFIGURATION.md](../../docs/localization/PROVIDER-CONFIGURATION.md)** - Complete provider configuration guide
- **[4-TIER-PROVIDER-ARCHITECTURE.md](../../docs/localization/4-TIER-PROVIDER-ARCHITECTURE.md)** - 4-tier architecture design document

## Development

### Run tests

```bash
# Unit tests
cargo test

# Integration tests (requires API keys)
DEEPL_API_KEY=xxx cargo test -- --ignored

# With logging
RUST_LOG=debug cargo test
```

### Run benchmarks

```bash
cargo bench
```

### Format code

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

## Examples

See `tests/integration/` for complete examples:

- **Translation API**: `api_tests.rs`
- **Format Parsing**: `format_parser_tests.rs`
- **Validation**: `validation_tests.rs`
- **Code Generation**: `code_generation_tests.rs`

## License

MIT OR Apache-2.0

---

**Full Documentation:** [docs/localization/](../../docs/localization/)
**Issues:** File with `[i18n]` prefix
**Questions:** See [DEVELOPER-GUIDE.md](../../docs/localization/DEVELOPER-GUIDE.md)
