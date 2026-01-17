# GitLab API: Project Visibility and Archived Status

Research conducted: 2025-12-23

## Overview

GitLab provides comprehensive API support for filtering projects by visibility level and archived status. This document details the API endpoints, query parameters, field values, and idiosyncratic behaviors.

---

## 1. Visibility Types

### 1.1 The Three Visibility Levels

GitLab projects can have one of three visibility levels:

| Visibility   | Value (numeric) | Description                      | Access Requirements                        |
| ------------ | --------------- | -------------------------------- | ------------------------------------------ |
| **Private**  | 0               | Only approved members can access | Explicit project/group membership required |
| **Internal** | 10              | Authenticated users can access   | Must be logged in; external users excluded |
| **Public**   | 20              | Anyone can access                | No authentication required                 |

### 1.2 API Field

- **Field Name**: `visibility`
- **Type**: String
- **Valid Values**: `"private"`, `"internal"`, `"public"`

### 1.3 Example JSON Response

```json
{
  "id": 3,
  "name": "My Project",
  "description": "Lorem ipsum dolor sit amet",
  "default_branch": "main",
  "visibility": "internal",
  "ssh_url_to_repo": "git@example.com:namespace/project.git",
  "archived": false,
  ...
}
```

---

## 2. Archived Status

### 2.1 Archived Projects Behavior

- **Field Name**: `archived`
- **Type**: Boolean
- **Values**: `true` (archived), `false` (active)

### 2.2 Search Behavior (Important Quirk)

Starting from GitLab 16.x:

- **Default Behavior**: Archived projects are EXCLUDED from search results by default
- **To Include Archived**: Use the `include_archived=true` parameter in search APIs
- **UI Behavior**: Users must check "Include archived" checkbox in search interface

This is controlled by feature flags:

- `search_commits_hide_archived_projects` (for commit search)
- `search_milestones_hide_archived_projects` (for milestone search)

### 2.3 Archive/Unarchive Endpoints

**Archive a project:**

```bash
POST /api/v4/projects/:id/archive
```

**Unarchive a project:**

```bash
POST /api/v4/projects/:id/unarchive
```

Both endpoints are **idempotent**:

- Archiving an already-archived project does not change it
- Unarchiving a non-archived project does not change it

**Permissions Required**: Owner role on the project

---

## 3. API Endpoints

### 3.1 List All Projects

**Endpoint:**

```
GET /api/v4/projects
```

**Description**: Returns all visible projects across GitLab for the authenticated user. When accessed without authentication, only public projects with simple fields are returned.

### 3.2 Query Parameters

| Parameter                     | Type    | Description                         | Valid Values                                                           |
| ----------------------------- | ------- | ----------------------------------- | ---------------------------------------------------------------------- |
| `visibility`                  | string  | Filter by visibility level          | `"public"`, `"internal"`, `"private"`                                  |
| `archived`                    | boolean | Filter by archived status           | `true`, `false`                                                        |
| `owned`                       | boolean | Limit to owned projects             | `true`, `false`                                                        |
| `membership`                  | boolean | Limit to projects user is member of | `true`, `false`                                                        |
| `starred`                     | boolean | Limit to starred projects           | `true`, `false`                                                        |
| `search`                      | string  | Search projects by keyword          | Any string                                                             |
| `simple`                      | boolean | Return only basic fields            | `true`, `false`                                                        |
| `order_by`                    | string  | Sort field                          | `id`, `name`, `path`, `created_at`, `updated_at`, `last_activity_at`   |
| `sort`                        | string  | Sort direction                      | `asc`, `desc`                                                          |
| `statistics`                  | boolean | Include project statistics          | `true`, `false`                                                        |
| `with_issues_enabled`         | boolean | Filter projects with issues         | `true`, `false`                                                        |
| `with_merge_requests_enabled` | boolean | Filter projects with MRs            | `true`, `false`                                                        |
| `min_access_level`            | integer | Minimum access level                | 10 (Guest), 20 (Reporter), 30 (Developer), 40 (Maintainer), 50 (Owner) |

### 3.3 Example API Calls

**List internal visibility projects:**

```bash
curl --request GET \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects?visibility=internal"
```

**List archived projects:**

```bash
curl --request GET \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects?archived=true"
```

