use super::{ValidationResult, Validator};
use crate::formats::{TranslationMap, TranslationValue};

pub struct CoverageValidator;

impl CoverageValidator {
    fn count_keys(map: &TranslationMap) -> usize {
        map.values()
            .map(|value| match value {
                TranslationValue::String(_) => 1,
                TranslationValue::Plural(_) => 1,
                TranslationValue::Nested(nested) => Self::count_keys(nested),
            })
            .sum()
    }

    fn count_translated_keys(source: &TranslationMap, target: &TranslationMap) -> usize {
        source
            .iter()
            .map(|(key, value)| {
                if let Some(target_value) = target.get(key) {
                    match (value, target_value) {
                        (TranslationValue::String(_), TranslationValue::String(s)) => {
                            if !s.is_empty() {
                                1
                            } else {
                                0
                            }
                        }
                        (TranslationValue::Plural(_), TranslationValue::Plural(p)) => {
                            if !p.other.is_empty() {
                                1
                            } else {
                                0
                            }
                        }
                        (
                            TranslationValue::Nested(source_nested),
                            TranslationValue::Nested(target_nested),
                        ) => Self::count_translated_keys(source_nested, target_nested),
                        _ => 0,
                    }
                } else {
                    0
                }
            })
            .sum()
    }

    fn calculate_coverage(source_map: &TranslationMap, target_map: &TranslationMap) -> f32 {
        let total_keys = Self::count_keys(source_map);
        if total_keys == 0 {
            return 100.0;
        }

        let translated_keys = Self::count_translated_keys(source_map, target_map);
        (translated_keys as f32 / total_keys as f32) * 100.0
    }
}

impl Validator for CoverageValidator {
    fn validate(
        &self,
        source: &[(String, TranslationMap)],
        target: &[(String, TranslationMap)],
    ) -> crate::error::Result<ValidationResult> {
        let mut result = ValidationResult::new("coverage_validator");

        for (namespace, source_map) in source {
            if let Some((_, target_map)) = target.iter().find(|(ns, _)| ns == namespace) {
                let coverage = Self::calculate_coverage(source_map, target_map);

                if coverage < 100.0 {
                    result.add_warning(format!(
                        "{}: {:.1}% coverage ({:.1}% missing)",
                        namespace,
                        coverage,
                        100.0 - coverage
                    ));
                }
            }
        }

        Ok(result)
    }
}
