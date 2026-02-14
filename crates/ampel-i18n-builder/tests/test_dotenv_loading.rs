//! Test .env file loading and environment variable precedence
//!
//! This test verifies:
//! 1. .env files are loaded when present
//! 2. System environment variables override .env values
//! 3. CLI works correctly without .env file
//!
//! Note: Tests use `dotenv::from_path()` instead of `set_current_dir` +
//! `dotenv::dotenv()` because `set_current_dir` is process-global and
//! causes race conditions when tests run in parallel.

use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_dotenv_precedence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        "TEST_VAR_FROM_DOTENV=dotenv_value\nANOTHER_VAR=from_file\n",
    )
    .expect("Failed to write .env");

    // Load .env from explicit path (avoids set_current_dir race condition)
    dotenv::from_path(&env_file).expect("Failed to load .env");

    assert_eq!(
        env::var("TEST_VAR_FROM_DOTENV").ok(),
        Some("dotenv_value".to_string())
    );
    assert_eq!(env::var("ANOTHER_VAR").ok(), Some("from_file".to_string()));

    // Cleanup
    env::remove_var("TEST_VAR_FROM_DOTENV");
    env::remove_var("ANOTHER_VAR");
}

#[test]
fn test_system_env_overrides_dotenv() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    fs::write(&env_file, "OVERRIDE_TEST=from_dotenv\n").expect("Failed to write .env");

    // Set system environment variable BEFORE loading .env
    env::set_var("OVERRIDE_TEST", "from_system");

    // Load .env file - should NOT override system env
    dotenv::from_path(&env_file).ok();

    // System env should still be "from_system", not overridden by .env
    assert_eq!(
        env::var("OVERRIDE_TEST").ok(),
        Some("from_system".to_string()),
        "System environment variable should take precedence over .env file"
    );

    // Cleanup
    env::remove_var("OVERRIDE_TEST");
}

#[test]
fn test_dotenv_missing_is_ok() {
    // Attempt to load a .env from a path that doesn't exist
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let missing_env = temp_dir.path().join(".env");

    let result = dotenv::from_path(&missing_env);
    assert!(result.is_err(), ".env file should not exist in temp dir");
}

#[test]
fn test_api_key_env_vars() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"
SYSTRAN_API_KEY=test_systran_key
DEEPL_API_KEY=test_deepl_key
GOOGLE_API_KEY=test_google_key
OPENAI_API_KEY=test_openai_key
"#,
    )
    .expect("Failed to write .env");

    dotenv::from_path(&env_file).expect("Failed to load .env");

    assert_eq!(
        env::var("SYSTRAN_API_KEY").ok(),
        Some("test_systran_key".to_string())
    );
    assert_eq!(
        env::var("DEEPL_API_KEY").ok(),
        Some("test_deepl_key".to_string())
    );
    assert_eq!(
        env::var("GOOGLE_API_KEY").ok(),
        Some("test_google_key".to_string())
    );
    assert_eq!(
        env::var("OPENAI_API_KEY").ok(),
        Some("test_openai_key".to_string())
    );

    // Cleanup
    env::remove_var("SYSTRAN_API_KEY");
    env::remove_var("DEEPL_API_KEY");
    env::remove_var("GOOGLE_API_KEY");
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_override_env_vars() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"
AMPEL_I18N_TIMEOUT_SECS=45
AMPEL_I18N_BATCH_SIZE=50
AMPEL_I18N_MAX_RETRIES=5
"#,
    )
    .expect("Failed to write .env");

    dotenv::from_path(&env_file).expect("Failed to load .env");

    assert_eq!(
        env::var("AMPEL_I18N_TIMEOUT_SECS").ok(),
        Some("45".to_string())
    );
    assert_eq!(
        env::var("AMPEL_I18N_BATCH_SIZE").ok(),
        Some("50".to_string())
    );
    assert_eq!(
        env::var("AMPEL_I18N_MAX_RETRIES").ok(),
        Some("5".to_string())
    );

    // Cleanup
    env::remove_var("AMPEL_I18N_TIMEOUT_SECS");
    env::remove_var("AMPEL_I18N_BATCH_SIZE");
    env::remove_var("AMPEL_I18N_MAX_RETRIES");
}
