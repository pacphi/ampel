use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, Response},
    middleware::Next,
};
use metrics::{counter, histogram};
use std::time::Instant;

/// Middleware to record HTTP request metrics
pub async fn track_metrics(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, axum::http::StatusCode> {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();

    // Record metrics
    let labels = [
        ("method", method.to_string()),
        ("path", path.clone()),
        ("status", status.to_string()),
    ];

    counter!("http_requests_total", &labels).increment(1);
    histogram!("http_request_duration_seconds", &labels).record(latency);

    Ok(response)
}
