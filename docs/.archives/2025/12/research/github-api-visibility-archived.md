# GitHub API Research: Repository Visibility and Archived Status

**Research Date**: 2025-12-23
**API Version**: 2022-11-28 (current stable)
**Researcher**: Research Agent

---

## Executive Summary

This document details GitHub's REST API support for repository visibility (public/private/internal) and archived status designations, including specific endpoints, query parameters, API fields, and known quirks.

---

## 1. Repository Visibility Types

### 1.1 Visibility Options

GitHub supports **three visibility types** for repositories:

- **`public`**: Visible to the world, primarily for open source content
- **`private`**: Only visible to individuals/teams with explicit access
- **`internal`**: Read-accessible to all full members of enterprise accounts (Enterprise Cloud/Server 2.20+)

### 1.2 API Fields

**Repository Object Fields**:

- **`private`** (boolean): Legacy field indicating if repository is private
  - Example: `"private": false`

- **`visibility`** (string): Preferred field with values `"public"`, `"private"`, or `"internal"`
  - **Important**: The `visibility` field **overrides** any value set in the `private` field
  - Available since internal repository visibility was made generally available (Oct 2019)

### 1.3 Setting Visibility via API

**Create Repository**:

```bash
POST /user/repos
POST /orgs/{org}/repos

Body:
{
  "name": "repo-name",
  "visibility": "private"  # or "public" or "internal"
}
```

**Update Repository**:

```bash
PATCH /repos/{owner}/{repo}

Body:
{
  "visibility": "private"  # changes repository visibility
}
```

**Example**:

```bash
curl -X PATCH \
  -H "Authorization: token YOUR_PERSONAL_ACCESS_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  -d '{"visibility": "private"}' \
  https://api.github.com/repos/OWNER/REPO
```

### 1.4 Required Permissions

**OAuth Scopes**:

- **Public repositories**: Requires `public_repo` scope or `repo` scope
- **Private repositories**: Requires `repo` scope

**Fine-grained tokens**: Can work without authentication if only public resources are requested

---

## 2. Archived Repository Status

### 2.1 API Field

**Repository Object Field**:

- **`archived`** (boolean): Indicates if repository is archived
  - Example: `"archived": true`
  - Default: `false`

### 2.2 Setting Archived Status via API

**Update Repository to Archive**:

```bash
PATCH /repos/{owner}/{repo}

Body:
{
  "archived": true
}
```

**Important Limitation**: The API **does not currently support unarchiving** repositories. Once archived via API, you must use the GitHub web interface to unarchive.

### 2.3 Effects of Archiving

When a repository is archived:

- It becomes read-only (no pushes, issues, or PRs)
- It's excluded from the GitHub Archive Program
- Public forks are **not** automatically archived

---

## 3. API Endpoints and Query Parameters

### 3.1 List User Repositories

**Endpoint**: `GET /user/repos`

**Authentication**: Required (returns 401 without auth)

**Query Parameters**:

| Parameter     | Type    | Values                                         | Default                                   | Description          |
| ------------- | ------- | ---------------------------------------------- | ----------------------------------------- | -------------------- |
| `visibility`  | string  | `all`, `public`, `private`                     | `all`                                     | Filter by visibility |
| `affiliation` | string  | `owner`, `collaborator`, `organization_member` | `owner,collaborator,organization_member`  | Comma-separated list |
| `type`        | string  | `all`, `owner`, `public`, `private`, `member`  | `owner`                                   | Filter by type       |
| `sort`        | string  | `created`, `updated`, `pushed`, `full_name`    | `full_name`                               | Sort property        |
| `direction`   | string  | `asc`, `desc`                                  | `asc` when using `full_name`, else `desc` | Sort direction       |
| `per_page`    | integer | 1-100                                          | 30                                        | Results per page     |
| `page`        | integer | ≥1                                             | 1                                         | Page number          |

**Example**:

```bash
curl -H "Authorization: token YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "https://api.github.com/user/repos?visibility=private&type=all&per_page=100"
```

