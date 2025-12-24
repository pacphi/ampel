use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

/// Cache TTL for dashboard summary (5 minutes in seconds)
pub const DASHBOARD_CACHE_TTL: u64 = 300;

/// Redis connection manager type alias
pub type RedisConnectionManager = redis::aio::ConnectionManager;

/// Get dashboard cache key for a user
fn dashboard_cache_key(user_id: Uuid) -> String {
    format!("dashboard:summary:{}", user_id)
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
}
