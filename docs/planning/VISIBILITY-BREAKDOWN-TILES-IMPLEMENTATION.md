# Technical Implementation Plan: Repository Visibility Breakdown Tiles

**Document Version**: 1.0
**Created**: 2025-12-24
**Status**: Planning
**Target Branch**: feature/visibility-breakdown-tiles

---

## Executive Summary

This document outlines the implementation plan for adding a second row of dashboard tiles that display repository and PR counts broken down by visibility type (public, private, archived). These new tiles will appear below the existing top-level summary tiles, providing users with granular insights into their repository portfolio composition.

### Key Deliverables

1. **New Breakdown Tiles Row**: Add 4 tiles showing visibility breakdowns for:
   - Total Repositories (by public/private/archived)
   - Open PRs (by repository visibility)
   - Ready to Merge (by repository visibility)
   - Needs Attention (by repository visibility)

2. **Backend API Extensions**: Extend `/api/dashboard/summary` endpoint to return visibility breakdowns
3. **Reusable UI Components**: Create BreakdownTile component for consistent visualization
4. **Consistent Iconography**: Use Lock (private), Globe (public), Archive (archived) icons
5. **Loading & Error States**: Handle data loading and error scenarios gracefully

### Success Criteria

- Dashboard displays 8 total tiles (4 top-level + 4 breakdown tiles)
- Breakdown tiles use existing visibility icons and color scheme
- Data accurately reflects repository visibility distribution
- Mobile responsive design maintains readability
- No performance degradation (API response < 500ms)

---

## 1. Current State Analysis

### 1.1 Existing Dashboard Architecture

**File**: `/alt/home/developer/workspace/projects/ampel/frontend/src/pages/Dashboard.tsx`

The current dashboard displays 4 summary cards:

1. **Total Repositories** - Shows count of all repositories
2. **Open PRs** - Shows count of all open pull requests
3. **Ready to Merge** - Shows count of green-status PRs (calculated client-side)
4. **Needs Attention** - Shows count of red-status PRs

**Layout Structure**:

```tsx
// Lines 127-168 in Dashboard.tsx
<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
  <Card>Total Repositories</Card>
  <Card>Open PRs</Card>
  <Card>Ready to Merge</Card>
  <Card>Needs Attention</Card>
</div>
```

### 1.2 Existing Data Models

**Backend** (`crates/ampel-api/src/handlers/dashboard.rs`):

```rust
pub struct DashboardSummary {
    pub total_repositories: i32,
    pub total_open_prs: i32,
    pub status_counts: StatusCounts,      // green, yellow, red
    pub provider_counts: ProviderCounts,  // github, gitlab, bitbucket
}
```

**Frontend** (`frontend/src/types/index.ts`):

```typescript
export interface DashboardSummary {
  totalRepositories: number;
  totalOpenPrs: number;
  statusCounts: { green: number; yellow: number; red: number };
  providerCounts: { github: number; gitlab: number; bitbucket: number };
}
```

**Repository Fields Available**:

- `isPrivate: boolean` - True for private repositories
- `isArchived: boolean` - True for archived repositories (always false for Bitbucket)

### 1.3 Existing Icon System

**File**: `/alt/home/developer/workspace/projects/ampel/frontend/src/components/dashboard/RepositoryStatusIcons.tsx`

Already implements visibility icons:

- **Public**: `<Globe className="text-green-600" />`
- **Private**: `<Lock className="text-amber-600" />`
- **Archived**: `<Archive className="text-gray-500" />`

These icons are currently used in RepoCard and ListView components.

### 1.4 API Endpoint Analysis

**Current Endpoint**: `GET /api/dashboard/summary`

**Query Performance**:

```rust
// Lines 37-103 in crates/ampel-api/src/handlers/dashboard.rs
// Iterates through all repos and PRs
// Calculates status counts by fetching CI checks and reviews
// Current implementation: O(n*m) where n=repos, m=PRs per repo
```

---

## 2. Goal State Architecture

### 2.1 Enhanced Dashboard Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Dashboard Header                                 [Refresh] │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │  Total  │  │  Open   │  │ Ready   │  │ Needs   │       │
│  │  Repos  │  │   PRs   │  │ to Merge│  │Attention│       │
│  │   42    │  │   15    │  │    8    │  │    3    │       │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
│                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────┐│
│  │Total Repos  │  │ Open PRs    │  │Ready to Mrg │  │Needs││
│  │ Breakdown   │  │ Breakdown   │  │ Breakdown   │  │Attn ││
│  │             │  │             │  │             │  │Brkdn││
│  │🌐 Public:20 │  │🌐 Public: 8 │  │🌐 Public: 4 │  │🌐 1 ││
│  │🔒 Private:18│  │🔒 Private:5 │  │🔒 Private:3 │  │🔒 2 ││
│  │📦 Archived:4│  │📦 Archived:2│  │📦 Archived:1│  │📦 0 ││
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────┘│
│                                                               │
│  [Grid View of Repositories]                                 │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Enhanced Data Model

