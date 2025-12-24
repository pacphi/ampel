use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use metrics_util::MetricKindMask;
use serde::Serialize;
use std::time::Duration;

use crate::AppState;

/// Initialize Prometheus metrics exporter
///
/// This function is safe to call multiple times - if a recorder is already
/// installed, it will return a new handle to a local (non-global) recorder.
/// This ensures tests can run in parallel without panicking.
pub fn init_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(Duration::from_secs(10)),
        )
        .install_recorder()
        .unwrap_or_else(|_| {
            // Recorder already installed (common in tests), create a local handle
            // This is safe because the handle still works for rendering metrics
            tracing::debug!("Prometheus recorder already installed, using fallback handle");
            PrometheusBuilder::new()
                .idle_timeout(
                    MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
                    Some(Duration::from_secs(10)),
                )
                .build_recorder()
                .handle()
        })
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "Observability",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = HealthResponse)
    )
)]
pub async fn health_handler(State(state): State<AppState>) -> Response {
    let db_healthy = check_database_health(&state).await;

    let response = HealthResponse {
        status: if db_healthy { "healthy" } else { "unhealthy" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: HealthChecks {
            database: db_healthy,
        },
    };

    let status = if db_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response)).into_response()
}

/// Readiness check endpoint
#[utoipa::path(
    get,
    path = "/ready",
    tag = "Observability",
    responses(
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Service is not ready", body = ReadinessResponse)
    )
)]
pub async fn readiness_handler(State(state): State<AppState>) -> Response {
    let db_ready = check_database_health(&state).await;

    let response = ReadinessResponse {
        ready: db_ready,
        checks: ReadinessChecks { database: db_ready },
    };

    let status = if db_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response)).into_response()
}

/// Prometheus metrics endpoint
pub async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}

async fn check_database_health(state: &AppState) -> bool {
    // Simple ping to check database connectivity
    state.db.ping().await.is_ok()
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HealthChecks {
    pub database: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub checks: ReadinessChecks,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReadinessChecks {
    pub database: bool,
}
