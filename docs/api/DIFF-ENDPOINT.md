# Git Diff API Endpoint

## Overview

The Git Diff API provides unified access to pull request file changes across GitHub, GitLab, and Bitbucket. It returns a standardized diff format with syntax-highlighted patches, making it easy to integrate with frontend diff viewers like `@git-diff-view/react`.

## Endpoint

```
GET /api/v1/pull-requests/:id/diff
```

## Authentication

All requests require Bearer token authentication:

```bash
Authorization: Bearer <your-jwt-token>
```

## Request Parameters

### Path Parameters

| Parameter | Type | Required | Description                  |
| --------- | ---- | -------- | ---------------------------- |
| `id`      | UUID | Yes      | Pull request UUID from Ampel |

### Query Parameters

| Parameter | Type   | Required | Default   | Description                            |
| --------- | ------ | -------- | --------- | -------------------------------------- |
| `format`  | string | No       | `unified` | Diff format: `unified` or `split`      |
| `context` | number | No       | `3`       | Number of context lines around changes |

## Response

### Success Response (200 OK)

```json
{
  "success": true,
  "data": {
    "files": [
      {
        "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
        "old_path": "src/components/Button.tsx",
        "new_path": "src/components/Button.tsx",
        "status": "modified",
        "additions": 15,
        "deletions": 3,
        "changes": 18,
        "patch": "@@ -1,7 +1,19 @@\n import React from 'react';\n+import { cn } from '@/lib/utils';\n ..."
      }
    ],
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
}
```

### File Status Enum

| Status     | Description                               |
| ---------- | ----------------------------------------- |
| `added`    | New file created                          |
| `deleted`  | File removed                              |
| `modified` | File content changed                      |
| `renamed`  | File moved/renamed (old_path != new_path) |
| `copied`   | File copied from another location         |

## Error Responses

### 400 Bad Request

Invalid request parameters:

```json
{
  "success": false,
  "error": {
    "code": "INVALID_FORMAT",
    "message": "Invalid format parameter. Must be 'unified' or 'split'",
    "details": {
      "field": "format",
      "value": "invalid_value"
    }
  }
}
```

### 401 Unauthorized

Missing or invalid authentication:

```json
{
  "success": false,
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid or expired authentication token",
    "details": null
  }
}
```

### 404 Not Found

Pull request not found or user has no access:

```json
{
  "success": false,
  "error": {
    "code": "PR_NOT_FOUND",
    "message": "Pull request not found or access denied",
    "details": {
      "pr_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

### 500 Internal Server Error

Server-side error:

```json
{
  "success": false,
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "Failed to fetch diff from provider",
    "details": {
      "provider": "github",
      "error": "API connection timeout"
    }
  }
}
```

### 503 Service Unavailable

Provider API unavailable or rate limited:

```json
{
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
}
```

## Usage Examples

### Basic Request

```bash
curl -X GET \
  'http://localhost:8080/api/v1/pull-requests/550e8400-e29b-41d4-a716-446655440000/diff' \
  -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIs...'
```

### Split View with More Context

```bash
curl -X GET \
  'http://localhost:8080/api/v1/pull-requests/550e8400-e29b-41d4-a716-446655440000/diff?format=split&context=5' \
  -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIs...'
