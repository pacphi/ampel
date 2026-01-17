# Data Transformation Flow: Git Diff Integration

**Document Version:** 1.0
**Date:** 2025-12-25
**Status:** Architecture Design

## Overview

This document describes the complete data transformation pipeline for git diff integration in Ampel, from provider API responses to rendered UI components.

## Transformation Stages

```
┌─────────────────────────────────────────────────────────────────────┐
│                      DATA TRANSFORMATION PIPELINE                     │
└─────────────────────────────────────────────────────────────────────┘

[Provider API]
     │
     ├── GitHub:   GET /repos/{owner}/{repo}/pulls/{number}/files
     ├── GitLab:   GET /projects/{id}/merge_requests/{iid}/changes
     └── Bitbucket: GET /repositories/{workspace}/{slug}/pullrequests/{id}/diffstat
     │
     ▼
┌─────────────────────────────────────────────────────────────────────┐
│ STAGE 1: Provider-Specific Parsing                                  │
│ Location: crates/ampel-providers/src/{github,gitlab,bitbucket}.rs   │
└─────────────────────────────────────────────────────────────────────┘
     │
     ├── Parse JSON response → Provider-specific structs
     ├── Handle provider quirks (renamed files, binary files)
     └── Extract commit SHAs, file paths, patch strings
     │
     ▼
┌─────────────────────────────────────────────────────────────────────┐
│ STAGE 2: Normalization to Unified Model                             │
│ Location: crates/ampel-providers/src/traits.rs                      │
└─────────────────────────────────────────────────────────────────────┘
     │
     ├── Transform to ProviderDiffFile structs
     ├── Normalize status values (added/deleted/modified/renamed)
     ├── Calculate aggregate statistics (total additions/deletions)
     └── Detect file language from extension
     │
     ▼
┌─────────────────────────────────────────────────────────────────────┐
│ STAGE 3: API Response Serialization                                 │
│ Location: crates/ampel-api/src/handlers/pull_requests.rs            │
└─────────────────────────────────────────────────────────────────────┘
     │
     ├── Convert ProviderDiff → DiffResponse (JSON)
     ├── Add metadata (provider, fetchedAt timestamp)
     ├── Cache in Redis (5-60 min TTL)
     └── Return HTTP 200 with JSON body
     │
     ▼
┌─────────────────────────────────────────────────────────────────────┐
│ STAGE 4: Frontend Deserialization                                   │
│ Location: frontend/src/hooks/usePullRequestDiff.ts                  │
└─────────────────────────────────────────────────────────────────────┘
     │
     ├── TanStack Query fetches JSON
     ├── Parse to PullRequestDiff TypeScript interface
     ├── Cache in React Query cache (5-10 min)
     └── Trigger re-render
     │
     ▼
┌─────────────────────────────────────────────────────────────────────┐
│ STAGE 5: Component Rendering                                        │
│ Location: frontend/src/components/diff/*.tsx                        │
└─────────────────────────────────────────────────────────────────────┘
     │
     ├── DiffFileList: Render file tree
     ├── DiffFileItem: Render each file header
     ├── DiffView: @git-diff-view/react renders patch
     └── Syntax highlighting applied
     │
     ▼
[User's Browser]
```

## Stage Details

### Stage 1: Provider-Specific Parsing

**GitHub Example:**

```rust
// crates/ampel-providers/src/github.rs

#[derive(Debug, Deserialize)]
struct GitHubFile {
    sha: String,
    filename: String,
    previous_filename: Option<String>,
    status: String, // "added", "removed", "modified", "renamed"
    additions: i32,
    deletions: i32,
    changes: i32,
    patch: Option<String>,
    blob_url: String,
    raw_url: String,
}

impl GitHubProvider {
    async fn fetch_pr_files(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<GitHubFile>> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}/files",
            self.base_url, owner, repo, pr_number
        );

        let response = self.client
            .get(&url)
            .bearer_auth(credentials.token())
            .send()
            .await?;

        let files: Vec<GitHubFile> = response.json().await?;
        Ok(files)
    }
}
```

**Data Example (GitHub):**

```json
{
  "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
  "filename": "src/components/Button.tsx",
  "status": "modified",
  "additions": 50,
  "deletions": 10,
  "changes": 60,
  "patch": "@@ -1,4 +1,5 @@\n import React from 'react';\n+import { cn } from '@/lib/utils';\n..."
}
```

