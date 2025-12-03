use axum::{extract::State, http::StatusCode, Json};

use ampel_core::models::{AuthTokens, RefreshTokenRequest, UserResponse};
use ampel_db::queries::UserQueries;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

// Note: Register and Login endpoints have been removed.
// User authentication is now OAuth-only via GitHub/Google social login.
// Social auth endpoints will be added in a future update.

/// Refresh access token
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<AuthTokens>>, ApiError> {
    // Validate refresh token
    let claims = state
        .auth_service
        .validate_refresh_token(&req.refresh_token)
        .map_err(|_| ApiError::unauthorized("Invalid or expired refresh token"))?;

    // Verify user still exists
    let user = UserQueries::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or_else(|| ApiError::unauthorized("User not found"))?;

    // Generate new tokens
    let tokens = state
        .auth_service
        .generate_tokens(user.id, &user.email)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(tokens)))
}

/// Get current user
pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<UserResponse>>, ApiError> {
    let user = UserQueries::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    let response: UserResponse = ampel_core::models::User::from(user).into();
    Ok(Json(ApiResponse::success(response)))
}

/// Logout (client should discard tokens)
pub async fn logout() -> StatusCode {
    // In a stateless JWT setup, logout is handled client-side
    // For enhanced security, you could maintain a token blacklist
    StatusCode::NO_CONTENT
}
