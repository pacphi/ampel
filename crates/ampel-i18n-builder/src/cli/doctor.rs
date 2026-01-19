//! Health check and diagnostic tool for ampel-i18n-builder.

use crate::cli::DoctorArgs;
use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use std::process::Command;

pub async fn execute(args: DoctorArgs) -> Result<()> {
    println!(
        "{}",
        "🩺 ampel-i18n-builder Health Check".bright_cyan().bold()
    );
    println!("{}", "=================================".bright_cyan());
    println!();

    let mut issues = vec![];
    let mut warnings = vec![];

    // Check 1: Rust and Cargo versions
    check_rust_cargo(&mut issues, &mut warnings, args.verbose);

    // Check 2: Config file exists and is valid
    check_config_file(&mut issues, &mut warnings, args.verbose);

    // Check 3: API keys configured
    check_api_keys(&mut issues, &mut warnings, args.verbose);

    // Check 4: Translation directory exists
    check_translation_dir(&mut issues, &mut warnings, args.verbose);

    // Check 5: At least one provider enabled
    check_providers(&mut issues, &mut warnings, args.verbose);

    // Summary
    println!();
    println!("{}", "═══════════════════════════════".bright_cyan());
    println!();

    if issues.is_empty() && warnings.is_empty() {
        println!(
            "{}",
            "✅ All checks passed! Ready to translate."
                .bright_green()
                .bold()
        );
    } else {
        if !warnings.is_empty() {
            println!(
                "{} {}",
                "⚠️ ".yellow(),
                format!("{} warnings found:", warnings.len())
                    .yellow()
                    .bold()
            );
            for warning in &warnings {
                println!("   {}", warning.yellow());
            }
            println!();
        }

        if !issues.is_empty() {
            println!(
                "{} {}",
                "❌".red(),
                format!("{} issues found:", issues.len()).red().bold()
            );
            for issue in &issues {
                println!("   {}", issue.red());
            }
            println!();

            if args.fix {
                println!("{}", "Attempting automatic fixes...".yellow());
                attempt_fixes(&issues)?;
            } else {
                println!(
                    "{}",
                    "Run with --fix to attempt automatic fixes".bright_white()
                );
            }
        }
    }

    Ok(())
}

fn check_rust_cargo(issues: &mut Vec<String>, _warnings: &mut Vec<String>, verbose: bool) {
    if verbose {
        println!("{}", "Checking Rust and Cargo...".bright_white());
    }

    // Check cargo version
    match Command::new("cargo").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            if verbose || !output.status.success() {
                println!(
                    "  {} Cargo: {}",
                    if output.status.success() {
                        "✓".green()
                    } else {
                        "✗".red()
                    },
                    version.trim()
                );
            } else {
                println!("  {} Cargo installed", "✓".green());
            }
            if !output.status.success() {
                issues.push("Cargo not found or not working".to_string());
            }
        }
        Err(_) => {
            println!("  {} Cargo: not found", "✗".red());
            issues.push("Cargo is not installed. Install from https://rustup.rs/".to_string());
        }
    }

    // Check rustc version
    match Command::new("rustc").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            if verbose {
                println!(
                    "  {} Rust: {}",
                    if output.status.success() {
                        "✓".green()
                    } else {
                        "✗".red()
                    },
                    version.trim()
                );
            }
        }
        Err(_) => {
            if verbose {
                println!("  {} Rust: not found", "✗".red());
            }
        }
    }

    println!();
}

fn check_config_file(issues: &mut Vec<String>, warnings: &mut Vec<String>, verbose: bool) {
    if verbose {
        println!("{}", "Checking configuration file...".bright_white());
    }

    if !Path::new(".ampel-i18n.yaml").exists() {
        println!("  {} Config file: not found", "✗".red());
        issues.push(".ampel-i18n.yaml not found. Run 'ampel-i18n init' to create it".to_string());
    } else {
        println!("  {} Config file: found", "✓".green());

        // Try to load and validate config
        match Config::load() {
            Ok(config) => {
                if verbose {
                    println!("  {} Config is valid", "✓".green());
                    println!(
                        "    └── Translation dir: {}",
                        config.translation_dir.display()
                    );
                }
            }
            Err(e) => {
                println!("  {} Config validation failed", "✗".red());
                issues.push(format!("Config file is invalid: {}", e));
            }
        }

        // Check for deprecated fields
        if let Ok(content) = std::fs::read_to_string(".ampel-i18n.yaml") {
            if content.contains("timeout_secs:") && !content.contains("default_timeout_secs:") {
                warnings.push(
                    "Using deprecated 'timeout_secs'. Use 'default_timeout_secs' instead"
                        .to_string(),
                );
            }
            if content.contains("batch_size:") && !content.contains("default_batch_size:") {
                warnings.push(
                    "Using deprecated 'batch_size'. Use 'default_batch_size' instead".to_string(),
                );
            }
        }
    }

    println!();
}

