use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use ampel_db::queries::RepositoryFilterQueries;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFiltersResponse {
    pub include_public: bool,
    pub include_private: bool,
    pub include_archived: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRepositoryFiltersRequest {
    pub include_public: Option<bool>,
    pub include_private: Option<bool>,
    pub include_archived: Option<bool>,
}

/// Get user's repository filter settings
pub async fn get_repository_filters(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<RepositoryFiltersResponse>>, ApiError> {
    let filters = RepositoryFilterQueries::get_or_create_default(&state.db, auth.user_id).await?;

    Ok(Json(ApiResponse::success(RepositoryFiltersResponse {
        include_public: filters.include_public,
        include_private: filters.include_private,
        include_archived: filters.include_archived,
    })))
}

/// Update user's repository filter settings
pub async fn update_repository_filters(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateRepositoryFiltersRequest>,
) -> Result<Json<ApiResponse<RepositoryFiltersResponse>>, ApiError> {
    // Validate that at least one filter type is enabled
    let existing = RepositoryFilterQueries::get_or_create_default(&state.db, auth.user_id).await?;

    let final_public = req.include_public.unwrap_or(existing.include_public);
    let final_private = req.include_private.unwrap_or(existing.include_private);

    // At least one of public or private must be enabled
    if !final_public && !final_private {
        return Err(ApiError::bad_request(
            "At least one of public or private repositories must be enabled",
        ));
    }

    let updated = RepositoryFilterQueries::update(
        &state.db,
        auth.user_id,
        req.include_public,
        req.include_private,
        req.include_archived,
    )
    .await?;

    Ok(Json(ApiResponse::success(RepositoryFiltersResponse {
        include_public: updated.include_public,
        include_private: updated.include_private,
        include_archived: updated.include_archived,
    })))
}