**Backend Extension**:

```rust
// Add to crates/ampel-api/src/handlers/dashboard.rs
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityBreakdown {
    pub public: i32,
    pub private: i32,
    pub archived: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub total_repositories: i32,
    pub total_open_prs: i32,
    pub status_counts: StatusCounts,
    pub provider_counts: ProviderCounts,

    // NEW FIELDS
    pub repository_breakdown: VisibilityBreakdown,
    pub open_prs_breakdown: VisibilityBreakdown,
    pub ready_to_merge_breakdown: VisibilityBreakdown,
    pub needs_attention_breakdown: VisibilityBreakdown,
}
```

**Frontend Extension**:

```typescript
// Add to frontend/src/types/index.ts
export interface VisibilityBreakdown {
  public: number;
  private: number;
  archived: number;
}

export interface DashboardSummary {
  totalRepositories: number;
  totalOpenPrs: number;
  statusCounts: { green: number; yellow: number; red: number };
  providerCounts: { github: number; gitlab: number; bitbucket: number };

  // NEW FIELDS
  repositoryBreakdown: VisibilityBreakdown;
  openPrsBreakdown: VisibilityBreakdown;
  readyToMergeBreakdown: VisibilityBreakdown;
  needsAttentionBreakdown: VisibilityBreakdown;
}
```

---

## 3. Detailed Implementation Plan

### 3.1 Milestone 1: Backend API Enhancement

**Objective**: Extend `/api/dashboard/summary` to include visibility breakdowns

**Success Criteria**:

- API returns all 4 visibility breakdowns
- Response time remains < 500ms for 100 repos
- Breakdown totals match top-level counts

#### Step 1.1: Update Rust Data Models

**File**: `crates/ampel-api/src/handlers/dashboard.rs`

```rust
// Add new struct (insert after ProviderCounts)
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityBreakdown {
    pub public: i32,
    pub private: i32,
    pub archived: i32,
}

impl Default for VisibilityBreakdown {
    fn default() -> Self {
        Self {
            public: 0,
            private: 0,
            archived: 0,
        }
    }
}

// Update DashboardSummary struct (lines 11-18)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub total_repositories: i32,
    pub total_open_prs: i32,
    pub status_counts: StatusCounts,
    pub provider_counts: ProviderCounts,

    // NEW FIELDS
    pub repository_breakdown: VisibilityBreakdown,
    pub open_prs_breakdown: VisibilityBreakdown,
    pub ready_to_merge_breakdown: VisibilityBreakdown,
    pub needs_attention_breakdown: VisibilityBreakdown,
}
```

#### Step 1.2: Update get_summary Handler Logic

**File**: `crates/ampel-api/src/handlers/dashboard.rs` (lines 37-104)

