//! Integration tests for recursive translation of nested objects and plural forms

use ampel_i18n_builder::formats::{
    JsonFormat, PluralForms, TranslationFormat, TranslationMap, TranslationValue,
};
use std::collections::BTreeMap;

#[test]
fn test_flatten_nested_structure() {
    let mut map = TranslationMap::new();

    // Create nested structure: app.title, app.name, auth.login
    let mut app_nested = BTreeMap::new();
    app_nested.insert(
        "title".to_string(),
        TranslationValue::String("Ampel PR Dashboard".to_string()),
    );
    app_nested.insert(
        "name".to_string(),
        TranslationValue::String("Ampel".to_string()),
    );

    let mut auth_nested = BTreeMap::new();
    auth_nested.insert(
        "login".to_string(),
        TranslationValue::String("Login".to_string()),
    );
    auth_nested.insert(
        "logout".to_string(),
        TranslationValue::String("Logout".to_string()),
    );

    map.insert("app".to_string(), TranslationValue::Nested(app_nested));
    map.insert("auth".to_string(), TranslationValue::Nested(auth_nested));

    // Verify structure
    if let Some(TranslationValue::Nested(app)) = map.get("app") {
        assert_eq!(
            app.get("title"),
            Some(&TranslationValue::String("Ampel PR Dashboard".to_string()))
        );
        assert_eq!(
            app.get("name"),
            Some(&TranslationValue::String("Ampel".to_string()))
        );
    } else {
        panic!("Expected nested app structure");
    }
}

#[test]
fn test_parse_nested_json() {
    let json = r#"{
        "app": {
            "title": "Ampel PR Dashboard",
            "name": "Ampel",
            "description": "Unified pull request management"
        },
        "auth": {
            "login": "Login",
            "logout": "Logout"
        }
    }"#;

    let format = JsonFormat::new();
    let map = format.parse(json).unwrap();

    // Verify nested structure was parsed correctly
    assert!(map.contains_key("app"));
    assert!(map.contains_key("auth"));

    if let Some(TranslationValue::Nested(app)) = map.get("app") {
        assert_eq!(app.len(), 3);
        assert!(app.contains_key("title"));
        assert!(app.contains_key("name"));
        assert!(app.contains_key("description"));
    } else {
        panic!("Expected nested app structure");
    }
}

#[test]
fn test_parse_plural_forms() {
    let json = r#"{
        "pullRequests": {
            "count_one": "{{count}} pull request",
            "count_other": "{{count}} pull requests"
        }
    }"#;

    let format = JsonFormat::new();
    let map = format.parse(json).unwrap();

    if let Some(TranslationValue::Nested(pr)) = map.get("pullRequests") {
        assert!(pr.contains_key("count_one"));
        assert!(pr.contains_key("count_other"));

        if let Some(TranslationValue::String(text)) = pr.get("count_one") {
            assert!(text.contains("{{count}}"));
        } else {
            panic!("Expected string value for count_one");
        }
    } else {
        panic!("Expected nested pullRequests structure");
    }
}

#[test]
fn test_write_nested_json() {
    let mut map = TranslationMap::new();

    let mut app_nested = BTreeMap::new();
    app_nested.insert(
        "title".to_string(),
        TranslationValue::String("Ampel".to_string()),
    );
    app_nested.insert(
        "name".to_string(),
        TranslationValue::String("Ampel PR".to_string()),
    );

    map.insert("app".to_string(), TranslationValue::Nested(app_nested));
    map.insert(
        "simple".to_string(),
        TranslationValue::String("Simple value".to_string()),
    );

    let format = JsonFormat::new();
    let json = format.write(&map).unwrap();

    // Parse back and verify
    let parsed = format.parse(&json).unwrap();
    assert_eq!(parsed.len(), 2);
    assert!(parsed.contains_key("app"));
    assert!(parsed.contains_key("simple"));
}