**Important**: The `/user/repos` endpoint returns **authenticated user's repositories**, including private ones. The `/users/{username}/repos` endpoint returns only **public repositories** unless authenticated.

### 3.2 List Organization Repositories

**Endpoint**: `GET /orgs/{org}/repos`

**Authentication**: Optional (can work without auth for public repos only)

**Query Parameters**:

| Parameter   | Type    | Values                                                   | Default                                 | Description      |
| ----------- | ------- | -------------------------------------------------------- | --------------------------------------- | ---------------- |
| `type`      | string  | `all`, `public`, `private`, `forks`, `sources`, `member` | `all`                                   | Filter by type   |
| `sort`      | string  | `created`, `updated`, `pushed`, `full_name`              | `created`                               | Sort property    |
| `direction` | string  | `asc`, `desc`                                            | `desc` when using `created`, else `asc` | Sort direction   |
| `per_page`  | integer | 1-100                                                    | 30                                      | Results per page |
| `page`      | integer | ≥1                                                       | 1                                       | Page number      |

**Example**:

```bash
curl -H "Authorization: token YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "https://api.github.com/orgs/my-org/repos?type=all&sort=full_name"
```

**Permission Note**: To see the `security_and_analysis` block, you need:

- Admin permissions for the repository, OR
- Owner/security manager role for the organization

### 3.3 Search Repositories API

**Endpoint**: `GET /search/repositories`

**Query Parameter**: `q` (search query with qualifiers)

**Visibility Qualifiers**:

- `is:public` - Search public repositories
- `is:private` - Search private repositories (requires authentication)
- `is:internal` - Search internal repositories (Enterprise only)

**Archived Qualifiers**:

- `is:archived` - Include only archived repositories
- `NOT is:archived` - Exclude archived repositories
- **Default behavior**: Includes archived repositories but down-ranks them in results

**Examples**:

```bash
# Search user's repositories excluding archived
curl -H "Authorization: token YOUR_TOKEN" \
  "https://api.github.com/search/repositories?q=user:USERNAME+NOT+is:archived"

# Search private repositories
curl -H "Authorization: token YOUR_TOKEN" \
  "https://api.github.com/search/repositories?q=user:USERNAME+is:private"

# Combine filters
curl -H "Authorization: token YOUR_TOKEN" \
  "https://api.github.com/search/repositories?q=user:USERNAME+is:public+NOT+is:archived"
```

**Important**: Without authentication, the search API returns only public repositories. With authentication, it returns both public and private repositories.

---

## 4. Filtering Archived Repositories

### 4.1 API-Level Filtering

**Problem**: The `/user/repos` and `/orgs/{org}/repos` endpoints **do not have** an `archived` query parameter to filter archived repositories directly.

**Solution**: Client-side filtering is required:

```javascript
// Fetch all repos and filter
const response = await fetch('https://api.github.com/user/repos?per_page=100', {
  headers: { Authorization: `token ${token}` },
});
const repos = await response.json();
const activeRepos = repos.filter((repo) => !repo.archived);
```

### 4.2 Search API Filtering

**Recommended Approach**: Use the Search API with `NOT is:archived` qualifier:

```bash
GET /search/repositories?q=user:USERNAME+NOT+is:archived
```

**Benefits**:

- Server-side filtering (more efficient)
- Works with other qualifiers (visibility, language, etc.)
- Supports pagination

### 4.3 GitHub CLI Alternative

The GitHub CLI provides built-in archived filtering:

```bash
# Exclude archived repositories
gh search repos --archived=false

# Include only archived repositories
gh search repos --archived=true
```

### 4.4 Known Limitations

- `archived:false` qualifier works for **issues search** but **not** for code search on some platforms
- On GitHub Enterprise Server 3.17+, `archived:false` and `NOT is:archived` **do not work** on the Code tab (but work for repository search)
- Different behavior between legacy code search and GitHub Code Search (new)

---

## 5. Rate Limiting

