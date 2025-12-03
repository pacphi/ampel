use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::errors::{AmpelError, AmpelResult};
use crate::models::{AuthTokens, TokenClaims, TokenType};

pub struct AuthService {
    jwt_secret: String,
    access_token_expiry_minutes: i64,
    refresh_token_expiry_days: i64,
}

impl AuthService {
    pub fn new(
        jwt_secret: String,
        access_token_expiry_minutes: i64,
        refresh_token_expiry_days: i64,
    ) -> Self {
        Self {
            jwt_secret,
            access_token_expiry_minutes,
            refresh_token_expiry_days,
        }
    }

    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> AmpelResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AmpelError::InternalError(format!("Failed to hash password: {}", e)))
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, password_hash: &str) -> AmpelResult<bool> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| AmpelError::InternalError(format!("Invalid password hash: {}", e)))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Generate access and refresh tokens for a user
    pub fn generate_tokens(&self, user_id: Uuid, email: &str) -> AmpelResult<AuthTokens> {
        let access_token = self.generate_token(user_id, email, TokenType::Access)?;
        let refresh_token = self.generate_token(user_id, email, TokenType::Refresh)?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry_minutes * 60,
        })
    }

    /// Generate a single token (access or refresh)
    fn generate_token(
        &self,
        user_id: Uuid,
        email: &str,
        token_type: TokenType,
    ) -> AmpelResult<String> {
        let now = Utc::now();
        let expiry = match token_type {
            TokenType::Access => now + Duration::minutes(self.access_token_expiry_minutes),
            TokenType::Refresh => now + Duration::days(self.refresh_token_expiry_days),
        };

        let claims = TokenClaims {
            sub: user_id,
            email: email.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AmpelError::InternalError(format!("Failed to generate token: {}", e)))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> AmpelResult<TokenClaims> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AmpelError::TokenExpired,
            _ => AmpelError::InvalidToken(e.to_string()),
        })
    }

    /// Validate that a token is an access token
    pub fn validate_access_token(&self, token: &str) -> AmpelResult<TokenClaims> {
        let claims = self.validate_token(token)?;
        if claims.token_type != TokenType::Access {
            return Err(AmpelError::InvalidToken(
                "Expected access token".to_string(),
            ));
        }
        Ok(claims)
    }

    /// Validate that a token is a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> AmpelResult<TokenClaims> {
        let claims = self.validate_token(token)?;
        if claims.token_type != TokenType::Refresh {
            return Err(AmpelError::InvalidToken(
                "Expected refresh token".to_string(),
            ));
        }
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_auth_service() -> AuthService {
        AuthService::new("test-secret-key-32-characters!!".to_string(), 15, 7)
    }

    #[test]
    fn test_password_hashing() {
        let service = create_auth_service();
        let password = "mysecurepassword123";

        let hash = service.hash_password(password).unwrap();
        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn test_token_generation_and_validation() {
        let service = create_auth_service();
        let user_id = Uuid::new_v4();
        let email = "test@example.com";

        let tokens = service.generate_tokens(user_id, email).unwrap();

        let access_claims = service.validate_access_token(&tokens.access_token).unwrap();
        assert_eq!(access_claims.sub, user_id);
        assert_eq!(access_claims.email, email);
        assert_eq!(access_claims.token_type, TokenType::Access);

        let refresh_claims = service
            .validate_refresh_token(&tokens.refresh_token)
            .unwrap();
        assert_eq!(refresh_claims.sub, user_id);
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_wrong_token_type_validation() {
        let service = create_auth_service();
        let tokens = service
            .generate_tokens(Uuid::new_v4(), "test@example.com")
            .unwrap();

        // Access token should fail refresh validation
        assert!(service
            .validate_refresh_token(&tokens.access_token)
            .is_err());

        // Refresh token should fail access validation
        assert!(service
            .validate_access_token(&tokens.refresh_token)
            .is_err());
    }
}
