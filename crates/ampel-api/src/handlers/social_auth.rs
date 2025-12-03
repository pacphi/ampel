use axum::{
    extract::{Query, State},
    response::Redirect,
    Json,
};
use chrono::Utc;
use reqwest::Client;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::models::AuthTokens;
use ampel_db::entities::{user, user_oauth_account};
use ampel_db::queries::UserQueries;

use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct OAuthUrlResponse {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

// =============================================================================
// GitHub OAuth for User Authentication
// =============================================================================

/// Get GitHub OAuth authorization URL
pub async fn github_auth_url(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<OAuthUrlResponse>>, ApiError> {
    if state.config.github_auth_client_id.is_empty() {
        return Err(ApiError::bad_request("GitHub OAuth not configured"));
    }

    let csrf_state = Uuid::new_v4().to_string();

    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email&state={}",
        state.config.github_auth_client_id,
        urlencoding::encode(&state.config.github_auth_redirect_uri),
        csrf_state
    );

    Ok(Json(ApiResponse::success(OAuthUrlResponse { url })))
}

/// Handle GitHub OAuth callback
pub async fn github_auth_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, ApiError> {
    if state.config.github_auth_client_id.is_empty() {
        return Err(ApiError::bad_request("GitHub OAuth not configured"));
    }

    let client = Client::new();

    // Exchange code for access token
    let token_response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", &state.config.github_auth_client_id),
            ("client_secret", &state.config.github_auth_client_secret),
            ("code", &query.code),
            ("redirect_uri", &state.config.github_auth_redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| ApiError::internal(format!("GitHub OAuth error: {}", e)))?;

    let token_data: GitHubTokenResponse = token_response
        .json()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to parse GitHub token: {}", e)))?;

    if let Some(error) = token_data.error {
        return Err(ApiError::bad_request(format!("GitHub OAuth error: {}", error)));
    }

    let access_token = token_data
        .access_token
        .ok_or_else(|| ApiError::internal("No access token in response"))?;

    // Get user info from GitHub
    let user_response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "Ampel/1.0")
        .send()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get GitHub user: {}", e)))?;

    let github_user: GitHubUser = user_response
        .json()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to parse GitHub user: {}", e)))?;

    // Get user email (may need separate API call)
    let email = if let Some(email) = github_user.email {
        email
    } else {
        // Fetch emails from GitHub API
        let emails_response = client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "Ampel/1.0")
            .send()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to get GitHub emails: {}", e)))?;

        let emails: Vec<GitHubEmail> = emails_response.json().await.unwrap_or_default();

        emails
            .into_iter()
            .find(|e| e.primary)
            .map(|e| e.email)
            .ok_or_else(|| ApiError::bad_request("No email found on GitHub account"))?
    };

    // Find or create user
    let (_user, tokens) = find_or_create_user_from_oauth(
        &state,
        "github",
        &github_user.id.to_string(),
        &email,
        github_user.login.as_deref(),
        github_user.avatar_url.as_deref(),
    )
    .await?;

    // Redirect to frontend with tokens
    let frontend_url = state.config.cors_origins.first()
        .cloned()
        .unwrap_or_else(|| "http://localhost:3000".to_string());

    Ok(Redirect::temporary(&format!(
        "{}/?access_token={}&refresh_token={}&token_type=Bearer",
        frontend_url, tokens.access_token, tokens.refresh_token
    )))
}

// =============================================================================
// Google OAuth for User Authentication
// =============================================================================

/// Get Google OAuth authorization URL
pub async fn google_auth_url(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<OAuthUrlResponse>>, ApiError> {
    if state.config.google_client_id.is_empty() {
        return Err(ApiError::bad_request("Google OAuth not configured"));
    }

    let csrf_state = Uuid::new_v4().to_string();

    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&state={}",
        state.config.google_client_id,
        urlencoding::encode(&state.config.google_redirect_uri),
        csrf_state
    );

    Ok(Json(ApiResponse::success(OAuthUrlResponse { url })))
}