### 5.1 Standard Rate Limits

| Authentication Method       | Requests/Hour        | Notes                             |
| --------------------------- | -------------------- | --------------------------------- |
| Unauthenticated             | 60                   | IP-based limit                    |
| Personal Access Token (PAT) | 5,000                | Per user                          |
| OAuth App                   | 5,000                | Per app                           |
| GitHub App (base)           | 5,000                | Scales with repos/users           |
| GitHub App (scaled)         | 5,000 + (50 × repos) | Max ~12,500 for orgs              |
| GITHUB_TOKEN (Actions)      | 1,000                | Per repo/hour                     |
| GITHUB_TOKEN (Enterprise)   | 15,000               | Per repo/hour on Enterprise Cloud |

### 5.2 Visibility-Specific Limits

**Key Finding**: Certain rate limits for creating content **do not apply** to:

- Private repositories
- GitHub Enterprise installations

**Implication**: Operations on private repos may have different (often more lenient) secondary rate limits.

### 5.3 Secondary Rate Limits

GitHub enforces **secondary rate limits** to prevent abuse:

- **Concurrent requests**: Maximum 100 concurrent requests
- **Point system**:
  - `GET`, `HEAD`, `OPTIONS`: 1 point
  - `POST`, `PATCH`, `PUT`, `DELETE`: 5 points
- **Purpose**: Prevent rapid-fire requests that could overwhelm the API

### 5.4 Rate Limit Monitoring

**Check Current Limits**:

```bash
GET /rate_limit
```

**Response includes**:

```json
{
  "resources": {
    "core": {
      "limit": 5000,
      "remaining": 4999,
      "reset": 1372700873,
      "used": 1
    },
    "search": {
      "limit": 30,
      "remaining": 29,
      "reset": 1372697452,
      "used": 1
    }
  }
}
```

**Note**: Search API has a **separate, lower rate limit** (30 requests/minute for authenticated users).

---

## 6. Idiosyncratic Behavior and Gotchas

### 6.1 Visibility Field Priority

**Behavior**: When both `private` and `visibility` fields are provided, `visibility` **takes precedence**.

**Recommendation**: Use `visibility` field exclusively for new integrations.

### 6.2 Archived Repository Limitations

**Cannot Unarchive via API**: Once archived through the API, you must use the GitHub web UI to unarchive.

**Workaround**: Consider using `archived` as a read-only flag via API and direct users to the web UI for unarchiving.

### 6.3 Endpoint Differences for Private Repos

**Critical Distinction**:

- `/users/{username}/repos` - Returns **only public repositories** (even with authentication)
- `/user/repos` - Returns **authenticated user's repositories** (including private)

**Example**:

```bash
# Returns only public repos (even with auth)
curl -H "Authorization: token TOKEN" \
  https://api.github.com/users/octocat/repos

# Returns all repos (including private) for authenticated user
curl -H "Authorization: token TOKEN" \
  https://api.github.com/user/repos
```

### 6.4 Search API Quirks

**Default Archived Behavior**: The search API **includes archived repositories by default**, but down-ranks them. This means:

- You'll see archived repos in results if they're the best match
- Actively maintained repos are prioritized
- Use `NOT is:archived` to explicitly exclude

**Code Search Inconsistency**:

- **Legacy code search**: Completely excludes archived repositories (no option to include)
- **GitHub Code Search (new)**: Supports `NOT is:archived` qualifier

### 6.5 Internal Visibility (Enterprise Only)

**Availability**: `internal` visibility is **only available** on:

- GitHub Enterprise Cloud
- GitHub Enterprise Server 2.20+

**API Behavior**: Setting `visibility: "internal"` on non-Enterprise accounts will return an error.

**Search Qualifier**: Use `is:internal` to search internal repositories (Enterprise only).

### 6.6 Visibility Changes and Forks

**Important**: When changing a public repository to private:

- **Public forks are NOT made private** automatically
- Forks are detached and moved to a new network
- GitHub Pages sites are **automatically unpublished**
- Custom domains should be removed/updated to avoid domain takeover risks

