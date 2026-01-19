//! String extraction framework for finding translatable text in source code
//!
//! This module provides infrastructure for extracting translatable strings from
//! various programming languages (TypeScript, Rust, etc.) and generating translation keys.

pub mod extractor;
pub mod key_generator;
pub mod merger;
pub mod rust;
pub mod typescript;

// Re-export main types for convenience
pub use extractor::{ExtractedString, Extractor, StringContext};
pub use key_generator::{KeyGenerator, KeyStrategy};
pub use merger::{MergeReport, Merger};
pub use rust::RustExtractor;
pub use typescript::TypeScriptExtractor;
