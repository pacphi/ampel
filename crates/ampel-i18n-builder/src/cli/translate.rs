//! Translate command implementation.

use crate::cli::TranslateArgs;
use crate::config::Config;
use crate::error::Result;
use crate::formats::{JsonFormat, YamlFormat, TranslationFormat, TranslationMap, TranslationValue};
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
        println!("{} Override global timeout: {}s", "⚙".cyan(), timeout);
    }
    if let Some(batch_size) = args.batch_size {
        config.translation.default_batch_size = batch_size;
        println!("{} Override batch size: {}", "⚙".cyan(), batch_size);
    }
    if let Some(max_retries) = args.max_retries {
        config.translation.default_max_retries = max_retries;
        println!("{} Override max retries: {}", "⚙".cyan(), max_retries);
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
        // List all JSON and YAML files in source directory
        fs::read_dir(&source_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let ext = path.extension()?.to_str()?;
                if ext == "json" || ext == "yml" || ext == "yaml" {
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

    // Validate flag usage
    if args.force && args.detect_untranslated {
        return Err(crate::error::Error::Config(
            "Cannot use both --force and --detect-untranslated. Choose one strategy.".to_string(),
        ));
    }

    // Show mode
    if args.force {
        println!("{} Force mode: Retranslating ALL keys", "!".yellow().bold());
    } else if args.detect_untranslated {
        println!(
            "{} Smart detection: Retranslating English/untranslated values only",
            "🔍".cyan().bold()
        );
    }

    // Create options struct
    let options = TranslateOptions {
        dry_run: args.dry_run,
        force: args.force,
        detect_untranslated: args.detect_untranslated,
    };

    // Process each namespace
    for namespace in namespaces {
        process_namespace(
            &namespace,
            &source_dir,
            &target_dir,
            &args.lang,
            translator.as_ref(),
            &options,
        )
        .await?;
    }

    println!("{} Translation complete!", "✓".green().bold());

    Ok(())
}

struct TranslateOptions {
    dry_run: bool,
    force: bool,
    detect_untranslated: bool,
}

async fn process_namespace(
    namespace: &str,
    source_dir: &Path,
    target_dir: &Path,
    target_lang: &str,
    translator: &dyn TranslationService,
    options: &TranslateOptions,
) -> Result<()> {
    // Auto-detect file format from source directory
    let (source_file, formatter): (std::path::PathBuf, Box<dyn TranslationFormat>) =
        if source_dir.join(format!("{}.json", namespace)).exists() {
            (source_dir.join(format!("{}.json", namespace)), Box::new(JsonFormat::new()))
        } else if source_dir.join(format!("{}.yml", namespace)).exists() {
            (source_dir.join(format!("{}.yml", namespace)), Box::new(YamlFormat))
        } else if source_dir.join(format!("{}.yaml", namespace)).exists() {
            (source_dir.join(format!("{}.yaml", namespace)), Box::new(YamlFormat))
        } else {
            return Err(crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source file not found for namespace '{}' (.json, .yml, or .yaml)", namespace),
            )));
        };

    // Determine target file extension (same as source)
    let target_extension = source_file.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("json");

    // Load source translations
    let source_content = fs::read_to_string(&source_file)?;
    let source_map = formatter.parse(&source_content)?;

    // Load existing target translations (if any)
    let target_file = target_dir.join(format!("{}.{}", namespace, target_extension));
    let mut target_map = if target_file.exists() {
        let content = fs::read_to_string(&target_file)?;
        formatter.parse(&content)?
    } else {
        TranslationMap::new()
    };

    // Find keys to translate based on mode
    let keys_to_translate = if options.force {
        // Force mode: retranslate all keys
        collect_all_keys(&source_map)
    } else if options.detect_untranslated {
        // Smart detection: find English values
        find_untranslated_keys(&source_map, &target_map)
    } else {
        // Default: only missing keys
        find_missing_keys(&source_map, &target_map)
    };

    if keys_to_translate.is_empty() {
        println!(
            "  {} {} - {} up to date",
            "✓".green(),
            namespace.cyan(),
            "already".dimmed()
        );
        return Ok(());
    }

    // Show what will be translated
    let mode_label = if options.force {
        "all keys (force)"
    } else if options.detect_untranslated {
        "keys with English values"
    } else {
        "missing key(s)"
    };

    println!(
        "  {} {} - {} {}",
        "→".cyan(),
        namespace,
        keys_to_translate.len().to_string().yellow(),
        mode_label.dimmed()
    );

    // Flatten nested structures for translation
    let mut texts_to_translate: HashMap<String, serde_json::Value> = HashMap::new();

    for key in &keys_to_translate {
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
    if !options.dry_run {
        let output = formatter.write(&target_map)?;
        fs::write(&target_file, output)?;
        println!(
            "    {} Wrote {} translations to {}",
            "✓".green(),
            keys_to_translate.len(),
            target_file.display()
        );
    } else {
        println!(
            "    {} Would write {} translations (dry run)",
            "!".yellow(),
            keys_to_translate.len()
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

/// Collect all keys from source map (for --force mode)
fn collect_all_keys(source: &TranslationMap) -> Vec<String> {
    let mut all_keys = Vec::new();

    for (key, value) in source {
        match value {
            TranslationValue::String(_) | TranslationValue::Plural(_) => {
                all_keys.push(key.clone());
            }
            TranslationValue::Nested(nested_map) => {
                // Recursively collect nested keys
                let nested_keys = collect_all_keys(nested_map);
                for nested_key in nested_keys {
                    all_keys.push(format!("{}.{}", key, nested_key));
                }
            }
        }
    }

    all_keys
}

/// Find keys with English/untranslated values (for --detect-untranslated mode)
fn find_untranslated_keys(source: &TranslationMap, target: &TranslationMap) -> Vec<String> {
    let mut untranslated = Vec::new();

    for (key, source_value) in source {
        if !target.contains_key(key) {
            // Key doesn't exist - needs translation
            untranslated.push(key.clone());
        } else {
            let target_value = target.get(key).unwrap();

            match (source_value, target_value) {
                (TranslationValue::String(source_text), TranslationValue::String(target_text)) => {
                    // Check if target value is still in English
                    if is_english_value(source_text, target_text) {
                        untranslated.push(key.clone());
                    }
                }
                (
                    TranslationValue::Nested(source_nested),
                    TranslationValue::Nested(target_nested),
                ) => {
                    // Recursively check nested structures
                    let nested_untranslated = find_untranslated_keys(source_nested, target_nested);
                    for nested_key in nested_untranslated {
                        untranslated.push(format!("{}.{}", key, nested_key));
                    }
                }
                (
                    TranslationValue::Plural(source_plural),
                    TranslationValue::Plural(target_plural),
                ) => {
                    // Check 'other' form (always present)
                    if is_english_value(&source_plural.other, &target_plural.other) {
                        untranslated.push(format!("{}.other", key));
                    }
                    // Check 'one' form if present
                    if let (Some(src), Some(tgt)) = (&source_plural.one, &target_plural.one) {
                        if is_english_value(src, tgt) {
                            untranslated.push(format!("{}.one", key));
                        }
                    }
                    // Check other optional forms
                    if let (Some(src), Some(tgt)) = (&source_plural.zero, &target_plural.zero) {
                        if is_english_value(src, tgt) {
                            untranslated.push(format!("{}.zero", key));
                        }
                    }
                    if let (Some(src), Some(tgt)) = (&source_plural.two, &target_plural.two) {
                        if is_english_value(src, tgt) {
                            untranslated.push(format!("{}.two", key));
                        }
                    }
                    if let (Some(src), Some(tgt)) = (&source_plural.few, &target_plural.few) {
                        if is_english_value(src, tgt) {
                            untranslated.push(format!("{}.few", key));
                        }
                    }
                    if let (Some(src), Some(tgt)) = (&source_plural.many, &target_plural.many) {
                        if is_english_value(src, tgt) {
                            untranslated.push(format!("{}.many", key));
                        }
                    }
                }
                _ => {
                    // Type mismatch - needs retranslation
                    untranslated.push(key.clone());
                }
            }
        }
    }

    untranslated
}

/// Check if a value appears to be in English (not translated)
fn is_english_value(source_text: &str, target_text: &str) -> bool {
    // Exact match with source - definitely not translated
    if source_text == target_text {
        return true;
    }

    // Common English words that indicate untranslated text
    let english_indicators = [
        "the",
        "is",
        "and",
        "or",
        "to",
        "a",
        "an",
        "in",
        "on",
        "at",
        "for",
        "loading",
        "error",
        "success",
        "warning",
        "info",
        "cancel",
        "save",
        "delete",
        "edit",
        "add",
        "remove",
        "close",
        "back",
        "next",
        "retry",
        "apply",
        "clear",
        "filter",
        "search",
        "login",
        "logout",
        "password",
        "email",
        "username",
        "dashboard",
        "settings",
        "profile",
        "please",
        "enter",
        "valid",
        "required",
        "invalid",
        "already",
        "exists",
    ];

    let lower_target = target_text.to_lowercase();

    // Count how many English indicator words are in the target text
    let english_word_count = english_indicators
        .iter()
        .filter(|&&word| {
            // Check for whole word matches (not substrings)
            lower_target
                .split(|c: char| !c.is_alphanumeric())
                .any(|w| w == word)
        })
        .count();

    // If multiple English words found, likely untranslated
    english_word_count >= 2
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
