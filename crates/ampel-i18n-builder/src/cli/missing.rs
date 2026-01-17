//! Missing command implementation - lists keys missing from each language.

use crate::cli::MissingArgs;
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat, TranslationValue};
use colored::Colorize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

pub async fn execute(args: MissingArgs) -> Result<()> {
    println!("{} Checking for missing translations", "→".cyan().bold());

    // Load source (English) translations
    let source_dir = args.translation_dir.join("en");
    if !source_dir.exists() {
        return Err(crate::error::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory not found: {}", source_dir.display()),
        )));
    }

    let format = JsonFormat::new();
    let source_keys = collect_keys(&source_dir, &format)?;

    println!(
        "{} Source language (en): {} key(s)",
        "✓".green().bold(),
        source_keys.len()
    );

    // Discover languages to check
    let languages = if let Some(lang) = args.lang {
        vec![lang]
    } else {
        discover_languages(&args.translation_dir, "en")?
    };

    if languages.is_empty() {
        println!("{} No target languages found", "!".yellow());
        return Ok(());
    }

    let mut total_missing = 0;
    let mut languages_with_missing = Vec::new();

    for lang in &languages {
        let target_dir = args.translation_dir.join(lang);
        if !target_dir.exists() {
            println!(
                "\n{} {}: Directory not found - all {} keys missing",
                "✗".red().bold(),
                lang,
                source_keys.len()
            );
            total_missing += source_keys.len();
            languages_with_missing.push((lang.clone(), source_keys.clone()));
            continue;
        }

        let target_keys = collect_keys(&target_dir, &format)?;
        let missing: BTreeSet<_> = source_keys.difference(&target_keys).cloned().collect();

        if !missing.is_empty() {
            println!(
                "\n{} {}: {} missing key(s)",
                "✗".red().bold(),
                lang,
                missing.len()
            );
            for key in &missing {
                println!("  - {}", key);
            }
            total_missing += missing.len();
            languages_with_missing.push((lang.clone(), missing));
        }
    }

    // Summary
    println!();
    if total_missing > 0 {
        println!(
            "{} Missing translations found: {} key(s) across {} language(s)",
            "✗".red().bold(),
            total_missing,
            languages_with_missing.len()
        );
        std::process::exit(1);
    } else {
        println!(
            "{} All translations complete - no missing keys",
            "✓".green().bold()
        );
    }

    Ok(())
}

fn collect_keys(dir: &PathBuf, format: &JsonFormat) -> Result<BTreeSet<String>> {
    let mut keys = BTreeSet::new();

    if !dir.exists() {
        return Ok(keys);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let namespace = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let content = fs::read_to_string(&path)?;
            let map = format.parse(&content)?;
            collect_keys_recursive(&map, namespace, &mut keys);
        }
    }

    Ok(keys)
}

fn collect_keys_recursive(
    map: &BTreeMap<String, TranslationValue>,
    prefix: &str,
    keys: &mut BTreeSet<String>,
) {
    for (key, value) in map {
        let full_key = format!("{}.{}", prefix, key);
        match value {
            TranslationValue::String(_) => {
                keys.insert(full_key);
            }
            TranslationValue::Plural(_) => {
                keys.insert(full_key);
            }
            TranslationValue::Nested(nested) => {
                collect_keys_recursive(nested, &full_key, keys);
            }
        }
    }
}

fn discover_languages(translation_dir: &PathBuf, exclude: &str) -> Result<Vec<String>> {
    let mut languages = Vec::new();

    if !translation_dir.exists() {
        return Ok(languages);
    }

    for entry in fs::read_dir(translation_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(lang) = path.file_name().and_then(|n| n.to_str()) {
                if lang != exclude && !lang.starts_with('.') {
                    languages.push(lang.to_string());
                }
            }
        }
    }

    languages.sort();
    Ok(languages)
}
