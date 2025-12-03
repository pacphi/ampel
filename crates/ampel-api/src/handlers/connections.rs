use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use ampel_core::models::{
    AddConnectionRequest, GitProvider, ProviderConnectionResponse, UpdateConnectionRequest,
};
use ampel_db::entities::provider_connection;

use crate::extractors::{AuthUser, ValidatedJson};
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

/// Add a new PAT connection
pub async fn add_connection(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(req): ValidatedJson<AddConnectionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ProviderConnectionResponse>>), ApiError> {
    // Check if connection with same name already exists
    let existing = provider_connection::Entity::find()
        .filter(provider_connection::Column::UserId.eq(auth.user_id))
        .filter(provider_connection::Column::Provider.eq(req.provider.to_string()))
        .filter(provider_connection::Column::ConnectionName.eq(&req.connection_name))
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(ApiError::bad_request(format!(
            "Connection '{}' already exists for {}",
            req.connection_name, req.provider
        )));
    }

    // Create provider client with optional base URL
    let provider = state.provider_factory.create_with_base_url(req.provider, req.base_url.clone());

    // Validate the PAT by fetching user info
    let provider_user = provider
        .get_user(&req.access_token)
        .await
        .map_err(|e| ApiError::bad_request(format!("Invalid access token: {}", e)))?;

    // Encrypt the PAT
    let access_token_encrypted = state
        .encryption_service
        .encrypt(&req.access_token)
        .map_err(|e| ApiError::internal(format!("Encryption error: {}", e)))?;

    let now = Utc::now();

    // Create new connection
    let connection = provider_connection::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(auth.user_id),
        provider: Set(req.provider.to_string()),
        connection_name: Set(req.connection_name.clone()),
        provider_user_id: Set(provider_user.id),
        provider_username: Set(provider_user.username.clone()),
        access_token_encrypted: Set(access_token_encrypted),
        scopes: Set(None),
        base_url: Set(req.base_url.clone()),
        is_validated: Set(true),
        validation_error: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let saved = connection.insert(&state.db).await?;

    let response = ProviderConnectionResponse {
        id: saved.id,
        provider: req.provider,
        connection_name: saved.connection_name,
        provider_username: saved.provider_username,
        base_url: saved.base_url,
        is_validated: saved.is_validated,
        validation_error: saved.validation_error,
        created_at: saved.created_at,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// List all connections for the user
pub async fn list_connections(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<ProviderConnectionResponse>>>, ApiError> {
    let connections = provider_connection::Entity::find()
        .filter(provider_connection::Column::UserId.eq(auth.user_id))
        .all(&state.db)
        .await?;

    let responses: Vec<ProviderConnectionResponse> = connections
        .into_iter()
        .map(|c| {
            let provider: GitProvider = c.provider.parse().unwrap_or(GitProvider::GitHub);

            ProviderConnectionResponse {
                id: c.id,
                provider,
                connection_name: c.connection_name,
                provider_username: c.provider_username,
                base_url: c.base_url,
                is_validated: c.is_validated,
                validation_error: c.validation_error,
                created_at: c.created_at,
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(responses)))
}

/// Get a single connection
pub async fn get_connection(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProviderConnectionResponse>>, ApiError> {
    let connection = provider_connection::Entity::find_by_id(id)
        .filter(provider_connection::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Connection not found"))?;

    let provider: GitProvider = connection
        .provider
        .parse()
        .unwrap_or(GitProvider::GitHub);

    let response = ProviderConnectionResponse {
        id: connection.id,
        provider,
        connection_name: connection.connection_name,
        provider_username: connection.provider_username,
        base_url: connection.base_url,
        is_validated: connection.is_validated,
        validation_error: connection.validation_error,
        created_at: connection.created_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update a connection (rename or rotate PAT)
pub async fn update_connection(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateConnectionRequest>,
) -> Result<Json<ApiResponse<ProviderConnectionResponse>>, ApiError> {
    let connection = provider_connection::Entity::find_by_id(id)
        .filter(provider_connection::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Connection not found"))?;

    let mut active: provider_connection::ActiveModel = connection.clone().into();
    active.updated_at = Set(Utc::now());

    // Update connection name if provided
    if let Some(name) = &req.connection_name {
        active.connection_name = Set(name.clone());
    }

    // Update access token if provided
    if let Some(token) = &req.access_token {
        let provider: GitProvider = connection
            .provider
            .parse()
            .map_err(|_| ApiError::internal("Invalid provider"))?;

        // Validate the new PAT
        let provider_client =
            state
                .provider_factory
                .create_with_base_url(provider, connection.base_url.clone());

        let provider_user = provider_client
            .get_user(token)
            .await
            .map_err(|e| ApiError::bad_request(format!("Invalid access token: {}", e)))?;

        // Encrypt and update
        let access_token_encrypted = state
            .encryption_service
            .encrypt(token)
            .map_err(|e| ApiError::internal(format!("Encryption error: {}", e)))?;

        active.access_token_encrypted = Set(access_token_encrypted);
        active.provider_user_id = Set(provider_user.id);
        active.provider_username = Set(provider_user.username);
        active.is_validated = Set(true);
        active.validation_error = Set(None);
    }

    let updated = active.update(&state.db).await?;

    let provider: GitProvider = updated.provider.parse().unwrap_or(GitProvider::GitHub);

    let response = ProviderConnectionResponse {
        id: updated.id,
        provider,
        connection_name: updated.connection_name,
        provider_username: updated.provider_username,
        base_url: updated.base_url,
        is_validated: updated.is_validated,
        validation_error: updated.validation_error,
        created_at: updated.created_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Delete a connection
pub async fn delete_connection(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let connection = provider_connection::Entity::find_by_id(id)
        .filter(provider_connection::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Connection not found"))?;

    provider_connection::Entity::delete_by_id(connection.id)
        .exec(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Validate a connection (re-test the PAT)
pub async fn validate_connection(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProviderConnectionResponse>>, ApiError> {
    let connection = provider_connection::Entity::find_by_id(id)
        .filter(provider_connection::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Connection not found"))?;

    let provider: GitProvider = connection
        .provider
        .parse()
        .map_err(|_| ApiError::internal("Invalid provider"))?;

    // Decrypt access token
    let access_token = state
        .encryption_service
        .decrypt(&connection.access_token_encrypted)
        .map_err(|e| ApiError::internal(format!("Failed to decrypt token: {}", e)))?;

    // Create provider client
    let provider_client =
        state
            .provider_factory
            .create_with_base_url(provider, connection.base_url.clone());

    // Try to validate
    let mut active: provider_connection::ActiveModel = connection.clone().into();
    active.updated_at = Set(Utc::now());

    match provider_client.get_user(&access_token).await {
        Ok(provider_user) => {
            active.is_validated = Set(true);
            active.validation_error = Set(None);
            active.provider_user_id = Set(provider_user.id);
            active.provider_username = Set(provider_user.username);
        }
        Err(e) => {
            active.is_validated = Set(false);
            active.validation_error = Set(Some(e.to_string()));
        }
    }

    let updated = active.update(&state.db).await?;

    let response = ProviderConnectionResponse {
        id: updated.id,
        provider,
        connection_name: updated.connection_name,
        provider_username: updated.provider_username,
        base_url: updated.base_url,
        is_validated: updated.is_validated,
        validation_error: updated.validation_error,
        created_at: updated.created_at,
    };

    Ok(Json(ApiResponse::success(response)))
}
