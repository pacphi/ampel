use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_db::queries::PrFilterQueries;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrFilterResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub allowed_actors: Vec<String>,
    pub skip_labels: Vec<String>,
    pub max_age_days: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePrFilterRequest {
    pub allowed_actors: Option<Vec<String>>,
    pub skip_labels: Option<Vec<String>>,
    pub max_age_days: Option<i32>,
}

/// Get PR filter settings for the current user
pub async fn get_pr_filters(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<PrFilterResponse>>, ApiError> {
    let filter = PrFilterQueries::find_by_user(&state.db, auth.user_id).await?;

    let response = if let Some(f) = filter {
        let allowed_actors: Vec<String> =
            serde_json::from_str(&f.allowed_actors).unwrap_or_default();
        let skip_labels: Vec<String> = serde_json::from_str(&f.skip_labels).unwrap_or_default();
        PrFilterResponse {
            id: f.id,
            user_id: f.user_id,
            allowed_actors,
            skip_labels,
            max_age_days: f.max_age_days,
        }
    } else {
        // Return defaults
        PrFilterResponse {
            id: Uuid::nil(),
            user_id: auth.user_id,
            allowed_actors: vec![
                "dependabot[bot]".to_string(),
                "renovate[bot]".to_string(),
                "snyk-bot".to_string(),
            ],
            skip_labels: vec![
                "do-not-merge".to_string(),
                "wip".to_string(),
                "draft".to_string(),
                "hold".to_string(),
            ],
            max_age_days: None,
        }
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update PR filter settings for the current user
pub async fn update_pr_filters(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdatePrFilterRequest>,
) -> Result<Json<ApiResponse<PrFilterResponse>>, ApiError> {
    // Get existing or use defaults
    let existing = PrFilterQueries::find_by_user(&state.db, auth.user_id).await?;

    let current_allowed: Vec<String> = existing
        .as_ref()
        .map(|f| serde_json::from_str(&f.allowed_actors).unwrap_or_default())
        .unwrap_or_else(|| {
            vec![
                "dependabot[bot]".to_string(),
                "renovate[bot]".to_string(),
                "snyk-bot".to_string(),
            ]
        });

    let current_skip: Vec<String> = existing
        .as_ref()
        .map(|f| serde_json::from_str(&f.skip_labels).unwrap_or_default())
        .unwrap_or_else(|| {
            vec![
                "do-not-merge".to_string(),
                "wip".to_string(),
                "draft".to_string(),
                "hold".to_string(),
            ]
        });

    let current_max_age = existing.as_ref().and_then(|f| f.max_age_days);

    let allowed_actors = req.allowed_actors.unwrap_or(current_allowed);
    let skip_labels = req.skip_labels.unwrap_or(current_skip);
    let max_age_days = req.max_age_days.or(current_max_age);

    let allowed_actors_json = serde_json::to_string(&allowed_actors)
        .map_err(|e| ApiError::internal(format!("Failed to serialize allowed_actors: {}", e)))?;
    let skip_labels_json = serde_json::to_string(&skip_labels)
        .map_err(|e| ApiError::internal(format!("Failed to serialize skip_labels: {}", e)))?;

    let updated = PrFilterQueries::upsert(
        &state.db,
        auth.user_id,
        allowed_actors_json,
        skip_labels_json,
        max_age_days,
    )
    .await?;

    let response_allowed: Vec<String> =
        serde_json::from_str(&updated.allowed_actors).unwrap_or_default();
    let response_skip: Vec<String> = serde_json::from_str(&updated.skip_labels).unwrap_or_default();

    Ok(Json(ApiResponse::success(PrFilterResponse {
        id: updated.id,
        user_id: updated.user_id,
        allowed_actors: response_allowed,
        skip_labels: response_skip,
        max_age_days: updated.max_age_days,
    })))
}

/// Reset PR filter settings to defaults
pub async fn reset_pr_filters(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<PrFilterResponse>>, ApiError> {
    PrFilterQueries::delete(&state.db, auth.user_id).await?;

    Ok(Json(ApiResponse::success(PrFilterResponse {
        id: Uuid::nil(),
        user_id: auth.user_id,
        allowed_actors: vec![
            "dependabot[bot]".to_string(),
            "renovate[bot]".to_string(),
            "snyk-bot".to_string(),
        ],
        skip_labels: vec![
            "do-not-merge".to_string(),
            "wip".to_string(),
            "draft".to_string(),
            "hold".to_string(),
        ],
        max_age_days: None,
    })))
}
