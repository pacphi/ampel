# ADR-004: Error Recovery Strategy for Provider API Failures

**Status:** Accepted
**Date:** 2025-12-25
**Decision Makers:** Architecture Team
**Technical Story:** Resilient Diff Fetching

## Context

Fetching diffs from external provider APIs introduces multiple failure modes:

1. **Network Errors**: Timeouts, connection failures, DNS issues
2. **Provider API Errors**: 500 errors, rate limiting (429), maintenance (503)
3. **Authentication Errors**: Invalid tokens, expired credentials, insufficient permissions
4. **Data Errors**: Malformed responses, missing fields, encoding issues
5. **Cache Errors**: Redis unavailable, serialization failures

The system must handle these gracefully without degrading user experience.

## Decision Drivers

- **Reliability**: Graceful degradation, no data loss
- **User Experience**: Clear error messages, actionable feedback
- **Observability**: Detailed logging for debugging
- **Recovery**: Automatic retries with exponential backoff
- **Fail-Safe**: Never crash the application

## Considered Options

### Option 1: Multi-Layer Error Handling with Circuit Breaker (SELECTED)

Implement defense-in-depth strategy:

1. **Request-Level Retries**: Exponential backoff for transient errors
2. **Circuit Breaker**: Stop hitting failing provider after threshold
3. **Fallback Mechanisms**: Serve stale cache, degraded mode
4. **User Feedback**: Clear error messages with recovery actions

**Pros:**

- Comprehensive coverage of failure modes
- Prevents cascade failures (circuit breaker)
- Users understand what's wrong and how to fix it

**Cons:**

- More complex implementation
- Requires careful tuning of retry/circuit breaker parameters

### Option 2: Simple Retry-Only

Retry failed requests 2-3 times with fixed backoff.

**Pros:**

- Simple to implement
- Handles transient errors

**Cons:**

- No protection against persistent failures
- Can hammer failing provider (thundering herd)
- No stale cache fallback

### Option 3: Fail Fast (No Retries)

Return error immediately on first failure.

**Pros:**

- Fastest failure feedback
- Simplest code

**Cons:**

- Poor UX (users see errors for transient issues)
- No resilience

## Decision Outcome

**Chosen Option:** Multi-Layer Error Handling with Circuit Breaker

### Error Classification

```rust
// crates/ampel-providers/src/error.rs

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    // Retryable errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Rate limit exceeded. Retry after {retry_after}s")]
    RateLimit { retry_after: u64 },

    #[error("Provider timeout after {timeout}s")]
    Timeout { timeout: u64 },

    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },

    // Non-retryable errors
    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Authorization denied: {message}")]
    AuthorizationDenied { message: String },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Invalid response: {message}")]
    InvalidResponse { message: String },

    // Internal errors
    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl ProviderError {
    /// Determine if error should trigger retry
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ProviderError::Network(_)
                | ProviderError::RateLimit { .. }
                | ProviderError::Timeout { .. }
                | ProviderError::ServiceUnavailable { .. }
        )
    }

    /// Get HTTP status code for API response
    pub fn status_code(&self) -> StatusCode {
        match self {
            ProviderError::AuthenticationFailed { .. } => StatusCode::UNAUTHORIZED,
            ProviderError::AuthorizationDenied { .. } => StatusCode::FORBIDDEN,
            ProviderError::NotFound { .. } => StatusCode::NOT_FOUND,
            ProviderError::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            ProviderError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

### Retry Strategy

```rust
// crates/ampel-core/src/services/retry.rs

use tokio::time::{sleep, Duration};

pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, T, E>(
    policy: &RetryPolicy,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> BoxFuture<'static, Result<T, E>>,
    E: std::error::Error + 'static,
{
    let mut backoff_ms = policy.initial_backoff_ms;

    for attempt in 0..=policy.max_retries {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    tracing::info!(attempt, "Operation succeeded after retry");
                    counter!("ampel_retry_success_total", "attempt" => attempt.to_string())
                        .increment(1);
                }
                return Ok(result);
            }
            Err(err) => {
                if attempt == policy.max_retries {
                    tracing::error!(attempt, error = %err, "Operation failed after max retries");
                    counter!("ampel_retry_failure_total").increment(1);
                    return Err(err);
                }

                tracing::warn!(
                    attempt,
                    backoff_ms,
                    error = %err,
                    "Operation failed, retrying..."
                );

                sleep(Duration::from_millis(backoff_ms)).await;

                // Exponential backoff with jitter
                backoff_ms = (backoff_ms as f64 * policy.backoff_multiplier) as u64;
                backoff_ms = backoff_ms.min(policy.max_backoff_ms);
                backoff_ms += rand::thread_rng().gen_range(0..100); // Jitter
            }
        }
    }

    unreachable!("Loop should always return within max_retries")
}
```

### Circuit Breaker

```rust
// crates/ampel-core/src/services/circuit_breaker.rs

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, reject requests
    HalfOpen,    // Testing recovery
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout_seconds: u64,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout_seconds: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_threshold,
            success_threshold,
            timeout_seconds,
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> BoxFuture<'static, Result<T, E>>,
        E: std::error::Error,
    {
        // Check if circuit is open
        let state = self.state.read().await;
        if *state == CircuitState::Open {
            // Check if timeout expired
            let last_failure = self.last_failure_time.read().await;
            if let Some(time) = *last_failure {
                let elapsed = Utc::now().signed_duration_since(time).num_seconds() as u64;
                if elapsed > self.timeout_seconds {
                    // Transition to half-open
                    drop(state);
                    *self.state.write().await = CircuitState::HalfOpen;
                    tracing::info!("Circuit breaker transitioning to HalfOpen");
                } else {
                    return Err(CircuitBreakerError::Open);
                }
            }
        }
        drop(state);

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(CircuitBreakerError::Execution(err))
            }
        }
    }

    async fn on_success(&self) {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::HalfOpen => {
                let mut success_count = self.success_count.write().await;
                *success_count += 1;

                if *success_count >= self.success_threshold {
                    *self.state.write().await = CircuitState::Closed;
                    *success_count = 0;
                    *self.failure_count.write().await = 0;
                    tracing::info!("Circuit breaker closed");
                    gauge!("ampel_circuit_breaker_state", "provider" => "github").set(0);
                }
            }
            CircuitState::Closed => {
                *self.failure_count.write().await = 0;
            }
            _ => {}
        }
    }

    async fn on_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;

        if *failure_count >= self.failure_threshold {
            *self.state.write().await = CircuitState::Open;
            *self.last_failure_time.write().await = Some(Utc::now());
            tracing::error!("Circuit breaker opened after {} failures", failure_count);
            gauge!("ampel_circuit_breaker_state", "provider" => "github").set(1);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    Open,

    #[error("Execution failed: {0}")]
    Execution(E),
}
```

### Fallback Mechanisms

```rust
// crates/ampel-api/src/handlers/pull_requests.rs

pub async fn get_pull_request_diff(
    State(state): State<AppState>,
    Path(pr_id): Path<Uuid>,
) -> Result<Json<DiffResponse>, ApiError> {
    let pr = state.db.find_pull_request(pr_id).await?;

    // 1. Try cache first (fast path)
    if let Some(diff) = state.diff_cache.get(...).await {
        return Ok(Json(DiffResponse::from_cached(diff)));
    }

    // 2. Fetch from provider with retries + circuit breaker
    let provider = state.provider_factory.create_provider(...)?;

    let diff_result = state.circuit_breakers
        .get(&pr.repository.provider)
        .unwrap()
        .call(|| {
            Box::pin(async {
                retry_with_backoff(
                    &RetryPolicy::default(),
                    || Box::pin(provider.get_pull_request_diff(...)),
                ).await
            })
        })
        .await;

    match diff_result {
        Ok(diff) => {
            // Cache successful result
            state.diff_cache.set(..., &diff, ttl).await?;
            Ok(Json(DiffResponse::from(diff)))
        }
        Err(CircuitBreakerError::Open) => {
            // Circuit open: try stale cache
            if let Some(stale_diff) = state.diff_cache.get_stale(...).await {
                tracing::warn!("Serving stale diff due to circuit breaker");
                return Ok(Json(DiffResponse::from_stale(stale_diff)));
            }

            // No stale cache: return error with actionable message
            Err(ApiError::ServiceUnavailable {
                message: format!(
                    "{} provider is temporarily unavailable. Please try again in a few minutes.",
                    pr.repository.provider
                ),
                retry_after: Some(60),
            })
        }
        Err(CircuitBreakerError::Execution(err)) => {
            if err.is_retryable() {
                // Temporary error: suggest retry
                Err(ApiError::TemporaryError {
                    message: err.to_string(),
                    retry_after: Some(5),
                })
            } else {
                // Permanent error: user action required
                Err(ApiError::from(err))
            }
        }
    }
}
```

### User-Facing Error Messages

```typescript
// frontend/src/components/diff/DiffErrorDisplay.tsx