```rust
pub async fn get_summary(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<DashboardSummary>>, ApiError> {
    let repos = RepoQueries::find_by_user_id(&state.db, auth.user_id).await?;

    let mut total_open_prs = 0;
    let mut green_count = 0;
    let mut yellow_count = 0;
    let mut red_count = 0;
    let mut github_count = 0;
    let mut gitlab_count = 0;
    let mut bitbucket_count = 0;

    // NEW: Initialize visibility breakdown counters
    let mut repo_breakdown = VisibilityBreakdown::default();
    let mut open_prs_breakdown = VisibilityBreakdown::default();
    let mut ready_breakdown = VisibilityBreakdown::default();
    let mut needs_attention_breakdown = VisibilityBreakdown::default();

    for repo in &repos {
        // NEW: Count repository by visibility
        if repo.is_archived {
            repo_breakdown.archived += 1;
        } else if repo.is_private {
            repo_breakdown.private += 1;
        } else {
            repo_breakdown.public += 1;
        }

        // Get all open PRs for this repository
        let open_prs = PrQueries::find_open_by_repository(&state.db, repo.id).await?;
        total_open_prs += open_prs.len() as i32;

        // Count by provider (existing code)
        match repo.provider.as_str() {
            "github" => github_count += 1,
            "gitlab" => gitlab_count += 1,
            "bitbucket" => bitbucket_count += 1,
            _ => {}
        }

        // Calculate actual PR statuses based on CI checks and reviews
        for pr_model in &open_prs {
            // NEW: Count open PRs by repo visibility
            if repo.is_archived {
                open_prs_breakdown.archived += 1;
            } else if repo.is_private {
                open_prs_breakdown.private += 1;
            } else {
                open_prs_breakdown.public += 1;
            }

            // Load CI checks and reviews for this PR (existing code)
            let ci_checks = CICheckQueries::find_by_pull_request(&state.db, pr_model.id).await?;
            let reviews = ReviewQueries::find_by_pull_request(&state.db, pr_model.id).await?;

            // Convert database models to core models (existing code)
            let pr: ampel_core::models::PullRequest = pr_model.clone().into();
            let ci_checks: Vec<ampel_core::models::CICheck> =
                ci_checks.into_iter().map(|c| c.into()).collect();
            let reviews: Vec<ampel_core::models::Review> =
                reviews.into_iter().map(|r| r.into()).collect();

            // Calculate status using the actual logic (existing code)
            let status = AmpelStatus::for_pull_request(&pr, &ci_checks, &reviews);

            // Count by status (existing code)
            match status {
                AmpelStatus::Green => {
                    green_count += 1;
                    // NEW: Count ready to merge by repo visibility
                    if repo.is_archived {
                        ready_breakdown.archived += 1;
                    } else if repo.is_private {
                        ready_breakdown.private += 1;
                    } else {
                        ready_breakdown.public += 1;
                    }
                }
                AmpelStatus::Yellow => yellow_count += 1,
                AmpelStatus::Red => {
                    red_count += 1;
                    // NEW: Count needs attention by repo visibility
                    if repo.is_archived {
                        needs_attention_breakdown.archived += 1;
                    } else if repo.is_private {
                        needs_attention_breakdown.private += 1;
                    } else {
                        needs_attention_breakdown.public += 1;
                    }
                }
                AmpelStatus::None => {}
            }
        }
    }

    Ok(Json(ApiResponse::success(DashboardSummary {
        total_repositories: repos.len() as i32,
        total_open_prs,
        status_counts: StatusCounts {
            green: green_count,
            yellow: yellow_count,
            red: red_count,
        },
        provider_counts: ProviderCounts {
            github: github_count,
            gitlab: gitlab_count,
            bitbucket: bitbucket_count,
        },
        // NEW: Add visibility breakdowns
        repository_breakdown: repo_breakdown,
        open_prs_breakdown,
        ready_to_merge_breakdown: ready_breakdown,
        needs_attention_breakdown,
    })))
}
```

#### Step 1.3: Test Backend Changes

**Test File**: `crates/ampel-api/tests/integration/dashboard_tests.rs`

```rust
#[tokio::test]
async fn test_dashboard_summary_visibility_breakdowns() {
    let state = setup_test_state().await;
    let user = create_test_user(&state.db).await;

    // Create test repositories with different visibility
    create_test_repo(&state.db, user.id, true, false).await;   // private
    create_test_repo(&state.db, user.id, false, false).await;  // public
    create_test_repo(&state.db, user.id, false, true).await;   // archived

    let response = get_summary(State(state), AuthUser { user_id: user.id })
        .await
        .unwrap();

    let summary = response.0.data.unwrap();

    // Verify breakdown counts
    assert_eq!(summary.repository_breakdown.public, 1);
    assert_eq!(summary.repository_breakdown.private, 1);
    assert_eq!(summary.repository_breakdown.archived, 1);
    assert_eq!(summary.total_repositories, 3);
}
```

**Expected Results**:

- ✅ All breakdown fields are present in response
- ✅ Breakdown totals sum to top-level counts
- ✅ Public + Private + Archived = Total Repositories
- ✅ Response time < 500ms with 100 repos

---

### 3.2 Milestone 2: Frontend Type Updates

**Objective**: Update TypeScript types to match new backend structure

**Success Criteria**:

- TypeScript compilation succeeds
- No type errors in IDE
- API client correctly typed

#### Step 2.1: Update Types

**File**: `frontend/src/types/index.ts`

```typescript
// Add new interface (insert after line 135)
export interface VisibilityBreakdown {
  public: number;
  private: number;
  archived: number;
}

// Update DashboardSummary interface (lines 122-135)
export interface DashboardSummary {
  totalRepositories: number;
  totalOpenPrs: number;
  statusCounts: {
    green: number;
    yellow: number;
    red: number;
  };
  providerCounts: {
    github: number;
    gitlab: number;
    bitbucket: number;
  };

  // NEW FIELDS
  repositoryBreakdown: VisibilityBreakdown;
  openPrsBreakdown: VisibilityBreakdown;
  readyToMergeBreakdown: VisibilityBreakdown;
  needsAttentionBreakdown: VisibilityBreakdown;
}
```

