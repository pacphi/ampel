# ampel-i18n-builder

Translation file parser, validator, and code generator for Ampel internationalization system.

## Features

- **Format Parsing**: Parse and validate YAML (backend) and JSON (frontend) translation files
- **Translation API Integration**: DeepL, Google Cloud Translation, and OpenAI support
- **Code Generation**: Generate TypeScript and Rust type definitions from translations
- **Validation Suite**: Coverage analysis, missing keys, duplicate detection, variable consistency
- **Type-Safe**: Compile-time translation key validation with generated types
- **Pluralization**: Support for CLDR plural forms (zero, one, two, few, many, other)

## Installation

```bash
# Build the CLI tool
cargo build --release --bin i18n-builder

# Or use via cargo run
cargo run --bin i18n-builder -- --help
```

## Quick Start

### 1. Configure API Keys

```bash
# Create .ampel-i18n.yaml (optional, uses env vars by default)
cat > .ampel-i18n.yaml <<EOF
translation_dir: "frontend/public/locales"
translation:
  timeout_secs: 30
  batch_size: 50
EOF

# Set API keys
export DEEPL_API_KEY="your-deepl-api-key"
export GOOGLE_API_KEY="your-google-api-key"  # For Thai, Arabic
```

### 2. Translate Files

```bash
# Translate YAML to Finnish
cargo run --bin i18n-builder -- translate \
  --provider deepl \
  --input crates/ampel-api/locales/en/common.yml \
  --output crates/ampel-api/locales/fi/common.yml \
  --target-lang fi

# Translate JSON to Spanish
cargo run --bin i18n-builder -- translate \
  --provider deepl \
  --input frontend/public/locales/en/dashboard.json \
  --output frontend/public/locales/es/dashboard.json \
  --target-lang es
```

### 3. Validate Translations

```bash
# Check coverage (requires ≥95%)
cargo run --bin i18n-builder -- validate \
  --input-dir crates/ampel-api/locales \
  --base-locale en \
  --min-coverage 95

# Check for missing keys
cargo run --bin i18n-builder -- validate \
  --input-dir frontend/public/locales \
  --check missing

# Check for variable mismatches
cargo run --bin i18n-builder -- validate \
  --check variables
```

### 4. Generate Type Definitions

```bash
# Generate TypeScript types
cargo run --bin i18n-builder -- codegen \
  --input frontend/public/locales/en/dashboard.json \
  --output frontend/src/types/i18n.generated.ts \
  --language typescript
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
use ampel_i18n_builder::codegen::{typescript::TypeScriptGenerator, CodeGenerator, GeneratorOptions};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let generator = TypeScriptGenerator::new();
    let options = GeneratorOptions {
        pretty_print: true,
        include_metadata: true,
        split_by_namespace: false,
        create_index: true,
    };

    let result = generator.generate(
        &translations,
        "en",
        &PathBuf::from("frontend/src/types"),
        options
    ).await?;

    println!("Generated {} files", result.files_created.len());
    Ok(())
}
```

## Architecture

```
ampel-i18n-builder/
├── src/
│   ├── translator/          # Translation API clients
│   │   ├── deepl.rs         # DeepL API integration
│   │   ├── google.rs        # Google Cloud Translation
│   │   └── openai.rs        # OpenAI fallback
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
│   └── config.rs            # Configuration management
│
└── tests/
    ├── integration/         # Integration tests
    └── fixtures/            # Test data
```

## Supported Languages

### DeepL (Primary - 18 languages)
English, Portuguese (Brazil), Spanish (Spain), Dutch, German, Serbian, Russian, Hebrew, French, Italian, Polish, Chinese (Simplified), Japanese, Finnish, Swedish, Norwegian, Danish, Czech

### Google Cloud Translation (Fallback - 2 languages)
Thai, Arabic

### Provider Selection

The tool automatically selects the best provider:
- **DeepL**: For 18/20 languages (highest quality)
- **Google**: For Thai and Arabic (DeepL doesn't support)
- **OpenAI**: Emergency fallback for all languages

## Documentation

- **[DEVELOPER_GUIDE.md](../../docs/localization/DEVELOPER_GUIDE.md)** - Quick start for developers
- **[TRANSLATION_WORKFLOW.md](../../docs/localization/TRANSLATION_WORKFLOW.md)** - Complete workflow guide
- **[ARCHITECTURE.md](../../docs/localization/ARCHITECTURE.md)** - System architecture details

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
**Questions:** See [DEVELOPER_GUIDE.md](../../docs/localization/DEVELOPER_GUIDE.md)
