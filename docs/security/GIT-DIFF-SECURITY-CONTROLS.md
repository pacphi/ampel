# Git Diff View Security Controls

This document describes the comprehensive security controls implemented for the Git diff viewing feature to ensure safe production deployment.

## Overview

The Git diff feature displays user-generated code from external Git providers (GitHub, GitLab, Bitbucket). This presents several security risks that have been mitigated through defense-in-depth controls.

## Security Controls Implemented

### 1. Rate Limiting (MANDATORY) ✅

**Priority**: Critical
**Status**: Implemented

#### Implementation

- **Redis-based distributed rate limiting** for multi-instance deployments
- **Location**: `crates/ampel-api/src/middleware/rate_limit.rs`
- **Limits**:
  - 100 requests per hour per user
  - 20 requests per second burst allowance
  - Sliding window algorithm for accuracy

#### Configuration

```rust
// Middleware applied to diff endpoint
.route(
    "/api/v1/pull-requests/:id/diff",
    get(pull_requests_diff::get_pull_request_diff)
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_diff))
)
```

#### Response Headers

When rate limiting is active:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 3540
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1735142400
```

#### Error Response

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Please try again later.",
    "details": {
      "limit": 100,
      "window": "1 hour",
      "retry_after_seconds": 3540
    }
  }
}
```

#### Testing

```bash
# Run rate limiting tests
cargo test --package ampel-api rate_limit --all-features

# Manual testing with curl
for i in {1..101}; do
  curl -H "Authorization: Bearer $TOKEN" \
       http://localhost:8080/api/v1/pull-requests/$PR_ID/diff
done
```

### 2. XSS Prevention (MANDATORY) ✅

**Priority**: Critical
**Status**: Implemented with comprehensive test coverage

#### Attack Vectors Mitigated

1. **Script tag injection** in file paths and diff content
2. **Event handler injection** (`onclick`, `onerror`, `onload`)
3. **HTML entity injection** in code blocks
4. **URL injection** (`javascript:`, `data:` schemes)
5. **Malicious metadata** in language tags and file modes

#### Implementation

**DOMPurify Sanitization** (`frontend/src/utils/sanitization.ts`):

```typescript
// Sanitize file paths (user-controlled via repository names)
export function sanitizeFilePath(filePath: string): string {
  return DOMPurify.sanitize(filePath, {
    ALLOWED_TAGS: [], // No HTML tags in file paths
    ALLOWED_ATTR: [],
    KEEP_CONTENT: true,
  });
}

// Sanitize metadata fields
export function sanitizeMetadata(value: string): string {
  return DOMPurify.sanitize(value, {
    ALLOWED_TAGS: [],
    ALLOWED_ATTR: [],
    KEEP_CONTENT: true,
  });
}
```

**React Automatic Escaping**:

- Code content rendered via JSX automatically escapes HTML
- `@git-diff-view/react` library does NOT use `dangerouslySetInnerHTML`
- Verified through automated tests

#### Test Coverage

**Location**: `frontend/src/components/diff/__tests__/xss-prevention.test.tsx`

**Test Cases** (12 comprehensive tests):

1. Script tag injection in file paths
2. Script tags in diff content
3. `onclick` handler injection
4. `onerror` handler injection
5. `onload` handler injection
6. HTML entity handling
7. Mixed HTML entities and special characters
8. React escaping verification
9. `dangerouslySetInnerHTML` detection
10. `@git-diff-view/react` library safety
11. Multi-file diff XSS protection
12. `javascript:` and `data:` URL prevention

#### Running XSS Tests

```bash
cd frontend
pnpm test -- xss-prevention.test.tsx --run
```

**Expected Output**:

```
✓ frontend/src/components/diff/__tests__/xss-prevention.test.tsx (12)
  ✓ DiffViewer - XSS Prevention (12)
    ✓ Script Tag Injection (2)
    ✓ Event Handler Injection (2)
    ✓ HTML Entity Injection (2)
    ✓ React Escaping Verification (2)
    ✓ @git-diff-view/react Library Safety (1)
    ✓ Multi-File Diff XSS Prevention (1)
    ✓ URL Injection Protection (2)

Test Files  1 passed (1)
Tests  12 passed (12)
```

### 3. Large Diff Protection (MANDATORY) ✅

**Priority**: High
**Status**: Implemented

#### Implementation

**Limit**: 5,000 lines per file maximum

**Location**: `frontend/src/components/diff/DiffViewer.tsx`

```typescript
const MAX_LINES_PER_FILE = 5000;

function countLinesInFile(file: DiffFile): number {
  return file.chunks.reduce((total, chunk) => {
    return total + (chunk.changes?.length || 0);
  }, 0);
}
```

