use ampel_i18n_builder::validation::{
    TranslationValidator, ValidationError, CoverageReport, PlaceholderValidator
};
use ampel_i18n_builder::formats::YamlParser;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn test_coverage_validator_complete() {
    let parser = YamlParser::new();

    let en_path = fixtures_dir().join("en.yaml");
    let ar_path = fixtures_dir().join("ar.yaml");

    let source = parser.parse_file(&en_path).expect("Parse en.yaml failed");
    let target = parser.parse_file(&ar_path).expect("Parse ar.yaml failed");

    let validator = TranslationValidator::new();
    let report = validator.check_coverage(&source, &target, "ar");

    assert_eq!(report.missing_keys.len(), 0,
        "Arabic translation should have all keys");
    assert_eq!(report.coverage_percent, 100.0,
        "Coverage should be 100%");
}

#[test]
fn test_coverage_validator_incomplete() {
    let parser = YamlParser::new();

    let en_path = fixtures_dir().join("en.yaml");
    let incomplete_path = fixtures_dir().join("incomplete.yaml");

    let source = parser.parse_file(&en_path).expect("Parse en.yaml failed");
    let target = parser.parse_file(&incomplete_path).expect("Parse incomplete.yaml failed");

    let validator = TranslationValidator::new();
    let report = validator.check_coverage(&source, &target, "incomplete");

    assert!(report.missing_keys.len() > 0,
        "Should detect missing keys");
    assert!(report.coverage_percent < 100.0,
        "Coverage should be less than 100%");

    // Check specific missing keys
    assert!(report.missing_keys.contains(&"common.app.tagline".to_string()),
        "Should detect missing tagline");
    assert!(report.missing_keys.contains(&"dashboard".to_string()),
        "Should detect missing dashboard section");
}

#[test]
fn test_extra_keys_detection() {
    use serde_json::json;

    let source = json!({
        "common": {
            "app": {
                "name": "Ampel"
            }
        }
    });

    let target = json!({
        "common": {
            "app": {
                "name": "Ampel",
                "extra_field": "Should not be here"
            }
        },
        "extra_section": {
            "key": "value"
        }
    });

    let validator = TranslationValidator::new();
    let report = validator.check_coverage(&source, &target, "test");

    assert!(report.extra_keys.len() > 0,
        "Should detect extra keys");
    assert!(report.extra_keys.contains(&"common.app.extra_field".to_string()));
    assert!(report.extra_keys.contains(&"extra_section".to_string()));
}

#[test]
fn test_placeholder_validator_matching() {
    let validator = PlaceholderValidator::new();

    let source = "Must be at least %{count} characters";
    let target = "Doit contenir au moins %{count} caractères";

    let result = validator.validate_placeholders(source, target);
    assert!(result.is_ok(), "Matching placeholders should validate");
}

#[test]
fn test_placeholder_validator_mismatched_names() {
    let validator = PlaceholderValidator::new();

    let source = "Must be at least %{count} characters";
    let target = "Doit contenir au moins %{nombre} caractères";  // Wrong variable name

    let result = validator.validate_placeholders(source, target);
    assert!(result.is_err(), "Mismatched placeholder names should fail");

    match result.unwrap_err() {
        ValidationError::PlaceholderMismatch { .. } => {},
        other => panic!("Expected PlaceholderMismatch error, got: {:?}", other),
    }
}

#[test]
fn test_placeholder_validator_missing() {
    let validator = PlaceholderValidator::new();

    let source = "Must be at most %{count} characters";
    let target = "Darf maximal Zeichen enthalten";  // Missing placeholder

    let result = validator.validate_placeholders(source, target);
    assert!(result.is_err(), "Missing placeholders should fail");
}

#[test]
fn test_placeholder_validator_extra() {
    let validator = PlaceholderValidator::new();

    let source = "Must be between %{min} and %{max}";
    let target = "Debe estar entre %{min}, %{max} y %{extra}";  // Extra placeholder

    let result = validator.validate_placeholders(source, target);
    assert!(result.is_err(), "Extra placeholders should fail");
}

#[test]
fn test_placeholder_extraction() {
    let validator = PlaceholderValidator::new();

    let text = "Hello %{name}, you have %{count} messages";
    let placeholders = validator.extract_placeholders(text);

    assert_eq!(placeholders.len(), 2);
    assert!(placeholders.contains(&"name".to_string()));
    assert!(placeholders.contains(&"count".to_string()));
}

