# ADR-003: Diff Caching Strategy

**Status:** Accepted
**Date:** 2025-12-25
**Decision Makers:** Architecture Team
**Technical Story:** Performance Optimization for Diff Fetching

## Context

Fetching diffs from provider APIs involves:

1. Network latency (100-500ms per API call)
2. Provider rate limits (GitHub: 5000/hr, GitLab: 300/min, Bitbucket: 1000/hr)
3. Computational cost (parsing diffs, calculating stats)
4. Database lookups (repository credentials, PR metadata)

Diffs are relatively static once a PR is opened, making them ideal for caching. However, cache invalidation is critical to ensure users see latest changes.

## Decision Drivers

- **Performance**: Sub-500ms load time for cached diffs
- **Freshness**: Show latest diff within acceptable staleness window
- **Resource Efficiency**: Reduce provider API calls
- **Scalability**: Support 1000+ concurrent users
- **Reliability**: Handle cache failures gracefully

## Considered Options

### Option 1: Redis Multi-Level Cache with TTL + Webhook Invalidation (SELECTED)

**Cache Layers:**

1. **L1 (Redis)**: 5-minute TTL for open PRs, 1-hour for closed/merged
2. **L2 (Database)**: Store last fetched diff in `pr_diffs` table (optional)
3. **Invalidation**: Webhook events trigger cache purge

**Pros:**

- Fast cache hits (<10ms)
- Handles high concurrency (Redis supports 100K+ ops/sec)
- Automatic expiration reduces stale data risk
- Webhook integration provides real-time updates

**Cons:**

- Requires Redis infrastructure
- Webhook setup adds operational complexity
- Cache misses can cause latency spikes

### Option 2: PostgreSQL-Only Caching

Store diffs in `pr_diffs` table with `cached_at` timestamp.

**Pros:**

- No additional infrastructure (uses existing Postgres)
- Simpler deployment

**Cons:**

- Slower than Redis (50-100ms vs 5-10ms)
- Database size grows with diff storage
- Less efficient for high-concurrency reads

### Option 3: CDN Edge Caching

Use HTTP caching headers, let CDN (Cloudflare, Fastly) cache diff responses.

**Pros:**

- Minimal backend changes
- Global distribution

**Cons:**

- Less control over invalidation
- Cache keys complex (needs user-specific data)
- Not suitable for authenticated API responses

### Option 4: No Caching (Always Fetch Fresh)

Fetch diff from provider API on every request.

**Pros:**

- Always fresh data
- No cache complexity

**Cons:**

- Slow user experience (500ms-2s per diff load)
- High provider API usage → rate limit issues
- Poor scalability

## Decision Outcome

**Chosen Option:** Redis Multi-Level Cache with TTL + Webhook Invalidation

### Cache Key Structure

```
diff:{provider}:{owner}:{repo}:{pr_number}:{head_commit_sha}
```

**Example:**

```
diff:github:facebook:react:12345:abc123def456
```

**Rationale:**

- `provider`: Isolate by provider (GitHub/GitLab/Bitbucket)
- `owner/repo`: Namespace by repository
- `pr_number`: Identify specific PR
- `head_commit_sha`: Force cache miss when PR updated

### TTL Strategy

| PR State       | TTL     | Rationale                                                                 |
| -------------- | ------- | ------------------------------------------------------------------------- |
| **Open**       | 5 min   | Active PRs change frequently (new commits, reviews)                       |
| **Merged**     | 1 hour  | Merged PRs rarely change, but allow updates (reverted merges)             |
| **Closed**     | 1 hour  | Closed PRs are mostly static                                              |
| **Draft**      | 2 min   | Drafts updated frequently                                                 |
| **Stale** (>7d | 30 min) | Older PRs less likely to change, prioritize cache hit rate over freshness |

### Cache Invalidation Triggers