**List internal archived projects:**

```bash
curl --request GET \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects?visibility=internal&archived=true"
```

**List only owned, active projects:**

```bash
curl --request GET \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects?owned=true&archived=false"
```

### 3.4 Python-GitLab Library Examples

```python
import gitlab

gl = gitlab.Gitlab('https://gitlab.example.com', private_token='<token>')

# List all projects
projects = gl.projects.list(get_all=True)

# Archived projects only
projects = gl.projects.list(archived=True, get_all=True)

# Public visibility projects
projects = gl.projects.list(visibility='public', get_all=True)

# Internal visibility projects
projects = gl.projects.list(visibility='internal', get_all=True)

# Owned projects that are not archived
projects = gl.projects.list(owned=True, archived=False, get_all=True)

# Search with visibility filter
projects = gl.projects.list(search='keyword', visibility='private', get_all=True)
```

---

## 4. Idiosyncratic Behaviors and Quirks

### 4.1 Internal Visibility is Disabled on GitLab.com

**Critical Quirk**: As of July 2019, the "Internal" visibility setting is **disabled for NEW projects, groups, and snippets on GitLab.com**.

- Existing projects/groups with "internal" visibility retain this setting
- Self-hosted GitLab instances still support all three visibility levels
- GitLab.com only offers "public" and "private" for new projects

**Reason**: Internal visibility can be misleading on a shared public service like GitLab.com.

### 4.2 External Users and Internal Visibility

- **External users** cannot access "internal" projects, even though other authenticated users can
- External users must be given explicit membership (at least Reporter role) to access internal projects
- External users have limited access: only to groups and projects where they are direct members

### 4.3 Visibility Hierarchy Constraints

**Parent-Child Visibility Rules**:

- A project's visibility must be AT LEAST as restrictive as its parent group
- You cannot set a group to "private" if it contains a "public" project
- You cannot set a project to "public" if its parent group is "private"

**Examples**:

- Private groups can ONLY have private subgroups and projects
- Public groups can have public, internal, OR private subgroups and projects
- Internal groups can have internal OR private subgroups and projects

### 4.4 Permission Requirements

**To View Projects**:

- Public: No authentication required
- Internal: Must be authenticated AND not an external user
- Private: Must have explicit membership

**To Change Visibility**:

- Must have **Owner role** on the project
- For groups: visibility can only be changed if ALL subgroups and projects have the same or lower visibility

### 4.5 Restricted Visibility Levels (Admin Setting)

Administrators can restrict which visibility levels are available:

- Cannot restrict a visibility level that is set as the default
- Cannot set a restricted visibility level as the default
- **Known Bug**: Cannot uncheck more than one visibility level at a time in admin UI
- When visibility is restricted, user profiles are only visible to authenticated users

### 4.6 API Response Fields Vary by Permission

**Important**: The fields returned in API responses vary based on:

- Whether the user is authenticated
- The user's role/permissions on the project
- Whether `simple=true` parameter is used

**Simple Response** (fewer fields):

- `id`, `description`, `name`, `path`, `created_at`, `default_branch`
- `topics`, URLs, `star_count`, `last_activity_at`, `visibility`, `namespace`

**Full Response** (authenticated with permissions):

- All simple fields PLUS statistics, permissions, CI/CD settings, etc.

### 4.7 Pagination Considerations

- GitLab supports **keyset pagination** for better performance
- Keyset pagination ONLY works with `order_by=id`
- Other `order_by` options require offset pagination
- Endpoint returns maximum 100 items per page by default

### 4.8 Archived Projects in Search APIs

- **Global Search API**: Excludes archived by default; use `include_archived=true` to include
- **Group Search API**: Same behavior
- **Commit Search**: Controlled by `search_commits_hide_archived_projects` feature flag
- **Milestones Search**: Controlled by `search_milestones_hide_archived_projects` feature flag

---

## 5. Comparison with GitHub

### Key Differences

| Feature                     | GitLab                                       | GitHub                       |
| --------------------------- | -------------------------------------------- | ---------------------------- |
| **Visibility Levels**       | 3 levels: public, internal, private          | 2 levels: public, private    |
| **Internal Visibility**     | Yes (except GitLab.com new projects)         | No equivalent                |
| **Archived Filter**         | `archived=true/false`                        | `archived=true/false`        |
| **Default Search Behavior** | Excludes archived by default (recent change) | Includes archived by default |
| **Numeric Values**          | Visibility has numeric codes (0/10/20)       | Not applicable               |

