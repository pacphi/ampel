use ampel_i18n_builder::validation::{PluralValidator, PluralRule, PluralForm};

#[test]
fn test_english_two_forms() {
    let validator = PluralValidator::for_language("en");
    let required_forms = validator.required_forms();

    assert_eq!(required_forms.len(), 2);
    assert!(required_forms.contains(&PluralForm::One));
    assert!(required_forms.contains(&PluralForm::Other));
}

#[test]
fn test_arabic_six_forms() {
    let validator = PluralValidator::for_language("ar");
    let required_forms = validator.required_forms();

    assert_eq!(required_forms.len(), 6,
        "Arabic should have 6 plural forms");
    assert!(required_forms.contains(&PluralForm::Zero));
    assert!(required_forms.contains(&PluralForm::One));
    assert!(required_forms.contains(&PluralForm::Two));
    assert!(required_forms.contains(&PluralForm::Few));
    assert!(required_forms.contains(&PluralForm::Many));
    assert!(required_forms.contains(&PluralForm::Other));
}

#[test]
fn test_polish_three_forms() {
    let validator = PluralValidator::for_language("pl");
    let required_forms = validator.required_forms();

    assert_eq!(required_forms.len(), 3,
        "Polish should have 3 plural forms");
    assert!(required_forms.contains(&PluralForm::One));
    assert!(required_forms.contains(&PluralForm::Few));
    assert!(required_forms.contains(&PluralForm::Many));
}

#[test]
fn test_russian_three_forms() {
    let validator = PluralValidator::for_language("ru");
    let required_forms = validator.required_forms();

    assert_eq!(required_forms.len(), 3);
    assert!(required_forms.contains(&PluralForm::One));
    assert!(required_forms.contains(&PluralForm::Few));
    assert!(required_forms.contains(&PluralForm::Many));
}

#[test]
fn test_czech_four_forms() {
    let validator = PluralValidator::for_language("cs");
    let required_forms = validator.required_forms();

    // Czech has: zero, one, few, many, other (5 forms)
    assert!(required_forms.len() >= 4,
        "Czech should have at least 4 plural forms");
}

#[test]
fn test_finnish_two_forms() {
    let validator = PluralValidator::for_language("fi");
    let required_forms = validator.required_forms();

    // Finnish uses standard two forms despite complex grammar
    assert_eq!(required_forms.len(), 2);
}

#[test]
fn test_plural_form_validation_complete() {
    use std::collections::HashMap;

    let validator = PluralValidator::for_language("ar");

    let mut forms = HashMap::new();
    forms.insert(PluralForm::Zero, "لا توجد");
    forms.insert(PluralForm::One, "واحد");
    forms.insert(PluralForm::Two, "اثنان");
    forms.insert(PluralForm::Few, "قليل");
    forms.insert(PluralForm::Many, "كثير");
    forms.insert(PluralForm::Other, "آخر");

    let result = validator.validate_forms(&forms);
    assert!(result.is_ok(), "All Arabic forms present, should validate");
}

#[test]
fn test_plural_form_validation_missing() {
    use std::collections::HashMap;

    let validator = PluralValidator::for_language("ar");

    let mut forms = HashMap::new();
    forms.insert(PluralForm::One, "واحد");
    forms.insert(PluralForm::Other, "آخر");
    // Missing zero, two, few, many

    let result = validator.validate_forms(&forms);
    assert!(result.is_err(), "Missing Arabic plural forms should fail validation");

    let error = result.unwrap_err();
    assert!(error.to_string().contains("missing"),
        "Error should mention missing forms: {}", error);
}

#[test]
fn test_plural_form_validation_extra_forms() {
    use std::collections::HashMap;

    let validator = PluralValidator::for_language("en");

    let mut forms = HashMap::new();
    forms.insert(PluralForm::One, "one item");
    forms.insert(PluralForm::Other, "other items");
    forms.insert(PluralForm::Few, "few items"); // Extra for English

    // Extra forms should be allowed (for forward compatibility)
    let result = validator.validate_forms(&forms);
    assert!(result.is_ok(), "Extra plural forms should not cause validation failure");
}

