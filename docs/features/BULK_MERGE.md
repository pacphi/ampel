# Bulk Merge Feature

## Overview

Bulk Merge allows you to select multiple pull requests across repositories and merge them all at once with a single API call. This feature significantly speeds up PR management workflows by:

- Merging up to 50 PRs simultaneously
- Automatic retry and error handling
- Pre-flight verification to avoid stale data issues
- Detailed operation tracking and results
- Configurable merge strategies and options

## Architecture

### Database Schema

```
merge_operations
├── id (UUID, primary key)
├── user_id (UUID, references users)
├── started_at (DateTime)
├── completed_at (Optional<DateTime>)
├── total_count (i32)
├── success_count (i32)
├── failed_count (i32)
├── skipped_count (i32)
├── status (String: "in_progress" | "completed" | "failed")
└── notification_sent (bool)

merge_operation_items
├── id (UUID, primary key)
├── merge_operation_id (UUID, references merge_operations)
├── pull_request_id (UUID, references pull_requests)
├── repository_id (UUID, references repositories)
├── status (String: "pending" | "success" | "failed" | "skipped")
├── error_message (Optional<String>)
├── merge_sha (Optional<String>)
└── merged_at (Optional<DateTime>)
```

### Entity Relationships

- **MergeOperation → User**: Each operation belongs to one user
- **MergeOperation ↔ Items**: One-to-many relationship with merge items
- **MergeOperationItem → PullRequest**: References the PR being merged
- **MergeOperationItem → Repository**: Tracks which repository the PR belongs to

## API Endpoints

All bulk merge endpoints are authenticated and located under `/api/merge`.

### Bulk Merge PRs

Merge multiple pull requests in a single operation.

**Endpoint:** `POST /api/merge/bulk`

**Authentication:** Required

**Request Body:**

```json
{
  "pullRequestIds": ["uuid1", "uuid2", "uuid3"],
  "strategy": "squash",
  "deleteBranch": true
}
```

**Parameters:**

- `pullRequestIds` (required): Array of PR UUIDs to merge (max 50)
- `strategy` (optional): Merge strategy - "merge", "squash", or "rebase"
  - Defaults to user's default merge strategy from settings
- `deleteBranch` (optional): Delete source branch after merge
  - Defaults to user's delete branches setting

**Response:** `200 OK`

```json
{
  "success": true,
  "data": {
    "operationId": "uuid",
    "status": "completed",
    "total": 3,
    "success": 2,
    "failed": 1,
    "skipped": 0,
    "results": [
      {
        "pullRequestId": "uuid1",
        "repositoryName": "owner/repo",
        "prNumber": 123,
        "prTitle": "Add feature X",
        "status": "success",
        "errorMessage": null,
        "mergeSha": "abc123..."
      },
      {
        "pullRequestId": "uuid2",
        "repositoryName": "owner/repo",
        "prNumber": 124,
        "prTitle": "Fix bug Y",
        "status": "success",
        "errorMessage": null,
        "mergeSha": "def456..."
      },
      {
        "pullRequestId": "uuid3",
        "repositoryName": "owner/other-repo",
        "prNumber": 50,
        "prTitle": "Update dependencies",
        "status": "failed",
        "errorMessage": "PR has merge conflicts",
        "mergeSha": null
      }
    ]
  }
}
```

**Error Responses:**

- `400 Bad Request`: Invalid input (no PRs, too many PRs, etc.)
- `404 Not Found`: One or more PRs not found or not owned by user
- `401 Unauthorized`: Missing or invalid authentication

### Get Merge Operation

Retrieve details for a specific merge operation.

**Endpoint:** `GET /api/merge/operations/:id`

**Authentication:** Required (must be operation owner)

**Response:**

```json
{
  "success": true,
  "data": {
    "operationId": "uuid",
    "status": "completed",
    "total": 3,
    "success": 2,
    "failed": 1,
    "skipped": 0,
    "results": [...]
  }
}
```

### List Merge Operations

List recent merge operations for the authenticated user.

**Endpoint:** `GET /api/merge/operations`

**Authentication:** Required

**Query Parameters:**

