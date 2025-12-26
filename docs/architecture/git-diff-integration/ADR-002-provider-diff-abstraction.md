# ADR-002: Provider Diff API Abstraction

**Status:** Accepted
**Date:** 2025-12-25
**Decision Makers:** Architecture Team
**Technical Story:** Multi-Provider Diff Support

## Context

Ampel integrates with three git providers (GitHub, GitLab, Bitbucket), each with different API structures for diff data:

- **GitHub**: JSON `/files` endpoint with `patch` field
- **GitLab**: JSON `/changes` endpoint with `diff` field
- **Bitbucket**: JSON `/diffstat` endpoint + per-file diff requests (Server)

The system must provide a unified diff interface to the frontend while handling provider-specific quirks.

## Decision Drivers

- **Consistency**: Unified data model for frontend consumption
- **Maintainability**: Single source of truth for diff structure
- **Extensibility**: Easy to add new providers in future
- **Performance**: Minimize transformation overhead
- **Type Safety**: Leverage Rust's type system for correctness

## Considered Options

### Option 1: Unified Rust Trait with Provider-Specific Transformations (SELECTED)

Extend `GitProvider` trait with `get_pull_request_diff()` method. Each provider implements transformation from native format to unified `ProviderDiff` struct.

**Pros:**

- Type-safe transformations enforced by Rust compiler
- Centralized data model in `ampel-providers/src/traits.rs`
- Provider-specific logic isolated in provider implementations
- Easy to add unit tests for each provider's transformation

**Cons:**

- Requires Rust code changes for new diff fields
- More upfront design required for unified model

### Option 2: Frontend Adapter Pattern

Let backend pass raw provider responses, handle normalization in TypeScript adapters.

**Pros:**

- Faster iteration on diff model changes
- No backend deployments for frontend-only changes

**Cons:**

- Duplicated transformation logic (violates DRY)
- Type safety only at frontend boundary
- Harder to test (JavaScript vs Rust testing)
- Risk of inconsistent transformations

### Option 3: GraphQL Schema

Define GraphQL schema for diffs, let providers implement resolvers.

**Pros:**

- Strong typed schema
- Frontend can query exactly what it needs

**Cons:**

- Overkill for simple REST API
- Adds complexity (GraphQL server, code generation)
- Larger bundle size

## Decision Outcome

**Chosen Option:** Unified Rust Trait with Provider-Specific Transformations

### Unified Data Model

```rust
// crates/ampel-providers/src/traits.rs

/// Unified diff file representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDiffFile {
    pub sha: String,
    pub old_path: Option<String>,
    pub new_path: String,
    pub status: FileStatus,  // Enum: Added, Deleted, Modified, Renamed, Copied
    pub additions: i32,
    pub deletions: i32,
    pub changes: i32,
    pub patch: String,       // Unified diff format
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDiff {
    pub files: Vec<ProviderDiffFile>,
    pub total_additions: i32,
    pub total_deletions: i32,
    pub total_files: i32,
    pub base_commit: String,
    pub head_commit: String,
}

/// Add to GitProvider trait
#[async_trait]
pub trait GitProvider: Send + Sync {
    async fn get_pull_request_diff(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<ProviderDiff>;
}
```

### Provider Implementations

Each provider transforms native format to unified model:

```rust
// GitHub: Straightforward mapping
impl GitProvider for GitHubProvider {
    async fn get_pull_request_diff(...) -> ProviderResult<ProviderDiff> {
        let files = self.fetch_pr_files(...).await?;
        Ok(ProviderDiff {
            files: files.into_iter().map(|f| f.into()).collect(),
            // ... aggregate fields
        })
    }
}

// GitLab: Handle renamed file detection
impl GitProvider for GitLabProvider {
    async fn get_pull_request_diff(...) -> ProviderResult<ProviderDiff> {
        let changes = self.fetch_mr_changes(...).await?;
        Ok(ProviderDiff {
            files: changes.into_iter().map(|c| {
                ProviderDiffFile {
                    old_path: if c.renamed_file { Some(c.old_path) } else { None },
                    status: self.normalize_status(c),
                    // ...
                }
            }).collect(),
        })
    }
}

// Bitbucket: Server requires per-file diff fetching
impl GitProvider for BitbucketProvider {
    async fn get_pull_request_diff(...) -> ProviderResult<ProviderDiff> {
        // 1. Fetch diffstat for file list
        let diffstat = self.fetch_diffstat(...).await?;

        // 2. For each file, fetch individual diff
        let mut files = Vec::new();
        for stat in diffstat.values {
            let patch = self.fetch_file_diff(..., &stat.path).await?;
            files.push(ProviderDiffFile {
                patch,
                // ...
            });
        }

        Ok(ProviderDiff { files, /* ... */ })
    }
}
```