/// Handle Google OAuth callback
pub async fn google_auth_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, ApiError> {
    if state.config.google_client_id.is_empty() {
        return Err(ApiError::bad_request("Google OAuth not configured"));
    }

    let client = Client::new();

    // Exchange code for access token
    let token_response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", &state.config.google_client_id),
            ("client_secret", &state.config.google_client_secret),
            ("code", &query.code),
            ("redirect_uri", &state.config.google_redirect_uri),
            ("grant_type", &"authorization_code".to_string()),
        ])
        .send()
        .await
        .map_err(|e| ApiError::internal(format!("Google OAuth error: {}", e)))?;

    let token_data: GoogleTokenResponse = token_response
        .json()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to parse Google token: {}", e)))?;

    if let Some(error) = token_data.error {
        return Err(ApiError::bad_request(format!("Google OAuth error: {}", error)));
    }

    let access_token = token_data
        .access_token
        .ok_or_else(|| ApiError::internal("No access token in response"))?;

    // Get user info from Google
    let user_response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get Google user: {}", e)))?;

    let google_user: GoogleUser = user_response
        .json()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to parse Google user: {}", e)))?;

    let email = google_user
        .email
        .ok_or_else(|| ApiError::bad_request("No email found on Google account"))?;

    // Find or create user
    let (_user, tokens) = find_or_create_user_from_oauth(
        &state,
        "google",
        &google_user.id,
        &email,
        google_user.name.as_deref(),
        google_user.picture.as_deref(),
    )
    .await?;

    // Redirect to frontend with tokens
    let frontend_url = state.config.cors_origins.first()
        .cloned()
        .unwrap_or_else(|| "http://localhost:3000".to_string());

    Ok(Redirect::temporary(&format!(
        "{}/?access_token={}&refresh_token={}&token_type=Bearer",
        frontend_url, tokens.access_token, tokens.refresh_token
    )))
}

// =============================================================================
// Helper Functions
// =============================================================================

async fn find_or_create_user_from_oauth(
    state: &AppState,
    provider: &str,
    provider_user_id: &str,
    email: &str,
    username: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<(user::Model, AuthTokens), ApiError> {
    let now = Utc::now();

    // Check if OAuth account already exists
    let existing_oauth = user_oauth_account::Entity::find()
        .filter(user_oauth_account::Column::Provider.eq(provider))
        .filter(user_oauth_account::Column::ProviderUserId.eq(provider_user_id))
        .one(&state.db)
        .await?;

    let user = if let Some(oauth_account) = existing_oauth {
        // User exists, fetch them
        UserQueries::find_by_id(&state.db, oauth_account.user_id)
            .await?
            .ok_or_else(|| ApiError::internal("OAuth account exists but user not found"))?
    } else {
        // Check if user with this email exists (link accounts)
        let existing_user = UserQueries::find_by_email(&state.db, email).await?;

        let user = if let Some(user) = existing_user {
            user
        } else {
            // Create new user
            UserQueries::create(
                &state.db,
                email.to_string(),
                username.map(String::from),
                avatar_url.map(String::from),
            )
            .await?
        };

        // Create OAuth account link
        let oauth_account = user_oauth_account::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            provider: Set(provider.to_string()),
            provider_user_id: Set(provider_user_id.to_string()),
            provider_email: Set(Some(email.to_string())),
            provider_username: Set(username.map(String::from)),
            avatar_url: Set(avatar_url.map(String::from)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        oauth_account.insert(&state.db).await?;

        user
    };

    // Generate JWT tokens
    let tokens = state
        .auth_service
        .generate_tokens(user.id, &user.email)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok((user, tokens))
}

// =============================================================================
// Response Types
// =============================================================================

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubTokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubUser {
    id: i64,
    login: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleTokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<i64>,
    id_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleUser {
    id: String,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}