**GitLab Example:**

```rust
// crates/ampel-providers/src/gitlab.rs

#[derive(Debug, Deserialize)]
struct GitLabChange {
    old_path: String,
    new_path: String,
    a_mode: String,
    b_mode: String,
    new_file: bool,
    renamed_file: bool,
    deleted_file: bool,
    diff: String,
}

// GitLab uses flags instead of single status field
fn determine_status(change: &GitLabChange) -> String {
    if change.new_file {
        "added".to_string()
    } else if change.deleted_file {
        "deleted".to_string()
    } else if change.renamed_file {
        "renamed".to_string()
    } else {
        "modified".to_string()
    }
}
```

**Bitbucket Cloud Example:**

```rust
// crates/ampel-providers/src/bitbucket.rs

#[derive(Debug, Deserialize)]
struct BitbucketDiffStat {
    #[serde(rename = "type")]
    diff_type: String,
    status: String, // "added", "removed", "modified"
    lines_removed: i32,
    lines_added: i32,
    old: Option<BitbucketFileInfo>,
    new: BitbucketFileInfo,
}

// Bitbucket requires second API call for actual patch
async fn fetch_file_patch(
    &self,
    workspace: &str,
    repo: &str,
    pr_id: i32,
    file_path: &str,
) -> ProviderResult<String> {
    let url = format!(
        "{}/repositories/{}/{}/pullrequests/{}/diff/{}",
        self.base_url, workspace, repo, pr_id, file_path
    );

    let patch = self.client.get(&url).send().await?.text().await?;
    Ok(patch)
}
```

### Stage 2: Normalization to Unified Model

**Unified Struct:**

```rust
// crates/ampel-providers/src/traits.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDiffFile {
    /// Git object SHA for this file
    pub sha: String,

    /// Original file path (for renames/deletes)
    pub old_path: Option<String>,

    /// New file path
    pub new_path: String,

    /// File status
    pub status: FileStatus,

    /// Lines added
    pub additions: i32,

    /// Lines deleted
    pub deletions: i32,

    /// Total changes
    pub changes: i32,

    /// Unified diff patch
    pub patch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
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
```

**Transformation Example (GitHub → Unified):**

```rust
impl From<GitHubFile> for ProviderDiffFile {
    fn from(file: GitHubFile) -> Self {
        Self {
            sha: file.sha,
            old_path: file.previous_filename,
            new_path: file.filename,
            status: match file.status.as_str() {
                "added" => FileStatus::Added,
                "removed" => FileStatus::Deleted,
                "modified" => FileStatus::Modified,
                "renamed" => FileStatus::Renamed,
                _ => FileStatus::Modified, // Fallback
            },
            additions: file.additions,
            deletions: file.deletions,
            changes: file.changes,
            patch: file.patch.unwrap_or_default(),
        }
    }
}

// GitLab transformation
impl From<GitLabChange> for ProviderDiffFile {
    fn from(change: GitLabChange) -> Self {
        let status = if change.new_file {
            FileStatus::Added
        } else if change.deleted_file {
            FileStatus::Deleted
        } else if change.renamed_file {
            FileStatus::Renamed
        } else {
            FileStatus::Modified
        };

        // Parse diff to count lines
        let (additions, deletions) = count_diff_lines(&change.diff);

        Self {
            sha: hash_file_content(&change.diff), // Generate SHA from content
            old_path: if change.renamed_file {
                Some(change.old_path.clone())
            } else {
                None
            },
            new_path: change.new_path,
            status,
            additions,
            deletions,
            changes: additions + deletions,
            patch: change.diff,
        }
    }
}
```

### Stage 3: API Response Serialization

**Backend Handler:**

