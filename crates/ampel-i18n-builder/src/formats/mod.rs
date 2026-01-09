use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

pub mod json;
pub mod yaml;

pub use json::JsonFormat;
pub use yaml::YamlFormat;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Failed to parse format: {0}")]
    ParseError(String),

    #[error("Failed to write format: {0}")]
    WriteError(String),

    #[error("Invalid schema: {0}")]
    SchemaError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type TranslationMap = BTreeMap<String, TranslationValue>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TranslationValue {
    String(String),
    Plural(PluralForms),
    Nested(TranslationMap),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluralForms {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub few: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub many: Option<String>,
    pub other: String,
}

pub trait TranslationFormat {
    /// Parse translation file into key-value map
    fn parse(&self, content: &str) -> Result<TranslationMap, FormatError>;

    /// Write key-value map to translation file
    fn write(&self, map: &TranslationMap) -> Result<String, FormatError>;

    /// Validate format schema
    fn validate(&self, content: &str) -> Result<(), FormatError>;
}

impl TranslationValue {
    /// Get all string values recursively
    pub fn all_strings(&self) -> Vec<&str> {
        match self {
            TranslationValue::String(s) => vec![s.as_str()],
            TranslationValue::Plural(forms) => {
                let mut strings = vec![forms.other.as_str()];
                if let Some(zero) = &forms.zero {
                    strings.push(zero.as_str());
                }
                if let Some(one) = &forms.one {
                    strings.push(one.as_str());
                }
                if let Some(two) = &forms.two {
                    strings.push(two.as_str());
                }
                if let Some(few) = &forms.few {
                    strings.push(few.as_str());
                }
                if let Some(many) = &forms.many {
                    strings.push(many.as_str());
                }
                strings
            }
            TranslationValue::Nested(map) => map.values().flat_map(|v| v.all_strings()).collect(),
        }
    }
}