```

### TypeScript/JavaScript (Fetch API)

```typescript
async function fetchPullRequestDiff(prId: string): Promise<DiffResponse> {
  const response = await fetch(`http://localhost:8080/api/v1/pull-requests/${prId}/diff`, {
    headers: {
      Authorization: `Bearer ${localStorage.getItem('access_token')}`,
    },
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${await response.text()}`);
  }

  return response.json();
}
```

### Axios

```typescript
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:8080/api/v1',
  headers: {
    Authorization: `Bearer ${localStorage.getItem('access_token')}`,
  },
});

// Fetch diff
const diff = await api.get(`/pull-requests/${prId}/diff`, {
  params: {
    format: 'unified',
    context: 3,
  },
});
```

## Provider-Specific Behavior

### GitHub

- **API Endpoint**: `GET /repos/{owner}/{repo}/pulls/{number}/files`
- **Patch Format**: Unified diff in `patch` field
- **Rate Limits**: 5,000 requests/hour (authenticated)
- **Max Files**: Returns all changed files (no pagination limit in practice)
- **Caching**: Recommended 5-minute cache TTL

**GitHub-Specific Notes:**

- Binary files return `patch: null`
- Large diffs may be truncated (check `status: "renamed"` field)
- Submodule changes show as modified with special patch format

### GitLab

- **API Endpoint**: `GET /projects/{id}/merge_requests/{iid}/changes`
- **Patch Format**: Unified diff in `diff` field
- **Rate Limits**: 10 requests/second (recommended)
- **Max Files**: 100 files per request (paginated via `page` param)
- **Caching**: Recommended 5-minute cache TTL

**GitLab-Specific Notes:**

- Renamed files include `renamed_file: true` flag
- `old_path` and `new_path` differ for renames
- Binary files return empty `diff` string

### Bitbucket Cloud

- **API Endpoint**: `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}/diffstat`
- **Patch Format**: Must fetch per-file via `/diff/{path}`
- **Rate Limits**: 1,000 requests/hour
- **Max Files**: 50 files per diffstat response
- **Caching**: Recommended 10-minute cache TTL

**Bitbucket-Specific Notes:**

- Requires two API calls: diffstat + per-file diff
- Binary files return `"binary": true` in diffstat
- Renames detected via `old_path != new_path` heuristic

### Bitbucket Server

- **API Endpoint**: `GET /rest/api/1.0/projects/{project}/repos/{repo}/pull-requests/{id}/changes`
- **Patch Format**: Server returns hunks array (requires assembly)
- **Rate Limits**: Depends on server configuration
- **Max Files**: 500 files (configurable)
- **Caching**: Recommended 10-minute cache TTL

**Server-Specific Notes:**

- Requires pagination via `start` parameter
- Diff hunks need assembly into unified format
- Large diffs may trigger server-side truncation

## Caching Strategy

### Redis Caching (Production)

When Redis is configured, diffs are cached with the following strategy:

```
Key Format: diff:pr:{pr_id}:{format}:{context}
TTL: 300 seconds (5 minutes)
```

**Cache Invalidation Events:**

- New commit pushed to PR branch
- PR merged or closed
- Manual refresh via `/refresh` endpoint

### Cache Headers

```http
X-Cache-Status: HIT | MISS | BYPASS
X-Cache-Age: 120
Cache-Control: private, max-age=300
```

### Bypass Cache

Force fresh data from provider:

```bash
curl -X GET \
  'http://localhost:8080/api/v1/pull-requests/{id}/diff' \
  -H 'Authorization: Bearer ...' \
  -H 'Cache-Control: no-cache'
```

## Rate Limiting

API implements per-user rate limiting:

- **Default**: 100 requests/minute per user
- **Burst**: 20 requests/second
- **Headers**:
  ```http
  X-RateLimit-Limit: 100
  X-RateLimit-Remaining: 87
  X-RateLimit-Reset: 1735142400
  ```

**Rate Limit Exceeded (429):**

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Retry after 45 seconds",
    "details": {
      "limit": 100,
      "retry_after": 45
    }
  }
}
```

## Performance Considerations

### Large Diffs

For PRs with 500+ files:

1. **Pagination**: Consider implementing client-side pagination
2. **Lazy Loading**: Load diffs on-demand per file
3. **Virtual Scrolling**: Use `@git-diff-view/react` virtual scrolling
4. **Streaming**: Future improvement for massive diffs (>1000 files)

### Timeouts

- **Default Timeout**: 30 seconds
- **Provider Timeout**: 15 seconds per provider API call
- **Recommended**: Show loading state after 2 seconds

## Troubleshooting

### Issue: "Provider API timeout"

**Cause**: Provider API took >15 seconds to respond

**Solutions:**

1. Check provider API status page
2. Retry request (may be cached on retry)
3. Reduce `context` parameter to speed up response

### Issue: "Binary file diff truncated"

**Cause**: Binary files cannot be displayed as text diffs

**Solutions:**

1. Check `patch` field for `null` value
2. Display "Binary file changed" message
3. Provide download link to view file

### Issue: "Diff missing files"

**Cause**: Provider pagination limit reached

**Solutions:**

1. Check `total_files` vs `files.length`
2. Implement pagination for GitLab (>100 files)
3. Show "Showing X of Y files" message

### Issue: "Cache stale after new commit"

**Cause**: Cache not invalidated on PR update

**Solutions:**

1. Use `/refresh` endpoint to force update
2. Implement webhook-based cache invalidation
3. Reduce cache TTL for active PRs

## Frontend Integration

### React Example with @git-diff-view/react

```tsx
import { DiffView } from '@git-diff-view/react';
import '@git-diff-view/react/styles/diff-view.css';

function PullRequestDiff({ prId }: { prId: string }) {
  const { data, isLoading, error } = useQuery({
    queryKey: ['pr-diff', prId],
    queryFn: () => fetchPullRequestDiff(prId),
  });

  if (isLoading) return <LoadingSpinner />;
  if (error) return <ErrorAlert error={error} />;

  return (
    <div className="diff-container">
      {data.data.files.map((file) => (
        <DiffView
          key={file.sha}
          data={file.patch}
          viewType="unified"
          language={getLanguageFromPath(file.new_path)}
        />
      ))}
    </div>
  );
}
```

## Security Considerations

1. **Authentication**: Always validate JWT token before serving diffs
2. **Authorization**: Verify user owns the repository before returning diff
3. **Rate Limiting**: Prevent abuse via per-user quotas
4. **Token Encryption**: Provider PATs stored encrypted (AES-256-GCM)
5. **No PII in Diffs**: Sanitize commit messages with email addresses

## API Versioning

Current version: `v1`

**Breaking Changes:**

- URL path includes `/v1/` prefix
- New versions will use `/v2/`, `/v3/`, etc.
- Deprecation notices sent 6 months before removal

## Support

- **Documentation**: [https://docs.ampel.dev](https://docs.ampel.dev)
- **API Status**: [https://status.ampel.dev](https://status.ampel.dev)
- **Issues**: [https://github.com/ampel/ampel/issues](https://github.com/ampel/ampel/issues)
- **Contact**: support@ampel.dev
