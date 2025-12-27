use super::{ValidationError, ValidationResult, Validator};
use crate::formats::{TranslationMap, TranslationValue};

pub struct MissingKeysValidator;

impl MissingKeysValidator {
    fn find_missing_keys(
        source: &TranslationMap,
        target: &TranslationMap,
        prefix: String,
    ) -> Vec<String> {
        let mut missing = Vec::new();

        for (key, value) in source.iter() {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            if let Some(target_value) = target.get(key) {
                if let (TranslationValue::Nested(source_nested), TranslationValue::Nested(target_nested)) =
                    (value, target_value)
                {
                    missing.extend(
                        Self::find_missing_keys(source_nested, target_nested, full_key)
                    );
                }
            } else {
                missing.push(full_key);
            }
        }

        missing
    }
}

impl Validator for MissingKeysValidator {
    fn validate(
        &self,
        source: &[(String, TranslationMap)],
        target: &[(String, TranslationMap)],
    ) -> crate::error::Result<ValidationResult> {
        let mut result = ValidationResult::new("missing_keys");

        for (namespace, source_map) in source {
            if let Some((_, target_map)) = target.iter().find(|(ns, _)| ns == namespace) {
                let missing = Self::find_missing_keys(source_map, target_map, String::new());

                for key in missing {
                    result.add_error(ValidationError::MissingKey {
                        key: format!("{}.{}", namespace, key),
                    });
                }
            }
        }

        Ok(result)
    }
}