**Expected Results**:

- ✅ `npm run typecheck` passes
- ✅ No TypeScript errors in VSCode
- ✅ API response correctly typed

---

### 3.3 Milestone 3: Create BreakdownTile Component

**Objective**: Build reusable component for displaying visibility breakdowns

**Success Criteria**:

- Component displays all 3 visibility counts
- Icons and colors match existing design
- Responsive on mobile devices
- Loading state implemented

#### Step 3.1: Create BreakdownTile Component

**File**: `frontend/src/components/dashboard/BreakdownTile.tsx` (NEW FILE)

```typescript
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Globe, Lock, Archive, LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { VisibilityBreakdown } from '@/types';

interface BreakdownTileProps {
  title: string;
  breakdown: VisibilityBreakdown;
  icon: LucideIcon;
  isLoading?: boolean;
}

export default function BreakdownTile({
  title,
  breakdown,
  icon: Icon,
  isLoading,
}: BreakdownTileProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-4">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary" />
          </div>
        ) : (
          <div className="space-y-2">
            {/* Public Count */}
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-2">
                <Globe className="h-3.5 w-3.5 text-green-600" />
                <span className="text-muted-foreground">Public</span>
              </div>
              <span className="font-semibold">{breakdown.public}</span>
            </div>

            {/* Private Count */}
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-2">
                <Lock className="h-3.5 w-3.5 text-amber-600" />
                <span className="text-muted-foreground">Private</span>
              </div>
              <span className="font-semibold">{breakdown.private}</span>
            </div>

            {/* Archived Count */}
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-2">
                <Archive className="h-3.5 w-3.5 text-gray-500" />
                <span className="text-muted-foreground">Archived</span>
              </div>
              <span className="font-semibold">{breakdown.archived}</span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
```

#### Step 3.2: Create Component Tests

**File**: `frontend/src/components/dashboard/BreakdownTile.test.tsx` (NEW FILE)

```typescript
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import BreakdownTile from './BreakdownTile';
import { Boxes } from 'lucide-react';

describe('BreakdownTile', () => {
  const mockBreakdown = {
    public: 10,
    private: 5,
    archived: 2,
  };

  it('renders title correctly', () => {
    render(
      <BreakdownTile
        title="Test Breakdown"
        breakdown={mockBreakdown}
        icon={Boxes}
      />
    );
    expect(screen.getByText('Test Breakdown')).toBeInTheDocument();
  });

  it('displays all visibility counts', () => {
    render(
      <BreakdownTile
        title="Repositories"
        breakdown={mockBreakdown}
        icon={Boxes}
      />
    );

    expect(screen.getByText('10')).toBeInTheDocument(); // Public
    expect(screen.getByText('5')).toBeInTheDocument();  // Private
    expect(screen.getByText('2')).toBeInTheDocument();  // Archived
  });

  it('shows loading state when isLoading is true', () => {
    render(
      <BreakdownTile
        title="Loading Test"
        breakdown={mockBreakdown}
        icon={Boxes}
        isLoading={true}
      />
    );

    // Should show spinner, not counts
    expect(screen.queryByText('10')).not.toBeInTheDocument();
  });

  it('displays correct icon labels', () => {
    render(
      <BreakdownTile
        title="Repos"
        breakdown={mockBreakdown}
        icon={Boxes}
      />
    );

    expect(screen.getByText('Public')).toBeInTheDocument();
    expect(screen.getByText('Private')).toBeInTheDocument();
    expect(screen.getByText('Archived')).toBeInTheDocument();
  });
});
```

**Expected Results**:

- ✅ Component renders correctly
- ✅ All 3 visibility types displayed
- ✅ Icons match design system
- ✅ Loading state works
- ✅ Tests pass: `npm run test`

---

### 3.4 Milestone 4: Integrate Breakdown Tiles into Dashboard

**Objective**: Add second row of breakdown tiles to Dashboard page

**Success Criteria**:

- 4 breakdown tiles displayed below summary cards
- Data flows correctly from API
- Responsive layout maintained
- No visual regressions

#### Step 4.1: Update Dashboard Component

**File**: `frontend/src/pages/Dashboard.tsx`

