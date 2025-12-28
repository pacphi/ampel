use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use rust_i18n::t;
use tokio::sync::Mutex;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RateLimitLayer {
    requests_per_minute: u32,
    state: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

#[derive(Clone)]
struct RateLimitState {
    count: u32,
    window_start: Instant,
}

impl RateLimitLayer {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            requests_per_minute: self.requests_per_minute,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    requests_per_minute: u32,
    state: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let requests_per_minute = self.requests_per_minute;
        let state = self.state.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract client identifier (IP or user ID from token)
            let client_id = req
                .headers()
                .get("x-forwarded-for")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let now = Instant::now();
            let window_duration = Duration::from_secs(60);

            let mut state_guard = state.lock().await;

            let entry = state_guard.entry(client_id).or_insert(RateLimitState {
                count: 0,
                window_start: now,
            });

            // Reset window if expired
            if now.duration_since(entry.window_start) >= window_duration {
                entry.count = 0;
                entry.window_start = now;
            }

            // Check rate limit
            if entry.count >= requests_per_minute {
                drop(state_guard);
                let response = Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("Retry-After", "60")
                    .body(Body::from(t!("errors.rate_limit.exceeded")))
                    .unwrap();
                return Ok(response);
            }

            entry.count += 1;
            drop(state_guard);

            inner.call(req).await
        })
    }
}
