//! Coverage command implementation.

use crate::cli::CoverageArgs;
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat, TranslationValue};
use colored::Colorize;
use std::collections::BTreeMap;
use std::fs;

pub async fn execute(args: CoverageArgs) -> Result<()> {
    println!("{} Analyzing translation coverage", "→".cyan().bold());

    // Load source (English) translations
    let source_dir = args.translation_dir.join("en");
    if !source_dir.exists() {
        return Err(crate::error::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory not found: {}", source_dir.display()),
        )));
    }

    let format = JsonFormat::new();
    let source_stats = calculate_stats(&source_dir, &format)?;

    println!(
        "{} Source language (en): {} key(s) in {} namespace(s)",
        "✓".green().bold(),
        source_stats.total_keys,
        source_stats.namespaces.len()
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

    // Calculate coverage for each language
    let mut coverage_results = Vec::new();

    for lang in &languages {
        let target_dir = args.translation_dir.join(lang);
        if !target_dir.exists() {
            coverage_results.push(CoverageResult {
                language: lang.clone(),
                coverage: 0.0,
                translated: 0,
                total: source_stats.total_keys,
                missing_keys: source_stats.total_keys,
            });
            continue;
        }

        let target_stats = calculate_stats(&target_dir, &format)?;
        let coverage = target_stats.total_keys as f32 / source_stats.total_keys as f32;
        let missing = source_stats.total_keys - target_stats.total_keys;

        coverage_results.push(CoverageResult {
            language: lang.clone(),
            coverage,
            translated: target_stats.total_keys,
            total: source_stats.total_keys,
            missing_keys: missing,
        });
    }

    // Display results
    println!("\n{} Coverage Report", "=".cyan().bold());
    println!("{:<12} {:>10} {:>10} {:>10} {:>12}", "Language", "Coverage", "Translated", "Total", "Missing");
    println!("{}", "─".repeat(60));

    let mut has_failures = false;

    for result in &coverage_results {
        let coverage_str = format!("{:.1}%", result.coverage * 100.0);
        let coverage_colored = if result.coverage >= 0.9 {
            coverage_str.green()
        } else if result.coverage >= 0.7 {
            coverage_str.yellow()
        } else {
            coverage_str.red()
        };

        println!(
            "{:<12} {:>10} {:>10} {:>10} {:>12}",
            result.language,
            coverage_colored,
            result.translated,
            result.total,
            result.missing_keys
        );

        // Check threshold
        if let Some(min_coverage) = args.min_coverage {
            if result.coverage < min_coverage {
                has_failures = true;
            }
        }
    }

    // Summary
    let avg_coverage = coverage_results.iter().map(|r| r.coverage).sum::<f32>()
        / coverage_results.len() as f32;

    println!("\n{} Average Coverage: {:.1}%", "=".cyan().bold(), avg_coverage * 100.0);

    // Check threshold
    if let Some(min_coverage) = args.min_coverage {
        if has_failures {
            println!(
                "\n{} Coverage check failed (minimum: {:.1}%)",
                "✗".red().bold(),
                min_coverage * 100.0
            );
            std::process::exit(1);
        } else {
            println!(
                "\n{} Coverage check passed (minimum: {:.1}%)",
                "✓".green().bold(),
                min_coverage * 100.0
            );
        }
    }

    Ok(())
}

struct TranslationStats {
    total_keys: usize,
    namespaces: BTreeMap<String, usize>,
}

struct CoverageResult {
    language: String,
    coverage: f32,
    translated: usize,
    total: usize,
    missing_keys: usize,
}

fn calculate_stats(dir: &std::path::PathBuf, format: &JsonFormat) -> Result<TranslationStats> {
    let mut total_keys = 0;
    let mut namespaces = BTreeMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let map = format.parse(&content)?;
            let key_count = count_keys(&map);

            total_keys += key_count;

            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                namespaces.insert(name.to_string(), key_count);
            }
        }
    }

    Ok(TranslationStats {
        total_keys,
        namespaces,
    })
}

fn count_keys(map: &BTreeMap<String, TranslationValue>) -> usize {
    let mut count = 0;

    for value in map.values() {
        match value {
            TranslationValue::String(_) => count += 1,
            TranslationValue::Plural(_) => count += 1,
            TranslationValue::Nested(nested) => count += count_keys(nested),
        }
    }

    count
}

fn discover_languages(
    translation_dir: &std::path::PathBuf,
    exclude: &str,
) -> Result<Vec<String>> {
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
