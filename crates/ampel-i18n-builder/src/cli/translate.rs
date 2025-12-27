//! Translate command implementation.

use crate::cli::TranslateArgs;
use crate::config::Config;
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat, TranslationMap, TranslationValue};
use crate::translator::Translator;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub async fn execute(args: TranslateArgs) -> Result<()> {
    println!(
        "{} Translating to {} using {:?}",
        "→".cyan().bold(),
        args.lang.green(),
        args.provider
    );

    // Load configuration
    let config = Config::load()?;

    // Initialize translator
    let translator = Translator::new(args.provider, &config)?;

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
            &translator,
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
    translator: &Translator,
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

    // Prepare texts for translation
    let texts_to_translate: HashMap<String, serde_json::Value> = missing_keys
        .iter()
        .filter_map(|key| {
            if let Some(TranslationValue::String(text)) = source_map.get(key) {
                Some((key.clone(), serde_json::Value::String(text.clone())))
            } else {
                None
            }
        })
        .collect();

    if texts_to_translate.is_empty() {
        println!("    {} No simple strings to translate (only nested/plural forms)", "!".yellow());
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

    // Update target map
    for (key, value) in translations {
        if let serde_json::Value::String(text) = value {
            target_map.insert(key, TranslationValue::String(text));
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