#### Fallback UI

When a file exceeds the limit:

```tsx
<Card className="p-8">
  <AlertTriangle className="h-16 w-16 text-yellow-600" />
  <h3>Diff Too Large to Display</h3>
  <p>
    This file has {totalLines.toLocaleString()} lines, exceeding the limit of{' '}
    {MAX_LINES_PER_FILE.toLocaleString()}.
  </p>
  <a href={externalDiffUrl} target="_blank">
    View on Provider <ExternalLink />
  </a>
</Card>
```

#### Rationale

1. **Browser performance**: Rendering 10,000+ line diffs can freeze the browser
2. **Memory exhaustion**: Large diffs consume significant client memory
3. **Security**: Prevents DoS via maliciously large diffs
4. **User experience**: Provides clear alternative via "View on Provider" link

### 4. Redis Caching with TTL

**Status**: Already implemented
**Location**: `crates/ampel-api/src/handlers/pull_requests_diff.rs`

- **Cache Key**: `diff:pr:{pr_id}:{format}:{context}`
- **TTL**: 5 minutes
- **Bypass**: `Cache-Control: no-cache` header

Reduces provider API calls and improves response time while preventing stale data.

## Security Testing Checklist

Before production deployment:

- [x] Rate limiting returns 429 after quota exceeded
- [x] All XSS tests pass (12/12)
- [x] Large diff fallback displays correctly
- [x] DOMPurify installed and configured
- [x] Redis rate limiting functional
- [x] Retry-After header present in 429 responses
- [x] No script execution in diff content
- [x] No `dangerouslySetInnerHTML` usage
- [x] File paths sanitized
- [x] External links use `rel="noopener noreferrer"`

## Production Configuration

### Environment Variables

```bash
# Redis (required for rate limiting)
REDIS_URL=redis://localhost:6379

# Rate limiting (optional overrides)
DIFF_RATE_LIMIT_REQUESTS_PER_HOUR=100
DIFF_RATE_LIMIT_BURST_ALLOWANCE=20
```

### Monitoring

Monitor these metrics in production:

1. **Rate limit hits**: Track 429 responses
2. **Oversized diffs**: Count files exceeding 5,000 lines
3. **Cache hit rate**: Redis cache effectiveness
4. **Response times**: Diff endpoint latency

### Alerts

Set up alerts for:

- High rate of 429 responses (potential attack)
- Redis connection failures
- Spike in oversized diff fallbacks
- XSS attempt patterns in logs

## Threat Model

### Mitigated Threats

| Threat                          | Mitigation                          | Severity |
| ------------------------------- | ----------------------------------- | -------- |
| XSS via malicious file paths    | DOMPurify sanitization              | Critical |
| XSS via malicious diff content  | React automatic escaping            | Critical |
| DoS via excessive requests      | Redis-based rate limiting           | High     |
| DoS via large diffs             | 5,000 line limit with fallback      | High     |
| Session fixation                | JWT with short expiry               | Medium   |
| Provider API exhaustion         | Rate limiting + Redis caching       | Medium   |
| Memory exhaustion (client-side) | Large diff protection               | Medium   |
| Clickjacking                    | CSP headers (existing nginx config) | Low      |
| CSRF                            | JWT auth (no cookies for state)     | Low      |

### Residual Risks

1. **Provider API failures**: Handled with graceful error messages
2. **Redis failures**: Requests allowed to prevent service disruption
3. **Malicious repository content**: Users only see their own repositories

## Incident Response

If XSS vulnerability discovered:

1. **Immediate**: Deploy hotfix to sanitize affected fields
2. **Short-term**: Add test case to prevent regression
3. **Long-term**: Review all user-generated content rendering

If DoS attack detected:

1. **Immediate**: Reduce rate limits via environment variables
2. **Short-term**: Identify attacking IPs and block at nginx level
3. **Long-term**: Implement CAPTCHA for suspicious patterns

## References

- [OWASP XSS Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
- [DOMPurify Documentation](https://github.com/cure53/DOMPurify)
- [React Security Best Practices](https://reactjs.org/docs/dom-elements.html#dangerouslysetinnerhtml)
- [Redis Rate Limiting Patterns](https://redis.io/docs/manual/patterns/rate-limiter/)

## Maintenance

This security control documentation should be reviewed:

- Before each production release
- After any changes to diff rendering
- Quarterly as part of security review
- After any security incidents

**Last Updated**: 2025-12-25
**Next Review**: 2026-03-25