export function DiffErrorDisplay({ error }: { error: ApiError }) {
  const errorMessages = {
    401: {
      title: 'Authentication Failed',
      message: 'Your provider token has expired or is invalid.',
      action: 'Please update your credentials in Settings',
      actionLink: '/settings/accounts',
    },
    403: {
      title: 'Permission Denied',
      message: 'You do not have access to view this diff.',
      action: 'Ensure your token has the required scopes',
      actionLink: '/docs/permissions',
    },
    404: {
      title: 'Pull Request Not Found',
      message: 'This PR may have been deleted or moved.',
      action: 'Check the repository on the provider',
    },
    429: {
      title: 'Rate Limit Exceeded',
      message: `Provider API rate limit reached. Retry after ${error.retryAfter || 60}s.`,
      action: 'Wait and try again, or upgrade provider account',
    },
    503: {
      title: 'Provider Unavailable',
      message: `${error.provider || 'Provider'} is temporarily unavailable.`,
      action: 'We will retry automatically. Please wait...',
    },
  };

  const config = errorMessages[error.status] || {
    title: 'Error Loading Diff',
    message: error.message,
    action: 'Please try refreshing the page',
  };

  return (
    <div className="diff-error">
      <AlertCircle className="icon-error" />
      <h3>{config.title}</h3>
      <p>{config.message}</p>
      {config.action && (
        <div className="error-actions">
          {config.actionLink ? (
            <Link to={config.actionLink}>
              <Button variant="outline">{config.action}</Button>
            </Link>
          ) : (
            <p className="text-muted">{config.action}</p>
          )}
        </div>
      )}
    </div>
  );
}
```

## Consequences

### Positive

- **Reliability**: System handles transient failures gracefully
- **User Experience**: Clear, actionable error messages
- **Observability**: Detailed metrics for error tracking
- **Resource Protection**: Circuit breaker prevents cascade failures
- **Resilience**: Stale cache fallback ensures some data availability

### Negative

- **Complexity**: More code to maintain (retry logic, circuit breaker)
- **Latency**: Retries add latency to failed requests (acceptable trade-off)
- **State Management**: Circuit breaker state requires synchronization (using `Arc<RwLock>`)

### Mitigation Strategies

1. **Complexity**: Thorough unit tests, integration tests for error paths
2. **Latency**: Set aggressive retry timeouts (max 5s total)
3. **Circuit Breaker Tuning**: Monitor false positives, adjust thresholds

## Monitoring & Alerts

```rust
// Metrics
counter!("ampel_provider_errors_total", "provider" => "github", "error_type" => "network");
histogram!("ampel_retry_duration_seconds");
gauge!("ampel_circuit_breaker_state", "provider" => "github"); // 0=closed, 1=open
counter!("ampel_stale_cache_served_total");

// Alerts
// - circuit_breaker_open_duration > 5 minutes
// - provider_error_rate > 10% for 5 minutes
// - stale_cache_served_count > 100/hour
```

## Related Decisions

- ADR-003: Caching Strategy
- ADR-002: Provider Diff Abstraction

## References

- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Exponential Backoff Best Practices](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
