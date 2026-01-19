//! Extract command implementation - extracts translatable strings from source code

use crate::cli::{ExtractionFormat, KeyStrategyArg};
use crate::error::Result;
use crate::extraction::{ExtractedString, KeyStrategy, Merger};
use crate::formats::{JsonFormat, PropertiesFormat, TranslationFormat, TranslationMap, YamlFormat};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};

pub async fn execute(args: crate::cli::ExtractArgs) -> Result<()> {
    println!("{} Starting string extraction...", "→".cyan().bold());

    // Collect source files
    let files = collect_source_files(&args.source, &args.patterns)?;

    if files.is_empty() {
        println!("{} No files found matching patterns", "!".yellow());
        return Ok(());
    }

    println!("{} Found {} files to scan", "✓".green().bold(), files.len());

    // Extract strings from all files
    let progress = ProgressBar::new(files.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut all_extracted: Vec<ExtractedString> = Vec::new();

    for file in &files {
        progress.set_message(format!("Scanning {}", file.display()));

        // Detect language from extension
        let extractor = create_extractor(file)?;

        if let Some(extractor) = extractor {
            let extracted = extractor.extract_file(file).await?;
            all_extracted.extend(extracted);
        }

        progress.inc(1);
    }

    progress.finish_with_message("Extraction complete");

    // Deduplicate strings (same text = one entry, track all locations)
    let unique_strings = deduplicate_strings(all_extracted);

    println!(
        "{} Extracted {} unique strings",
        "✓".green().bold(),
        unique_strings.len()
    );

    if unique_strings.is_empty() {
        println!("{} No translatable strings found", "!".yellow());
        return Ok(());
    }

    // Load existing translations if merging
    let existing = if args.merge {
        load_existing_translations(&args.output, &args.format)?
    } else {
        TranslationMap::new()
    };

    // Merge with existing translations
    let key_strategy = match args.key_strategy {
        KeyStrategyArg::Semantic => KeyStrategy::Semantic,
        KeyStrategyArg::Hash => KeyStrategy::Hash,
        KeyStrategyArg::Incremental => KeyStrategy::Incremental,
    };

    let merger = Merger::new(key_strategy);
    let (merged, report) = merger.merge(&existing, unique_strings)?;

    // Display merge report
    println!("\n{} Merge report:", "→".cyan().bold());
    println!("  Added: {}", report.added.to_string().green());
    println!(
        "  Skipped (existing): {}",
        report.skipped.to_string().yellow()
    );

    if !report.conflicts.is_empty() {
        println!("  Conflicts: {}", report.conflicts.len().to_string().red());
        for conflict in &report.conflicts {
            println!("    - {}", conflict);
        }
    }

    // Write translations to file if not dry-run
    if args.dry_run {
        println!("\n{} Dry-run mode: no files written", "!".yellow().bold());
        println!("\nWould write to: {}", args.output.display());
    } else {
        write_translations(&args.output, &merged, &args.format)?;
        println!(
            "\n{} Wrote translations to {}",
            "✓".green().bold(),
            args.output.display()
        );
    }

    // Show key mapping if verbose (sample)
    if !args.dry_run && report.added > 0 {
        println!("\n{} Sample generated keys:", "→".cyan().bold());
        let mut count = 0;
        for (key, extracted) in report.key_mapping.iter().take(5) {
            println!("  {} → \"{}\"", key.cyan(), extracted.text);
            count += 1;
        }
        if report.added > count {
            println!("  ... and {} more", report.added - count);
        }
    }

    Ok(())
}

/// Collect source files matching patterns
fn collect_source_files(source_dirs: &[PathBuf], patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for dir in source_dirs {
        if !dir.exists() {
            return Err(crate::error::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source directory not found: {}", dir.display()),
            )));
        }

        for entry in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() {
                // Check if file matches any pattern
                if matches_pattern(path, patterns) {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

/// Check if file matches any of the glob patterns
fn matches_pattern(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return true; // No patterns = match all
    }

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    for pattern in patterns {
        // Simple pattern matching: "*.tsx" → ends_with(".tsx")
        if let Some(ext) = pattern.strip_prefix("*.") {
            if file_name.ends_with(ext) {
                return true;
            }
        } else if pattern == file_name {
            return true;
        }
    }

    false
}

/// Create appropriate extractor based on file extension
fn create_extractor(file: &Path) -> Result<Option<Box<dyn crate::extraction::Extractor>>> {
    let extension = file.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "ts" | "tsx" | "js" | "jsx" => Ok(Some(Box::new(
            crate::extraction::TypeScriptExtractor::new(),
        ))),
        "rs" => Ok(Some(Box::new(crate::extraction::RustExtractor::new()))),
        _ => Ok(None),
    }
}

/// Deduplicate extracted strings by text content
///
/// Keeps the first occurrence, discards duplicates
fn deduplicate_strings(mut extracted: Vec<ExtractedString>) -> Vec<ExtractedString> {
    let mut seen = std::collections::HashSet::new();
    let mut unique = Vec::new();

    for item in extracted.drain(..) {
        if seen.insert(item.text.clone()) {
            unique.push(item);
        }
    }

    unique
}

/// Load existing translations from file
fn load_existing_translations(output: &Path, format: &ExtractionFormat) -> Result<TranslationMap> {
    if !output.exists() {
        return Ok(TranslationMap::new());
    }

    let content = fs::read_to_string(output)?;

    let formatter: Box<dyn TranslationFormat> = match format {
        ExtractionFormat::Json => Box::new(JsonFormat::new()),
        ExtractionFormat::Yaml => Box::new(YamlFormat),
        ExtractionFormat::Properties => Box::new(PropertiesFormat::new()),
    };

    let map = formatter.parse(&content)?;
    Ok(map)
}

/// Write translations to file
fn write_translations(
    output: &Path,
    translations: &TranslationMap,
    format: &ExtractionFormat,
) -> Result<()> {
    // Ensure output directory exists
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let formatter: Box<dyn TranslationFormat> = match format {
        ExtractionFormat::Json => Box::new(JsonFormat::new()),
        ExtractionFormat::Yaml => Box::new(YamlFormat),
        ExtractionFormat::Properties => Box::new(PropertiesFormat::new()),
    };

    let content = formatter.write(translations)?;
    fs::write(output, content)?;

    Ok(())
}