### GitLab-Specific Considerations

1. **Three-tier visibility model**: GitLab's "internal" level has no GitHub equivalent
2. **GitLab.com restrictions**: Internal visibility disabled for new projects
3. **External users**: Special user type with limited access to internal projects
4. **Hierarchy constraints**: Parent-child visibility relationships are strictly enforced
5. **Search defaults**: Recent change to exclude archived projects by default

---

## 6. Implementation Recommendations for Ampel

### 6.1 Provider Abstraction Mapping

Map GitLab's three visibility levels to a unified model:

```rust
pub enum ProjectVisibility {
    Public,
    Internal,  // GitLab-specific
    Private,
}

// For GitHub compatibility, map Internal to Private
impl From<ProjectVisibility> for UnifiedVisibility {
    fn from(vis: ProjectVisibility) -> Self {
        match vis {
            ProjectVisibility::Public => UnifiedVisibility::Public,
            ProjectVisibility::Internal => UnifiedVisibility::Private,
            ProjectVisibility::Private => UnifiedVisibility::Private,
        }
    }
}
```

### 6.2 Archived Status Handling

```rust
// Always explicitly filter archived projects
let params = ListProjectsParams {
    archived: Some(false),  // Exclude archived by default
    visibility: Some("internal"),
    ..Default::default()
};
```

### 6.3 API Query Best Practices

1. **Always specify `archived=false`** unless you explicitly want archived projects
2. **Handle three visibility levels**: Store the original GitLab value, map for display
3. **Consider pagination**: Use keyset pagination with `order_by=id` for large datasets
4. **Check GitLab.com vs self-hosted**: Internal visibility may not be available
5. **Handle external users**: Document that some authenticated users can't see "internal" projects

### 6.4 User-Facing Considerations

- Display all three visibility levels in UI for GitLab
- Show warning if GitLab.com (internal not available for new projects)
- Default to excluding archived projects with toggle to include
- Document that "internal" projects require authentication

---

## 7. References and Sources

### Official Documentation

- [Projects API | GitLab Docs](https://docs.gitlab.com/api/projects/)
- [Project and group visibility | GitLab Docs](https://docs.gitlab.com/user/public_access/)
- [Control access and visibility | GitLab Docs](https://docs.gitlab.com/administration/settings/visibility_and_access_controls/)
- [REST API resources | GitLab Docs](https://docs.gitlab.com/ee/api/api_resources.html)
- [Search API | GitLab Docs](https://docs.gitlab.com/api/search/)

### Python-GitLab Library

- [Projects - python-gitlab v7.0.0](https://python-gitlab.readthedocs.io/en/stable/gl_objects/projects.html)

### GitLab Issues and Merge Requests

- [Disable `Internal` visibility setting on GitLab.com (#12388)](https://gitlab.com/gitlab-org/gitlab/-/issues/12388)
- [Exclude archived projects in search results by default (!121981)](https://gitlab.com/gitlab-org/gitlab/-/merge_requests/121981)
- [Hide archived projects in search results by default (#18262)](https://gitlab.com/gitlab-org/gitlab/-/issues/18262)
- [API: Use `visibility` as string parameter everywhere (#27501)](https://gitlab.com/gitlab-org/gitlab-foss/-/issues/27501)
- [Project API archived param (#32301)](https://gitlab.com/gitlab-org/gitlab-foss/-/issues/32301)

---

## Appendix: Quick Reference

### API Endpoint

```
GET https://gitlab.example.com/api/v4/projects
```

### Key Query Parameters

```
?visibility=internal&archived=false&owned=true&order_by=last_activity_at&sort=desc
```

### Response Fields to Extract

```json
{
  "id": 123,
  "visibility": "internal",
  "archived": false,
  "name": "Project Name",
  "path": "project-path",
  "namespace": { ... }
}
```

### Visibility Value Mapping

- `"private"` → Private (0)
- `"internal"` → Internal (10) - GitLab-specific
- `"public"` → Public (20)
