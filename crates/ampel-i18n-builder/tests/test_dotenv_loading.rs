//! Test .env file loading and environment variable precedence
//!
//! This test verifies:
//! 1. .env files are loaded when present
//! 2. System environment variables override .env values
//! 3. CLI works correctly without .env file

use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_dotenv_precedence() {
    // Create a temporary directory for test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    // Write test .env file
    fs::write(
        &env_file,
        "TEST_VAR_FROM_DOTENV=dotenv_value\nANOTHER_VAR=from_file\n",
    )
    .expect("Failed to write .env");

    // Change to temp directory
    let original_dir = env::current_dir().expect("Failed to get current dir");
    env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Load .env file
    dotenv::dotenv().expect("Failed to load .env");

    // Verify .env values are loaded
    assert_eq!(
        env::var("TEST_VAR_FROM_DOTENV").ok(),
        Some("dotenv_value".to_string())
    );
    assert_eq!(env::var("ANOTHER_VAR").ok(), Some("from_file".to_string()));

    // Cleanup - restore directory and remove env vars
    env::set_current_dir(original_dir).expect("Failed to restore dir");
    env::remove_var("TEST_VAR_FROM_DOTENV");
    env::remove_var("ANOTHER_VAR");
}

#[test]
fn test_system_env_overrides_dotenv() {
    // Create a temporary directory for test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    // Write test .env file
    fs::write(&env_file, "OVERRIDE_TEST=from_dotenv\n").expect("Failed to write .env");

    // Set system environment variable BEFORE loading .env
    env::set_var("OVERRIDE_TEST", "from_system");

    // Change to temp directory
    let original_dir = env::current_dir().ok();
    env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Load .env file - should NOT override system env
    dotenv::dotenv().ok(); // Ignore error if system var prevents loading

    // System env should still be "from_system", not overridden by .env
    assert_eq!(
        env::var("OVERRIDE_TEST").ok(),
        Some("from_system".to_string()),
        "System environment variable should take precedence over .env file"
    );

    // Cleanup - restore directory if possible
    if let Some(dir) = original_dir {
        let _ = env::set_current_dir(dir); // Ignore error if dir no longer exists
    }
    env::remove_var("OVERRIDE_TEST");
}

#[test]
fn test_dotenv_missing_is_ok() {
    // Create a temporary directory WITHOUT .env file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = env::current_dir().expect("Failed to get current dir");
    env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Loading .env when file doesn't exist should return an error
    // but application should handle gracefully (this is the pattern in main.rs)
    let result = dotenv::dotenv();
    assert!(result.is_err(), ".env file should not exist in temp dir");

    // This demonstrates the pattern used in main.rs:
    // if let Err(_e) = dotenv::dotenv() {
    //     // Silent failure - application continues
    // }

    // Cleanup
    env::set_current_dir(original_dir).expect("Failed to restore dir");
}

#[test]
fn test_api_key_env_vars() {
    // This test verifies that common API key environment variables
    // can be loaded from .env files

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    // Write .env with all supported API keys
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

    let original_dir = env::current_dir().expect("Failed to get current dir");
    env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    // Load .env
    dotenv::dotenv().expect("Failed to load .env");

    // Verify all API keys are loaded
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
    env::set_current_dir(original_dir).expect("Failed to restore dir");
    env::remove_var("SYSTRAN_API_KEY");
    env::remove_var("DEEPL_API_KEY");
    env::remove_var("GOOGLE_API_KEY");
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_override_env_vars() {
    // Test configuration override environment variables

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

    let original_dir = env::current_dir().expect("Failed to get current dir");
    env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    dotenv::dotenv().expect("Failed to load .env");

    // Verify config override vars are loaded
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
    env::set_current_dir(original_dir).expect("Failed to restore dir");
    env::remove_var("AMPEL_I18N_TIMEOUT_SECS");
    env::remove_var("AMPEL_I18N_BATCH_SIZE");
    env::remove_var("AMPEL_I18N_MAX_RETRIES");
}
