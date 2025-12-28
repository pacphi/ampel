//! Translate command implementation.

use crate::cli::TranslateArgs;
use crate::config::Config;
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat, TranslationMap, TranslationValue};
use crate::translator::fallback::FallbackTranslationRouter;
use crate::translator::{TranslationService, Translator};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub async fn execute(args: TranslateArgs) -> Result<()> {
    // Load configuration
    let mut config = Config::load()?;

    // Apply CLI overrides to configuration
    if let Some(timeout) = args.timeout {
        config.translation.default_timeout_secs = timeout;
        println!(
            "{} Override global timeout: {}s",
            "⚙".cyan(),
            timeout
        );
    }
    if let Some(batch_size) = args.batch_size {
        config.translation.default_batch_size = batch_size;
        println!(
            "{} Override batch size: {}",
            "⚙".cyan(),
            batch_size
        );
    }
    if let Some(max_retries) = args.max_retries {
        config.translation.default_max_retries = max_retries;
        println!(
            "{} Override max retries: {}",
            "⚙".cyan(),
            max_retries
        );
    }

    // Warn about disabled providers
    if !args.disabled_providers.is_empty() {
        println!(
            "{} Disabled providers: {}",
            "!".yellow(),
            args.disabled_providers.join(", ")
        );
    }

    // Initialize translator based on mode
    let translator: Box<dyn TranslationService> = if args.no_fallback {
        // Single provider mode (backward compatibility)
        let provider = args.provider.ok_or_else(|| {
            crate::error::Error::Config(
                "Provider must be specified when using --no-fallback".to_string(),
            )
        })?;

        println!(
            "{} Single provider mode: {:?} (no fallback)",
            "→".cyan().bold(),
            provider
        );

        Box::new(Translator::new(provider, &config)?)
    } else {
        // Fallback router mode (default)
        if let Some(provider) = args.provider {
            println!(
                "{} Provider hint: {:?} will be prioritized",
                "!".yellow(),
                provider
            );
        }

        let router = FallbackTranslationRouter::new(&config)?;

        println!(
            "{} Fallback mode enabled: Translating to {}",
            "→".cyan().bold(),
            args.lang.green()
        );

        Box::new(router)
    };

    // Load source (English) translations
    let source_dir = args.translation_dir.join("en");
    if !source_dir.exists() {
        return Err(crate::error::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory not found: {}", source_dir.display()),
        )));
    }

    // Load target translations (if exists)
    let target_dir = args.translation_dir.join(&args.lang);
    fs::create_dir_all(&target_dir)?;

    // Process namespaces
    let namespaces = if let Some(ns) = &args.namespace {
        vec![ns.clone()]
    } else {
        // List all JSON files in source directory
        fs::read_dir(&source_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "json" {
                    path.file_stem()?.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect()
    };

    println!(
        "{} Found {} namespace(s): {}",
        "✓".green().bold(),
        namespaces.len(),
        namespaces.join(", ")
    );

    // Process each namespace
    for namespace in namespaces {
        process_namespace(
            &namespace,
            &source_dir,
            &target_dir,
            &args.lang,
            translator.as_ref(),
            args.dry_run,
        )
        .await?;
    }

    println!("{} Translation complete!", "✓".green().bold());

    Ok(())
}

async fn process_namespace(
    namespace: &str,
    source_dir: &Path,
    target_dir: &Path,
    target_lang: &str,
    translator: &dyn TranslationService,
    dry_run: bool,
) -> Result<()> {
    let format = JsonFormat::new();

    // Load source translations
    let source_file = source_dir.join(format!("{}.json", namespace));
    let source_content = fs::read_to_string(&source_file)?;
    let source_map = format.parse(&source_content)?;

    // Load existing target translations (if any)
    let target_file = target_dir.join(format!("{}.json", namespace));
    let mut target_map = if target_file.exists() {
        let content = fs::read_to_string(&target_file)?;
        format.parse(&content)?
    } else {
        TranslationMap::new()
    };

    // Find missing keys
    let missing_keys = find_missing_keys(&source_map, &target_map);

    if missing_keys.is_empty() {
        println!(
            "  {} {} - {} up to date",
            "✓".green(),
            namespace.cyan(),
            "already".dimmed()
        );
        return Ok(());
    }

    println!(
        "  {} {} - {} missing key(s)",
        "→".cyan(),
        namespace,
        missing_keys.len().to_string().yellow()
    );

    // Flatten nested structures for translation
    let mut texts_to_translate: HashMap<String, serde_json::Value> = HashMap::new();

    for key in &missing_keys {
        if let Some(value) = get_translation_value(&source_map, key) {
            flatten_for_translation(key, value, &mut texts_to_translate);
        }
    }

    if texts_to_translate.is_empty() {
        println!("    {} No translatable content found", "!".yellow());
        return Ok(());
    }

    // Show progress bar
    let pb = ProgressBar::new(texts_to_translate.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("    {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Translate
    pb.set_message("Translating...");
    let translations = translator
        .translate_batch(&texts_to_translate, target_lang)
        .await?;
    pb.finish_with_message("Done!");

    // Reconstruct nested structure from flattened translations
    for (key, value) in translations {
        if let serde_json::Value::String(text) = value {
            set_translation_value(&mut target_map, &key, text);
        }
    }

    // Write to file
    if !dry_run {
        let output = format.write(&target_map)?;
        fs::write(&target_file, output)?;
        println!(
            "    {} Wrote {} translations to {}",
            "✓".green(),
            missing_keys.len(),
            target_file.display()
        );
    } else {
        println!(
            "    {} Would write {} translations (dry run)",
            "!".yellow(),
            missing_keys.len()
        );
    }

    Ok(())
}

fn find_missing_keys(source: &TranslationMap, target: &TranslationMap) -> Vec<String> {
    let mut missing = Vec::new();

    for (key, value) in source {
        if !target.contains_key(key) {
            missing.push(key.clone());
        } else if let TranslationValue::Nested(source_nested) = value {
            if let Some(TranslationValue::Nested(target_nested)) = target.get(key) {
                let nested_missing = find_missing_keys(source_nested, target_nested);
                for nested_key in nested_missing {
                    missing.push(format!("{}.{}", key, nested_key));
                }
            }
        }
    }

    missing
}

/// Get translation value from nested path (e.g., "app.title" or "auth.login")
fn get_translation_value<'a>(map: &'a TranslationMap, key: &str) -> Option<&'a TranslationValue> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return None;
    }

    if parts.len() == 1 {
        return map.get(key);
    }

    // Navigate nested structure
    let mut current_value = map.get(parts[0])?;

    for part in &parts[1..] {
        match current_value {
            TranslationValue::Nested(nested_map) => {
                current_value = nested_map.get(*part)?;
            }
            _ => return None,
        }
    }

    Some(current_value)
}

