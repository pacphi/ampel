# Code Generation Module

Type-safe code generation for translation keys in TypeScript and Rust.

## Features

- **TypeScript Generator**: Creates type-safe interfaces and utility types
- **Rust Generator**: Creates compile-time const declarations
- **Integration**: Works with rust-i18n and react-i18next
- **Namespace Support**: Organized code with namespace modules
- **Compile-Time Validation**: Catch translation key typos at compile time

## Usage

### TypeScript Generation

```rust
use ampel_i18n_builder::codegen::{
    typescript::TypeScriptGenerator,
    CodeGenerator,
    GeneratorOptions
};
use std::path::PathBuf;

let generator = TypeScriptGenerator::new();
let options = GeneratorOptions::default();

generator.generate(
    &translations,
    "en",
    &PathBuf::from("frontend/src/i18n"),
    options
).await?;
```

Output: `frontend/src/i18n/types.ts`

### Rust Generation

```rust
use ampel_i18n_builder::codegen::{
    rust::RustGenerator,
    CodeGenerator,
    GeneratorOptions
};

let generator = RustGenerator::new();
let options = GeneratorOptions {
    split_by_namespace: true,
    ..Default::default()
};

generator.generate(
    &translations,
    "en",
    &PathBuf::from("crates/ampel-api/src/i18n"),
    options
).await?;
```

Output: `crates/ampel-api/src/i18n/keys.rs`

## Generator Options

```rust
pub struct GeneratorOptions {
    pub pretty_print: bool,         // Format output (default: true)
    pub include_metadata: bool,     // Add metadata comments (default: true)
    pub split_by_namespace: bool,   // Create namespace modules (default: false)
    pub create_index: bool,         // Generate index file (default: true)
}
```

## API

### TypeScript Generator

```rust
impl TypeScriptGenerator {
    pub fn new() -> Self;

    pub fn generate_types(
        &self,
        translations: &TranslationMap,
        language: &str,
        options: &GeneratorOptions
    ) -> String;
}
```

### Rust Generator

```rust
impl RustGenerator {
    pub fn new() -> Self;

    pub fn generate_consts(
        &self,
        translations: &TranslationMap,
        language: &str,
        options: &GeneratorOptions
    ) -> String;
}
```

## Testing

Run tests:

```bash
cargo test -p ampel-i18n-builder --lib codegen
```

All generators include comprehensive unit tests:
- Translation key flattening
- Identifier validation and sanitization
- Namespace extraction
- Generated code syntax validation

## Examples

See `examples/generate_code.rs` for a complete working example.
