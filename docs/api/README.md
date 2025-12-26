# Ampel API Documentation

## Overview

Comprehensive API documentation for the Ampel unified PR management dashboard. This documentation covers all REST endpoints, authentication, error handling, and provider-specific behavior.

## Quick Links

- **[Diff Endpoint Guide](./DIFF-ENDPOINT.md)** - Complete documentation for the git diff API
- **[Swagger UI](http://localhost:8080/api/docs)** - Interactive API documentation (when server is running)
- **[OpenAPI Spec](http://localhost:8080/api/openapi.json)** - Machine-readable API specification

## API Documentation Structure

### Core Documentation

- **[DIFF-ENDPOINT.md](./DIFF-ENDPOINT.md)** - Git diff endpoint comprehensive guide
  - Request/response formats
  - Provider-specific behavior (GitHub, GitLab, Bitbucket)
  - Caching strategy
  - Rate limiting
  - Error handling
  - Troubleshooting

### Example Code

Located in `docs/api/examples/`:

- **[diff-request.sh](./examples/diff-request.sh)** - Bash/curl examples for all use cases
- **[diff-typescript-client.ts](./examples/diff-typescript-client.ts)** - TypeScript client implementation
- **[diff-response-github.json](./examples/diff-response-github.json)** - GitHub provider response example
- **[diff-response-gitlab.json](./examples/diff-response-gitlab.json)** - GitLab provider response example
- **[diff-response-bitbucket.json](./examples/diff-response-bitbucket.json)** - Bitbucket provider response example

## Getting Started

### 1. Authentication

All API requests require JWT authentication:

```bash
# Login to get token
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "your-password"}'

# Response includes access_token
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "...",
    "expires_in": 900
  }
}
```

### 2. Using the API

Include the token in the Authorization header:

```bash
curl -X GET http://localhost:8080/api/v1/pull-requests/{id}/diff \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..."
```

### 3. Interactive Documentation

When the API server is running, visit:

- **Swagger UI**: http://localhost:8080/api/docs
- **OpenAPI JSON**: http://localhost:8080/api/openapi.json

## API Endpoints

### Pull Request Diff

```
GET /api/v1/pull-requests/:id/diff
GET /api/repositories/:repo_id/pull-requests/:pr_id/diff
```

Returns unified diff for all files changed in a pull request.

**Features:**

- Unified response format across all providers
- Redis caching (5-minute TTL)
- Rate limiting (100 req/min per user)
- Provider-agnostic file status (added, deleted, modified, renamed, copied)

**See:** [DIFF-ENDPOINT.md](./DIFF-ENDPOINT.md) for complete documentation.

## API Conventions

### Success Response Format

```json
{
  "success": true,
  "data": {
    /* response data */
  },
  "metadata": {
    /* optional metadata */
  }
}
```

### Error Response Format

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      /* optional error details */
    }
  }
}
```

### HTTP Status Codes

| Code | Meaning               | Usage                           |
| ---- | --------------------- | ------------------------------- |
| 200  | OK                    | Successful request              |
| 400  | Bad Request           | Invalid parameters              |
| 401  | Unauthorized          | Missing/invalid auth token      |
| 404  | Not Found             | Resource not found or no access |
| 429  | Too Many Requests     | Rate limit exceeded             |
| 500  | Internal Server Error | Server-side error               |
| 503  | Service Unavailable   | Provider API unavailable        |

## Rate Limiting

All authenticated endpoints have rate limits:

- **Default**: 100 requests/minute per user
- **Burst**: 20 requests/second
- **Headers**:
  ```http
  X-RateLimit-Limit: 100
  X-RateLimit-Remaining: 87
  X-RateLimit-Reset: 1735142400
  ```

## Caching

API responses are cached in Redis when configured:

- **Cache Headers**:

  ```http
  X-Cache-Status: HIT | MISS | BYPASS
  X-Cache-Age: 120
  Cache-Control: private, max-age=300
  ```

- **Bypass Cache**:
  ```bash
  curl -H "Cache-Control: no-cache" ...
  ```

## Provider Support

Ampel integrates with three git providers:

| Provider         | Status          | Notes                                  |
| ---------------- | --------------- | -------------------------------------- |
| GitHub           | ✅ Full Support | All features available                 |
| GitLab           | ✅ Full Support | Pagination handled automatically       |
| Bitbucket Cloud  | ✅ Full Support | May be slower due to per-file fetching |
| Bitbucket Server | ✅ Full Support | Requires server-side configuration     |

## Development

### Running Locally

```bash
# Start API server
make dev-api

# Access Swagger UI
open http://localhost:8080/api/docs
```

### Testing

```bash
# Backend tests
make test-backend

# Run specific API tests
cargo test --test test_pull_requests -- --nocapture
```

### Adding New Endpoints

1. Create handler in `crates/ampel-api/src/handlers/`
2. Add utoipa annotations using `#[utoipa::path(...)]`
3. Register in `crates/ampel-api/src/docs.rs`
4. Add route in `crates/ampel-api/src/routes/mod.rs`
5. Document in `docs/api/`

## API Versioning

- Current version: `v1`
- Version included in URL: `/api/v1/...`
- Deprecation notices: 6 months before removal
- Breaking changes: New version (`v2`, `v3`, etc.)

## Security

- **Authentication**: JWT with 15-minute access tokens
- **Token Storage**: Provider PATs encrypted with AES-256-GCM
- **Password Hashing**: Argon2id
- **CORS**: Configurable allowed origins
- **Rate Limiting**: Per-user quotas

## Support

- **Documentation**: https://docs.ampel.dev
- **API Status**: https://status.ampel.dev
- **Issues**: https://github.com/ampel/ampel/issues
- **Email**: support@ampel.dev

## License

MIT License - see LICENSE file for details
