use super::{ValidationError, ValidationResult, Validator};
use crate::formats::{TranslationMap, TranslationValue};
use regex::Regex;
use std::collections::HashSet;

pub struct VariableValidator;

impl VariableValidator {
    fn extract_variables(text: &str) -> Vec<String> {
        let mut variables = Vec::new();

        // {{var}}
        let re1 = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        for cap in re1.captures_iter(text) {
            variables.push(cap[1].to_string());
        }

        // %{var}
        let re2 = Regex::new(r"%\{(\w+)\}").unwrap();
        for cap in re2.captures_iter(text) {
            variables.push(cap[1].to_string());
        }

        // {var}
        let re3 = Regex::new(r"\{(\w+)\}").unwrap();
        for cap in re3.captures_iter(text) {
            let var = cap[1].to_string();
            if !variables.contains(&var) {
                variables.push(var);
            }
        }

        variables
    }

    fn check_string_variables(
        key: &str,
        source_str: &str,
        target_str: &str,
    ) -> Vec<ValidationError> {
        let source_vars = Self::extract_variables(source_str);
        let target_vars = Self::extract_variables(target_str);

        let source_set: HashSet<_> = source_vars.iter().collect();
        let target_set: HashSet<_> = target_vars.iter().collect();

        if source_set != target_set {
            vec![ValidationError::VariableMismatch {
                key: key.to_string(),
                source_vars,
                translation_vars: target_vars,
            }]
        } else {
            Vec::new()
        }
    }

    fn validate_variables_recursive(
        source: &TranslationMap,
        target: &TranslationMap,
        prefix: String,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (key, source_value) in source.iter() {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            if let Some(target_value) = target.get(key) {
                match (source_value, target_value) {
                    (TranslationValue::String(source_str), TranslationValue::String(target_str)) => {
                        errors.extend(Self::check_string_variables(&full_key, source_str, target_str));
                    }
                    (TranslationValue::Plural(source_plural), TranslationValue::Plural(target_plural)) => {
                        errors.extend(Self::check_string_variables(
                            &format!("{}.other", full_key),
                            &source_plural.other,
                            &target_plural.other,
                        ));

                        if let (Some(s), Some(t)) = (&source_plural.one, &target_plural.one) {
                            errors.extend(Self::check_string_variables(&format!("{}.one", full_key), s, t));
                        }
                        if let (Some(s), Some(t)) = (&source_plural.few, &target_plural.few) {
                            errors.extend(Self::check_string_variables(&format!("{}.few", full_key), s, t));
                        }
                    }
                    (TranslationValue::Nested(source_nested), TranslationValue::Nested(target_nested)) => {
                        errors.extend(Self::validate_variables_recursive(source_nested, target_nested, full_key));
                    }
                    _ => {}
                }
            }
        }

        errors
    }
}

impl Validator for VariableValidator {
    fn validate(
        &self,
        source: &[(String, TranslationMap)],
        target: &[(String, TranslationMap)],
    ) -> crate::error::Result<ValidationResult> {
        let mut result = ValidationResult::new("variable_validator");

        for (namespace, source_map) in source {
            if let Some((_, target_map)) = target.iter().find(|(ns, _)| ns == namespace) {
                let errors = Self::validate_variables_recursive(source_map, target_map, String::new());

                for error in errors {
                    result.add_error(error);
                }
            }
        }

        Ok(result)
    }
}
