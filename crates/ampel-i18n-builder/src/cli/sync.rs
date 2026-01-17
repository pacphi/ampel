//! Sync command implementation.

use crate::cli::{SyncArgs, TranslateArgs};
use crate::error::Result;
use colored::Colorize;
use std::fs;

pub async fn execute(args: SyncArgs) -> Result<()> {
    println!(
        "{} Syncing all languages from {} using {:?}",
        "→".cyan().bold(),
        args.source.green(),
        args.provider
    );

    // Discover all target languages
    let languages = discover_languages(&args.translation_dir, &args.source)?;

    if languages.is_empty() {
        println!("{} No target languages found", "!".yellow());
        return Ok(());
    }

    println!(
        "{} Found {} language(s): {}",
        "✓".green().bold(),
        languages.len(),
        languages.join(", ")
    );

    // Translate each language
    for lang in languages {
        println!("\n{} Processing {}", "→".cyan().bold(), lang.green());

        let translate_args = TranslateArgs {
            lang,
            provider: Some(args.provider),
            namespace: None,
            dry_run: args.dry_run,
            translation_dir: args.translation_dir.clone(),
            timeout: None,
            batch_size: None,
            max_retries: None,
            disabled_providers: vec![],
            no_fallback: true, // Sync uses explicit provider, no fallback
            force: false,
            detect_untranslated: false,
        };

        crate::cli::translate::execute(translate_args).await?;
    }

    println!("\n{} Sync complete!", "✓".green().bold());

    Ok(())
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
