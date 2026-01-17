//! Validate command implementation.

use crate::cli::ValidateArgs;
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat};
use crate::validation::{
    CoverageValidator, DuplicateKeysValidator, MissingKeysValidator, ValidationResult,
    ValidationResults, Validator, VariableValidator,
};
use colored::Colorize;
use std::fs;

pub async fn execute(args: ValidateArgs) -> Result<()> {
    println!("{} Validating translations", "→".cyan().bold());

    // Discover languages to validate
    let languages = if args.all {
        discover_all_languages(&args.translation_dir)?
    } else if let Some(lang) = args.lang {
        vec![lang]
    } else {
        return Err(crate::error::Error::Validation(
            "Must specify --all or --lang".to_string(),
        ));
    };

    println!(
        "{} Checking {} language(s): {}",
        "✓".green().bold(),
        languages.len(),
        languages.join(", ")
    );

    let format = JsonFormat::new();
    let mut all_results = ValidationResults::default();
    let mut has_errors = false;

    // Create validators
    let missing_validator = MissingKeysValidator;
    let duplicate_validator = DuplicateKeysValidator;
    let variable_validator = VariableValidator;
    let coverage_validator = CoverageValidator;

    // Validate each language
    for lang in &languages {
        println!("\n{} Validating {}", "→".cyan(), lang.green());

        let lang_dir = args.translation_dir.join(lang);
        if !lang_dir.exists() {
            println!("  {} Directory not found", "✗".red());
            continue;
        }

        // Load all namespace files
        let namespaces = load_namespaces(&lang_dir, &format)?;

        if namespaces.is_empty() {
            println!("  {} No translation files found", "!".yellow());
            continue;
        }

        // Run validators
        let source_namespaces = if lang != "en" {
            let source_dir = args.translation_dir.join("en");
            load_namespaces(&source_dir, &format)?
        } else {
            namespaces.clone()
        };

        // Missing keys validation
        let missing_results = missing_validator.validate(&source_namespaces, &namespaces)?;
        if !missing_results.errors.is_empty() {
            has_errors = true;
            println!("  {} Missing keys:", "✗".red());
            for error in &missing_results.errors {
                println!("    - {}", error);
            }
        }

        // Duplicate keys validation
        let duplicate_results = duplicate_validator.validate(&source_namespaces, &namespaces)?;
        if !duplicate_results.errors.is_empty() {
            has_errors = true;
            println!("  {} Duplicate keys:", "✗".red());
            for error in &duplicate_results.errors {
                println!("    - {}", error);
            }
        }

        // Variable validation
        let variable_results = variable_validator.validate(&source_namespaces, &namespaces)?;
        if !variable_results.errors.is_empty() {
            has_errors = true;
            println!("  {} Variable mismatches:", "✗".red());
            for error in &variable_results.errors {
                println!("    - {}", error);
            }
        }

        // Coverage validation
        let coverage_results = coverage_validator.validate(&source_namespaces, &namespaces)?;
        if !coverage_results.warnings.is_empty() {
            println!("  {} Coverage warnings:", "!".yellow());
            for warning in &coverage_results.warnings {
                println!("    - {}", warning);
            }
        }

        // Merge results
        for error in missing_results.errors {
            all_results.add_result(ValidationResult {
                validator_name: "missing_keys".to_string(),
                errors: vec![error],
                warnings: vec![],
            });
        }
        for error in duplicate_results.errors {
            all_results.add_result(ValidationResult {
                validator_name: "duplicate_keys".to_string(),
                errors: vec![error],
                warnings: vec![],
            });
        }
        for error in variable_results.errors {
            all_results.add_result(ValidationResult {
                validator_name: "variable_validator".to_string(),
                errors: vec![error],
                warnings: vec![],
            });
        }
        for warning in coverage_results.warnings {
            let mut result = ValidationResult::new("coverage");
            result.add_warning(warning);
            all_results.add_result(result);
        }

        if !has_errors {
            println!("  {} All validations passed", "✓".green());
        }
    }

    // Summary
    println!("\n{} Validation Summary", "=".cyan().bold());
    println!(
        "  Errors:   {}",
        all_results.total_errors().to_string().red()
    );
    println!(
        "  Warnings: {}",
        all_results.total_warnings().to_string().yellow()
    );

    if has_errors {
        println!("\n{} Validation failed", "✗".red().bold());
        std::process::exit(1);
    } else {
        println!("\n{} Validation passed", "✓".green().bold());
    }

    Ok(())
}

fn discover_all_languages(translation_dir: &std::path::PathBuf) -> Result<Vec<String>> {
    let mut languages = Vec::new();

    if !translation_dir.exists() {
        return Ok(languages);
    }

    for entry in fs::read_dir(translation_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(lang) = path.file_name().and_then(|n| n.to_str()) {
                if !lang.starts_with('.') {
                    languages.push(lang.to_string());
                }
            }
        }
    }

    languages.sort();
    Ok(languages)
}

fn load_namespaces(
    dir: &std::path::PathBuf,
    format: &JsonFormat,
) -> Result<Vec<(String, crate::formats::TranslationMap)>> {
    let mut namespaces = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let map = format.parse(&content)?;

            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                namespaces.push((name.to_string(), map));
            }
        }
    }

    Ok(namespaces)
}
