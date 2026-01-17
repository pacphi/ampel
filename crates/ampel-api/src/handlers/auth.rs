use axum::{extract::State, http::StatusCode, Json};
use rust_i18n::t;

use ampel_core::models::{
    AuthTokens, CreateUserRequest, LoginRequest, RefreshTokenRequest, UpdateProfileRequest,
    UserResponse,
};
use ampel_db::queries::UserQueries;

use crate::extractors::{AuthUser, ValidatedJson};
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AuthTokens>>), ApiError> {
    // Check if user already exists
    if UserQueries::find_by_email(&state.db, &req.email)
        .await?
        .is_some()
    {
        return Err(ApiError::bad_request(t!("errors.auth.email_registered")));
    }

    // Hash password
    let password_hash = state
        .auth_service
        .hash_password(&req.password)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Create user
    let user = UserQueries::create(
        &state.db,
        req.email.clone(),
        password_hash,
        req.display_name,
    )
    .await?;

    // Generate tokens
    let tokens = state
        .auth_service
        .generate_tokens(user.id, &user.email)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(tokens))))
}

/// Login with email and password
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> Result<Json<ApiResponse<AuthTokens>>, ApiError> {
    // Find user
    let user = UserQueries::find_by_email(&state.db, &req.email)
        .await?
        .ok_or_else(|| ApiError::unauthorized(t!("errors.auth.invalid_credentials")))?;

    // Verify password
    let valid = state
        .auth_service
        .verify_password(&req.password, &user.password_hash)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    if !valid {
        return Err(ApiError::unauthorized(t!(
            "errors.auth.invalid_credentials"
        )));
    }

    // Generate tokens
    let tokens = state
        .auth_service
        .generate_tokens(user.id, &user.email)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(tokens)))
}

/// Refresh access token
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<AuthTokens>>, ApiError> {
    // Validate refresh token
    let claims = state
        .auth_service
        .validate_refresh_token(&req.refresh_token)
        .map_err(|_| ApiError::unauthorized(t!("errors.auth.invalid_refresh_token")))?;

    // Verify user still exists
    let user = UserQueries::find_by_id(&state.db, claims.sub)
        .await?
        .ok_or_else(|| ApiError::unauthorized(t!("errors.auth.user_not_found")))?;

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
        .ok_or_else(|| ApiError::not_found(t!("errors.auth.user_not_found")))?;

    let response: UserResponse = ampel_core::models::User::from(user).into();
    Ok(Json(ApiResponse::success(response)))
}

/// Update current user profile
pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, ApiError> {
    // If email is being changed, check it's not already taken
    if let Some(ref new_email) = req.email {
        if let Some(existing) = UserQueries::find_by_email(&state.db, new_email).await? {
            if existing.id != auth.user_id {
                return Err(ApiError::bad_request(t!("errors.auth.email_in_use")));
            }
        }
    }

    // Convert display_name to Option<Option<String>> for the query
    // Some(value) means update, None means don't change
    let display_name_update = if req.display_name.is_some() {
        Some(req.display_name)
    } else {
        None
    };

    let user = UserQueries::update_profile(&state.db, auth.user_id, req.email, display_name_update)
        .await?;

    let response: UserResponse = ampel_core::models::User::from(user).into();
    Ok(Json(ApiResponse::success(response)))
}

/// Logout (client should discard tokens)
pub async fn logout() -> StatusCode {
    // In a stateless JWT setup, logout is handled client-side
    // For enhanced security, you could maintain a token blacklist
    StatusCode::NO_CONTENT
}
