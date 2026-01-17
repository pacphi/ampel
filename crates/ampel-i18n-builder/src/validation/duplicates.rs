use super::{ValidationError, ValidationResult, Validator};
use crate::formats::TranslationMap;
use std::collections::HashSet;

pub struct DuplicateKeysValidator;

impl DuplicateKeysValidator {
    fn collect_keys(
        map: &TranslationMap,
        prefix: String,
        seen: &mut HashSet<String>,
        duplicates: &mut Vec<String>,
    ) {
        for (key, value) in map.iter() {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            if !seen.insert(full_key.clone()) {
                duplicates.push(full_key.clone());
            }

            if let crate::formats::TranslationValue::Nested(nested) = value {
                Self::collect_keys(nested, full_key, seen, duplicates);
            }
        }
    }
}

impl Validator for DuplicateKeysValidator {
    fn validate(
        &self,
        _source: &[(String, TranslationMap)],
        target: &[(String, TranslationMap)],
    ) -> crate::error::Result<ValidationResult> {
        let mut result = ValidationResult::new("duplicate_keys");

        for (namespace, target_map) in target {
            let mut seen = HashSet::new();
            let mut duplicates = Vec::new();

            Self::collect_keys(target_map, String::new(), &mut seen, &mut duplicates);

            for key in duplicates {
                result.add_error(ValidationError::DuplicateKey {
                    key: format!("{}.{}", namespace, key),
                    line: 0,
                });
            }
        }

        Ok(result)
    }
}