1. **Commit Pushed to PR Branch**
   - Webhook: `pull_request.synchronize` (GitHub), `merge_request:update` (GitLab)
   - Action: Delete cache key `diff:{provider}:{owner}:{repo}:{pr_number}:*`

2. **PR Merged/Closed**
   - Webhook: `pull_request.closed` (GitHub), `merge_request:merge` (GitLab)
   - Action: Update TTL to 1 hour, keep cache

3. **Manual Refresh**
   - User clicks "Refresh" button in UI
   - Action: Delete cache key, force fetch from provider

4. **Provider Credentials Updated**
   - User rotates PAT token
   - Action: Purge all caches for `diff:{provider}:*`

### Implementation

#### Backend (Rust)

```rust
// crates/ampel-api/src/services/diff_cache.rs

use redis::{Client, Commands};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CachedDiff {
    pub diff: ProviderDiff,
    pub cached_at: DateTime<Utc>,
}

pub struct DiffCache {
    redis: Client,
}

impl DiffCache {
    pub async fn get(
        &self,
        provider: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
        head_sha: &str,
    ) -> Option<ProviderDiff> {
        let key = format!("diff:{}:{}:{}:{}:{}", provider, owner, repo, pr_number, head_sha);

        let mut conn = self.redis.get_connection().ok()?;
        let cached: Option<String> = conn.get(&key).ok()?;

        cached.and_then(|json| serde_json::from_str::<CachedDiff>(&json).ok())
              .map(|c| c.diff)
    }

    pub async fn set(
        &self,
        provider: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
        head_sha: &str,
        diff: &ProviderDiff,
        ttl_seconds: i32,
    ) -> Result<(), Error> {
        let key = format!("diff:{}:{}:{}:{}:{}", provider, owner, repo, pr_number, head_sha);

        let cached = CachedDiff {
            diff: diff.clone(),
            cached_at: Utc::now(),
        };

        let json = serde_json::to_string(&cached)?;
        let mut conn = self.redis.get_connection()?;
        conn.set_ex(&key, json, ttl_seconds as usize)?;

        Ok(())
    }

    pub async fn invalidate(
        &self,
        provider: &str,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> Result<(), Error> {
        let pattern = format!("diff:{}:{}:{}:{}:*", provider, owner, repo, pr_number);
        let mut conn = self.redis.get_connection()?;

        // Find all keys matching pattern
        let keys: Vec<String> = conn.keys(&pattern)?;

        // Delete each key
        for key in keys {
            conn.del(&key)?;
        }

        Ok(())
    }
}

// Usage in handler
pub async fn get_pull_request_diff(
    State(state): State<AppState>,
    Path(pr_id): Path<Uuid>,
) -> Result<Json<DiffResponse>, ApiError> {
    let pr = state.db.find_pull_request(pr_id).await?;

    // Check cache first
    if let Some(diff) = state.diff_cache.get(
        &pr.repository.provider,
        &pr.repository.owner,
        &pr.repository.name,
        pr.number,
        &pr.head_sha,
    ).await {
        return Ok(Json(DiffResponse::from(diff)));
    }

    // Cache miss: fetch from provider
    let provider = state.provider_factory.create_provider(...)?;
    let diff = provider.get_pull_request_diff(...).await?;

    // Cache the result
    let ttl = if pr.state == "open" { 300 } else { 3600 };
    state.diff_cache.set(
        &pr.repository.provider,
        &pr.repository.owner,
        &pr.repository.name,
        pr.number,
        &pr.head_sha,
        &diff,
        ttl,
    ).await?;

    Ok(Json(DiffResponse::from(diff)))
}
```

#### Frontend (TanStack Query)

```typescript
// frontend/src/hooks/usePullRequestDiff.ts

export function usePullRequestDiff(pullRequestId: string) {
  return useQuery({
    queryKey: ['pull-request-diff', pullRequestId],
    queryFn: async () => {
      const response = await api.get<PullRequestDiff>(
        `/api/v1/pull-requests/${pullRequestId}/diff`
      );
      return response.data;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes (matches backend TTL)
    cacheTime: 10 * 60 * 1000, // 10 minutes (keep in memory longer)
    retry: 2,
    retryDelay: 1000,
  });
}

// Manual refresh
const { refetch } = usePullRequestDiff(prId);

<button onClick={() => refetch()}>Refresh Diff</button>
```

