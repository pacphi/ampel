use thiserror::Error;

#[derive(Error, Debug)]
pub enum AmpelError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Rate limit exceeded for provider: {0}")]
    RateLimitExceeded(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl AmpelError {
    pub fn status_code(&self) -> u16 {
        match self {
            AmpelError::AuthenticationFailed(_) => 401,
            AmpelError::AuthorizationDenied(_) => 403,
            AmpelError::NotFound(_) => 404,
            AmpelError::ValidationError(_) => 400,
            AmpelError::TokenExpired => 401,
            AmpelError::InvalidToken(_) => 401,
            AmpelError::RateLimitExceeded(_) => 429,
            _ => 500,
        }
    }
}

pub type AmpelResult<T> = Result<T, AmpelError>;
