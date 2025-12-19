use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Rate limit exceeded. Resets at: {0}")]
    RateLimitExceeded(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Provider API error: {message} (status: {status_code})")]
    ApiError { status_code: u16, message: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::NetworkError("Request timed out".to_string())
        } else if err.is_connect() {
            ProviderError::NetworkError("Connection failed".to_string())
        } else {
            ProviderError::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::SerializationError(err.to_string())
    }
}

pub type ProviderResult<T> = Result<T, ProviderError>;
