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
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferencesResponse {
    pub email_enabled: bool,
    pub slack_enabled: bool,
    pub slack_webhook_url: Option<String>,
    pub push_enabled: bool,
    pub notify_on_pr_ready: bool,
    pub notify_on_pr_failed: bool,
    pub notify_on_review_requested: bool,
    pub digest_frequency: String,
    // Email SMTP settings
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i32>,
    pub smtp_username: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_to_emails: Option<Vec<String>>,
    pub smtp_use_tls: bool,
    // Merge notification settings
    pub notify_on_merge_success: bool,
    pub notify_on_merge_failure: bool,
    pub slack_channel: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotificationPreferencesRequest {
    pub email_enabled: Option<bool>,
    pub slack_enabled: Option<bool>,
    pub slack_webhook_url: Option<String>,
    pub push_enabled: Option<bool>,
    pub notify_on_pr_ready: Option<bool>,
    pub notify_on_pr_failed: Option<bool>,
    pub notify_on_review_requested: Option<bool>,
    pub digest_frequency: Option<String>,
    // Email SMTP settings
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i32>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>, // Will be encrypted before storage
    pub smtp_from_email: Option<String>,
    pub smtp_to_emails: Option<Vec<String>>,
    pub smtp_use_tls: Option<bool>,
    // Merge notification settings
    pub notify_on_merge_success: Option<bool>,
    pub notify_on_merge_failure: Option<bool>,
    pub slack_channel: Option<String>,
}

fn parse_smtp_to_emails(json_str: Option<&String>) -> Option<Vec<String>> {
    json_str.and_then(|s| serde_json::from_str(s).ok())
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
            smtp_host: p.smtp_host,
            smtp_port: p.smtp_port,
            smtp_username: p.smtp_username,
            smtp_from_email: p.smtp_from_email,
            smtp_to_emails: parse_smtp_to_emails(p.smtp_to_emails.as_ref()),
            smtp_use_tls: p.smtp_use_tls,
            notify_on_merge_success: p.notify_on_merge_success,
            notify_on_merge_failure: p.notify_on_merge_failure,
            slack_channel: p.slack_channel,
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
            smtp_host: None,
            smtp_port: None,
            smtp_username: None,
            smtp_from_email: None,
            smtp_to_emails: None,
            smtp_use_tls: true,
            notify_on_merge_success: true,
            notify_on_merge_failure: true,
            slack_channel: None,
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

    // Convert smtp_to_emails Vec to JSON string
    let smtp_to_emails_json = req
        .smtp_to_emails
        .as_ref()
        .map(|emails| serde_json::to_string(emails).unwrap_or_else(|_| "[]".to_string()));

    // Encrypt SMTP password if provided
    let smtp_password_encrypted = if let Some(ref password) = req.smtp_password {
        Some(
            state
                .encryption_service
                .encrypt(password)
                .map_err(|e| ApiError::internal(format!("Failed to encrypt password: {}", e)))?,
        )
    } else {
        None
    };

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
        // Email SMTP settings
        if req.smtp_host.is_some() {
            active.smtp_host = Set(req.smtp_host);
        }
        if req.smtp_port.is_some() {
            active.smtp_port = Set(req.smtp_port);
        }
        if req.smtp_username.is_some() {
            active.smtp_username = Set(req.smtp_username);
        }
        if smtp_password_encrypted.is_some() {
            active.smtp_password_encrypted = Set(smtp_password_encrypted);
        }
        if req.smtp_from_email.is_some() {
            active.smtp_from_email = Set(req.smtp_from_email);
        }
        if smtp_to_emails_json.is_some() {
            active.smtp_to_emails = Set(smtp_to_emails_json);
        }
        if let Some(v) = req.smtp_use_tls {
            active.smtp_use_tls = Set(v);
        }
        // Merge notification settings
        if let Some(v) = req.notify_on_merge_success {
            active.notify_on_merge_success = Set(v);
        }
        if let Some(v) = req.notify_on_merge_failure {
            active.notify_on_merge_failure = Set(v);
        }
        if req.slack_channel.is_some() {
            active.slack_channel = Set(req.slack_channel);
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
            smtp_host: Set(req.smtp_host),
            smtp_port: Set(req.smtp_port),
            smtp_username: Set(req.smtp_username),
            smtp_password_encrypted: Set(smtp_password_encrypted),
            smtp_from_email: Set(req.smtp_from_email),
            smtp_to_emails: Set(smtp_to_emails_json),
            smtp_use_tls: Set(req.smtp_use_tls.unwrap_or(true)),
            notify_on_merge_success: Set(req.notify_on_merge_success.unwrap_or(true)),
            notify_on_merge_failure: Set(req.notify_on_merge_failure.unwrap_or(true)),
            slack_channel: Set(req.slack_channel),
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
            smtp_host: updated.smtp_host,
            smtp_port: updated.smtp_port,
            smtp_username: updated.smtp_username,
            smtp_from_email: updated.smtp_from_email,
            smtp_to_emails: parse_smtp_to_emails(updated.smtp_to_emails.as_ref()),
            smtp_use_tls: updated.smtp_use_tls,
            notify_on_merge_success: updated.notify_on_merge_success,
            notify_on_merge_failure: updated.notify_on_merge_failure,
            slack_channel: updated.slack_channel,
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

/// Test email SMTP configuration
pub async fn test_email_smtp(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<bool>>, ApiError> {
    let prefs = notification_preferences::Entity::find()
        .filter(notification_preferences::Column::UserId.eq(auth.user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::bad_request("No notification preferences configured"))?;

    let smtp_host = prefs
        .smtp_host
        .ok_or_else(|| ApiError::bad_request("SMTP host not configured"))?;
    let smtp_port = prefs
        .smtp_port
        .ok_or_else(|| ApiError::bad_request("SMTP port not configured"))?;
    let smtp_username = prefs
        .smtp_username
        .ok_or_else(|| ApiError::bad_request("SMTP username not configured"))?;
    let smtp_password_encrypted = prefs
        .smtp_password_encrypted
        .ok_or_else(|| ApiError::bad_request("SMTP password not configured"))?;
    let smtp_from = prefs
        .smtp_from_email
        .ok_or_else(|| ApiError::bad_request("SMTP from email not configured"))?;
    let smtp_to_emails: Vec<String> = prefs
        .smtp_to_emails
        .as_ref()
        .and_then(|s| serde_json::from_str(s).ok())
        .ok_or_else(|| ApiError::bad_request("SMTP recipient emails not configured"))?;

    if smtp_to_emails.is_empty() {
        return Err(ApiError::bad_request("No recipient emails configured"));
    }

    // Decrypt password
    let smtp_password = state
        .encryption_service
        .decrypt(&smtp_password_encrypted)
        .map_err(|e| ApiError::internal(format!("Failed to decrypt SMTP password: {}", e)))?;

    // Build email transport
    use lettre::{
        message::header::ContentType, transport::smtp::authentication::Credentials, Message,
        SmtpTransport, Transport,
    };

    let creds = Credentials::new(smtp_username.clone(), smtp_password);

    let mailer = if prefs.smtp_use_tls {
        SmtpTransport::starttls_relay(&smtp_host)
            .map_err(|e| ApiError::bad_request(format!("Invalid SMTP host: {}", e)))?
            .port(smtp_port as u16)
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::builder_dangerous(&smtp_host)
            .port(smtp_port as u16)
            .credentials(creds)
            .build()
    };

    // Send test email
    let email = Message::builder()
        .from(
            smtp_from
                .parse()
                .map_err(|_| ApiError::bad_request("Invalid from email address"))?,
        )
        .to(smtp_to_emails[0]
            .parse()
            .map_err(|_| ApiError::bad_request("Invalid recipient email address"))?)
        .subject("Test Email from Ampel")
        .header(ContentType::TEXT_PLAIN)
        .body(
            "This is a test notification from Ampel! Your email integration is working correctly."
                .to_string(),
        )
        .map_err(|e| ApiError::internal(format!("Failed to build email: {}", e)))?;

    mailer
        .send(&email)
        .map_err(|e| ApiError::bad_request(format!("Failed to send test email: {}", e)))?;

    Ok(Json(ApiResponse::success(true)))
}
