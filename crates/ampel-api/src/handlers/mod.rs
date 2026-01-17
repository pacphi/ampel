pub mod accounts;
pub mod analytics;
pub mod auth;
pub mod bot_rules;
pub mod bulk_merge;
pub mod dashboard;
pub mod diff_types;
pub mod notifications;
pub mod pr_filters;
pub mod pull_requests;
pub mod pull_requests_diff;
pub mod repositories;
pub mod teams;
pub mod user_settings;

use axum::{http::StatusCode, response::IntoResponse, Json};
use metrics::counter;
use serde::Serialize;

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Error response type
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(ApiResponse::<()>::error(self.message));
        (self.status, body).into_response()
    }
}

impl From<ampel_core::AmpelError> for ApiError {
    fn from(err: ampel_core::AmpelError) -> Self {
        ApiError::new(
            StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            err.to_string(),
        )
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(err: sea_orm::DbErr) -> Self {
        tracing::error!("Database error: {}", err);
        // Record database error metric
        counter!("ampel_dashboard_errors_total", "error_type" => "database").increment(1);
        ApiError::internal("Database error")
    }
}