- `page` (optional): Page number (default: 1)
- `perPage` (optional): Results per page (default: 20, max: 100)

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "operationId": "uuid1",
      "status": "completed",
      "total": 5,
      "success": 5,
      "failed": 0,
      "skipped": 0,
      "results": [...]
    },
    {
      "operationId": "uuid2",
      "status": "completed",
      "total": 3,
      "success": 2,
      "failed": 1,
      "skipped": 0,
      "results": [...]
    }
  ]
}
```

## Implementation Details

### Merge Flow

Located in `crates/ampel-api/src/handlers/bulk_merge.rs`:

1. **Validation Phase**
   - Verify PR count (1-50)
   - Fetch user settings for defaults
   - Determine merge strategy and delete branch setting
   - Validate PR ownership and existence

2. **Grouping Phase**
   - Group PRs by repository to optimize API calls
   - Verify all PRs belong to the authenticated user

3. **Operation Creation**
   - Create merge_operation record
   - Create merge_operation_item for each PR

4. **Merge Execution Phase**
   - Process PRs repository by repository
   - Add delay between merges in same repo (configurable via user settings)
   - For each PR:
     - Get provider account and decrypt token
     - **Pre-flight check**: Fetch fresh PR state from provider
     - Verify PR is still open (locally and remotely)
     - Update local state if stale
     - Attempt merge via provider API
     - Update operation item status

5. **Completion Phase**
   - Update operation with final counts
   - Mark operation as completed/failed
   - (Future) Send notification

### Pre-flight Verification

A critical feature added in PR #2 to improve reliability:

```rust
// Pre-flight check: verify PR is still open on the provider
let fresh_pr = provider
    .get_pull_request(&credentials, &repo.owner, &repo.name, pr.number)
    .await?;

// Check if PR is still open (either locally or from fresh data)
if pr.state != "open" || fresh_pr.state != "open" {
    // Update local state if stale
    if pr.state == "open" && fresh_pr.state != "open" {
        PrQueries::update_state(&state.db, pr.id, fresh_pr.state, ...)
            .await?;
    }
    // Skip this PR
}
```

**Why this matters:**

- Prevents attempting to merge already-closed PRs
- Catches cases where local DB is out of sync
- Reduces failed merge attempts
- Provides accurate error messages

### Error Handling

The bulk merge implementation uses robust error handling:

1. **Token Errors**: Caught and reported per-PR
2. **Provider Errors**: Network/API failures tracked individually
3. **Merge Conflicts**: Detected and skipped gracefully
4. **State Mismatches**: Local DB updated to match remote state

**Status Values:**

- `success`: PR merged successfully
- `failed`: Merge attempt failed (with error message)
- `skipped`: PR not eligible for merge (closed, merged, etc.)
- `pending`: Not yet processed (should not appear in final results)

### Merge Delay

To avoid rate limiting and server overload, merges within the same repository are delayed:

```rust
let merge_delay = Duration::from_secs(settings.merge_delay_seconds as u64);

// Add delay between merges in same repo (except first)
if !is_first && merge_delay.as_secs() > 0 {
    sleep(merge_delay).await;
}
```

Configurable via user settings (default: 0 seconds).

### Provider Integration

Uses the provider abstraction from `ampel-providers`:

```rust
let provider = state.provider_factory.create(provider_type, account.instance_url);

provider.merge_pull_request(
    &credentials,
    &repo.owner,
    &repo.name,
    pr.number,
    &merge_request,
).await?;
```

Supports GitHub, GitLab, and Bitbucket through unified interface.

## Usage Examples

### Basic Bulk Merge

```bash
curl -X POST http://localhost:8080/api/merge/bulk \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "pullRequestIds": [
      "uuid-1",
      "uuid-2",
      "uuid-3"
    ],
    "strategy": "squash",
    "deleteBranch": true
  }'
```

### Check Operation Status

```bash
curl http://localhost:8080/api/merge/operations/operation-uuid \
  -H "Authorization: Bearer $TOKEN"
```

### List Recent Operations

```bash
curl http://localhost:8080/api/merge/operations?page=1&perPage=10 \
  -H "Authorization: Bearer $TOKEN"