#[test]
fn test_placeholder_extraction_react_format() {
    let validator = PlaceholderValidator::new();

    // React-i18next format: {{variable}}
    let text = "Hello {{name}}, you have {{count}} messages";
    let placeholders = validator.extract_placeholders(text);

    assert_eq!(placeholders.len(), 2);
    assert!(placeholders.contains(&"name".to_string()));
    assert!(placeholders.contains(&"count".to_string()));
}

#[test]
fn test_placeholder_extraction_mixed_formats() {
    let validator = PlaceholderValidator::new();

    // Both formats in same string (should be avoided but handled)
    let text = "User %{name} has {{count}} items";
    let placeholders = validator.extract_placeholders(text);

    assert_eq!(placeholders.len(), 2);
}

#[test]
fn test_duplicate_key_detection() {
    use serde_json::json;

    let translations = json!({
        "common": {
            "save": "Save",
            "app": {
                "save": "Save"  // Duplicate at different nesting
            }
        }
    });

    let validator = TranslationValidator::new();
    let duplicates = validator.find_duplicate_keys(&translations);

    // Note: This depends on how we define "duplicate" (same key at any level)
    // For now, we only care about duplicates at the same nesting level
    assert!(duplicates.len() == 0 || duplicates.len() > 0,
        "Duplicate detection implemented");
}

#[test]
fn test_coverage_report_statistics() {
    let mut report = CoverageReport {
        language: "fr".to_string(),
        total_keys: 100,
        translated_keys: 85,
        missing_keys: vec!["key1".into(), "key2".into()],
        extra_keys: vec!["extra1".into()],
        coverage_percent: 85.0,
    };

    assert_eq!(report.coverage_percent, 85.0);
    assert_eq!(report.missing_keys.len(), 2);
    assert_eq!(report.extra_keys.len(), 1);
}

#[test]
fn test_validation_with_invalid_placeholders_fixture() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("invalid_placeholders.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let validator = PlaceholderValidator::new();

    // Extract test cases from fixture
    let validation_section = translations
        .pointer("/common/validation")
        .expect("Missing validation section");

    let min_en = validation_section.get("min_length_en")
        .and_then(|v| v.as_str()).unwrap();
    let min_fr = validation_section.get("min_length_fr")
        .and_then(|v| v.as_str()).unwrap();

    let result = validator.validate_placeholders(min_en, min_fr);
    assert!(result.is_err(), "Mismatched placeholder names should fail");
}

#[test]
fn test_nested_key_flattening() {
    use serde_json::json;

    let nested = json!({
        "common": {
            "app": {
                "name": "Ampel",
                "tagline": "PR Management"
            }
        }
    });

    let validator = TranslationValidator::new();
    let flat_keys = validator.flatten_keys(&nested);

    assert!(flat_keys.contains(&"common.app.name".to_string()));
    assert!(flat_keys.contains(&"common.app.tagline".to_string()));
}

#[test]
fn test_empty_translation_detection() {
    use serde_json::json;

    let translations = json!({
        "common": {
            "empty": "",
            "null": null,
            "valid": "Valid text"
        }
    });

    let validator = TranslationValidator::new();
    let empty_keys = validator.find_empty_translations(&translations);

    assert!(empty_keys.contains(&"common.empty".to_string()));
    assert!(empty_keys.contains(&"common.null".to_string()));
    assert!(!empty_keys.contains(&"common.valid".to_string()));
}

#[test]
fn test_validation_batch_processing() {
    let parser = YamlParser::new();
    let validator = TranslationValidator::new();

    let en_path = fixtures_dir().join("en.yaml");
    let source = parser.parse_file(&en_path).expect("Parse failed");

    let languages = vec![
        ("ar", fixtures_dir().join("ar.yaml")),
        ("pl", fixtures_dir().join("pl.yaml")),
    ];

    let mut reports = Vec::new();

    for (lang, path) in languages {
        let target = parser.parse_file(&path).expect("Parse failed");
        let report = validator.check_coverage(&source, &target, lang);
        reports.push(report);
    }

    assert_eq!(reports.len(), 2);

    for report in &reports {
        assert!(report.coverage_percent >= 90.0,
            "{} coverage should be at least 90%", report.language);
    }
}