### 6.7 Enterprise Managed Users (EMU)

**Restriction**: Enterprises using EMUs **prevent**:

- Creation of public repositories
- Changing existing repository visibility to public

**Implication**: For EMU accounts, `visibility: "public"` will fail with a permissions error.

### 6.8 Deleted Repository Ghosts

**Known Issue**: Some API endpoints may return "ghost" entries for deleted repositories with minimal data.

**Mitigation**: Check for essential fields (e.g., `full_name`, `id`) before processing repository objects.

---

## 7. Best Practices and Recommendations

### 7.1 Querying Repositories

**For Active Repositories Only**:

```bash
# Use Search API for server-side filtering
GET /search/repositories?q=user:USERNAME+NOT+is:archived

# Or filter client-side from list endpoint
GET /user/repos?per_page=100
# Then filter where repo.archived === false
```

**For Visibility Filtering**:

```bash
# Use visibility parameter on list endpoints
GET /user/repos?visibility=private

# Or use is:public/is:private in search
GET /search/repositories?q=user:USERNAME+is:private
```

### 7.2 Pagination

**Always paginate** when listing repositories:

- Use `per_page=100` (maximum) to reduce API calls
- Check `Link` header for next page URL
- Be aware of rate limits when fetching many pages

### 7.3 Caching Strategy

**Cache Repository Metadata**:

- Store `visibility` and `archived` status locally
- Refresh periodically or use webhooks for real-time updates
- Use `If-None-Match` header with ETags to save rate limit quota

**Webhook Events**:

- `repository` event with `archived` action
- `repository` event with `publicized` action (public → private not covered)

### 7.4 Error Handling

**Handle Common Errors**:

- `401 Unauthorized`: Token expired or missing
- `403 Forbidden`: Insufficient scopes or rate limit exceeded
- `404 Not Found`: Repository doesn't exist or no access
- `422 Unprocessable Entity`: Invalid `visibility` value (e.g., `internal` on non-Enterprise)

### 7.5 Authentication Scopes

**Minimum Required Scopes**:

- **Read public repos**: No scope required
- **Read private repos**: `repo` scope
- **Update visibility**: `repo` scope

---

## 8. Code Examples

### 8.1 List Active Private Repositories (Node.js)

```javascript
const { Octokit } = require('@octokit/rest');

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });

async function listActivePrivateRepos(username) {
  try {
    // Use Search API for efficient server-side filtering
    const { data } = await octokit.search.repos({
      q: `user:${username} is:private NOT is:archived`,
      per_page: 100,
      sort: 'updated',
      order: 'desc',
    });

    return data.items;
  } catch (error) {
    console.error('Error fetching repos:', error.message);
    throw error;
  }
}

// Alternative: Client-side filtering with list endpoint
async function listActivePrivateReposAlt() {
  const repos = [];
  let page = 1;

  while (true) {
    const { data } = await octokit.repos.listForAuthenticatedUser({
      visibility: 'private',
      per_page: 100,
      page,
    });

    if (data.length === 0) break;

    // Filter out archived repos
    repos.push(...data.filter((repo) => !repo.archived));
    page++;
  }

  return repos;
}
```

### 8.2 Archive Repository (Rust)

```rust
use reqwest::Client;
use serde_json::json;

async fn archive_repository(
    client: &Client,
    token: &str,
    owner: &str,
    repo: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let response = client
        .patch(&url)
        .header("Authorization", format!("token {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "ampel-app")
        .json(&json!({
            "archived": true
        }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("Repository archived successfully");
        Ok(())
    } else {
        let error = response.text().await?;
        Err(format!("Failed to archive repository: {}", error).into())
    }
}
```

### 8.3 Filter Repositories by Visibility and Archived Status