```rust
// crates/ampel-api/src/handlers/pull_requests.rs

#[derive(Debug, Serialize)]
pub struct DiffResponse {
    pub pull_request_id: String,
    pub provider: String,
    pub files: Vec<DiffFileDto>,
    pub total_additions: i32,
    pub total_deletions: i32,
    pub total_files: i32,
    pub base_commit: String,
    pub head_commit: String,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DiffFileDto {
    pub id: String, // SHA
    pub old_path: Option<String>,
    pub new_path: String,
    pub status: String, // "added" | "deleted" | "modified" | "renamed" | "copied"
    pub additions: i32,
    pub deletions: i32,
    pub changes: i32,
    pub patch: String,
    pub language: Option<String>,
}

impl From<ProviderDiff> for DiffResponse {
    fn from(diff: ProviderDiff) -> Self {
        Self {
            files: diff.files.into_iter().map(|f| {
                DiffFileDto {
                    id: f.sha.clone(),
                    old_path: f.old_path,
                    new_path: f.new_path.clone(),
                    status: f.status.to_string(),
                    additions: f.additions,
                    deletions: f.deletions,
                    changes: f.changes,
                    patch: f.patch,
                    language: detect_language(&f.new_path),
                }
            }).collect(),
            total_additions: diff.total_additions,
            total_deletions: diff.total_deletions,
            total_files: diff.total_files,
            base_commit: diff.base_commit,
            head_commit: diff.head_commit,
            fetched_at: Utc::now(),
        }
    }
}

fn detect_language(filename: &str) -> Option<String> {
    let ext = filename.split('.').last()?;
    let lang_map = [
        ("ts", "typescript"),
        ("tsx", "tsx"),
        ("js", "javascript"),
        ("jsx", "jsx"),
        ("rs", "rust"),
        ("py", "python"),
        // ... more mappings
    ];

    lang_map.iter()
        .find(|(e, _)| e == &ext)
        .map(|(_, lang)| lang.to_string())
}
```

**JSON Response Example:**

```json
{
  "pullRequestId": "pr_123abc",
  "provider": "github",
  "files": [
    {
      "id": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
      "oldPath": null,
      "newPath": "src/components/Button.tsx",
      "status": "modified",
      "additions": 50,
      "deletions": 10,
      "changes": 60,
      "patch": "@@ -1,4 +1,5 @@\n import React from 'react';\n+import { cn } from '@/lib/utils';\n...",
      "language": "typescript"
    }
  ],
  "totalAdditions": 104,
  "totalDeletions": 23,
  "totalFiles": 5,
  "baseCommit": "abc123",
  "headCommit": "def456",
  "fetchedAt": "2025-12-25T10:00:00Z"
}
```

### Stage 4: Frontend Deserialization

**TypeScript Interfaces:**

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
  language?: string;
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

**TanStack Query Hook:**

```typescript
// frontend/src/hooks/usePullRequestDiff.ts

export function usePullRequestDiff(pullRequestId: string) {
  return useQuery<PullRequestDiff>({
    queryKey: ['pull-request-diff', pullRequestId],
    queryFn: async () => {
      const response = await api.get<ApiResponse<PullRequestDiff>>(
        `/api/v1/pull-requests/${pullRequestId}/diff`
      );

      const data = response.data.data!;

      // Transform fetchedAt string to Date object
      return {
        ...data,
        fetchedAt: new Date(data.fetchedAt),
      };
    },
    staleTime: 5 * 60 * 1000,
    cacheTime: 10 * 60 * 1000,
  });
}
```

### Stage 5: Component Rendering

**Component Tree:**

```tsx
// frontend/src/components/diff/FilesChangedTab.tsx

export function FilesChangedTab({ pullRequestId }: { pullRequestId: string }) {
  const { data: diff, isLoading } = usePullRequestDiff(pullRequestId);

  if (isLoading) return <DiffSkeleton />;
  if (!diff) return null;

  return (
    <div className="files-changed-tab">
      <DiffToolbar
        totalFiles={diff.totalFiles}
        additions={diff.totalAdditions}
        deletions={diff.totalDeletions}
      />

      <DiffFileList files={diff.files} />
    </div>
  );
}

// DiffFileList renders each file
function DiffFileList({ files }: { files: DiffFile[] }) {
  return (
    <div className="diff-file-list">
      {files.map((file) => (
        <DiffFileItem key={file.id} file={file} />
      ))}
    </div>
  );
}

// DiffFileItem renders file header + collapsible diff
function DiffFileItem({ file }: { file: DiffFile }) {
  const [isExpanded, setIsExpanded] = useState(true);

  return (
    <div className="diff-file-item">
      <DiffFileHeader
        file={file}
        isExpanded={isExpanded}
        onToggle={() => setIsExpanded(!isExpanded)}
      />

      {isExpanded && (
        <DiffView
          data={file.patch}
          diffType="unified"
          extendData={{
            oldFile: { fileName: file.oldPath || '', language: file.language },
            newFile: { fileName: file.newPath, language: file.language },
          }}
        />
      )}
    </div>
  );
}
```

