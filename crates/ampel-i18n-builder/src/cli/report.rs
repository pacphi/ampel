//! Report command implementation - generates coverage reports in various formats.

use crate::cli::{ReportArgs, ReportFormat};
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat, TranslationValue};
use std::collections::BTreeMap;
use std::fs;

pub async fn execute(args: ReportArgs) -> Result<()> {
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

    // Discover languages to check
    let languages = if let Some(lang) = args.lang {
        vec![lang]
    } else {
        discover_languages(&args.translation_dir, "en")?
    };

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
        let coverage = (target_stats.total_keys as f32 / source_stats.total_keys as f32).min(1.0);
        let missing = source_stats
            .total_keys
            .saturating_sub(target_stats.total_keys);

        coverage_results.push(CoverageResult {
            language: lang.clone(),
            coverage,
            translated: target_stats.total_keys.min(source_stats.total_keys),
            total: source_stats.total_keys,
            missing_keys: missing,
        });
    }

    // Calculate overall coverage
    let overall_coverage = if coverage_results.is_empty() {
        0.0
    } else {
        coverage_results.iter().map(|r| r.coverage).sum::<f32>() / coverage_results.len() as f32
    };

    // Output in requested format
    match args.format {
        ReportFormat::Json => {
            let report = JsonReport {
                overall_coverage: overall_coverage * 100.0,
                source_keys: source_stats.total_keys,
                languages: coverage_results
                    .iter()
                    .map(|r| LanguageReport {
                        language: r.language.clone(),
                        coverage: r.coverage * 100.0,
                        translated: r.translated,
                        total: r.total,
                        missing: r.missing_keys,
                    })
                    .collect(),
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        ReportFormat::Markdown => {
            println!("# Translation Coverage Report\n");
            println!("**Overall Coverage:** {:.1}%\n", overall_coverage * 100.0);
            println!("**Source Keys:** {}\n", source_stats.total_keys);
            println!("## Language Coverage\n");
            println!("| Language | Coverage | Translated | Total | Missing |");
            println!("|----------|----------|------------|-------|---------|");

            for result in &coverage_results {
                let status = if result.coverage >= 0.95 {
                    "✅"
                } else if result.coverage >= 0.80 {
                    "⚠️"
                } else {
                    "❌"
                };
                println!(
                    "| {} {} | {:.1}% | {} | {} | {} |",
                    result.language,
                    status,
                    result.coverage * 100.0,
                    result.translated,
                    result.total,
                    result.missing_keys
                );
            }
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct JsonReport {
    overall_coverage: f32,
    source_keys: usize,
    languages: Vec<LanguageReport>,
}

#[derive(serde::Serialize)]
struct LanguageReport {
    language: String,
    coverage: f32,
    translated: usize,
    total: usize,
    missing: usize,
}

struct TranslationStats {
    total_keys: usize,
    #[allow(dead_code)]
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

fn discover_languages(translation_dir: &std::path::PathBuf, exclude: &str) -> Result<Vec<String>> {
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
