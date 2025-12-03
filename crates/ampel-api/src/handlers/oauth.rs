use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::models::{GitProvider, ProviderConnectionResponse};
use ampel_db::entities::git_provider;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthUrlResponse {
    pub url: String,
}

/// Get OAuth authorization URL for a provider
pub async fn get_oauth_url(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(provider): Path<String>,
) -> Result<Json<ApiResponse<OAuthUrlResponse>>, ApiError> {
    let provider_type: GitProvider = provider
        .parse()
        .map_err(|_| ApiError::bad_request("Invalid provider"))?;

    let provider_client = state.provider_factory.create(provider_type);

    // Use user_id as state for CSRF protection
    let oauth_state = format!("{}:{}", auth.user_id, Uuid::new_v4());

    let url = provider_client.get_oauth_url(&oauth_state);

    Ok(Json(ApiResponse::success(OAuthUrlResponse { url })))
}

/// Handle OAuth callback
pub async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, ApiError> {
    let provider_type: GitProvider = provider
        .parse()
        .map_err(|_| ApiError::bad_request("Invalid provider"))?;

    // Parse state to get user_id
    let user_id: Uuid = query
        .state
        .split(':')
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ApiError::bad_request("Invalid state"))?;

    let provider_client = state.provider_factory.create(provider_type);

    // Exchange code for tokens
    let tokens = provider_client
        .exchange_code(&query.code)
        .await
        .map_err(|e| ApiError::bad_request(format!("OAuth error: {}", e)))?;

    // Get user info from provider
    let provider_user = provider_client
        .get_user(&tokens.access_token)
        .await
        .map_err(|e| ApiError::bad_request(format!("Failed to get user info: {}", e)))?;

    // Encrypt tokens
    let access_token_encrypted = state
        .encryption_service
        .encrypt(&tokens.access_token)
        .map_err(|e| ApiError::internal(format!("Encryption error: {}", e)))?;

    let refresh_token_encrypted = tokens
        .refresh_token
        .as_ref()
        .map(|rt| state.encryption_service.encrypt(rt))
        .transpose()
        .map_err(|e| ApiError::internal(format!("Encryption error: {}", e)))?;

    // Check if connection already exists
    let existing = git_provider::Entity::find()
        .filter(git_provider::Column::UserId.eq(user_id))
        .filter(git_provider::Column::Provider.eq(provider_type.to_string()))
        .one(&state.db)
        .await?;

    let now = Utc::now();
    let scopes_json = serde_json::to_string(&tokens.scopes).unwrap_or_else(|_| "[]".to_string());

    if let Some(existing) = existing {
        // Update existing connection
        let mut active: git_provider::ActiveModel = existing.into();
        active.provider_user_id = Set(provider_user.id);
        active.provider_username = Set(provider_user.username);
        active.access_token_encrypted = Set(access_token_encrypted);
        active.refresh_token_encrypted = Set(refresh_token_encrypted);
        active.token_expires_at = Set(tokens.expires_at);
        active.scopes = Set(scopes_json);
        active.updated_at = Set(now);
        active.update(&state.db).await?;
    } else {
        // Create new connection
        let connection = git_provider::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            provider: Set(provider_type.to_string()),
            provider_user_id: Set(provider_user.id),
            provider_username: Set(provider_user.username),
            access_token_encrypted: Set(access_token_encrypted),
            refresh_token_encrypted: Set(refresh_token_encrypted),
            token_expires_at: Set(tokens.expires_at),
            scopes: Set(scopes_json),
            instance_url: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        connection.insert(&state.db).await?;
    }

    // Redirect to frontend with success
    let default_url = "http://localhost:3000".to_string();
    let frontend_url = state.config.cors_origins.first().unwrap_or(&default_url);
    Ok(Redirect::temporary(&format!(
        "{}/settings/connections?success=true&provider={}",
        frontend_url, provider
    )))
}

/// List connected providers
pub async fn list_connections(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<ProviderConnectionResponse>>>, ApiError> {
    let connections = git_provider::Entity::find()
        .filter(git_provider::Column::UserId.eq(auth.user_id))
        .all(&state.db)
        .await?;

    let responses: Vec<ProviderConnectionResponse> = connections
        .into_iter()
        .map(|c| {
            let provider: GitProvider = c.provider.parse().unwrap_or(GitProvider::GitHub);
            let scopes: Vec<String> = serde_json::from_str(&c.scopes).unwrap_or_default();

            ProviderConnectionResponse {
                id: c.id,
                provider,
                provider_username: c.provider_username,
                scopes,
                token_expires_at: c.token_expires_at,
                created_at: c.created_at,
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(responses)))
}

/// Disconnect a provider
pub async fn disconnect_provider(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(provider): Path<String>,
) -> Result<StatusCode, ApiError> {
    let provider_type: GitProvider = provider
        .parse()
        .map_err(|_| ApiError::bad_request("Invalid provider"))?;

    git_provider::Entity::delete_many()
        .filter(git_provider::Column::UserId.eq(auth.user_id))
        .filter(git_provider::Column::Provider.eq(provider_type.to_string()))
        .exec(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
