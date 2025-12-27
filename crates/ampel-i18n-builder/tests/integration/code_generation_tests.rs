use ampel_i18n_builder::generator::{TypeScriptGenerator, RustGenerator, CodeGenerator};
use ampel_i18n_builder::formats::YamlParser;
use std::path::PathBuf;
use tempfile::TempDir;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn test_typescript_type_generation() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let generator = TypeScriptGenerator::new();
    let typescript_code = generator.generate(&translations)
        .expect("TypeScript generation failed");

    // Verify interface is generated
    assert!(typescript_code.contains("interface") || typescript_code.contains("type"),
        "Should generate TypeScript types");

    // Verify nested structures
    assert!(typescript_code.contains("common"),
        "Should include 'common' namespace");
    assert!(typescript_code.contains("dashboard"),
        "Should include 'dashboard' namespace");
    assert!(typescript_code.contains("settings"),
        "Should include 'settings' namespace");
}

#[test]
fn test_typescript_nested_types() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let generator = TypeScriptGenerator::new();
    let typescript_code = generator.generate(&translations)
        .expect("TypeScript generation failed");

    // Verify nested type definitions
    assert!(typescript_code.contains("app") || typescript_code.contains("App"),
        "Should include nested app type");

    // Should handle deep nesting: common.app.name
    assert!(typescript_code.contains("name") || typescript_code.contains("string"),
        "Should include string types for leaf nodes");
}

#[test]
fn test_typescript_plural_types() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("ar.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let generator = TypeScriptGenerator::new();
    let typescript_code = generator.generate(&translations)
        .expect("TypeScript generation failed");

    // Plurals should be typed
    assert!(typescript_code.contains("count") || typescript_code.contains("plural"),
        "Should include plural types");
}

#[test]
fn test_rust_const_generation() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let generator = RustGenerator::new();
    let rust_code = generator.generate(&translations)
        .expect("Rust generation failed");

    // Verify constants are generated
    assert!(rust_code.contains("pub const") || rust_code.contains("const"),
        "Should generate Rust constants");

    // Verify key format
    assert!(rust_code.contains("COMMON_APP_NAME") ||
            rust_code.contains("common.app.name"),
        "Should include constant for common.app.name");
}

#[test]
fn test_rust_key_format() {
    let generator = RustGenerator::new();

    // Test key transformation
    assert_eq!(
        generator.transform_key("common.app.name"),
        "COMMON_APP_NAME"
    );
    assert_eq!(
        generator.transform_key("dashboard.filters.all"),
        "DASHBOARD_FILTERS_ALL"
    );
}

#[test]
fn test_generated_code_compiles_typescript() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let generator = TypeScriptGenerator::new();
    let typescript_code = generator.generate(&translations)
        .expect("TypeScript generation failed");

    let temp_dir = TempDir::new().unwrap();
    let ts_file = temp_dir.path().join("translations.d.ts");

    std::fs::write(&ts_file, typescript_code)
        .expect("Failed to write TypeScript file");

    // Use tsc to validate TypeScript syntax
    let output = std::process::Command::new("npx")
        .arg("tsc")
        .arg("--noEmit")
        .arg(&ts_file)
        .output();

    if let Ok(output) = output {
        assert!(output.status.success(),
            "Generated TypeScript should compile without errors: {}",
            String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn test_generated_code_valid_syntax_rust() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let generator = RustGenerator::new();
    let rust_code = generator.generate(&translations)
        .expect("Rust generation failed");

    // Basic syntax validation
    assert!(rust_code.contains("pub const") || rust_code.contains("const"));
    assert!(!rust_code.contains("TODO") && !rust_code.contains("FIXME"),
        "Generated code should not contain placeholders");

    // Verify valid Rust identifiers
    for line in rust_code.lines() {
        if line.contains("pub const") {
            // Extract identifier
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let identifier = parts[2].trim_end_matches(':');
                assert!(identifier.chars().all(|c| c.is_alphanumeric() || c == '_'),
                    "Invalid Rust identifier: {}", identifier);
            }
        }
    }
}

#[test]
fn test_code_generation_escapes_strings() {
    use serde_json::json;

    let translations = json!({
        "common": {
            "quote": "He said \"Hello\"",
            "newline": "Line 1\nLine 2",
            "backslash": "Path: C:\\Users\\test"
        }
    });

    let generator = TypeScriptGenerator::new();
    let typescript_code = generator.generate(&translations)
        .expect("TypeScript generation failed");

    // Verify strings are properly escaped
    assert!(typescript_code.contains("\\\"") || typescript_code.contains("'"),
        "Quotes should be escaped");
    assert!(typescript_code.contains("\\n"),
        "Newlines should be escaped");
    assert!(typescript_code.contains("\\\\"),
        "Backslashes should be escaped");
}

#[test]
fn test_code_generation_empty_input() {
    use serde_json::json;

    let translations = json!({});

    let generator = TypeScriptGenerator::new();
    let result = generator.generate(&translations);

    assert!(result.is_ok(), "Should handle empty input");

    let typescript_code = result.unwrap();
    assert!(typescript_code.contains("interface") || typescript_code.is_empty());
}

#[test]
fn test_code_generation_preserves_structure() {
    use serde_json::json;

    let translations = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "Deep value"
                }
            }
        }
    });

    let generator = TypeScriptGenerator::new();
    let typescript_code = generator.generate(&translations)
        .expect("Generation failed");

    // Verify nested structure is preserved
    assert!(typescript_code.contains("level1"));
    assert!(typescript_code.contains("level2"));
    assert!(typescript_code.contains("level3"));
}

#[test]
fn test_typescript_readonly_types() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let mut generator = TypeScriptGenerator::new();
    generator.set_readonly(true);

    let typescript_code = generator.generate(&translations)
        .expect("TypeScript generation failed");

    // Verify readonly modifiers
    assert!(typescript_code.contains("readonly"),
        "Should generate readonly types");
}

#[test]
fn test_code_generation_performance() {
    use std::time::Instant;

    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let generator = TypeScriptGenerator::new();

    let start = Instant::now();
    let result = generator.generate(&translations);
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_millis() < 100,
        "Code generation should be fast (< 100ms), took: {}ms",
        duration.as_millis());
}

#[test]
fn test_multi_format_generation() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    // Generate TypeScript
    let ts_generator = TypeScriptGenerator::new();
    let ts_code = ts_generator.generate(&translations);
    assert!(ts_code.is_ok());

    // Generate Rust
    let rust_generator = RustGenerator::new();
    let rust_code = rust_generator.generate(&translations);
    assert!(rust_code.is_ok());

    // Both should succeed and not be empty
    assert!(!ts_code.unwrap().is_empty());
    assert!(!rust_code.unwrap().is_empty());
}

#[test]
fn test_comment_generation() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let mut generator = TypeScriptGenerator::new();
    generator.set_include_comments(true);

    let typescript_code = generator.generate(&translations)
        .expect("TypeScript generation failed");

    // Verify comments are included
    assert!(typescript_code.contains("//") || typescript_code.contains("/*"),
        "Should include comments");
}

#[test]
fn test_custom_output_format() {
    let parser = YamlParser::new();
    let path = fixtures_dir().join("en.yaml");
    let translations = parser.parse_file(&path).expect("Parse failed");

    let mut generator = TypeScriptGenerator::new();
    generator.set_export_style("named"); // vs "default"

    let typescript_code = generator.generate(&translations)
        .expect("TypeScript generation failed");

    // Verify export style
    assert!(typescript_code.contains("export"),
        "Should include export statements");
}
