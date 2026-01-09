# Bitbucket Cloud API: Repository Visibility and Archive Status

**Research Date**: 2025-12-23
**API Version**: Bitbucket Cloud REST API 2.0

## Summary

Bitbucket Cloud uses a simple boolean `is_private` field for visibility and **does not support** repository archiving natively. This document covers API behavior, limitations, and recommendations for Ampel integration.

---

## 1. Visibility Types

### API Field: `is_private` (boolean)

Bitbucket Cloud uses a straightforward boolean approach:

| Value   | Meaning            | Access                           |
| ------- | ------------------ | -------------------------------- |
| `false` | Public repository  | Visible to everyone              |
| `true`  | Private repository | Visible only to authorized users |

### API Response Example

```json
{
  "type": "repository",
  "uuid": "{abc123...}",
  "full_name": "workspace/repo-name",
  "name": "repo-name",
  "is_private": true,
  "scm": "git",
  "mainbranch": {
    "name": "main",
    "type": "branch"
  }
}
```

### Querying by Visibility

Filter repositories using query parameters:

```bash
# Private repositories only
GET /repositories/{workspace}?q=is_private%3Dtrue

# Public repositories only
GET /repositories/{workspace}?q=is_private%3Dfalse
```

**Note**: Query parameter values require URL encoding (`=` becomes `%3D`).

---

## 2. Archive Status

### Critical Finding: No Native Archive Support

**Bitbucket Cloud does NOT support repository archiving.**

| Platform              | Archive Support | API Field        |
| --------------------- | --------------- | ---------------- |
| GitHub                | Yes             | `archived: bool` |
| GitLab                | Yes             | `archived: bool` |
| Bitbucket Cloud       | **No**          | N/A              |
| Bitbucket Data Center | Yes             | `archived: bool` |

### Bitbucket's Recommended Workaround

Atlassian suggests using a dedicated "archive" project:

1. Create a project named "Archive" or similar
2. Move inactive repositories to this project
3. Restrict project permissions to workspace admins only
4. Repositories remain fully functional (not read-only)

**This is NOT a true archive** - repositories:

- Still appear in API listings
- Can still receive commits/PRs
- Are not marked as read-only
- Have no special API indicator

### Ampel Implementation

```rust
// crates/ampel-providers/src/bitbucket.rs
is_archived: false, // Bitbucket doesn't have archived concept
```

The current implementation correctly hardcodes `is_archived: false` for all Bitbucket Cloud repositories.

---

## 3. API Endpoints

### List Repositories

**Endpoint**: `GET /repositories/{workspace}`

**Parameters**:

| Parameter | Type    | Description                                                      |
| --------- | ------- | ---------------------------------------------------------------- |
| `role`    | string  | Filter by user's role: `admin`, `contributor`, `member`, `owner` |
| `q`       | string  | Query filter (e.g., `is_private=true`)                           |
| `sort`    | string  | Sort field (prefix `-` for descending)                           |
| `page`    | integer | Page number                                                      |
| `pagelen` | integer | Results per page (max: 100)                                      |
| `fields`  | string  | Specific fields to return                                        |

**Example Queries**:

```bash
# Private repos, sorted by most recent update
GET /repositories/{workspace}?q=is_private%3Dtrue&sort=-updated_on

# Filter by project
GET /repositories/{workspace}?q=project.key%3D"PROJ"

# Combine filters
GET /repositories/{workspace}?q=is_private%3Dtrue%20AND%20project.key%3D"PROJ"
```

### Get Single Repository

**Endpoint**: `GET /repositories/{workspace}/{repo_slug}`

Returns full repository object including `is_private` field.

---

## 4. Workspace Hierarchy

Bitbucket uses a unique hierarchical structure:

```
Workspace (formerly "teams")
└── Projects
    └── Repositories
```

### Key Differences from GitHub/GitLab

| Aspect    | GitHub      | GitLab     | Bitbucket                            |
| --------- | ----------- | ---------- | ------------------------------------ |
| Top-level | User/Org    | User/Group | Workspace                            |
| Grouping  | None (flat) | Subgroups  | Projects                             |
| Required  | No          | No         | Yes (all repos must be in a project) |

### Permission Inheritance

Permissions flow: Workspace → Project → Repository

- **Admin**: Full control
- **Create**: Can create repos, write access to all
- **Write**: Edit content
- **Read**: Read-only

**Important**: A private repository in a "public" project remains private. The `is_private` setting takes precedence.

---

## 5. Authentication

### App Passwords (PAT Equivalent)

Bitbucket requires **Basic Auth** (not Bearer tokens):

```
Authorization: Basic base64(username:app_password)
```

**Current Ampel Implementation**:

```rust
let auth = BASE64.encode(format!("{}:{}", username, token));
format!("Basic {}", auth)
```

### 2025-2026 Deprecation Timeline

| Date              | Event                               |
| ----------------- | ----------------------------------- |
| September 9, 2025 | No new app passwords can be created |
| June 9, 2026      | All existing app passwords disabled |

**Action Required**: Plan migration to OAuth 2.0 or workspace access tokens.

---

## 6. Rate Limiting

Bitbucket's rate limiting differs from GitHub/GitLab:

- Approximately 1,000 requests/hour per user
- Rate limits NOT exposed via API headers
- No `X-RateLimit-*` headers returned

**Current Ampel Handling**: Returns placeholder values (acceptable approach).

---

## 7. Differences: Cloud vs Data Center

| Feature            | Bitbucket Cloud         | Bitbucket Data Center        |
| ------------------ | ----------------------- | ---------------------------- |
| Archive support    | No                      | Yes                          |
| Rate limit headers | No                      | Yes                          |
| API base URL       | `api.bitbucket.org/2.0` | `{instance}/rest/api/latest` |
| Auth methods       | OAuth, App passwords    | Personal tokens, OAuth       |

### Data Center Archive Endpoint

```
PUT /rest/api/latest/projects/{projectKey}/repos/{repoSlug}
Content-Type: application/json

{
  "archived": true
}
```

Archived repositories become read-only:

- No new PRs, branches, or comments
- Can still clone and fork

---

## 8. Recommendations for Ampel

### Current Status: Correct Implementation

- `is_private` field: ✅ Correctly mapped
- `is_archived` field: ✅ Correctly hardcoded to `false`
- Authentication: ✅ Correctly uses Basic Auth

### Future Enhancements

1. **Bitbucket Data Center Support**
   - Detect Cloud vs Data Center via URL pattern
   - Enable archive field for Data Center instances
   - Update UI to show archived status when available

2. **Authentication Migration (Before June 2026)**
   - Add OAuth 2.0 flow
   - Support workspace access tokens (premium)
   - Provide migration guidance in UI

3. **UI Improvements**
   - Add note in UI that Bitbucket doesn't support archiving
   - Consider showing workspace/project hierarchy
   - Filter by project support

---

## Sources

- [Set repository privacy and forking options](https://support.atlassian.com/bitbucket-cloud/docs/set-repository-privacy-and-forking-options/)
- [Bitbucket Cloud REST API - Repositories](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-repositories/)
- [How To Archive Repositories (Workaround)](https://support.atlassian.com/bitbucket-cloud/kb/how-to-archive-repositories/)
- [Archive a repository - Bitbucket Data Center](https://confluence.atlassian.com/bitbucketserver/archive-a-repository-1128304317.html)
- [Workspace Access Token Permissions](https://support.atlassian.com/bitbucket-cloud/docs/workspace-access-token-permissions/)
- [Bitbucket API Changes for GDPR](https://developer.atlassian.com/cloud/bitbucket/bitbucket-api-changes-gdpr/)