### TypeScript Interface (Frontend)

Frontend receives consistent JSON:

```typescript
// frontend/src/types/diff.ts

export interface DiffFile {
  id: string;
  oldPath: string | null;
  newPath: string;
  status: 'added' | 'deleted' | 'modified' | 'renamed' | 'copied';
  additions: number;
  deletions: number;
  changes: number;
  patch: string;
  language?: string; // Detected by backend
}

export interface PullRequestDiff {
  pullRequestId: string;
  provider: 'github' | 'gitlab' | 'bitbucket';
  files: DiffFile[];
  totalAdditions: number;
  totalDeletions: number;
  totalFiles: number;
  baseCommit: string;
  headCommit: string;
  fetchedAt: Date;
}
```

## Rationale

1. **Type Safety**: Rust's type system catches transformation bugs at compile time
2. **Single Source of Truth**: `ProviderDiff` struct defines canonical diff structure
3. **Provider Isolation**: Each provider's transformation logic is self-contained
4. **Testability**: Easy to unit test transformations with mock provider responses
5. **Performance**: Transformation happens once in backend, not repeatedly in frontend

## Consequences

### Positive

- **Compile-Time Correctness**: Type errors caught before deployment
- **Consistent Frontend Experience**: All providers return identical structure
- **Easy Testing**: Mock provider responses → test transformation → verify output
- **Clear Ownership**: Backend owns data normalization, frontend owns presentation

### Negative

- **Backend Deployment Required**: Adding new diff fields requires Rust changes + deploy
- **Migration Complexity**: Changing unified model affects all three providers
- **Bitbucket Server Performance**: Per-file diff fetching can be slow for large PRs

### Mitigation Strategies

1. **Bitbucket Performance**: Implement concurrent fetching (fetch 5-10 files in parallel)
2. **Schema Evolution**: Use `#[serde(default)]` and `Option<T>` for new fields (backward compatible)
3. **Caching**: Cache provider responses for 5 minutes to reduce API calls
4. **Monitoring**: Track provider API latency, alert on >2s average response time

## Edge Cases Handled

### Binary Files

- **Detection**: File extension check + `patch` empty
- **Handling**: Return `patch: ""` with `status: "modified"`, frontend renders "Binary file changed"

### Very Large Diffs

- **GitHub**: API returns first 3000 lines, `patch` field may be truncated
- **Handling**: Backend checks for truncation, frontend shows "Diff too large, view on GitHub" message

### Renamed Files

- **GitHub**: `previous_filename` field present
- **GitLab**: `renamed_file: true` flag + `old_path` different from `new_path`
- **Bitbucket**: `type: "renamed"` in diffstat
- **Handling**: Unified as `status: Renamed`, `old_path: Some(old)`, `new_path: new`

### Deleted Files

- **All Providers**: `status: "deleted"`, `patch` shows full deletion
- **Handling**: Frontend shows red background, "File deleted" badge

## Related Decisions

- ADR-001: Diff Library Selection
- ADR-003: Diff Caching Strategy
- ADR-005: Language Detection for Syntax Highlighting

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_github_diff_transformation() {
        let mock_response = include_str!("../fixtures/github_files.json");
        let files: Vec<GitHubFile> = serde_json::from_str(mock_response).unwrap();

        let diff: ProviderDiff = files.into();

        assert_eq!(diff.files.len(), 5);
        assert_eq!(diff.files[0].status, FileStatus::Modified);
        assert_eq!(diff.total_additions, 104);
    }

    #[tokio::test]
    async fn test_gitlab_renamed_file() {
        let mock_change = GitLabChange {
            old_path: "old.rs",
            new_path: "new.rs",
            renamed_file: true,
            diff: "@@ ...",
        };

        let file: ProviderDiffFile = mock_change.into();

        assert_eq!(file.status, FileStatus::Renamed);
        assert_eq!(file.old_path, Some("old.rs".to_string()));
        assert_eq!(file.new_path, "new.rs");
    }
}
```

## References

- [GitHub API - PR Files](https://docs.github.com/en/rest/pulls/pulls#list-pull-requests-files)
- [GitLab API - MR Changes](https://docs.gitlab.com/api/merge_requests/#get-single-mr-changes)
- [Bitbucket API - Diffstat](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/#api-repositories-workspace-repo-slug-pullrequests-pull-request-id-diffstat-get)