**Final Rendered Output:**

```html
<div class="files-changed-tab">
  <!-- Toolbar with stats -->
  <div class="diff-toolbar">
    <span>5 files changed</span>
    <span class="text-green">+104 additions</span>
    <span class="text-red">-23 deletions</span>
  </div>

  <!-- File list -->
  <div class="diff-file-list">
    <!-- File item -->
    <div class="diff-file-item">
      <!-- File header -->
      <button class="file-header" aria-expanded="true">
        <span class="file-status-icon modified">M</span>
        <span class="file-path">src/components/Button.tsx</span>
        <span class="file-stats">+50 -10</span>
        <chevron-down-icon />
      </button>

      <!-- Diff content (from @git-diff-view/react) -->
      <div class="diff-view">
        <table class="diff-table">
          <tr class="diff-line diff-line-normal">
            <td class="line-number">1</td>
            <td class="line-content">
              <code>import React from 'react';</code>
            </td>
          </tr>
          <tr class="diff-line diff-line-add">
            <td class="line-number">2</td>
            <td class="line-content">
              <code
                ><span class="syntax-import">import</span> { cn }
                <span class="syntax-import">from</span>
                <span class="syntax-string">'@/lib/utils'</span>;</code
              >
            </td>
          </tr>
          <!-- ... more lines ... -->
        </table>
      </div>
    </div>
  </div>
</div>
```

## Performance Optimizations

### 1. Lazy Loading

Only load diffs when "Files Changed" tab is clicked:

```tsx
const FilesChangedTab = lazy(() => import('./FilesChangedTab'));

<Tabs>
  <TabPanel value="files">
    <Suspense fallback={<DiffSkeleton />}>
      <FilesChangedTab pullRequestId={prId} />
    </Suspense>
  </TabPanel>
</Tabs>;
```

### 2. Virtual Scrolling

For PRs with 100+ files, use react-window:

```tsx
import { FixedSizeList } from 'react-window';

<FixedSizeList height={600} itemCount={files.length} itemSize={400} width="100%">
  {({ index, style }) => (
    <div style={style}>
      <DiffFileItem file={files[index]} />
    </div>
  )}
</FixedSizeList>;
```

### 3. Memoization

Prevent unnecessary re-renders:

```tsx
const DiffFileItem = memo(
  ({ file }: { file: DiffFile }) => {
    // Component implementation
  },
  (prevProps, nextProps) => prevProps.file.id === nextProps.file.id
);
```

## Error Handling at Each Stage

| Stage       | Error Type             | Handling                                   |
| ----------- | ---------------------- | ------------------------------------------ |
| **Stage 1** | Network timeout        | Retry with exponential backoff             |
| **Stage 1** | 401 Unauthorized       | Return error → prompt user to update token |
| **Stage 1** | 429 Rate limit         | Return error → show retry countdown        |
| **Stage 2** | Parse error            | Log warning, skip file, continue           |
| **Stage 2** | Missing required field | Use default value, log warning             |
| **Stage 3** | Serialization error    | Return 500, log detailed error             |
| **Stage 4** | Network error          | TanStack Query automatic retry (3x)        |
| **Stage 4** | JSON parse error       | Show error toast, allow manual refresh     |
| **Stage 5** | Render error           | ErrorBoundary catches, shows fallback UI   |

## Monitoring Points

```typescript
// Track transformation latency at each stage
histogram!("ampel_diff_stage1_parse_duration_seconds");
histogram!("ampel_diff_stage2_normalize_duration_seconds");
histogram!("ampel_diff_stage3_serialize_duration_seconds");

// Track transformation success rates
counter!("ampel_diff_transformation_errors_total", "stage" => "parse");
counter!("ampel_diff_transformation_success_total");

// Track data size
histogram!("ampel_diff_file_count");
histogram!("ampel_diff_patch_size_bytes");
```

## Related Documents

- [ADR-002: Provider Diff Abstraction](/docs/architecture/git-diff-integration/ADR-002-provider-diff-abstraction.md)
- [Git Diff View Integration Plan](/docs/planning/GIT_DIFF_VIEW_INTEGRATION.md)
