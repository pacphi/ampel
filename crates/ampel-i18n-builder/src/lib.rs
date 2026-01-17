//! # Ampel i18n Builder
//!
//! Translation file parser and builder for Ampel internationalization system.
//!
//! ## Features
//!
//! - Parse YAML and JSON translation files
//! - Support nested translation keys (e.g., "dashboard.title.main")
//! - Handle plural forms (zero, one, two, few, many, other)
//! - Preserve variable placeholders ({{var}}, {var})
//! - Maintain key ordering with BTreeMap
//! - Code generation for TypeScript and Rust type-safe translations
//!
//! ## Example
//!
//! ```rust
//! use ampel_i18n_builder::formats::{TranslationFormat, JsonFormat};
//!
//! let json = r#"{
//!     "greeting": "Hello, {name}!",
//!     "items": {
//!         "one": "1 item",
//!         "other": "{{count}} items"
//!     }
//! }"#;
//!
//! let format = JsonFormat::new();
//! let bundle = format.parse(json).unwrap();
//!
//! assert_eq!(bundle.len(), 2);
//! ```

pub mod cli;
pub mod codegen;
pub mod config;
pub mod error;
pub mod formats;
pub mod translator;
pub mod validation;

pub use codegen::{CodeGenerator, GeneratorError, GeneratorOptions, GeneratorResult};
pub use config::Config;
pub use error::{Error, Result};
pub use formats::{
    FormatError, JsonFormat, PluralForms, TranslationFormat, TranslationMap, TranslationValue,
    YamlFormat,
};
pub use translator::{TranslationService, Translator};
pub use validation::{
    CoverageValidator, DuplicateKeysValidator, MissingKeysValidator, ValidationError,
    ValidationResult, ValidationResults, Validator, VariableValidator,
};
