use axum::{extract::State, Json};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_db::entities::notification_preferences;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationPreferencesResponse {
    pub email_enabled: bool,
    pub slack_enabled: bool,
    pub slack_webhook_url: Option<String>,
    pub push_enabled: bool,
    pub notify_on_pr_ready: bool,
    pub notify_on_pr_failed: bool,
    pub notify_on_review_requested: bool,
    pub digest_frequency: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub email_enabled: Option<bool>,
    pub slack_enabled: Option<bool>,
    pub slack_webhook_url: Option<String>,
    pub push_enabled: Option<bool>,
    pub notify_on_pr_ready: Option<bool>,
    pub notify_on_pr_failed: Option<bool>,
    pub notify_on_review_requested: Option<bool>,
    pub digest_frequency: Option<String>,
}

/// Get notification preferences
pub async fn get_preferences(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<NotificationPreferencesResponse>>, ApiError> {
    let prefs = notification_preferences::Entity::find()
        .filter(notification_preferences::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?;

    let response = match prefs {
        Some(p) => NotificationPreferencesResponse {
            email_enabled: p.email_enabled,
            slack_enabled: p.slack_enabled,
            slack_webhook_url: p.slack_webhook_url,
            push_enabled: p.push_enabled,
            notify_on_pr_ready: p.notify_on_pr_ready,
            notify_on_pr_failed: p.notify_on_pr_failed,
            notify_on_review_requested: p.notify_on_review_requested,
            digest_frequency: p.digest_frequency,
        },
        None => NotificationPreferencesResponse {
            email_enabled: true,
            slack_enabled: false,
            slack_webhook_url: None,
            push_enabled: false,
            notify_on_pr_ready: true,
            notify_on_pr_failed: true,
            notify_on_review_requested: true,
            digest_frequency: "daily".to_string(),
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update notification preferences
pub async fn update_preferences(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateNotificationPreferencesRequest>,
) -> Result<Json<ApiResponse<NotificationPreferencesResponse>>, ApiError> {
    let existing = notification_preferences::Entity::find()
        .filter(notification_preferences::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?;

    let now = Utc::now();

    let updated = if let Some(existing) = existing {
        let mut active: notification_preferences::ActiveModel = existing.into();

        if let Some(v) = req.email_enabled {
            active.email_enabled = Set(v);
        }
        if let Some(v) = req.slack_enabled {
            active.slack_enabled = Set(v);
        }
        if req.slack_webhook_url.is_some() {
            active.slack_webhook_url = Set(req.slack_webhook_url);
        }
        if let Some(v) = req.push_enabled {
            active.push_enabled = Set(v);
        }
        if let Some(v) = req.notify_on_pr_ready {
            active.notify_on_pr_ready = Set(v);
        }
        if let Some(v) = req.notify_on_pr_failed {
            active.notify_on_pr_failed = Set(v);
        }
        if let Some(v) = req.notify_on_review_requested {
            active.notify_on_review_requested = Set(v);
        }
        if let Some(v) = req.digest_frequency {
            active.digest_frequency = Set(v);
        }
        active.updated_at = Set(now);

        active.update(&state.db).await?
    } else {
        let new_prefs = notification_preferences::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(auth.user_id),
            email_enabled: Set(req.email_enabled.unwrap_or(true)),
            slack_enabled: Set(req.slack_enabled.unwrap_or(false)),
            slack_webhook_url: Set(req.slack_webhook_url),
            push_enabled: Set(req.push_enabled.unwrap_or(false)),
            notify_on_pr_ready: Set(req.notify_on_pr_ready.unwrap_or(true)),
            notify_on_pr_failed: Set(req.notify_on_pr_failed.unwrap_or(true)),
            notify_on_review_requested: Set(req.notify_on_review_requested.unwrap_or(true)),
            digest_frequency: Set(req.digest_frequency.unwrap_or_else(|| "daily".to_string())),
            updated_at: Set(now),
        };
        new_prefs.insert(&state.db).await?
    };

    Ok(Json(ApiResponse::success(
        NotificationPreferencesResponse {
            email_enabled: updated.email_enabled,
            slack_enabled: updated.slack_enabled,
            slack_webhook_url: updated.slack_webhook_url,
            push_enabled: updated.push_enabled,
            notify_on_pr_ready: updated.notify_on_pr_ready,
            notify_on_pr_failed: updated.notify_on_pr_failed,
            notify_on_review_requested: updated.notify_on_review_requested,
            digest_frequency: updated.digest_frequency,
        },
    )))
}

/// Test Slack webhook
pub async fn test_slack_webhook(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<bool>>, ApiError> {
    let prefs = notification_preferences::Entity::find()
        .filter(notification_preferences::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::bad_request("No notification preferences configured"))?;

    let webhook_url = prefs
        .slack_webhook_url
        .ok_or_else(|| ApiError::bad_request("Slack webhook URL not configured"))?;

    let client = reqwest::Client::new();
    let response = client
        .post(&webhook_url)
        .json(&serde_json::json!({
            "text": "Test notification from Ampel! Your Slack integration is working correctly."
        }))
        .send()
        .await
        .map_err(|e| ApiError::bad_request(format!("Failed to send test message: {}", e)))?;

    if response.status().is_success() {
        Ok(Json(ApiResponse::success(true)))
    } else {
        Err(ApiError::bad_request("Slack webhook test failed"))
    }
}