#[test]
fn test_plural_rule_selection() {
    let validator = PluralValidator::for_language("en");

    // Test number to plural form mapping
    assert_eq!(validator.select_form(0), PluralForm::Other);
    assert_eq!(validator.select_form(1), PluralForm::One);
    assert_eq!(validator.select_form(2), PluralForm::Other);
    assert_eq!(validator.select_form(100), PluralForm::Other);
}

#[test]
fn test_plural_rule_arabic_selection() {
    let validator = PluralValidator::for_language("ar");

    // Arabic plural rules (CLDR)
    assert_eq!(validator.select_form(0), PluralForm::Zero);
    assert_eq!(validator.select_form(1), PluralForm::One);
    assert_eq!(validator.select_form(2), PluralForm::Two);
    assert_eq!(validator.select_form(3), PluralForm::Few);
    assert_eq!(validator.select_form(5), PluralForm::Few);
    assert_eq!(validator.select_form(11), PluralForm::Many);
    assert_eq!(validator.select_form(100), PluralForm::Other);
}

#[test]
fn test_plural_rule_polish_selection() {
    let validator = PluralValidator::for_language("pl");

    // Polish plural rules
    assert_eq!(validator.select_form(0), PluralForm::Many);
    assert_eq!(validator.select_form(1), PluralForm::One);
    assert_eq!(validator.select_form(2), PluralForm::Few);
    assert_eq!(validator.select_form(4), PluralForm::Few);
    assert_eq!(validator.select_form(5), PluralForm::Many);
    assert_eq!(validator.select_form(22), PluralForm::Few);
    assert_eq!(validator.select_form(25), PluralForm::Many);
}

#[test]
fn test_unsupported_language_fallback() {
    let validator = PluralValidator::for_language("unknown-lang");
    let required_forms = validator.required_forms();

    // Unknown languages should fall back to standard two forms
    assert_eq!(required_forms.len(), 2);
    assert!(required_forms.contains(&PluralForm::One));
    assert!(required_forms.contains(&PluralForm::Other));
}

#[test]
fn test_plural_validation_with_fixtures() {
    use ampel_i18n_builder::formats::YamlParser;
    use std::path::PathBuf;

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let parser = YamlParser::new();

    // Test Arabic plural forms
    let ar_path = fixtures_dir.join("ar.yaml");
    let ar_translations = parser.parse_file(&ar_path).expect("Parse failed");

    let count_section = ar_translations
        .pointer("/common/pull_requests/count")
        .expect("Missing count section");

    let validator = PluralValidator::for_language("ar");
    let mut forms = std::collections::HashMap::new();

    for form_name in ["zero", "one", "two", "few", "many", "other"] {
        if let Some(value) = count_section.get(form_name).and_then(|v| v.as_str()) {
            let plural_form = PluralForm::from_string(form_name);
            forms.insert(plural_form, value);
        }
    }

    let result = validator.validate_forms(&forms);
    assert!(result.is_ok(), "Arabic fixture should have valid plural forms");
}

#[test]
fn test_plural_form_string_conversion() {
    assert_eq!(PluralForm::from_string("zero"), PluralForm::Zero);
    assert_eq!(PluralForm::from_string("one"), PluralForm::One);
    assert_eq!(PluralForm::from_string("two"), PluralForm::Two);
    assert_eq!(PluralForm::from_string("few"), PluralForm::Few);
    assert_eq!(PluralForm::from_string("many"), PluralForm::Many);
    assert_eq!(PluralForm::from_string("other"), PluralForm::Other);

    assert_eq!(PluralForm::Zero.to_string(), "zero");
    assert_eq!(PluralForm::One.to_string(), "one");
}