```

## Frontend Integration

Located in `frontend/src/pages/Merge.tsx`:

### Key Features

1. **PR Selection**
   - Shows mergeable PRs grouped by repository
   - Checkbox selection with select all/deselect all
   - Visual indicators for PR status

2. **Merge Configuration**
   - Strategy selector (merge/squash/rebase)
   - Delete branch toggle
   - Defaults loaded from user settings

3. **Execution Feedback**
   - Loading state during merge
   - Toast notifications for success/failure
   - Detailed results dialog

4. **Automatic Filtering**
   - Only shows PRs ready to merge
   - Respects user's skip review requirement setting
   - Displays blockers for non-mergeable PRs

### React Component Structure

```typescript
// State management
const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
const [strategy, setStrategy] = useState<'merge' | 'squash' | 'rebase'>('squash');
const [deleteBranch, setDeleteBranch] = useState(false);

// Fetch user settings for defaults
const { data: settings } = useQuery({
  queryKey: ['user-settings', 'behavior'],
  queryFn: () => settingsApi.getBehavior(),
});

// Bulk merge mutation
const bulkMergeMutation = useMutation({
  mutationFn: (request: BulkMergeRequest) => mergeApi.bulkMerge(request),
  onSuccess: (data) => {
    setMergeResults(data);
    setShowResults(true);
    queryClient.invalidateQueries({ queryKey: ['pull-requests'] });
  },
});
```

### Mergeable PR Criteria

```typescript
const mergeablePrs = prs.filter((pr) => {
  // Must not be draft and have no conflicts
  if (pr.isDraft || pr.hasConflicts) return false;

  // If green status, always mergeable
  if (pr.status === 'green') return true;

  // If skipReviewRequirement is enabled, check if the only blocker is review-related
  if (skipReviewRequirement && pr.status === 'yellow') {
    const blockers = getBlockers(pr, true);
    return blockers.length === 0; // Mergeable if no other blockers
  }

  return false;
});
```

### Results Dialog

`frontend/src/components/merge/MergeResultsDialog.tsx` displays:

- Overall operation status
- Success/failed/skipped counts
- Per-PR results with error messages
- Links to merged PRs

## User Settings Integration

Bulk merge respects user settings from `user_settings` table:

```rust
let settings = UserSettingsQueries::get_or_create_default(&state.db, auth.user_id).await?;

// Apply defaults
let merge_strategy = req.strategy.unwrap_or(match settings.default_merge_strategy.as_str() {
    "merge" => MergeStrategy::Merge,
    "rebase" => MergeStrategy::Rebase,
    _ => MergeStrategy::Squash,
});