```typescript
// Add import (line 8)
import BreakdownTile from '@/components/dashboard/BreakdownTile';
import { Boxes, GitPullRequest } from 'lucide-react';

// Update render (after line 168, before Repository/PR View)
export default function Dashboard() {
  // ... existing code ...

  return (
    <div className="space-y-6">
      {/* ... existing header ... */}

      {/* Summary Cards (EXISTING) */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Total Repositories</CardTitle>
            <Boxes className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{isLoading ? '-' : summary?.totalRepositories}</div>
          </CardContent>
        </Card>
        {/* ... other 3 existing cards ... */}
      </div>

      {/* NEW: Visibility Breakdown Tiles */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <BreakdownTile
          title="Repositories by Visibility"
          breakdown={summary?.repositoryBreakdown || { public: 0, private: 0, archived: 0 }}
          icon={Boxes}
          isLoading={isLoading}
        />
        <BreakdownTile
          title="Open PRs by Visibility"
          breakdown={summary?.openPrsBreakdown || { public: 0, private: 0, archived: 0 }}
          icon={GitPullRequest}
          isLoading={isLoading}
        />
        <BreakdownTile
          title="Ready to Merge by Visibility"
          breakdown={summary?.readyToMergeBreakdown || { public: 0, private: 0, archived: 0 }}
          icon={() => <span className="h-3 w-3 rounded-full bg-ampel-green" />}
          isLoading={isLoading}
        />
        <BreakdownTile
          title="Needs Attention by Visibility"
          breakdown={summary?.needsAttentionBreakdown || { public: 0, private: 0, archived: 0 }}
          icon={() => <span className="h-3 w-3 rounded-full bg-ampel-red" />}
          isLoading={isLoading}
        />
      </div>

      {/* Repository/PR View (EXISTING) */}
      {viewMode === 'prs' ? (
        <PRListView />
      ) : isLoading ? (
        // ... existing loading state ...
      ) : viewMode === 'grid' ? (
        <GridView repositories={filteredRepositories} />
      ) : (
        <ListView repositories={filteredRepositories} />
      )}
    </div>
  );
}
```

#### Step 4.2: Update Dashboard Tests

**File**: `frontend/src/pages/Dashboard.test.tsx`

```typescript
// Add test case
it('displays visibility breakdown tiles', async () => {
  const mockSummary = {
    totalRepositories: 20,
    totalOpenPrs: 10,
    statusCounts: { green: 5, yellow: 3, red: 2 },
    providerCounts: { github: 15, gitlab: 3, bitbucket: 2 },
    repositoryBreakdown: { public: 12, private: 6, archived: 2 },
    openPrsBreakdown: { public: 6, private: 3, archived: 1 },
    readyToMergeBreakdown: { public: 3, private: 2, archived: 0 },
    needsAttentionBreakdown: { public: 1, private: 1, archived: 0 },
  };

  dashboardApi.getSummary.mockResolvedValue(mockSummary);

  render(<Dashboard />);

  await waitFor(() => {
    expect(screen.getByText('Repositories by Visibility')).toBeInTheDocument();
    expect(screen.getByText('Open PRs by Visibility')).toBeInTheDocument();
  });

  // Verify breakdown counts are displayed
  expect(screen.getAllByText('12')).toHaveLength(1); // Public repos
  expect(screen.getAllByText('6')).toHaveLength(2);  // Private repos + Public PRs
});
```

**Expected Results**:

- ✅ 8 total tiles displayed (4 summary + 4 breakdown)
- ✅ Breakdown data renders correctly
- ✅ Layout responsive on mobile (tiles stack properly)
- ✅ Tests pass: `npm run test`

---

## 4. Testing Strategy

### 4.1 Backend Tests

**Test Coverage Requirements**: 80% minimum

1. **Unit Tests** (`crates/ampel-api/tests/unit/`):
   - VisibilityBreakdown struct serialization
   - Default values for breakdown
   - Breakdown calculation logic

2. **Integration Tests** (`crates/ampel-api/tests/integration/`):
   - Dashboard summary with mixed visibility repos
   - Breakdown totals match top-level counts
   - Edge cases: all private, all public, all archived
   - Performance: 100 repos < 500ms response time

3. **Test Scenarios**:

```rust
#[tokio::test]
async fn test_all_public_repositories() {
    // Create 5 public repos
    // Assert: breakdown.public = 5, private = 0, archived = 0
}

#[tokio::test]
async fn test_mixed_visibility_with_prs() {
    // Create 2 public, 3 private, 1 archived
    // Add PRs with different statuses
    // Assert: all breakdown counts correct
}

#[tokio::test]
async fn test_archived_repos_count_prs() {
    // Create archived repo with open PRs
    // Assert: PR counts appear in archived breakdown
}
```

### 4.2 Frontend Tests

**Test Coverage Requirements**: 80% minimum

