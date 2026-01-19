//! Interactive setup wizard for first-time users.

use crate::cli::InitArgs;
use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub async fn execute(args: InitArgs) -> Result<()> {
    println!(
        "{}",
        "🚀 ampel-i18n-builder Setup Wizard".bright_cyan().bold()
    );
    println!("{}", "================================".bright_cyan());
    println!();

    // Check if config already exists
    if Path::new(".ampel-i18n.yaml").exists() && !confirm("Config file already exists. Overwrite?")?
    {
        println!("{}", "Setup cancelled.".yellow());
        return Ok(());
    }

    // Step 1: Framework detection
    let framework = if let Some(f) = args.framework {
        f
    } else if args.non_interactive {
        "react".to_string()
    } else {
        prompt_framework()?
    };

    // Step 2: Translation directory
    let translation_dir = if let Some(dir) = args.translation_dir {
        dir.to_string_lossy().to_string()
    } else if args.non_interactive {
        get_default_translation_dir(&framework)
    } else {
        prompt_translation_dir(&framework)?
    };

    // Step 3: Target languages
    let languages = if let Some(langs) = args.languages {
        langs.split(',').map(|s| s.trim().to_string()).collect()
    } else if args.non_interactive {
        vec!["fr".to_string(), "de".to_string(), "es".to_string()]
    } else {
        prompt_languages()?
    };

    // Step 4: Provider selection
    let provider = if let Some(p) = args.provider {
        p
    } else if args.non_interactive {
        "openai".to_string()
    } else {
        prompt_provider()?
    };

    // Generate .ampel-i18n.yaml
    let config_content = generate_config(&translation_dir, &provider);
    fs::write(".ampel-i18n.yaml", config_content)?;
    println!("{} .ampel-i18n.yaml", "✓ Created".green().bold());

    // Generate .env template
    let env_content = generate_env_template(&provider);
    let env_path = ".env";
    if !Path::new(env_path).exists() {
        fs::write(env_path, env_content)?;
        println!("{} .env (with template)", "✓ Created".green().bold());
    } else {
        println!(
            "{} .env already exists, not overwriting",
            "⊙ Skipped".yellow()
        );
    }

    // Create translation directory structure
    create_translation_structure(&translation_dir, &languages)?;

    // Summary
    println!();
    println!("{}", "🎉 Setup Complete!".bright_green().bold());
    println!();
    println!("Next steps:");
    println!("  1. {} - Add your API key to .env", "Edit".cyan());
    println!("  2. {} - Add source translations", "Edit".cyan());
    println!("  3. {} - Generate translations", "Run".cyan());
    println!();
    println!("Example commands:");
    println!(
        "  {} - Check translation status",
        "ampel-i18n coverage".bright_white()
    );
    println!("  {} - Sync translations", "ampel-i18n sync".bright_white());
    println!();

    Ok(())
}

fn prompt_framework() -> Result<String> {
    println!("{}", "Step 1: What framework are you using?".bold());
    println!("  1) React (i18next)");
    println!("  2) Vue (vue-i18n)");
    println!("  3) Rust (rust-i18n)");
    println!("  4) Other");
    print!("Choice [1-4]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" | "react" | "React" => Ok("react".to_string()),
        "2" | "vue" | "Vue" => Ok("vue".to_string()),
        "3" | "rust" | "Rust" => Ok("rust".to_string()),
        "4" | "other" | "Other" => Ok("other".to_string()),
        _ => Ok("react".to_string()), // default
    }
}

fn prompt_translation_dir(framework: &str) -> Result<String> {
    let default = get_default_translation_dir(framework);
    println!();
    println!(
        "{}",
        "Step 2: Where should translation files be stored?".bold()
    );
    print!("Path [{}]: ", default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let dir = input.trim();
    Ok(if dir.is_empty() {
        default
    } else {
        dir.to_string()
    })
}

fn prompt_languages() -> Result<Vec<String>> {
    println!();
    println!(
        "{}",
        "Step 3: Which languages do you want to support?".bold()
    );
    println!(
        "Common choices: fr (French), de (German), es (Spanish), ja (Japanese), zh-CN (Chinese)"
    );
    print!("Languages (comma-separated) [fr,de,es]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();
    if input.is_empty() {
        Ok(vec!["fr".to_string(), "de".to_string(), "es".to_string()])
    } else {
        Ok(input.split(',').map(|s| s.trim().to_string()).collect())
    }
}

fn prompt_provider() -> Result<String> {
    println!();
    println!(
        "{}",
        "Step 4: Which translation provider do you want to use?".bold()
    );
    println!("  1) OpenAI (recommended - easiest to set up)");
    println!("  2) DeepL (best for European languages)");
    println!("  3) Google Translate (broadest coverage)");
    println!("  4) Systran (enterprise quality)");
    print!("Choice [1-4]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" | "openai" | "OpenAI" => Ok("openai".to_string()),
        "2" | "deepl" | "DeepL" => Ok("deepl".to_string()),
        "3" | "google" | "Google" => Ok("google".to_string()),
        "4" | "systran" | "Systran" => Ok("systran".to_string()),
        _ => Ok("openai".to_string()), // default
    }
}

fn confirm(message: &str) -> Result<bool> {
    print!("{} [y/N]: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn get_default_translation_dir(framework: &str) -> String {
    match framework {
        "react" => "src/locales".to_string(),
        "vue" => "src/i18n/locales".to_string(),
        "rust" => "locales".to_string(),
        _ => "locales".to_string(),
    }
}

fn generate_config(translation_dir: &str, provider: &str) -> String {
    format!(
        r#"# ampel-i18n-builder Configuration
# Generated by: ampel-i18n init

# Directory containing translation files
translation_dir: {}

# Translation provider configuration
translation:
  {}_api_key: "${{{}}}_API_KEY}}"

  # Default settings
  default_timeout_secs: 30
  default_batch_size: 50
  default_max_retries: 3

  # Provider-specific configuration
  providers:
    {}:
      enabled: true
      priority: 1
"#,
        translation_dir,
        provider,
        provider.to_uppercase(),
        provider
    )
}

fn generate_env_template(provider: &str) -> String {
    let provider_upper = provider.to_uppercase();
    let api_url = match provider {
        "openai" => "https://platform.openai.com/api-keys",
        "deepl" => "https://www.deepl.com/pro-api",
        "google" => "https://cloud.google.com/translate",
        "systran" => "https://platform.systran.net/",
        _ => "https://platform.openai.com/api-keys",
    };

    format!(
        r#"# Translation Provider API Key
# Get your API key from: {}
{}_API_KEY=your-api-key-here
"#,
        api_url, provider_upper
    )
}

fn create_translation_structure(base_dir: &str, _languages: &[String]) -> Result<()> {
    // Create base translation directory
    fs::create_dir_all(base_dir)?;

    // Create en (source) directory and sample file
    let en_dir = Path::new(base_dir).join("en");
    fs::create_dir_all(&en_dir)?;

    let sample_content = r#"{
  "welcome": "Welcome",
  "greeting": "Hello, {{name}}!",
  "itemCount": "You have {{count}} items"
}
"#;

    fs::write(en_dir.join("common.json"), sample_content)?;

    println!("{} {} structure", "✓ Created".green().bold(), base_dir);
    println!("  └── en/common.json (sample source file)");

    Ok(())
}
