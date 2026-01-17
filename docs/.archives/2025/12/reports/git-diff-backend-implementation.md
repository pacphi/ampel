# Git Diff Integration - Backend Implementation Summary

## Completed: 2025-12-25

### Phase 1: Core Implementation (COMPLETED)

#### 1. Provider Traits (`crates/ampel-providers/src/traits.rs`)

Added new data structures:

- `ProviderDiffFile`: File-level diff with filename, status, additions/deletions, patch content, and rename tracking
- `ProviderDiff`: Complete diff with file list and totals

Added new trait method:

```rust
async fn get_pull_request_diff(
    &self,
    credentials: &ProviderCredentials,
    owner: &str,
    repo: &str,
    pr_number: i32,
) -> ProviderResult<ProviderDiff>;
```

Both structs include:

- Serde serialization/deserialization
- Utoipa schema derivation for OpenAPI documentation

#### 2. GitHub Implementation (`crates/ampel-providers/src/github.rs`)

Endpoint: `GET /repos/{owner}/{repo}/pulls/{pr_number}/files`

Features:

- Fetches up to 100 files per request
- Includes full patch content
- Tracks file status (added, modified, removed, renamed)
- Aggregates totals for additions, deletions, changes

#### 3. GitLab Implementation (`crates/ampel-providers/src/gitlab.rs`)

Endpoint: `GET /projects/{id}/merge_requests/{mr_number}/diffs`

Features:

- Parses diff patch content to count additions/deletions
- Normalizes status values (new_file → added, deleted_file → removed, etc.)
- Handles renamed files with previous_filename tracking

#### 4. Bitbucket Implementation (`crates/ampel-providers/src/bitbucket.rs`)

Endpoint: `GET /repositories/{owner}/{repo}/pullrequests/{pr_number}/diffstat`

Features:

- Uses diffstat endpoint for statistics
- Handles Bitbucket's status format (added, removed, modified, renamed)
- Note: Patch content not available from diffstat endpoint (patch field is None)

#### 5. API Handler (`crates/ampel-api/src/handlers/pull_requests.rs`)

New handler: `get_pull_request_diff`

Features:

- Authentication and authorization checks
- Repository ownership verification
- Provider account credential decryption
- Error handling with proper HTTP status codes

#### 6. API Route (`crates/ampel-api/src/routes/mod.rs`)

New route: `GET /api/repositories/:repo_id/pull-requests/:pr_id/diff`

## API Response Format

```json
{
  "success": true,
  "data": {
    "files": [
      {
        "filename": "src/main.rs",
        "status": "modified",
        "additions": 15,
        "deletions": 3,
        "changes": 18,
        "patch": "@@ -1,5 +1,6 @@\n...",
        "previous_filename": null
      }
    ],
    "total_additions": 15,
    "total_deletions": 3,
    "total_changes": 18
  }
}
```

## Provider-Specific Normalization

### Status Values

All providers normalize to: `added`, `modified`, `removed`, `renamed`

### GitHub

- Native status values used directly
- Full patch content included

### GitLab

- `new_file` → `added`
- `deleted_file` → `removed`
- `renamed_file` → `renamed`
- Patch content parsed for line counts

### Bitbucket

- Lowercase status values
- No patch content (diffstat endpoint limitation)
- Rename detection via old/new path comparison

## Testing

Compilation verified:

- `cargo check --package ampel-providers` - PASSED
- `cargo check --package ampel-api` - IN PROGRESS

## Next Steps (Frontend Integration)

1. Create TypeScript types matching ProviderDiff structure
2. Add API client method for fetching diff
3. Implement diff viewer UI component
4. Add file-by-file navigation
5. Syntax highlighting for patches
6. Expandable/collapsible file sections

## Files Modified

- `/alt/home/developer/workspace/projects/ampel/crates/ampel-providers/src/traits.rs`
- `/alt/home/developer/workspace/projects/ampel/crates/ampel-providers/src/github.rs`
- `/alt/home/developer/workspace/projects/ampel/crates/ampel-providers/src/gitlab.rs`
- `/alt/home/developer/workspace/projects/ampel/crates/ampel-providers/src/bitbucket.rs`
- `/alt/home/developer/workspace/projects/ampel/crates/ampel-api/src/handlers/pull_requests.rs`
- `/alt/home/developer/workspace/projects/ampel/crates/ampel-api/src/routes/mod.rs`

## Notes

- All implementations follow Rust best practices with proper error handling
- Provider-specific quirks documented and handled appropriately
- OpenAPI schema integration ensures API documentation stays current
- Authentication and authorization consistently applied
