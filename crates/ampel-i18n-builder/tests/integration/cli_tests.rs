use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn ampel_i18n_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug/ampel-i18n")
}

#[test]
#[ignore = "Requires binary to be built"]
fn test_cli_help_command() {
    let output = Command::new(ampel_i18n_bin())
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ampel-i18n"));
    assert!(stdout.contains("USAGE:") || stdout.contains("Usage:"));
}

#[test]
#[ignore = "Requires binary to be built"]
fn test_cli_validate_command() {
    let output = Command::new(ampel_i18n_bin())
        .arg("validate")
        .arg("--source")
        .arg(fixtures_dir().join("en.yaml"))
        .arg("--target")
        .arg(fixtures_dir().join("ar.yaml"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(),
        "Validation should pass for complete translations");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("100%") || stdout.contains("valid"));
}

#[test]
#[ignore = "Requires binary to be built"]
fn test_cli_validate_incomplete() {
    let output = Command::new(ampel_i18n_bin())
        .arg("validate")
        .arg("--source")
        .arg(fixtures_dir().join("en.yaml"))
        .arg("--target")
        .arg(fixtures_dir().join("incomplete.yaml"))
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(),
        "Validation should fail for incomplete translations");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing") || stderr.contains("incomplete"));
}

#[test]
#[ignore = "Requires binary to be built"]
fn test_cli_coverage_report() {
    let temp_dir = TempDir::new().unwrap();
    let report_path = temp_dir.path().join("coverage.json");

    let output = Command::new(ampel_i18n_bin())
        .arg("coverage")
        .arg("--source")
        .arg(fixtures_dir().join("en.yaml"))
        .arg("--targets")
        .arg(fixtures_dir().join("ar.yaml"))
        .arg(fixtures_dir().join("pl.yaml"))
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&report_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(report_path.exists(), "Coverage report should be created");

    // Verify report content
    let report_content = std::fs::read_to_string(&report_path)
        .expect("Failed to read report");

    assert!(report_content.contains("coverage_percent"));
    assert!(report_content.contains("missing_keys"));
}

#[test]
#[ignore = "Requires binary to be built"]
fn test_cli_extract_keys() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("keys.txt");

    let output = Command::new(ampel_i18n_bin())
        .arg("extract-keys")
        .arg("--file")
        .arg(fixtures_dir().join("en.yaml"))
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(output_path.exists());

    let keys = std::fs::read_to_string(&output_path)
        .expect("Failed to read keys file");

    assert!(keys.contains("common.app.name"));
    assert!(keys.contains("dashboard.title"));
}

#[test]
#[ignore = "Requires binary to be built and API key"]
fn test_cli_translate_command() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("translated.yaml");

    let output = Command::new(ampel_i18n_bin())
        .arg("translate")
        .arg("--source")
        .arg(fixtures_dir().join("en.yaml"))
        .arg("--source-lang")
        .arg("en")
        .arg("--target-lang")
        .arg("de")
        .arg("--output")
        .arg(&output_path)
        .arg("--api-key")
        .arg("test-key")  // Will use mock in tests
        .arg("--dry-run")
        .output()
        .expect("Failed to execute command");

    // With --dry-run, should preview without creating file
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Would translate"));
}

#[test]
#[ignore = "Requires binary to be built"]
fn test_cli_check_placeholders() {
    let output = Command::new(ampel_i18n_bin())
        .arg("check-placeholders")
        .arg("--source")
        .arg(fixtures_dir().join("en.yaml"))
        .arg("--target")
        .arg(fixtures_dir().join("invalid_placeholders.yaml"))
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(),
        "Should fail on invalid placeholders");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("placeholder") || stderr.contains("mismatch"));
}

#[test]
#[ignore = "Requires binary to be built"]
fn test_cli_generate_types() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("translations.d.ts");

    let output = Command::new(ampel_i18n_bin())
        .arg("generate-types")
        .arg("--source")
        .arg(fixtures_dir().join("en.yaml"))
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("typescript")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(output_path.exists());

    let types = std::fs::read_to_string(&output_path)
        .expect("Failed to read types file");

    assert!(types.contains("interface") || types.contains("type"));
    assert!(types.contains("common"));
    assert!(types.contains("dashboard"));
}

#[test]
fn test_cli_argument_parsing() {
    use ampel_i18n_builder::cli::{Cli, Commands};
    use clap::Parser;

    let args = vec![
        "ampel-i18n",
        "validate",
        "--source", "en.yaml",
        "--target", "fr.yaml",
    ];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Should parse valid arguments");

    let cli = cli.unwrap();
    match cli.command {
        Commands::Validate { source, target } => {
            assert_eq!(source.to_str().unwrap(), "en.yaml");
            assert_eq!(target.to_str().unwrap(), "fr.yaml");
        }
        _ => panic!("Expected Validate command"),
    }
}

#[test]
fn test_cli_invalid_arguments() {
    use ampel_i18n_builder::cli::Cli;
    use clap::Parser;

    let args = vec![
        "ampel-i18n",
        "validate",
        // Missing required arguments
    ];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err(), "Should fail on missing required arguments");
}

#[test]
fn test_cli_version_flag() {
    use ampel_i18n_builder::cli::Cli;
    use clap::Parser;

    let args = vec!["ampel-i18n", "--version"];

    let cli = Cli::try_parse_from(args);
    // Version flag is handled by clap internally and exits
    // So this will error, but that's expected
    assert!(cli.is_err());
}

#[test]
fn test_cli_config_file_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("i18n.toml");

    std::fs::write(&config_path, r#"
        default_source_lang = "en"
        supported_languages = ["en", "fr", "de", "es"]
        translation_dir = "locales"
        api_provider = "deepl"
    "#).expect("Failed to write config");

    use ampel_i18n_builder::cli::Config;

    let config = Config::from_file(&config_path);
    assert!(config.is_ok(), "Should load valid config file");

    let config = config.unwrap();
    assert_eq!(config.default_source_lang, "en");
    assert_eq!(config.supported_languages.len(), 4);
}