/// Flatten translation value for batch translation
/// Converts nested structures to dot-notation keys
fn flatten_for_translation(
    prefix: &str,
    value: &TranslationValue,
    result: &mut HashMap<String, serde_json::Value>,
) {
    match value {
        TranslationValue::String(text) => {
            result.insert(prefix.to_string(), serde_json::Value::String(text.clone()));
        }
        TranslationValue::Nested(nested_map) => {
            for (key, nested_value) in nested_map {
                let nested_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_for_translation(&nested_key, nested_value, result);
            }
        }
        TranslationValue::Plural(forms) => {
            // Translate each plural form independently
            if let Some(zero) = &forms.zero {
                result.insert(
                    format!("{}.zero", prefix),
                    serde_json::Value::String(zero.clone()),
                );
            }
            if let Some(one) = &forms.one {
                result.insert(
                    format!("{}.one", prefix),
                    serde_json::Value::String(one.clone()),
                );
            }
            if let Some(two) = &forms.two {
                result.insert(
                    format!("{}.two", prefix),
                    serde_json::Value::String(two.clone()),
                );
            }
            if let Some(few) = &forms.few {
                result.insert(
                    format!("{}.few", prefix),
                    serde_json::Value::String(few.clone()),
                );
            }
            if let Some(many) = &forms.many {
                result.insert(
                    format!("{}.many", prefix),
                    serde_json::Value::String(many.clone()),
                );
            }
            result.insert(
                format!("{}.other", prefix),
                serde_json::Value::String(forms.other.clone()),
            );
        }
    }
}

/// Set translation value in nested structure from dot-notation key
fn set_translation_value(map: &mut TranslationMap, key: &str, value: String) {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        return;
    }

    // Handle plural form keys (e.g., "pullRequests.count_one.zero")
    if parts.len() > 1 {
        let last_part = parts[parts.len() - 1];
        if matches!(last_part, "zero" | "one" | "two" | "few" | "many" | "other") {
            // This is a plural form - reconstruct the plural key path
            let plural_key_parts = &parts[..parts.len() - 1];
            let plural_form = last_part;

            // Navigate to or create the nested structure
            let mut current_map = map;
            for part in &plural_key_parts[..plural_key_parts.len() - 1] {
                if !current_map.contains_key(*part) {
                    current_map.insert(
                        part.to_string(),
                        TranslationValue::Nested(TranslationMap::new()),
                    );
                }
                current_map = match current_map.get_mut(*part).unwrap() {
                    TranslationValue::Nested(nested) => nested,
                    _ => return, // Invalid structure, skip
                };
            }

            // Get or create the plural forms structure
            let plural_key = plural_key_parts.last().unwrap();
            let plural_forms = current_map
                .entry(plural_key.to_string())
                .or_insert_with(|| {
                    TranslationValue::Plural(crate::formats::PluralForms {
                        zero: None,
                        one: None,
                        two: None,
                        few: None,
                        many: None,
                        other: String::new(),
                    })
                });

            if let TranslationValue::Plural(forms) = plural_forms {
                match plural_form {
                    "zero" => forms.zero = Some(value),
                    "one" => forms.one = Some(value),
                    "two" => forms.two = Some(value),
                    "few" => forms.few = Some(value),
                    "many" => forms.many = Some(value),
                    "other" => forms.other = value,
                    _ => {}
                }
            }
            return;
        }
    }

    // Handle regular nested keys (e.g., "app.title", "auth.login")
    if parts.len() == 1 {
        map.insert(key.to_string(), TranslationValue::String(value));
        return;
    }

    let mut current_map = map;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - insert the value
            current_map.insert(part.to_string(), TranslationValue::String(value.clone()));
        } else {
            // Intermediate part - ensure nested map exists
            if !current_map.contains_key(*part) {
                current_map.insert(
                    part.to_string(),
                    TranslationValue::Nested(TranslationMap::new()),
                );
            }
            current_map = match current_map.get_mut(*part).unwrap() {
                TranslationValue::Nested(nested) => nested,
                _ => return, // Invalid structure, skip
            };
        }
    }
}
