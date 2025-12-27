use std::collections::HashMap;
use thiserror::Error;

pub mod coverage;
pub mod missing;
pub mod duplicates;
pub mod variables;

pub use coverage::CoverageValidator;
pub use missing::MissingKeysValidator;
pub use duplicates::DuplicateKeysValidator;
pub use variables::VariableValidator;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    #[error("Missing key: {key}")]
    MissingKey { key: String },

    #[error("Duplicate key: {key} at line {line}")]
    DuplicateKey { key: String, line: usize },

    #[error("Variable mismatch in key '{key}': source has {source_vars:?}, translation has {translation_vars:?}")]
    VariableMismatch {
        key: String,
        source_vars: Vec<String>,
        translation_vars: Vec<String>,
    },

    #[error("Coverage below threshold: {actual}% < {threshold}%")]
    InsufficientCoverage { actual: f32, threshold: f32 },

    #[error("Invalid format: {message}")]
    InvalidFormat { message: String },
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub validator_name: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new(validator_name: impl Into<String>) -> Self {
        Self {
            validator_name: validator_name.into(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResults {
    pub results: HashMap<String, ValidationResult>,
}

impl ValidationResults {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    pub fn add_result(&mut self, result: ValidationResult) {
        self.results.insert(result.validator_name.clone(), result);
    }

    pub fn is_valid(&self) -> bool {
        self.results.values().all(|r| r.is_valid())
    }

    pub fn total_errors(&self) -> usize {
        self.results.values().map(|r| r.errors.len()).sum()
    }

    pub fn total_warnings(&self) -> usize {
        self.results.values().map(|r| r.warnings.len()).sum()
    }

    pub fn get_errors(&self) -> Vec<(String, ValidationError)> {
        self.results
            .iter()
            .flat_map(|(name, result)| {
                result
                    .errors
                    .iter()
                    .map(move |e| (name.clone(), e.clone()))
            })
            .collect()
    }
}

impl Default for ValidationResults {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Validator {
    /// Run validation on source and target namespaces and return results
    fn validate(
        &self,
        source: &[(String, crate::formats::TranslationMap)],
        target: &[(String, crate::formats::TranslationMap)],
    ) -> crate::error::Result<ValidationResult>;

    /// Get validator name
    fn name(&self) -> &str {
        "validator"
    }
}
