use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Format error: {0}")]
    Format(#[from] crate::formats::FormatError),

    #[error("Validation error: {0}")]
    ValidationError(#[from] crate::validation::ValidationError),

    #[error("Extraction error: {0}")]
    Extraction(#[from] crate::extraction::extractor::ExtractionError),

    #[error("Merge error: {0}")]
    Merge(#[from] crate::extraction::merger::MergeError),

    #[error("Refactor error: {0}")]
    Refactor(#[from] crate::refactor::RefactorError),

    #[error("Translation error: {0}")]
    Translation(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Coverage error: {0}")]
    Coverage(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
