use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use redis::AsyncCommands;

use crate::AppState;

/// Redis-based distributed rate limiter
///
/// Uses Redis for rate limiting to support multiple API instances.
/// Implements sliding window algorithm with burst allowance.
#[derive(Clone)]
pub struct RedisRateLimiter {
    redis: redis::aio::ConnectionManager,
    requests_per_hour: u32,
    burst_allowance: u32,
    window_seconds: u64,
}

impl RedisRateLimiter {
    pub fn new(
        redis: redis::aio::ConnectionManager,
        requests_per_hour: u32,
        burst_allowance: u32,
    ) -> Self {
        Self {
            redis,
            requests_per_hour,
            burst_allowance,
            window_seconds: 3600, // 1 hour
        }
    }

    /// Check if request is allowed under rate limit
    ///
    /// Returns (allowed, remaining, reset_timestamp)
    pub async fn check_rate_limit(
        &mut self,
        client_id: &str,
    ) -> Result<(bool, u32, u64), redis::RedisError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let key = format!("rate_limit:{}", client_id);
        let window_key = format!("{}:window", key);
        let burst_key = format!("{}:burst", key);

        // Get current count in window
        let count: u32 = self.redis.get(&key).await.unwrap_or(0);

        // Get burst tokens used in last second
        let burst_count: u32 = self.redis.get(&burst_key).await.unwrap_or(0);

        // Check burst allowance first (20 requests/second)
        if burst_count >= self.burst_allowance {
            let reset = now + 1; // Burst resets every second
            return Ok((false, 0, reset));
        }

        // Check hourly limit
        if count >= self.requests_per_hour {
            // Get window start time
            let window_start: u64 = self.redis.get(&window_key).await.unwrap_or(now);
            let reset = window_start + self.window_seconds;
            return Ok((false, 0, reset));
        }

        // Increment counters
        let _: () = redis::pipe()
            .atomic()
            .incr(&key, 1)
            .expire(&key, self.window_seconds as i64)
            .incr(&burst_key, 1)
            .expire(&burst_key, 1) // Burst window is 1 second
            .query_async(&mut self.redis)
            .await?;

        // Set window start if first request
        if count == 0 {
            let _: () = self
                .redis
                .set_ex(&window_key, now, self.window_seconds)
                .await?;
        }

        let remaining = self.requests_per_hour.saturating_sub(count + 1);
        let window_start: u64 = self.redis.get(&window_key).await.unwrap_or(now);
        let reset = window_start + self.window_seconds;

        Ok((true, remaining, reset))
    }
}

/// Axum middleware for rate limiting diff endpoint
///
/// Applies 100 requests/hour per user with 20 requests/second burst allowance
pub async fn rate_limit_diff(
    state: axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, Response<Body>> {
    // Extract user ID from auth headers
    let client_id = extract_client_id(&req);

    // Skip rate limiting if no Redis available (development mode)
    let Some(redis) = state.redis.as_ref() else {
        tracing::warn!("Redis not available, skipping rate limiting");
        return Ok(next.run(req).await);
    };

    let mut limiter = RedisRateLimiter::new(
        redis.clone(),
        100,  // 100 requests/hour
        20,   // 20 requests/second burst
    );

    match limiter.check_rate_limit(&client_id).await {
        Ok((allowed, remaining, reset)) => {
            if allowed {
                let mut response = next.run(req).await;
                let headers = response.headers_mut();
                headers.insert("X-RateLimit-Limit", "100".parse().unwrap());
                headers.insert(
                    "X-RateLimit-Remaining",
                    remaining.to_string().parse().unwrap(),
                );
                headers.insert("X-RateLimit-Reset", reset.to_string().parse().unwrap());
                Ok(response)
            } else {
                let retry_after = reset
                    .saturating_sub(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    )
                    .max(1);

                Err(Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("Retry-After", retry_after.to_string())
                    .header("X-RateLimit-Limit", "100")
                    .header("X-RateLimit-Remaining", "0")
                    .header("X-RateLimit-Reset", reset.to_string())
                    .body(Body::from(
                        serde_json::json!({
                            "success": false,
                            "error": {
                                "code": "RATE_LIMIT_EXCEEDED",
                                "message": "Rate limit exceeded. Please try again later.",
                                "details": {
                                    "limit": 100,
                                    "window": "1 hour",
                                    "retry_after_seconds": retry_after
                                }
                            }
                        })
                        .to_string(),
                    ))
                    .unwrap())
            }
        }
        Err(e) => {
            tracing::error!("Rate limit check failed: {}", e);
            // Allow request on error to prevent service disruption
            Ok(next.run(req).await)
        }
    }
}

/// Extract client identifier from request
///
/// Priority: User ID from auth > IP address > unknown
fn extract_client_id(req: &Request) -> String {
    // Try to get user ID from auth extension (set by auth middleware)
    if let Some(auth_user) = req.extensions().get::<crate::extractors::AuthUser>() {
        return format!("user:{}", auth_user.user_id);
    }

    // Fallback to IP address
    req.headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .map(|ip| format!("ip:{}", ip))
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        // This test requires a Redis instance
        // Skip in CI if REDIS_URL not set
        if std::env::var("REDIS_URL").is_err() {
            return;
        }

        let client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .unwrap();

        let mut limiter = RedisRateLimiter::new(conn, 100, 20);

        let (allowed, remaining, _) = limiter.check_rate_limit("test_user").await.unwrap();
        assert!(allowed);
        assert!(remaining > 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_burst_protection() {
        if std::env::var("REDIS_URL").is_err() {
            return;
        }

        let client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .unwrap();

        let mut limiter = RedisRateLimiter::new(conn, 100, 20);

        // Send 21 requests rapidly (burst limit is 20)
        for i in 0..21 {
            let (allowed, _, _) = limiter
                .check_rate_limit("burst_test_user")
                .await
                .unwrap();
            if i < 20 {
                assert!(allowed, "Request {} should be allowed", i);
            } else {
                assert!(!allowed, "Request {} should be rate limited", i);
            }
        }
    }
}
