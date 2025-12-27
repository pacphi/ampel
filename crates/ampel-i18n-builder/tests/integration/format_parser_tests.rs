use ampel_i18n_builder::formats::{FormatParser, YamlParser, JsonParser, ParseError};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn test_yaml_parser_basic_structure() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");

    let result = parser.parse_file(&path);
    assert!(result.is_ok(), "Failed to parse valid YAML: {:?}", result.err());

    let translations = result.unwrap();

    // Verify top-level keys
    assert!(translations.contains_key("common"));
    assert!(translations.contains_key("dashboard"));
    assert!(translations.contains_key("settings"));
}

#[test]
fn test_yaml_parser_nested_values() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    // Test nested access
    let common = translations.get("common").expect("Missing common section");
    assert!(common.is_object());

    // Verify deep nesting: common.app.name
    let app_name = common
        .get("app")
        .and_then(|app| app.get("name"))
        .and_then(|name| name.as_str());

    assert_eq!(app_name, Some("Ampel"));
}

#[test]
fn test_yaml_parser_plural_forms() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("ar.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    // Arabic has 6 plural forms
    let count_section = translations
        .pointer("/common/pull_requests/count")
        .expect("Missing count section");

    assert!(count_section.get("zero").is_some(), "Missing 'zero' form");
    assert!(count_section.get("one").is_some(), "Missing 'one' form");
    assert!(count_section.get("two").is_some(), "Missing 'two' form");
    assert!(count_section.get("few").is_some(), "Missing 'few' form");
    assert!(count_section.get("many").is_some(), "Missing 'many' form");
    assert!(count_section.get("other").is_some(), "Missing 'other' form");
}

#[test]
fn test_yaml_parser_polish_three_forms() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("pl.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    // Polish has 3 plural forms
    let count_section = translations
        .pointer("/common/pull_requests/count")
        .expect("Missing count section");

    assert!(count_section.get("one").is_some(), "Missing 'one' form");
    assert!(count_section.get("few").is_some(), "Missing 'few' form");
    assert!(count_section.get("many").is_some(), "Missing 'many' form");
}

#[test]
fn test_json_parser_basic_structure() {
    let parser = JsonParser::new();
    let path = fixtures_dir().join("en.json");

    let result = parser.parse_file(&path);
    assert!(result.is_ok(), "Failed to parse valid JSON: {:?}", result.err());

    let translations = result.unwrap();

    // Verify top-level keys
    assert!(translations.contains_key("common"));
    assert!(translations.contains_key("dashboard"));
    assert!(translations.contains_key("settings"));
}

#[test]
fn test_json_parser_plural_forms() {
    let parser = JsonParser::new();
    let path = fixtures_dir().join("en.json");
    let translations = parser.parse_file(&path).expect("Parse failed");

    // React-i18next format: count_one, count_other
    let pull_requests = translations
        .pointer("/common/pullRequests")
        .expect("Missing pullRequests section");

    assert!(pull_requests.get("count_one").is_some());
    assert!(pull_requests.get("count_other").is_some());
}

#[test]
fn test_yaml_parser_invalid_file() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("nonexistent.yaml");

    let result = parser.parse_file(&path);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::FileNotFound(_) => {},
        other => panic!("Expected FileNotFound error, got: {:?}", other),
    }
}

#[test]
fn test_yaml_parser_malformed_content() {
    let parser = YamlParser::new();
    let malformed = r#"
    invalid: yaml: content:
    - missing
      proper
        indentation
    "#;

    let result = parser.parse_string(malformed);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::InvalidFormat(_) => {},
        other => panic!("Expected InvalidFormat error, got: {:?}", other),
    }
}

#[test]
fn test_json_parser_malformed_content() {
    let parser = JsonParser::new();
    let malformed = r#"
    {
        "invalid": "json",
        "missing": "comma"
        "syntax": "error"
    }
    "#;

    let result = parser.parse_string(malformed);
    assert!(result.is_err());
}

#[test]
fn test_parser_preserves_variable_placeholders() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let min_length = translations
        .pointer("/common/validation/min_length")
        .and_then(|v| v.as_str())
        .expect("Missing validation.min_length");

    // Verify placeholder is preserved
    assert!(min_length.contains("%{count}"),
        "Placeholder not preserved: {}", min_length);
}

#[test]
fn test_parser_handles_rtl_content() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("ar.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let app_name = translations
        .pointer("/common/app/name")
        .and_then(|v| v.as_str())
        .expect("Missing app.name");

    // Verify Arabic text is correctly parsed
    assert_eq!(app_name, "أمبل");

    // Verify RTL text contains Arabic characters
    assert!(app_name.chars().any(|c| c >= '\u{0600}' && c <= '\u{06FF}'),
        "Expected Arabic characters in RTL text");
}

#[test]
fn test_parser_empty_file() {
    let parser = YamlParser::new();
    let empty_yaml = "";

    let result = parser.parse_string(empty_yaml);

    // Empty YAML should parse to empty object
    assert!(result.is_ok());
    let translations = result.unwrap();
    assert!(translations.is_object());
}

#[test]
fn test_yaml_to_json_conversion() {
    let yaml_parser = YamlParser::new();
    let json_parser = JsonParser::new();

    let yaml_path = fixtures_dir().join("en.yaml");
    let yaml_data = yaml_parser.parse_file(&yaml_path).expect("YAML parse failed");

    // Convert to JSON string
    let json_string = serde_json::to_string_pretty(&yaml_data)
        .expect("JSON serialization failed");

    // Parse back
    let json_data = json_parser.parse_string(&json_string)
        .expect("JSON parse failed");

    // Verify structure matches
    assert_eq!(
        yaml_data.get("common").and_then(|c| c.get("app")).and_then(|a| a.get("name")),
        json_data.get("common").and_then(|c| c.get("app")).and_then(|a| a.get("name"))
    );
}

#[test]
fn test_parser_large_file_performance() {
    use std::time::Instant;

    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");

    let start = Instant::now();
    let result = parser.parse_file(&path);
    let duration = start.elapsed();

    assert!(result.is_ok());

    // Parsing should be fast (< 100ms for small files)
    assert!(duration.as_millis() < 100,
        "Parsing took too long: {}ms", duration.as_millis());
}