```typescript
interface GitHubRepo {
  id: number;
  name: string;
  full_name: string;
  private: boolean;
  visibility: 'public' | 'private' | 'internal';
  archived: boolean;
  disabled: boolean;
}

async function filterRepositories(
  repos: GitHubRepo[],
  options: {
    includeArchived?: boolean;
    visibility?: 'public' | 'private' | 'internal' | 'all';
  } = {}
): Promise<GitHubRepo[]> {
  const { includeArchived = false, visibility = 'all' } = options;

  return repos.filter((repo) => {
    // Filter archived
    if (!includeArchived && repo.archived) {
      return false;
    }

    // Filter visibility
    if (visibility !== 'all' && repo.visibility !== visibility) {
      return false;
    }

    // Optionally filter disabled repos
    if (repo.disabled) {
      return false;
    }

    return true;
  });
}
```

---

## 9. Summary Table

### API Fields

| Field        | Type    | Values                          | Scope             | Notes                       |
| ------------ | ------- | ------------------------------- | ----------------- | --------------------------- |
| `private`    | boolean | `true`, `false`                 | Repository object | Legacy, prefer `visibility` |
| `visibility` | string  | `public`, `private`, `internal` | Repository object | Overrides `private`         |
| `archived`   | boolean | `true`, `false`                 | Repository object | Cannot unarchive via API    |

### List Endpoints

| Endpoint               | Auth Required | `visibility` param | `archived` param | Returns Private |
| ---------------------- | ------------- | ------------------ | ---------------- | --------------- |
| `/user/repos`          | Yes           | Yes                | No               | Yes             |
| `/users/{user}/repos`  | No            | No                 | No               | No              |
| `/orgs/{org}/repos`    | No\*          | No                 | No               | Yes\*           |
| `/search/repositories` | No\*\*        | Via `is:`          | Via `is:`        | Yes\*\*         |

\* With auth and proper permissions
\*\* With auth

### Query Parameters

| Endpoint               | Visibility Filter    | Archived Filter     | Client-side Filter Required |
| ---------------------- | -------------------- | ------------------- | --------------------------- |
| `/user/repos`          | `visibility=private` | None                | Yes                         |
| `/orgs/{org}/repos`    | `type=private`       | None                | Yes                         |
| `/search/repositories` | `q=is:private`       | `q=NOT+is:archived` | No                          |

---

## 10. References and Sources

- [REST API endpoints for repositories - GitHub Docs](https://docs.github.com/en/rest/repos/repos)
- [Setting repository visibility - GitHub Docs](https://docs.github.com/articles/setting-repository-visibility)
- [Searching for repositories - GitHub Docs](https://docs.github.com/en/search-github/searching-on-github/searching-for-repositories)
- [Rate limits for the REST API - GitHub Docs](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)
- [How to Use the GitHub API to List Repositories - Stateful](https://stateful.com/blog/github-api-list-repositories)
- [Internal repository visibility is now generally available - GitHub Changelog](https://github.blog/changelog/2019-10-28-internal-repository-visibility-is-now-generally-available/)
- [Please, please support excluding archived repos - GitHub Community Discussion](https://github.com/orgs/community/discussions/8591)
- [How to get list of PRIVATE repositories via api call - GitHub Community Discussion](https://github.com/orgs/community/discussions/24382)
- [GitHub CLI Manual - gh search repos](https://cli.github.com/manual/gh_search_repos)
- [A Developer's Guide: Managing Rate Limits for the GitHub API](https://www.lunar.dev/post/a-developers-guide-managing-rate-limits-for-the-github-api)

---

## Appendix A: Quick Reference Card

```bash
# List all private repos (authenticated user)
GET /user/repos?visibility=private

# List active (non-archived) repos using Search API
GET /search/repositories?q=user:USERNAME+NOT+is:archived

# Get single repo (includes visibility and archived)
GET /repos/{owner}/{repo}

# Archive a repository
PATCH /repos/{owner}/{repo}
Body: {"archived": true}

# Change visibility to private
PATCH /repos/{owner}/{repo}
Body: {"visibility": "private"}

# Check rate limit
GET /rate_limit
```

---

**End of Research Document**
