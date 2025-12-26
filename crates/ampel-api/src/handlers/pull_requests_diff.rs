use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use chrono::Utc;
use sea_orm::EntityTrait;
use uuid::Uuid;

use ampel_core::models::GitProvider;
use ampel_db::entities::provider_account;
use ampel_db::queries::{PrQueries, RepoQueries};
use ampel_providers::traits::ProviderCredentials;

use crate::extractors::AuthUser;
use crate::handlers::diff_types::{
    DiffApiResponse, DiffErrorResponse, DiffFile, DiffMetadata, DiffQuery,
    DiffResponse, FileStatus,
};
use crate::handlers::ApiError;
use crate::AppState;

/// Get pull request diff
///
/// Returns the unified diff for all files changed in a pull request.
/// Supports GitHub, GitLab, and Bitbucket providers with a unified response format.
///
/// # Caching
/// Diff responses are cached in Redis (if configured) for 5 minutes.
/// Use `Cache-Control: no-cache` header to bypass cache.
///
/// # Rate Limiting
/// Subject to per-user rate limits (100 requests/minute by default).
///
/// # Provider Notes
/// - **GitHub**: Returns all changed files without pagination
/// - **GitLab**: May paginate for >100 files (automatically handled)
/// - **Bitbucket**: Requires per-file diff fetching (may be slower)
#[utoipa::path(
    get,
    path = "/api/v1/pull-requests/{id}/diff",
    tag = "Pull Requests",
    params(
        ("id" = Uuid, Path, description = "Pull request UUID"),
        ("format" = Option<String>, Query, description = "Diff format: 'unified' or 'split' (default: unified)"),
        ("context" = Option<i32>, Query, description = "Number of context lines around changes (default: 3)")
    ),
    responses(
        (status = 200, description = "Diff retrieved successfully", body = DiffApiResponse,
            example = json!({
                "success": true,
                "data": {
                    "files": [{
                        "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
                        "old_path": "src/components/Button.tsx",
                        "new_path": "src/components/Button.tsx",
                        "status": "modified",
                        "additions": 15,
                        "deletions": 3,
                        "changes": 18,
                        "patch": "@@ -1,7 +1,19 @@\n import React from 'react';\n+import { cn } from '@/lib/utils';"
                    }],
                    "total_additions": 142,
                    "total_deletions": 38,
                    "total_files": 8,
                    "base_commit": "abc123...",
                    "head_commit": "def456..."
                },
                "metadata": {
                    "provider": "github",
                    "cached": true,
                    "cache_age_seconds": 120,
                    "timestamp": "2025-12-25T15:30:00Z"
                }
            })
        ),
        (status = 400, description = "Invalid request parameters", body = DiffErrorResponse,
            example = json!({
                "success": false,
                "error": {
                    "code": "INVALID_FORMAT",
                    "message": "Invalid format parameter. Must be 'unified' or 'split'",
                    "details": {
                        "field": "format",
                        "value": "invalid_value"
                    }
                }
            })
        ),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = DiffErrorResponse,
            example = json!({
                "success": false,
                "error": {
                    "code": "UNAUTHORIZED",
                    "message": "Invalid or expired authentication token",
                    "details": null
                }
            })
        ),
        (status = 404, description = "Pull request not found", body = DiffErrorResponse,
            example = json!({
                "success": false,
                "error": {
                    "code": "PR_NOT_FOUND",
                    "message": "Pull request not found or access denied",
                    "details": {
                        "pr_id": "550e8400-e29b-41d4-a716-446655440000"
                    }
                }
            })
        ),
        (status = 500, description = "Internal server error", body = DiffErrorResponse,
            example = json!({
                "success": false,
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": "Failed to fetch diff from provider",
                    "details": {
                        "provider": "github",
                        "error": "API connection timeout"
                    }
                }
            })
        ),
        (status = 503, description = "Provider API unavailable", body = DiffErrorResponse,
            example = json!({
                "success": false,
                "error": {
                    "code": "PROVIDER_UNAVAILABLE",
                    "message": "GitHub API is currently unavailable",
                    "details": {
                        "provider": "github",
                        "retry_after": 60,
                        "status": "rate_limited"
                    }
                }
            })
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_pull_request_diff(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(pr_id): Path<Uuid>,
    Query(query): Query<DiffQuery>,
    headers: HeaderMap,
) -> Result<Json<DiffApiResponse>, ApiError> {
    // Validate query parameters
    if query.format != "unified" && query.format != "split" {
        return Err(ApiError::bad_request(
            "Invalid format parameter. Must be 'unified' or 'split'",
        ));
    }

    if query.context < 0 || query.context > 20 {
        return Err(ApiError::bad_request(
            "Context must be between 0 and 20 lines",
        ));
    }

    // Check cache bypass
    let bypass_cache = headers
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("no-cache"))
        .unwrap_or(false);

    // Try to get from cache first (if not bypassing)
    let cache_key = format!("diff:pr:{}:{}:{}", pr_id, query.format, query.context);

    if !bypass_cache {
        if let Some(redis) = &state.redis {
            if let Ok(Some(cached)) = redis::cmd("GET")
                .arg(&cache_key)
                .query_async::<Option<String>>(&mut redis.clone())
                .await
            {
                if let Ok(response) = serde_json::from_str::<DiffApiResponse>(&cached) {
                    return Ok(Json(response));
                }
            }
        }
    }

    // Fetch PR and verify ownership
    let pr = PrQueries::find_by_id(&state.db, pr_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Pull request not found"))?;

    let repo = RepoQueries::find_by_id(&state.db, pr.repository_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;

    // Verify user owns the repository
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Pull request not found"));
    }

    // Get provider
    let provider_type: GitProvider = repo
        .provider
        .parse()
        .map_err(|_| ApiError::internal("Invalid provider"))?;

    // Get provider account
    let account = provider_account::Entity::find_by_id(
        repo.provider_account_id
            .ok_or(ApiError::bad_request("Repository not linked to account"))?,
    )
    .one(&state.db)
    .await?
    .ok_or(ApiError::not_found("Provider account not found"))?;

    // Decrypt access token
    let access_token = state
        .encryption_service
        .decrypt(&account.access_token_encrypted)
        .map_err(|e| ApiError::internal(format!("Failed to decrypt token: {}", e)))?;

    // Create credentials
    let credentials = ProviderCredentials::Pat {
        token: access_token,
        username: account.auth_username.clone(),
    };

    // Create provider instance
    let provider = state
        .provider_factory
        .create(provider_type, account.instance_url.clone());

    // Fetch diff from provider
    let provider_diff = provider
        .get_pull_request_diff(&credentials, &repo.owner, &repo.name, pr.number)
        .await
        .map_err(|e| {
            ApiError::internal(format!("Failed to fetch diff from provider: {}", e))
        })?;

    // Convert to API response format
    let files: Vec<DiffFile> = provider_diff
        .files
        .into_iter()
        .map(|f| DiffFile {
            sha: f.sha,
            old_path: f.previous_filename.clone(),
            new_path: f.filename.clone(),
            status: match f.status.as_str() {
                "added" => FileStatus::Added,
                "deleted" => FileStatus::Deleted,
                "modified" => FileStatus::Modified,
                "renamed" => FileStatus::Renamed,
                "copied" => FileStatus::Copied,
                _ => FileStatus::Modified,
            },
            additions: f.additions,
            deletions: f.deletions,
            changes: f.changes,
            patch: f.patch.unwrap_or_default(),
        })
        .collect();

    let response = DiffApiResponse {
        success: true,
        data: DiffResponse {
            files,
            total_additions: provider_diff.total_additions,
            total_deletions: provider_diff.total_deletions,
            total_files: provider_diff.total_files,
            base_commit: provider_diff.base_commit,
            head_commit: provider_diff.head_commit,
        },
        metadata: DiffMetadata {
            provider: provider_type.to_string().to_lowercase(),
            cached: false,
            cache_age_seconds: 0,
            timestamp: Utc::now().to_rfc3339(),
        },
    };

    // Cache the response
    if let Some(redis) = &state.redis {
        let _ = redis::cmd("SETEX")
            .arg(&cache_key)
            .arg(300) // 5 minutes TTL
            .arg(serde_json::to_string(&response).unwrap_or_default())
            .query_async::<()>(&mut redis.clone())
            .await;
    }

    Ok(Json(response))
}