fn check_api_keys(issues: &mut Vec<String>, warnings: &mut Vec<String>, verbose: bool) {
    if verbose {
        println!("{}", "Checking API keys...".bright_white());
    }

    let providers = [
        ("SYSTRAN_API_KEY", "Systran"),
        ("DEEPL_API_KEY", "DeepL"),
        ("GOOGLE_API_KEY", "Google"),
        ("OPENAI_API_KEY", "OpenAI"),
    ];

    let mut found_count = 0;

    for (env_var, name) in providers {
        match std::env::var(env_var) {
            Ok(key) if !key.is_empty() && key != "your-api-key-here" => {
                println!("  {} {}: configured", "✓".green(), name);
                found_count += 1;
            }
            _ => {
                if verbose {
                    println!("  {} {}: not configured", "⊙".yellow(), name);
                }
            }
        }
    }

    if found_count == 0 {
        println!("  {} No API keys configured", "✗".red());
        issues.push("At least one translation provider API key is required in .env".to_string());
    } else if found_count == 1 {
        println!("  {} 1 provider configured (sufficient)", "✓".green());
    } else {
        println!(
            "  {} {} providers configured (redundancy enabled)",
            "✓".green(),
            found_count
        );
    }

    // Check for .env file
    if !Path::new(".env").exists() {
        warnings.push(".env file not found. Create it with provider API keys".to_string());
    }

    println!();
}

fn check_translation_dir(issues: &mut Vec<String>, _warnings: &mut Vec<String>, verbose: bool) {
    if verbose {
        println!("{}", "Checking translation directory...".bright_white());
    }

    // Try to load config to get translation_dir
    let translation_dir = match Config::load() {
        Ok(config) => config.translation_dir,
        Err(_) => {
            // Use default if config not found
            std::path::PathBuf::from("frontend/public/locales")
        }
    };

    if translation_dir.exists() {
        println!(
            "  {} Translation directory exists: {}",
            "✓".green(),
            translation_dir.display()
        );

        // Check for en (source) directory
        let en_dir = translation_dir.join("en");
        if en_dir.exists() {
            println!("  {} Source language directory found (en/)", "✓".green());
        } else {
            println!(
                "  {} Source language directory not found (en/)",
                "⊙".yellow()
            );
        }
    } else {
        println!(
            "  {} Translation directory not found: {}",
            "✗".red(),
            translation_dir.display()
        );
        issues.push(format!(
            "Translation directory does not exist: {}. Create it or update .ampel-i18n.yaml",
            translation_dir.display()
        ));
    }

    println!();
}

fn check_providers(issues: &mut Vec<String>, _warnings: &mut Vec<String>, verbose: bool) {
    if verbose {
        println!("{}", "Checking provider configuration...".bright_white());
    }

    match Config::load() {
        Ok(config) => {
            let enabled_providers = [
                ("Systran", config.translation.providers.systran.enabled),
                ("DeepL", config.translation.providers.deepl.enabled),
                ("Google", config.translation.providers.google.enabled),
                ("OpenAI", config.translation.providers.openai.enabled),
            ];

            let enabled_count = enabled_providers
                .iter()
                .filter(|(_, enabled)| *enabled)
                .count();

            if enabled_count == 0 {
                println!("  {} No providers enabled", "✗".red());
                issues
                    .push("At least one provider must be enabled in .ampel-i18n.yaml".to_string());
            } else {
                println!("  {} {} provider(s) enabled", "✓".green(), enabled_count);
                for (name, enabled) in enabled_providers {
                    if enabled && verbose {
                        println!("    └── {}", name);
                    }
                }
            }
        }
        Err(_) => {
            // Already reported in check_config_file
        }
    }

    println!();
}

fn attempt_fixes(issues: &[String]) -> Result<()> {
    for issue in issues {
        if issue.contains("ampel-i18n init") {
            println!(
                "  {} Suggested fix: Run 'ampel-i18n init' to set up configuration",
                "💡".yellow()
            );
        } else if issue.contains(".env") {
            println!(
                "  {} Suggested fix: Create .env file with API keys",
                "💡".yellow()
            );
        } else if issue.contains("Translation directory does not exist") {
            println!(
                "  {} Suggested fix: Create the translation directory manually",
                "💡".yellow()
            );
        } else if issue.contains("Config file is invalid") {
            println!(
                "  {} Suggested fix: Check YAML syntax in .ampel-i18n.yaml",
                "💡".yellow()
            );
        }
    }

    Ok(())
}
