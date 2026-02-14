# Repository Refresh Feature

## Overview

This feature adds real-time repository refresh functionality to the Dashboard, allowing users to manually trigger an immediate refresh of all repositories from their git providers (GitHub, GitLab, Bitbucket) with visual progress tracking.

## What Was Implemented

### Backend (Rust)

#### 1. New API Endpoints

**POST `/api/repositories/refresh-all`**
- Triggers immediate polling of all user repositories
- Returns a job ID for progress tracking
- Spawns a background task to refresh repositories concurrently

**GET `/api/repositories/refresh-status/:job_id`**
- Returns real-time status of a refresh job
- Includes progress (completed/total), current repository being refreshed, and completion status

#### 2. Core Components

**File: `crates/ampel-api/src/handlers/repositories.rs`**

- **`RefreshJobStatus`**: Struct tracking refresh progress
  - `job_id`: Unique identifier for the job
  - `total_repositories`: Total number of repositories to refresh
  - `completed`: Number of repositories already refreshed
  - `current_repository`: Name of repository currently being refreshed
  - `is_complete`: Whether the job has finished
  - `started_at` / `completed_at`: Timestamps

- **`refresh_all_repositories()`**: Handler that:
  - Fetches all user repositories
  - Creates a new refresh job with unique ID
  - Spawns background task to poll each repository
  - Returns job ID immediately for async tracking

- **`get_refresh_status()`**: Handler that returns current status of a refresh job

**File: `crates/ampel-worker/src/jobs/poll_repository.rs`**

- Made `poll_single_repo()` method public so it can be called from the API layer
- This method fetches latest PRs, CI checks, and reviews from the git provider

#### 3. In-Memory Job Tracking

- Uses `lazy_static` with `Arc<RwLock<HashMap>>` for thread-safe job status storage
- Automatically updates progress as each repository is refreshed
- Tracks which repository is currently being processed

### Frontend (React + TypeScript)

#### 1. API Client Updates

**File: `frontend/src/api/repositories.ts`**

Added two new methods:
```typescript
async refreshAll(): Promise<RefreshJobResponse>
async getRefreshStatus(jobId: string): Promise<RefreshJobStatus>
```

#### 2. Type Definitions

**File: `frontend/src/types/index.ts`**

```typescript
interface RefreshJobResponse {
  jobId: string;
}

interface RefreshJobStatus {
  jobId: string;
  totalRepositories: number;
  completed: number;
  currentRepository?: string;
  isComplete: boolean;
  startedAt: string;
  completedAt?: string;
}
```

#### 3. Dashboard Updates

**File: `frontend/src/pages/Dashboard.tsx`**

- **Refresh Button**: Now triggers actual repository refresh (not just cache invalidation)
- **Loading Animation**: Spinner animation while refresh is in progress
- **Progress Dialog**: Shows real-time refresh status in a modal dialog
- **Auto Polling**: Uses TanStack Query to poll refresh status every second
- **Cache Invalidation**: Automatically refreshes dashboard data when job completes

#### 4. New Components

**File: `frontend/src/components/dashboard/RefreshProgress.tsx`**

Real-time progress display showing:
- Progress bar with percentage
- "X of Y repositories refreshed"
- Current repository being refreshed (with animated spinner)
- Completion status (green checkmark when done)
- Start and completion timestamps

**File: `frontend/src/components/ui/progress.tsx`**

Custom Progress component using shadcn/ui patterns for visual progress bar.

#### 5. Internationalization

**File: `frontend/public/locales/en/dashboard.json`**

Added translation keys for:
- Dialog title and description
- Progress message
- Current repository label
- Completion message
- Timestamp labels

**File: `crates/ampel-api/locales/en/errors.yml`**

Added error message for "refresh job not found".

### Dependencies

**Backend:**
- Added `lazy_static = "1.5"` for static job storage
- Added `ampel-worker` dependency to reuse polling logic

**Frontend:**
- Used existing TanStack Query for polling
- Used existing shadcn/ui Dialog component

## How It Works

### User Flow

1. **User clicks Refresh button** on Dashboard
2. **Frontend calls** `POST /api/repositories/refresh-all`
3. **Backend returns** job ID immediately
4. **Frontend shows progress dialog** and starts polling
5. **Backend processes** each repository in background:
   - Updates job status after each repository
   - Fetches latest PRs, CI checks, and reviews from git provider
   - Marks stale PRs as closed if they're no longer open on provider
6. **Frontend polls** `GET /api/repositories/refresh-status/:job_id` every second
7. **Progress updates** in real-time showing current repository
8. **When complete**:
   - Job marked as complete
   - Frontend invalidates cache
   - Fresh data displayed on dashboard

### Technical Details

**Background Processing:**
- Uses `tokio::spawn` for non-blocking execution
- API returns immediately while work continues
- No impact on API responsiveness

**Progress Tracking:**
- In-memory HashMap with RwLock for thread-safe access
- Write lock held briefly to update status
- Read lock for status queries (no blocking)

**Polling Strategy:**
- Frontend polls every 1000ms while job is active
- TanStack Query manages polling lifecycle
- Automatic stop when job completes

**Error Handling:**
- Individual repository failures logged but don't stop the job
- Job continues processing remaining repositories
- Progress accurately reflects completed count

## Files Modified

### Backend
- `crates/ampel-api/Cargo.toml` - Added dependencies
- `crates/ampel-api/src/handlers/repositories.rs` - New handlers and job tracking
- `crates/ampel-api/src/routes/mod.rs` - New routes
- `crates/ampel-api/locales/en/errors.yml` - Error translations
- `crates/ampel-worker/src/jobs/poll_repository.rs` - Made method public

### Frontend
- `frontend/src/api/repositories.ts` - New API methods
- `frontend/src/types/index.ts` - New type definitions
- `frontend/src/pages/Dashboard.tsx` - Refresh logic and progress dialog
- `frontend/src/components/dashboard/RefreshProgress.tsx` - **NEW** Progress component
- `frontend/src/components/ui/progress.tsx` - **NEW** Base progress bar
- `frontend/public/locales/en/dashboard.json` - Translation keys

## Benefits

✅ **Real-time feedback** - Users see exactly what's happening during refresh

✅ **Non-blocking** - API remains responsive during background refresh

✅ **Progress visibility** - Shows which repository is currently being refreshed

✅ **Accurate tracking** - Progress updates in real-time as repositories complete

✅ **Graceful handling** - Individual failures don't stop the entire refresh

✅ **Reuses existing logic** - Leverages battle-tested `PollRepositoryJob` code

## Testing

To test the feature:

1. **Start the application:**
   ```bash
   make docker-up
   make dev-api
   make dev-frontend
   ```

2. **Navigate to Dashboard** in the browser

3. **Click the Refresh button** (with spinning icon)

4. **Observe the progress dialog:**
   - Progress bar should advance
   - Current repository name should update
   - Completion percentage should increase
   - Checkmark appears when done

5. **Verify data refresh:**
   - PRs should show latest state from providers
   - Stale PRs should be marked as closed
   - Dashboard stats should update

## Future Enhancements

Potential improvements:

- **WebSocket support** for push-based updates instead of polling
- **Selective refresh** - Refresh specific repositories instead of all
- **Retry logic** - Automatic retry for failed repositories
- **Notification** - Browser notification when refresh completes
- **Background refresh** - Close dialog but continue tracking in background
- **Refresh history** - View past refresh jobs and their results
- **Rate limiting** - Prevent excessive refresh requests
- **Caching** - Remember last refresh time, show "Last refreshed X minutes ago"