1. **Component Tests**:
   - BreakdownTile renders all visibility types
   - Loading state displays spinner
   - Icons and colors correct
   - Responsive layout

2. **Integration Tests**:
   - Dashboard fetches and displays breakdowns
   - API errors show gracefully
   - Breakdowns match summary totals

3. **Visual Regression Tests**:
   - Screenshot comparison of new tiles
   - Mobile layout verification

### 4.3 E2E Tests

**Test Scenarios**:

1. **Happy Path**:
   - User logs in
   - Dashboard loads with mixed repos
   - Breakdown tiles show correct counts
   - Totals match summary

2. **Edge Cases**:
   - Empty state (0 repositories)
   - All repositories same visibility
   - Large numbers (100+ repos)

3. **Performance**:
   - Dashboard loads < 2 seconds
   - No layout shift when data loads

---

## 5. Deployment Strategy

### 5.1 Rollout Plan

**Phase 1: Backend Deployment**

1. Deploy backend changes with new fields (backward compatible)
2. Old frontend ignores new fields (graceful degradation)
3. Monitor API response times and error rates

**Phase 2: Frontend Deployment**

1. Deploy frontend with breakdown tiles
2. New tiles consume new API fields
3. Monitor user engagement and error rates

**Phase 3: Verification**

1. Verify breakdown totals match summary
2. Check Sentry for JavaScript errors
3. Review user feedback

### 5.2 Rollback Strategy

**Backend Rollback**:

- Revert to previous API version
- Frontend continues to work (ignores missing fields)

**Frontend Rollback**:

- Revert frontend deployment
- Dashboard shows only 4 summary tiles
- Backend continues serving new data (unused)

**Rollback Triggers**:

- API error rate > 1%
- Response time > 1 second
- TypeScript errors in production
- Critical user complaints

---

## 6. Performance Considerations

### 6.1 Backend Optimization

**Current Performance**:

- `get_summary` iterates all repos and PRs: O(n\*m)
- For 100 repos with 10 PRs each = 1000 iterations

**Optimization Opportunities**:

1. **Database Aggregation**: Push visibility counting to SQL
2. **Caching**: Cache breakdown for 60 seconds
3. **Parallel Queries**: Fetch CI checks and reviews in parallel

**SQL Aggregation Example**:

```sql
SELECT
  COUNT(*) FILTER (WHERE is_private = false AND is_archived = false) as public,
  COUNT(*) FILTER (WHERE is_private = true AND is_archived = false) as private,
  COUNT(*) FILTER (WHERE is_archived = true) as archived
FROM repositories
WHERE user_id = $1;
```

**Expected Improvement**:

- Current: ~500ms for 100 repos
- Optimized: ~100ms for 100 repos

### 6.2 Frontend Optimization

**Optimizations**:

1. **Memoization**: Use `useMemo` for breakdown calculations
2. **Lazy Loading**: Load breakdown tiles only when visible
3. **Code Splitting**: Lazy load BreakdownTile component

**Current Bundle Impact**:

- New component: ~2KB
- Lucide icons: Already included (0KB additional)

---

## 7. Accessibility Considerations

### 7.1 Screen Reader Support

```typescript
// Add aria-labels to BreakdownTile
<Card role="region" aria-label={`${title} breakdown by visibility`}>
  <CardContent>
    <div className="space-y-2" role="list" aria-label="Visibility breakdown">
      <div role="listitem" aria-label={`Public repositories: ${breakdown.public}`}>
        <Globe className="h-3.5 w-3.5 text-green-600" aria-hidden="true" />
        <span className="text-muted-foreground">Public</span>
        <span className="font-semibold">{breakdown.public}</span>
      </div>
      {/* Similar for Private and Archived */}
    </div>
  </CardContent>
</Card>
```

### 7.2 Keyboard Navigation

- Tiles are not interactive (no keyboard focus needed)
- Screen reader announces counts on page load
- Proper heading hierarchy maintained

### 7.3 Color Contrast

**WCAG AA Compliance**:

- Green (Public): `#16a34a` - Contrast ratio 4.5:1 ✅
- Amber (Private): `#d97706` - Contrast ratio 4.5:1 ✅
- Gray (Archived): `#6b7280` - Contrast ratio 4.5:1 ✅

---

## 8. Documentation Updates

### 8.1 Files to Update

1. **README.md**: Add screenshot showing breakdown tiles
2. **docs/ARCHITECTURE.md**: Document new API fields
3. **API Documentation** (Swagger/utoipa): Update DashboardSummary schema
4. **CLAUDE.md**: Document BreakdownTile component usage

### 8.2 API Documentation

**Add to OpenAPI schema**:

```yaml
components:
  schemas:
    VisibilityBreakdown:
      type: object
      required:
        - public
        - private
        - archived
      properties:
        public:
          type: integer
          description: Count of public (non-private, non-archived) items
        private:
          type: integer
          description: Count of private (non-archived) items
        archived:
          type: integer
          description: Count of archived items (may also be private)

    DashboardSummary:
      type: object
      properties:
        # ... existing fields ...
        repositoryBreakdown:
          $ref: '#/components/schemas/VisibilityBreakdown'
        openPrsBreakdown:
          $ref: '#/components/schemas/VisibilityBreakdown'
        readyToMergeBreakdown:
          $ref: '#/components/schemas/VisibilityBreakdown'
        needsAttentionBreakdown:
          $ref: '#/components/schemas/VisibilityBreakdown'
```

---

## 9. Migration & Compatibility

### 9.1 Backward Compatibility

**Frontend Compatibility**:

- Old frontend (before this change) receives new API fields but ignores them
- No breaking changes to existing API contract
- Optional fields with defaults prevent crashes

**API Versioning**:

- No API version bump required (additive change)
- New fields are additions, not modifications

### 9.2 Database Migrations

**No database changes required**:

- `is_private` and `is_archived` fields already exist
- No new tables or columns needed

---

## 10. Monitoring & Metrics

### 10.1 Success Metrics

**Performance Metrics**:

- API response time (p95): < 500ms
- Frontend render time: < 100ms
- Bundle size increase: < 5KB

**Usage Metrics**:

- % of users viewing dashboard: Track
- Average time on dashboard: Track
- Click-through to filtered views: Track (future)

**Quality Metrics**:

- JavaScript error rate: < 0.1%
- API error rate: < 0.5%
- Test coverage: > 80%

### 10.2 Observability

**Logging**:

```rust
// Add to get_summary handler
tracing::info!(
    user_id = %auth.user_id,
    total_repos = repos.len(),
    public_repos = repo_breakdown.public,
    private_repos = repo_breakdown.private,
    archived_repos = repo_breakdown.archived,
    "Dashboard summary calculated"
);
```

**Metrics** (Prometheus):

- `ampel_dashboard_summary_duration_seconds` - Histogram
- `ampel_dashboard_breakdown_total` - Counter by visibility type
- `ampel_dashboard_errors_total` - Counter

---

## 11. Risk Assessment

| Risk                               | Impact | Probability | Mitigation                               |
| ---------------------------------- | ------ | ----------- | ---------------------------------------- |
| API performance degradation        | High   | Low         | Add SQL aggregation, caching             |
| Incorrect breakdown totals         | Medium | Medium      | Comprehensive tests, validation          |
| Mobile layout breaks               | Medium | Low         | Responsive design tests                  |
| TypeScript compilation errors      | Low    | Low         | CI/CD type checking                      |
| User confusion with archived count | Low    | Medium      | Add tooltip explaining archived behavior |
| Bitbucket always shows 0 archived  | Low    | High        | Document in tooltip, help text           |

---

## 12. Implementation Checklist

### 12.1 Backend Tasks

- [ ] Add `VisibilityBreakdown` struct to `dashboard.rs`
- [ ] Update `DashboardSummary` struct with new fields
- [ ] Implement breakdown calculation in `get_summary`
- [ ] Add unit tests for breakdown logic
- [ ] Add integration tests for API endpoint
- [ ] Performance test with 100+ repos
- [ ] Update OpenAPI/utoipa documentation
- [ ] Add tracing/logging for breakdowns

### 12.2 Frontend Tasks

- [ ] Add `VisibilityBreakdown` interface to types
- [ ] Update `DashboardSummary` interface
- [ ] Create `BreakdownTile` component
- [ ] Add unit tests for `BreakdownTile`
- [ ] Integrate breakdown tiles into Dashboard
- [ ] Update Dashboard tests
- [ ] Add accessibility attributes
- [ ] Test responsive layout on mobile
- [ ] Verify color contrast ratios
- [ ] Run Lighthouse accessibility audit

### 12.3 Testing Tasks

- [ ] Run full backend test suite: `make test-backend`
- [ ] Run full frontend test suite: `make test-frontend`
- [ ] E2E test: Dashboard with mixed repos
- [ ] E2E test: Dashboard with all public repos
- [ ] E2E test: Dashboard with all private repos
- [ ] Performance test: API < 500ms
- [ ] Visual regression test: Screenshot comparison

### 12.4 Documentation Tasks

- [ ] Update README with new screenshot
- [ ] Update ARCHITECTURE.md
- [ ] Update API docs (Swagger)
- [ ] Update CLAUDE.md with component usage
- [ ] Add inline code comments
- [ ] Create PR description with screenshots