#[test]
fn test_deeply_nested_structure() {
    let json = r#"{
        "level1": {
            "level2": {
                "level3": {
                    "deep": "Deep value"
                }
            }
        }
    }"#;

    let format = JsonFormat::new();
    let map = format.parse(json).unwrap();

    // Navigate to deep value
    if let Some(TranslationValue::Nested(l1)) = map.get("level1") {
        if let Some(TranslationValue::Nested(l2)) = l1.get("level2") {
            if let Some(TranslationValue::Nested(l3)) = l2.get("level3") {
                if let Some(TranslationValue::String(deep)) = l3.get("deep") {
                    assert_eq!(deep, "Deep value");
                } else {
                    panic!("Expected deep string value");
                }
            } else {
                panic!("Expected level3 nested structure");
            }
        } else {
            panic!("Expected level2 nested structure");
        }
    } else {
        panic!("Expected level1 nested structure");
    }
}

#[test]
fn test_all_strings_extraction() {
    let mut map = TranslationMap::new();

    let mut app_nested = BTreeMap::new();
    app_nested.insert(
        "title".to_string(),
        TranslationValue::String("Title".to_string()),
    );
    app_nested.insert(
        "name".to_string(),
        TranslationValue::String("Name".to_string()),
    );

    map.insert("app".to_string(), TranslationValue::Nested(app_nested));
    map.insert(
        "simple".to_string(),
        TranslationValue::String("Simple".to_string()),
    );

    // Extract all strings
    let mut all_strings = Vec::new();
    for value in map.values() {
        all_strings.extend(value.all_strings());
    }

    assert_eq!(all_strings.len(), 3);
    assert!(all_strings.contains(&"Title"));
    assert!(all_strings.contains(&"Name"));
    assert!(all_strings.contains(&"Simple"));
}

#[test]
fn test_plural_forms_structure() {
    let plural = PluralForms {
        zero: Some("No items".to_string()),
        one: Some("One item".to_string()),
        two: None,
        few: None,
        many: None,
        other: "{{count}} items".to_string(),
    };

    let value = TranslationValue::Plural(plural);
    let strings = value.all_strings();

    assert_eq!(strings.len(), 3);
    assert!(strings.contains(&"{{count}} items"));
    assert!(strings.contains(&"No items"));
    assert!(strings.contains(&"One item"));
}

#[test]
fn test_placeholder_preservation() {
    let json = r#"{
        "greeting": "Hello, {{name}}!",
        "count": "{{count}} items",
        "provider": "View on {{provider}}"
    }"#;

    let format = JsonFormat::new();
    let map = format.parse(json).unwrap();

    // Verify placeholders are in the strings
    if let Some(TranslationValue::String(greeting)) = map.get("greeting") {
        assert!(greeting.contains("{{name}}"));
    }

    if let Some(TranslationValue::String(count)) = map.get("count") {
        assert!(count.contains("{{count}}"));
    }

    if let Some(TranslationValue::String(provider)) = map.get("provider") {
        assert!(provider.contains("{{provider}}"));
    }
}

#[test]
fn test_mixed_structure() {
    let json = r#"{
        "simple": "Simple string",
        "nested": {
            "inner": "Nested string"
        },
        "plural": {
            "count_one": "{{count}} item",
            "count_other": "{{count}} items"
        }
    }"#;

    let format = JsonFormat::new();
    let map = format.parse(json).unwrap();

    assert_eq!(map.len(), 3);

    // Verify simple string
    assert!(matches!(
        map.get("simple"),
        Some(TranslationValue::String(_))
    ));

    // Verify nested structure
    assert!(matches!(
        map.get("nested"),
        Some(TranslationValue::Nested(_))
    ));

    // Verify plural structure
    assert!(matches!(
        map.get("plural"),
        Some(TranslationValue::Nested(_))
    ));
}

#[test]
fn test_roundtrip_preservation() {
    let original_json = r#"{
  "app": {
    "title": "Ampel PR Dashboard",
    "name": "Ampel"
  },
  "auth": {
    "login": "Login",
    "logout": "Logout"
  },
  "count": "{{count}} items"
}"#;

    let format = JsonFormat::new();

    // Parse
    let map = format.parse(original_json).unwrap();

    // Write back
    let written_json = format.write(&map).unwrap();

    // Parse again
    let reparsed = format.parse(&written_json).unwrap();

    // Verify structure is preserved
    assert_eq!(map.len(), reparsed.len());

    for (key, value) in &map {
        assert_eq!(Some(value), reparsed.get(key));
    }
}