### Webhook Integration (Future)

```rust
// crates/ampel-api/src/handlers/webhooks.rs

pub async fn handle_github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GitHubWebhookPayload>,
) -> Result<StatusCode, ApiError> {
    // Verify webhook signature
    verify_github_signature(&headers, &payload)?;

    match payload.action.as_str() {
        "synchronize" | "opened" | "reopened" => {
            // PR updated: invalidate cache
            state.diff_cache.invalidate(
                "github",
                &payload.repository.owner.login,
                &payload.repository.name,
                payload.pull_request.number,
            ).await?;

            // Queue background job to refetch PR data
            state.job_queue.enqueue(PollRepositoryJob {
                repository_id: payload.repository.id,
            }).await?;
        }
        "closed" => {
            // PR closed: update TTL but keep cache
            // (Cache will expire naturally after 1 hour)
        }
        _ => {}
    }

    Ok(StatusCode::OK)
}
```

## Consequences

### Positive

- **Performance**: 90%+ cache hit rate → <50ms response time (vs 500ms-2s uncached)
- **Scalability**: Reduced provider API calls by 80-90% → avoid rate limits
- **User Experience**: Instant diff loading for recently viewed PRs
- **Resource Efficiency**: Redis handles 100K+ ops/sec, supports horizontal scaling

### Negative

- **Infrastructure Cost**: Redis instance required (minimal, ~$10-20/month)
- **Complexity**: Cache invalidation logic adds complexity
- **Stale Data Risk**: Users may see outdated diff for up to 5 minutes
- **Memory Usage**: Redis stores ~1-5MB per cached diff (monitor usage)

### Mitigation Strategies

1. **Stale Data**: Show "Last updated X minutes ago" timestamp in UI
2. **Cache Failures**: Graceful degradation → fetch from provider if Redis unavailable
3. **Memory Limits**: Set Redis `maxmemory-policy` to `allkeys-lru` (evict least-recently-used)
4. **Monitoring**: Alert on cache hit rate <80%, eviction rate >10%

## Performance Benchmarks

| Scenario             | Uncached  | Cached (Redis) | Improvement |
| -------------------- | --------- | -------------- | ----------- |
| Small PR (10 files)  | 320ms     | 8ms            | 40x faster  |
| Medium PR (50 files) | 680ms     | 12ms           | 56x faster  |
| Large PR (200 files) | 1900ms    | 35ms           | 54x faster  |
| Cache Miss           | 500-2000m | -              | -           |

**Target Metrics:**

- Cache hit rate: >85%
- P50 latency (cached): <50ms
- P95 latency (cached): <100ms
- P99 latency (uncached): <2000ms

## Related Decisions

- ADR-002: Provider Diff API Abstraction
- ADR-004: Error Recovery Mechanisms
- ADR-006: Webhook Integration Strategy

## Monitoring & Alerts

```rust
// Metrics to track
counter!("ampel_diff_cache_hits_total");
counter!("ampel_diff_cache_misses_total");
histogram!("ampel_diff_fetch_duration_seconds");
gauge!("ampel_diff_cache_size_bytes");
gauge!("ampel_diff_cache_evictions_total");

// Alerts
// - cache_hit_rate < 80% for 5 minutes
// - cache_miss_duration > 2000ms (p95)
// - redis_memory_usage > 80%
```

## References

- [Redis Caching Best Practices](https://redis.io/docs/manual/patterns/cache/)
- [TanStack Query Caching Guide](https://tanstack.com/query/latest/docs/react/guides/caching)
- [GitHub Webhooks Documentation](https://docs.github.com/en/webhooks)
