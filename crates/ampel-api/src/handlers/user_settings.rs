use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use ampel_db::queries::UserSettingsQueries;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsResponse {
    pub merge_delay_seconds: i32,
    pub require_approval: bool,
    pub delete_branches_default: bool,
    pub default_merge_strategy: String,
    pub skip_review_requirement: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserSettingsRequest {
    pub merge_delay_seconds: Option<i32>,
    pub require_approval: Option<bool>,
    pub delete_branches_default: Option<bool>,
    pub default_merge_strategy: Option<String>,
    pub skip_review_requirement: Option<bool>,
}

/// Get user behavior settings
pub async fn get_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<UserSettingsResponse>>, ApiError> {
    let settings = UserSettingsQueries::get_or_create_default(&state.db, auth.user_id).await?;

    Ok(Json(ApiResponse::success(UserSettingsResponse {
        merge_delay_seconds: settings.merge_delay_seconds,
        require_approval: settings.require_approval,
        delete_branches_default: settings.delete_branches_default,
        default_merge_strategy: settings.default_merge_strategy,
        skip_review_requirement: settings.skip_review_requirement,
    })))
}

/// Update user behavior settings
pub async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateUserSettingsRequest>,
) -> Result<Json<ApiResponse<UserSettingsResponse>>, ApiError> {
    // Get existing settings or defaults
    let existing = UserSettingsQueries::get_or_create_default(&state.db, auth.user_id).await?;

    // Validate merge strategy if provided
    if let Some(ref strategy) = req.default_merge_strategy {
        if !["merge", "squash", "rebase"].contains(&strategy.as_str()) {
            return Err(ApiError::bad_request(
                "Invalid merge strategy. Must be 'merge', 'squash', or 'rebase'",
            ));
        }
    }

    // Validate merge delay if provided
    if let Some(delay) = req.merge_delay_seconds {
        if !(0..=300).contains(&delay) {
            return Err(ApiError::bad_request(
                "Merge delay must be between 0 and 300 seconds",
            ));
        }
    }

    // Apply updates
    let updated = UserSettingsQueries::upsert(
        &state.db,
        auth.user_id,
        req.merge_delay_seconds
            .unwrap_or(existing.merge_delay_seconds),
        req.require_approval.unwrap_or(existing.require_approval),
        req.delete_branches_default
            .unwrap_or(existing.delete_branches_default),
        req.default_merge_strategy
            .unwrap_or(existing.default_merge_strategy),
        req.skip_review_requirement
            .unwrap_or(existing.skip_review_requirement),
    )
    .await?;

    Ok(Json(ApiResponse::success(UserSettingsResponse {
        merge_delay_seconds: updated.merge_delay_seconds,
        require_approval: updated.require_approval,
        delete_branches_default: updated.delete_branches_default,
        default_merge_strategy: updated.default_merge_strategy,
        skip_review_requirement: updated.skip_review_requirement,
    })))
}
