//! Example: Generate TypeScript and Rust code from translation files
//!
//! This example demonstrates how to use the code generators to create
//! type-safe translation code for both TypeScript and Rust.

use ampel_i18n_builder::codegen::{
    rust::RustGenerator, typescript::TypeScriptGenerator, CodeGenerator, GeneratorOptions,
    TranslationMap, TranslationValue,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create example translations
    let translations = create_example_translations();

    // Output directories
    let ts_output = PathBuf::from("./generated/typescript");
    let rust_output = PathBuf::from("./generated/rust");

    // Generator options
    let options = GeneratorOptions {
        pretty_print: true,
        include_metadata: true,
        split_by_namespace: true,
        create_index: true,
    };

    // Generate TypeScript types
    println!("Generating TypeScript types...");
    let ts_gen = TypeScriptGenerator::new();
    let ts_result = ts_gen
        .generate(&translations, "en", &ts_output, options.clone())
        .await?;

    println!("TypeScript generation complete:");
    println!("  Files created: {:?}", ts_result.files_created);
    println!("  Keys written: {}", ts_result.keys_written);

    // Generate Rust consts
    println!("\nGenerating Rust constants...");
    let rust_gen = RustGenerator::new();
    let rust_result = rust_gen
        .generate(&translations, "en", &rust_output, options)
        .await?;

    println!("Rust generation complete:");
    println!("  Files created: {:?}", rust_result.files_created);
    println!("  Keys written: {}", rust_result.keys_written);

    // Display generated TypeScript
    println!("\n--- Generated TypeScript (types.ts) ---");
    let ts_content = std::fs::read_to_string(ts_output.join("types.ts"))?;
    println!(
        "{}",
        ts_content.lines().take(50).collect::<Vec<_>>().join("\n")
    );

    // Display generated Rust
    println!("\n--- Generated Rust (keys.rs) ---");
    let rust_content = std::fs::read_to_string(rust_output.join("keys.rs"))?;
    println!(
        "{}",
        rust_content.lines().take(50).collect::<Vec<_>>().join("\n")
    );

    Ok(())
}

fn create_example_translations() -> TranslationMap {
    let mut translations = BTreeMap::new();

    // Simple top-level keys
    translations.insert(
        "app_name".to_string(),
        TranslationValue::String("Ampel".to_string()),
    );

    // Common namespace
    let mut common = BTreeMap::new();
    common.insert(
        "hello".to_string(),
        TranslationValue::String("Hello".to_string()),
    );
    common.insert(
        "welcome".to_string(),
        TranslationValue::String("Welcome, {{username}}!".to_string()),
    );
    common.insert(
        "goodbye".to_string(),
        TranslationValue::String("Goodbye".to_string()),
    );

    translations.insert("common".to_string(), TranslationValue::Nested(common));

    // Dashboard namespace
    let mut dashboard = BTreeMap::new();
    dashboard.insert(
        "title".to_string(),
        TranslationValue::String("Dashboard".to_string()),
    );
    dashboard.insert(
        "pull_requests".to_string(),
        TranslationValue::String("Pull Requests".to_string()),
    );

    // Add plural form
    use ampel_i18n_builder::PluralForms;
    dashboard.insert(
        "pr_count".to_string(),
        TranslationValue::Plural(PluralForms {
            zero: None,
            one: Some("1 pull request".to_string()),
            two: None,
            few: None,
            many: None,
            other: "{{count}} pull requests".to_string(),
        }),
    );

    translations.insert("dashboard".to_string(), TranslationValue::Nested(dashboard));

    translations
}
