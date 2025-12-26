use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

/// Cache TTL for dashboard summary (5 minutes in seconds)
pub const DASHBOARD_CACHE_TTL: u64 = 300;

/// Cache TTL for open PR diffs (5 minutes in seconds)
pub const OPEN_PR_DIFF_CACHE_TTL: u64 = 300;

/// Cache TTL for merged/closed PR diffs (1 hour in seconds)
pub const CLOSED_PR_DIFF_CACHE_TTL: u64 = 3600;

/// Redis connection manager type alias
pub type RedisConnectionManager = redis::aio::ConnectionManager;

/// Get dashboard cache key for a user
fn dashboard_cache_key(user_id: Uuid) -> String {
    format!("dashboard:summary:{}", user_id)
}

/// Get PR diff cache key
fn pr_diff_cache_key(repo_id: Uuid, pr_id: &str) -> String {
    format!("pr:diff:{}:{}", repo_id, pr_id)
}

/// Get cached dashboard summary for a user
///
/// Returns None if cache miss or error occurs (fail gracefully)
pub async fn get_dashboard_cache<T: DeserializeOwned>(
    redis: &mut RedisConnectionManager,
    user_id: Uuid,
) -> Option<T> {
    let key = dashboard_cache_key(user_id);

    match redis.get::<_, String>(&key).await {
        Ok(data) => match serde_json::from_str::<T>(&data) {
            Ok(summary) => {
                tracing::debug!(
                    user_id = %user_id,
                    key = %key,
                    "Dashboard cache hit"
                );
                Some(summary)
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    user_id = %user_id,
                    key = %key,
                    "Failed to deserialize cached dashboard data"
                );
                None
            }
        },
        Err(e) => {
            tracing::debug!(
                error = %e,
                user_id = %user_id,
                key = %key,
                "Dashboard cache miss"
            );
            None
        }
    }
}

/// Set dashboard cache for a user with TTL
///
/// Logs errors but doesn't fail if cache set fails (fail gracefully)
pub async fn set_dashboard_cache<T: Serialize>(
    redis: &mut RedisConnectionManager,
    user_id: Uuid,
    data: &T,
) {
    let key = dashboard_cache_key(user_id);

    match serde_json::to_string(data) {
        Ok(json_data) => {
            match redis
                .set_ex::<_, _, ()>(&key, json_data, DASHBOARD_CACHE_TTL)
                .await
            {
                Ok(_) => {
                    tracing::debug!(
                        user_id = %user_id,
                        key = %key,
                        ttl_seconds = DASHBOARD_CACHE_TTL,
                        "Dashboard cache set successfully"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        user_id = %user_id,
                        key = %key,
                        "Failed to set dashboard cache"
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                user_id = %user_id,
                "Failed to serialize dashboard data for cache"
            );
        }
    }
}

/// Get cached PR diff data
pub async fn get_pr_diff_cache<T: DeserializeOwned>(
    redis: &mut RedisConnectionManager,
    repo_id: Uuid,
    pr_id: &str,
) -> Option<T> {
    let key = pr_diff_cache_key(repo_id, pr_id);

    match redis.get::<_, String>(&key).await {
        Ok(data) => match serde_json::from_str::<T>(&data) {
            Ok(diff) => {
                tracing::debug!(
                    repo_id = %repo_id,
                    pr_id = %pr_id,
                    key = %key,
                    "PR diff cache hit"
                );
                Some(diff)
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    repo_id = %repo_id,
                    pr_id = %pr_id,
                    key = %key,
                    "Failed to deserialize cached PR diff data"
                );
                None
            }
        },
        Err(e) => {
            tracing::debug!(
                error = %e,
                repo_id = %repo_id,
                pr_id = %pr_id,
                key = %key,
                "PR diff cache miss"
            );
            None
        }
    }
}

/// Set PR diff cache with appropriate TTL based on PR state
pub async fn set_pr_diff_cache<T: Serialize>(
    redis: &mut RedisConnectionManager,
    repo_id: Uuid,
    pr_id: &str,
    data: &T,
    is_open: bool,
) {
    let key = pr_diff_cache_key(repo_id, pr_id);
    let ttl = if is_open {
        OPEN_PR_DIFF_CACHE_TTL
    } else {
        CLOSED_PR_DIFF_CACHE_TTL
    };

    match serde_json::to_string(data) {
        Ok(json_data) => match redis.set_ex::<_, _, ()>(&key, json_data, ttl).await {
            Ok(_) => {
                tracing::debug!(
                    repo_id = %repo_id,
                    pr_id = %pr_id,
                    key = %key,
                    ttl_seconds = ttl,
                    is_open = is_open,
                    "PR diff cache set successfully"
                );
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    repo_id = %repo_id,
                    pr_id = %pr_id,
                    key = %key,
                    "Failed to set PR diff cache"
                );
            }
        },
        Err(e) => {
            tracing::warn!(
                error = %e,
                repo_id = %repo_id,
                pr_id = %pr_id,
                "Failed to serialize PR diff data for cache"
            );
        }
    }
}

/// Invalidate PR diff cache (called on PR updates/webhooks)
pub async fn invalidate_pr_diff_cache(
    redis: &mut RedisConnectionManager,
    repo_id: Uuid,
    pr_id: &str,
) {
    let key = pr_diff_cache_key(repo_id, pr_id);

    match redis.del::<_, ()>(&key).await {
        Ok(_) => {
            tracing::debug!(
                repo_id = %repo_id,
                pr_id = %pr_id,
                key = %key,
                "PR diff cache invalidated"
            );
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                repo_id = %repo_id,
                pr_id = %pr_id,
                key = %key,
                "Failed to invalidate PR diff cache"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_cache_key_format() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = dashboard_cache_key(user_id);
        assert_eq!(
            key,
            "dashboard:summary:550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_dashboard_cache_ttl() {
        assert_eq!(DASHBOARD_CACHE_TTL, 300);
    }

    #[test]
    fn test_pr_diff_cache_key_format() {
        let repo_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = pr_diff_cache_key(repo_id, "123");
        assert_eq!(key, "pr:diff:550e8400-e29b-41d4-a716-446655440000:123");
    }

    #[test]
    fn test_pr_diff_cache_ttls() {
        assert_eq!(OPEN_PR_DIFF_CACHE_TTL, 300);
        assert_eq!(CLOSED_PR_DIFF_CACHE_TTL, 3600);
    }
}