let delete_branch = req.delete_branch.unwrap_or(settings.delete_branches_default);
let merge_delay = Duration::from_secs(settings.merge_delay_seconds as u64);
```

**Related Settings:**

- `default_merge_strategy`: Default merge strategy
- `delete_branches_default`: Auto-delete branches after merge
- `merge_delay_seconds`: Delay between merges in same repo

## Database Queries

Located in `crates/ampel-db/src/queries/merge_operation_queries.rs`:

### Creating Operations

```rust
pub async fn create(
    db: &DatabaseConnection,
    user_id: Uuid,
    total_count: i32,
) -> Result<merge_operation::Model, DbErr> {
    let operation = merge_operation::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        started_at: Set(Utc::now()),
        completed_at: Set(None),
        total_count: Set(total_count),
        success_count: Set(0),
        failed_count: Set(0),
        skipped_count: Set(0),
        status: Set("in_progress".to_string()),
        notification_sent: Set(false),
    };

    operation.insert(db).await
}
```

### Updating Counts

```rust
pub async fn update_counts(
    db: &DatabaseConnection,
    operation_id: Uuid,
    success: i32,
    failed: i32,
    skipped: i32,
    status: &str,
) -> Result<merge_operation::Model, DbErr> {
    let mut operation: merge_operation::ActiveModel =
        merge_operation::Entity::find_by_id(operation_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Operation not found".to_string()))?
            .into();

    operation.success_count = Set(success);
    operation.failed_count = Set(failed);
    operation.skipped_count = Set(skipped);
    operation.status = Set(status.to_string());
    operation.completed_at = Set(Some(Utc::now()));

    operation.update(db).await
}
```

### Item Operations

```rust
pub async fn create(
    db: &DatabaseConnection,
    operation_id: Uuid,
    pr_id: Uuid,
    repo_id: Uuid,
) -> Result<merge_operation_item::Model, DbErr> {
    let item = merge_operation_item::ActiveModel {
        id: Set(Uuid::new_v4()),
        merge_operation_id: Set(operation_id),
        pull_request_id: Set(pr_id),
        repository_id: Set(repo_id),
        status: Set("pending".to_string()),
        error_message: Set(None),
        merge_sha: Set(None),
        merged_at: Set(None),
    };

    item.insert(db).await
}

pub async fn update_status(
    db: &DatabaseConnection,
    item_id: Uuid,
    status: &str,
    error_message: Option<String>,
    merge_sha: Option<String>,
) -> Result<merge_operation_item::Model, DbErr> {
    // Implementation
}
```

## Testing

### Backend Tests

```rust
#[tokio::test]
async fn test_bulk_merge_success() {
    let state = create_test_state().await;
    let user = create_test_user(&state.db).await;
    let repo = create_test_repository(&state.db, user.id).await;
    let pr1 = create_test_pr(&state.db, repo.id, "open").await;
    let pr2 = create_test_pr(&state.db, repo.id, "open").await;

    let req = BulkMergeRequest {
        pull_request_ids: vec![pr1.id, pr2.id],
        strategy: Some("squash".to_string()),
        delete_branch: Some(true),
    };

    let response = bulk_merge(
        State(state),
        AuthUser { user_id: user.id },
        Json(req)
    ).await;

    assert!(response.is_ok());
    let json = response.unwrap().0;
    assert_eq!(json.data.unwrap().total, 2);
}
```

### Frontend Tests

```typescript
describe('Merge page', () => {
  it('should allow selecting PRs', async () => {
    render(<Merge />);
    const checkbox = await screen.findByRole('checkbox');
    fireEvent.click(checkbox);
    expect(checkbox).toBeChecked();
  });

  it('should call bulk merge API', async () => {
    const mockMerge = jest.fn();
    render(<Merge />);
    // Select PRs and click merge button
    // Verify mockMerge was called with correct params
  });
});
```

## Performance Considerations

### Optimization Strategies

1. **Repository Grouping**: PRs from same repo are processed together
2. **Parallel Fetching**: Initial PR validation uses parallel queries
3. **Configurable Delays**: Prevents rate limiting
4. **Early Validation**: Checks ownership and existence before processing

### Limits

- **Maximum PRs**: 50 per operation
- **Default Delay**: 0 seconds between same-repo merges
- **Timeout**: Standard HTTP timeout (60s)

### Scalability

For very large operations:

- Consider moving to background job queue (Apalis)
- Implement operation status polling
- Add webhook notifications for completion

## Troubleshooting

### Common Issues

**Problem:** Merge fails with "PR not found"

- **Solution:** PR may have been closed/merged externally. Pre-flight check will detect this.

**Problem:** Token decryption error

- **Solution:** Provider account may be invalid. Re-authenticate the account.

**Problem:** Some PRs skipped

- **Solution:** Check error messages. PRs may be closed, merged, or have conflicts.

**Problem:** Operation stuck in "in_progress"

- **Solution:** Check server logs for errors. May need to manually update operation status.

**Problem:** Rate limiting errors

- **Solution:** Increase merge_delay_seconds in user settings.

### Debug Logging

Enable detailed logging:

```bash
export RUST_LOG=debug,ampel_api::handlers::bulk_merge=trace
make dev-api
```

## Future Enhancements

### Planned Features

1. **Background Processing**
   - Move to Apalis job queue for async processing
   - Support operations >50 PRs
   - Better progress tracking

2. **Advanced Scheduling**
   - Schedule merges for specific time
   - Recurring bulk merge patterns
   - Time-based merge windows

3. **Conditional Merging**
   - Wait for specific CI checks
   - Dependency-based ordering
   - Auto-merge on green status

4. **Notifications**
   - Email/Slack notifications on completion
   - Per-PR failure notifications
   - Daily summary reports

5. **Analytics**
   - Merge success rates
   - Time-to-merge metrics
   - Failure pattern analysis

## Related Files

- Backend:
  - `crates/ampel-api/src/handlers/bulk_merge.rs` (main handler)
  - `crates/ampel-db/src/entities/merge_operation.rs`
  - `crates/ampel-db/src/entities/merge_operation_item.rs`
  - `crates/ampel-db/src/queries/merge_operation_queries.rs`
  - `crates/ampel-api/src/routes/mod.rs`

- Frontend:
  - `frontend/src/pages/Merge.tsx`
  - `frontend/src/components/merge/MergeResultsDialog.tsx`
  - `frontend/src/api/merge.ts`
  - `frontend/src/components/settings/BehaviorSettings.tsx`