### 12.5 Deployment Tasks

- [ ] Deploy backend to staging
- [ ] Verify API response structure
- [ ] Deploy frontend to staging
- [ ] Manual QA on staging
- [ ] Deploy to production
- [ ] Monitor error rates (1 hour)
- [ ] Monitor performance (1 hour)
- [ ] Announce feature to users

---

## 13. Timeline Estimate

**Total Estimated Effort**: 2-3 days

| Phase   | Task                         | Estimate |
| ------- | ---------------------------- | -------- |
| Phase 1 | Backend API enhancement      | 4 hours  |
| Phase 2 | Frontend types & component   | 3 hours  |
| Phase 3 | Dashboard integration        | 2 hours  |
| Phase 4 | Testing (backend + frontend) | 4 hours  |
| Phase 5 | Documentation                | 2 hours  |
| Phase 6 | Code review & iterations     | 2 hours  |
| Phase 7 | Deployment & monitoring      | 1 hour   |

**Critical Path**: Backend API → Frontend Types → Component → Integration → Testing

---

## 14. Future Enhancements

### 14.1 Potential Features

1. **Clickable Breakdowns**: Click on visibility type to filter dashboard
2. **Trend Charts**: Show breakdown changes over time
3. **Custom Groupings**: Group by org, team, or custom tags
4. **Export Data**: Download breakdown data as CSV
5. **Notifications**: Alert when archived repos have open PRs

### 14.2 Performance Optimizations

1. **Redis Caching**: Cache breakdowns for 60 seconds
2. **WebSocket Updates**: Real-time breakdown updates
3. **Incremental Updates**: Only recalculate changed repos

---

## Appendix A: Code File Summary

### New Files

```
frontend/src/components/dashboard/BreakdownTile.tsx
frontend/src/components/dashboard/BreakdownTile.test.tsx
```

### Modified Files

```
crates/ampel-api/src/handlers/dashboard.rs
crates/ampel-api/tests/integration/dashboard_tests.rs
frontend/src/types/index.ts
frontend/src/pages/Dashboard.tsx
frontend/src/pages/Dashboard.test.tsx
docs/ARCHITECTURE.md (documentation)
README.md (documentation)
```

### Lines of Code

- **Backend**: ~80 new lines
- **Frontend**: ~150 new lines
- **Tests**: ~100 new lines
- **Total**: ~330 lines

---

## Appendix B: Visual Design Reference

### Breakdown Tile Mockup

```
┌─────────────────────────────┐
│ Repositories by Visibility  │  ← CardTitle
│                       📦     │  ← Icon
├─────────────────────────────┤
│                             │
│  🌐 Public          20      │  ← Green globe icon
│  🔒 Private         15      │  ← Amber lock icon
│  📦 Archived         5      │  ← Gray archive icon
│                             │
└─────────────────────────────┘
```

### Color Palette

- **Public Green**: `#16a34a` (text-green-600)
- **Private Amber**: `#d97706` (text-amber-600)
- **Archived Gray**: `#6b7280` (text-gray-500)
- **Ampel Green**: Custom green for ready to merge
- **Ampel Red**: Custom red for needs attention

---

## Appendix C: Example API Response

```json
{
  "success": true,
  "data": {
    "totalRepositories": 42,
    "totalOpenPrs": 18,
    "statusCounts": {
      "green": 8,
      "yellow": 7,
      "red": 3
    },
    "providerCounts": {
      "github": 30,
      "gitlab": 10,
      "bitbucket": 2
    },
    "repositoryBreakdown": {
      "public": 20,
      "private": 18,
      "archived": 4
    },
    "openPrsBreakdown": {
      "public": 10,
      "private": 6,
      "archived": 2
    },
    "readyToMergeBreakdown": {
      "public": 5,
      "private": 2,
      "archived": 1
    },
    "needsAttentionBreakdown": {
      "public": 2,
      "private": 1,
      "archived": 0
    }
  }
}
```

---

## Appendix D: Related Documentation

- **Repository Visibility Filter Implementation**: `/docs/planning/REPOSITORY-VISIBILITY-FILTER-IMPLEMENTATION.md`
- **Dashboard Architecture**: `/docs/ARCHITECTURE.md`
- **Testing Guide**: `/docs/TESTING.md`
- **API Documentation**: Available at `/api/docs` when running locally

---

**Document Prepared By**: Claude Code Analysis
**Review Required By**: Development Team Lead
**Implementation Target**: Sprint TBD
**Last Updated**: 2025-12-24
