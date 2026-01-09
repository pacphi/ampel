# Implementation Plan: Merge Operations & Notifications

## Overview

Add bulk merge operations with Slack + Email notifications to Ampel, replicating git-pr-manager functionality.

**Scope:**

- Bulk merge backend + frontend
- Notifications (Slack + Email) for merge results
- Settings UI: Notifications tab, Behavior tab
- Merge from both Dashboard/PRListView AND dedicated Merge page
- Global settings only (not per-repo)

---

## Phase 1: Database Schema

### New Migration: `m20250104_000004_merge_notifications.rs`

**1. Extend `notification_preferences` table:**

```sql
-- Email SMTP (encrypted password)
smtp_host VARCHAR(255),
smtp_port INTEGER,
smtp_username VARCHAR(255),
smtp_password_encrypted BYTEA,
smtp_from_email VARCHAR(255),
smtp_to_emails TEXT,  -- JSON array
email_use_tls BOOLEAN DEFAULT true,
email_enabled BOOLEAN DEFAULT false,

-- Merge notifications
notify_on_merge_success BOOLEAN DEFAULT true,
notify_on_merge_failure BOOLEAN DEFAULT true,
slack_channel VARCHAR(100)
```

**2. New `user_settings` table (behavior config):**

```sql
CREATE TABLE user_settings (
    id UUID PRIMARY KEY,
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    merge_delay_seconds INTEGER DEFAULT 5,
    require_approval BOOLEAN DEFAULT false,
    delete_branches_default BOOLEAN DEFAULT false,
    default_merge_strategy VARCHAR(20) DEFAULT 'squash',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

**3. New `merge_operations` table (bulk merge tracking):**

```sql
CREATE TABLE merge_operations (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    total_count INTEGER NOT NULL,
    success_count INTEGER DEFAULT 0,
    failed_count INTEGER DEFAULT 0,
    skipped_count INTEGER DEFAULT 0,
    status VARCHAR(20) DEFAULT 'in_progress',
    notification_sent BOOLEAN DEFAULT false
);

CREATE TABLE merge_operation_items (
    id UUID PRIMARY KEY,
    merge_operation_id UUID NOT NULL REFERENCES merge_operations(id) ON DELETE CASCADE,
    pull_request_id UUID NOT NULL REFERENCES pull_requests(id),
    repository_id UUID NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    error_message TEXT,
    merge_sha VARCHAR(40),
    merged_at TIMESTAMPTZ
);
```

### Files to Create/Modify

- `crates/ampel-db/src/migrations/m20250104_000004_merge_notifications.rs` (new)
- `crates/ampel-db/src/migrations/mod.rs` (add migration)
- `crates/ampel-db/src/entities/user_settings.rs` (new)
- `crates/ampel-db/src/entities/merge_operation.rs` (new)
- `crates/ampel-db/src/entities/merge_operation_item.rs` (new)
- `crates/ampel-db/src/entities/notification_preferences.rs` (extend)
- `crates/ampel-db/src/entities/mod.rs` (export new entities)

---

## Phase 2: Backend API

### 2.1 User Settings Handler (Behavior)

**File:** `crates/ampel-api/src/handlers/user_settings.rs` (new)

```rust
// GET/PUT /api/settings/behavior
pub struct UserSettingsResponse {
    merge_delay_seconds: i32,
    require_approval: bool,
    delete_branches_default: bool,
    default_merge_strategy: String,  // "merge"|"squash"|"rebase"
}
```

### 2.2 Extend Notifications Handler

**File:** `crates/ampel-api/src/handlers/notifications.rs` (modify)

Add email SMTP fields to existing preferences:

- `smtp_host`, `smtp_port`, `smtp_username`, `smtp_password` (encrypted)
- `smtp_from_email`, `smtp_to_emails`, `email_use_tls`, `email_enabled`
- `notify_on_merge_success`, `notify_on_merge_failure`, `slack_channel`

Add endpoint: `POST /api/notifications/test-email`

### 2.3 Bulk Merge Handler

**File:** `crates/ampel-api/src/handlers/bulk_merge.rs` (new)

```rust
// POST /api/merge/bulk
pub struct BulkMergeRequest {
    pull_request_ids: Vec<Uuid>,
    strategy: Option<MergeStrategy>,  // Override user default
    delete_branch: Option<bool>,
}

pub struct BulkMergeResponse {
    operation_id: Uuid,
    status: String,  // "in_progress"|"completed"|"failed"
    total: i32,
    success: i32,
    failed: i32,
    skipped: i32,
    results: Vec<MergeItemResult>,
}
```

**Merge Execution Logic:**

1. Validate all PRs owned by user and mergeable
2. Create `merge_operations` record
3. Group PRs by repository
4. Merge sequentially within each repo (with `merge_delay_seconds` between)
5. Merge different repos in parallel
6. Update operation status
7. Send notifications (Slack + Email)

### 2.4 Notification Service

**File:** `crates/ampel-core/src/services/notification_service.rs` (new)

```rust
impl NotificationService {
    async fn send_merge_notification(...);  // Orchestrates both channels
    async fn send_slack_message(webhook_url, channel, message);
    async fn send_email(smtp_config, to, subject, body);  // Using lettre crate
}
```

### 2.5 New Routes

**File:** `crates/ampel-api/src/routes/mod.rs` (modify)

```rust
// Bulk merge
.route("/api/merge/bulk", post(bulk_merge::bulk_merge))
.route("/api/merge/operations", get(bulk_merge::list_operations))
.route("/api/merge/operations/:id", get(bulk_merge::get_operation))

// User settings (behavior)
.route("/api/settings/behavior", get(user_settings::get).put(user_settings::update))

// Email test (extend existing notifications routes)
.route("/api/notifications/test-email", post(notifications::test_email))
```

### Files to Create/Modify

- `crates/ampel-api/src/handlers/bulk_merge.rs` (new)
- `crates/ampel-api/src/handlers/user_settings.rs` (new)
- `crates/ampel-api/src/handlers/notifications.rs` (extend)
- `crates/ampel-api/src/handlers/mod.rs` (export new handlers)
- `crates/ampel-api/src/routes/mod.rs` (add routes)
- `crates/ampel-core/src/services/notification_service.rs` (new)
- `crates/ampel-core/src/services/mod.rs` (new)
- `crates/ampel-core/src/lib.rs` (export services)
- `crates/ampel-core/Cargo.toml` (add `lettre` for email)
- `crates/ampel-db/src/queries/user_settings_queries.rs` (new)
- `crates/ampel-db/src/queries/merge_operation_queries.rs` (new)
- `crates/ampel-db/src/queries/mod.rs` (export)

---

## Phase 3: Frontend - Settings Tabs

### 3.1 Add Notifications Tab

**File:** `frontend/src/pages/Settings.tsx` (modify)

Add to `SettingsNav`:

```typescript
{ href: '/settings/notifications', label: 'Notifications', icon: Bell },
{ href: '/settings/behavior', label: 'Behavior', icon: Settings2 },
```

Add routes:

```typescript
<Route path="notifications" element={<NotificationsSettings />} />
<Route path="behavior" element={<BehaviorSettings />} />
```

### 3.2 NotificationsSettings Component

**File:** `frontend/src/components/settings/NotificationsSettings.tsx` (new)

Sections:

- **Slack**: Webhook URL input, channel input, enabled toggle, test button
- **Email SMTP**: Host, port, username, password, from, to (list), TLS toggle, enabled toggle, test button
- **Merge Notifications**: Success toggle, failure toggle

### 3.3 BehaviorSettings Component

**File:** `frontend/src/components/settings/BehaviorSettings.tsx` (new)

Settings:

- Merge delay slider (5-60 seconds)
- Require approval toggle
- Delete branches by default toggle
- Default merge strategy dropdown (merge/squash/rebase)

### 3.4 API Clients

**File:** `frontend/src/api/settings.ts` (new)

```typescript
export const settingsApi = {
  getBehavior(): Promise<UserSettings>,
  updateBehavior(settings: Partial<UserSettings>): Promise<UserSettings>,
};
```

**File:** `frontend/src/api/notifications.ts` (extend if exists, or add to settings.ts)

Add `testEmail()` function.

---

## Phase 4: Frontend - Merge Page & PRListView

### 4.1 Dedicated Merge Page

**File:** `frontend/src/pages/Merge.tsx` (new)

Features:

- List all mergeable PRs (green status, no conflicts)
- Bulk selection with checkboxes
- Strategy selection dropdown
- Delete branch toggle
- "Preview & Merge" button opens confirmation dialog
- Progress indicator during merge
- Results summary after completion

### 4.2 Merge Components

**File:** `frontend/src/components/merge/MergePreviewDialog.tsx` (new)

- Shows selected PRs before execution
- Confirm/cancel buttons

**File:** `frontend/src/components/merge/MergeProgress.tsx` (new)

- Real-time progress bar
- Per-PR status updates

**File:** `frontend/src/components/merge/MergeResultsSummary.tsx` (new)

- Success/failed/skipped counts
- Details for each PR

### 4.3 Update PRListView

**File:** `frontend/src/components/dashboard/PRListView.tsx` (modify)

- Replace inline merge with `mergeApi.bulkMerge()` call
- Add strategy dropdown before merge button
- Show progress dialog during merge
- Use user's default settings from behavior config

### 4.4 API Client for Merge

**File:** `frontend/src/api/merge.ts` (new)

```typescript
export const mergeApi = {
  bulkMerge(request: BulkMergeRequest): Promise<BulkMergeResponse>,
  getOperation(id: string): Promise<BulkMergeResponse>,
  listOperations(): Promise<PaginatedResponse<BulkMergeResponse>>,
};
```

### 4.5 Update Routing

**File:** `frontend/src/App.tsx` (modify)

Add route: `<Route path="merge" element={<Merge />} />`

**File:** `frontend/src/components/layout/Header.tsx` or Sidebar (modify)

Add navigation link to Merge page.

---

## Implementation Order

### Step 1: Database & Entities

1. Create migration `m20250104_000004_merge_notifications.rs`
2. Create entities: `user_settings.rs`, `merge_operation.rs`, `merge_operation_item.rs`
3. Extend `notification_preferences.rs` entity
4. Create queries: `user_settings_queries.rs`, `merge_operation_queries.rs`
5. Run migration

### Step 2: Backend - Settings

1. Create `user_settings` handler (GET/PUT)
2. Extend `notifications` handler with email fields
3. Add routes

### Step 3: Backend - Bulk Merge

1. Create `notification_service.rs` (Slack + Email)
2. Create `bulk_merge` handler with merge execution logic
3. Add routes
4. Add `lettre` dependency for email

### Step 4: Frontend - Settings

1. Create `NotificationsSettings` component
2. Create `BehaviorSettings` component
3. Update `Settings.tsx` with new tabs/routes
4. Create `frontend/src/api/settings.ts`

### Step 5: Frontend - Merge

1. Create `frontend/src/api/merge.ts`
2. Create merge components (dialog, progress, results)
3. Create `Merge.tsx` page
4. Update `PRListView.tsx` to use bulk merge API
5. Add merge page to routing + navigation

### Step 6: Testing & Polish

1. Test full merge flow
2. Test Slack notifications
3. Test Email notifications
4. Error handling edge cases

---

## Key Files Summary

| Category  | File                                                                     | Action |
| --------- | ------------------------------------------------------------------------ | ------ |
| Migration | `crates/ampel-db/src/migrations/m20250104_000004_merge_notifications.rs` | Create |
| Entity    | `crates/ampel-db/src/entities/user_settings.rs`                          | Create |
| Entity    | `crates/ampel-db/src/entities/merge_operation.rs`                        | Create |
| Handler   | `crates/ampel-api/src/handlers/bulk_merge.rs`                            | Create |
| Handler   | `crates/ampel-api/src/handlers/user_settings.rs`                         | Create |
| Handler   | `crates/ampel-api/src/handlers/notifications.rs`                         | Modify |
| Service   | `crates/ampel-core/src/services/notification_service.rs`                 | Create |
| Routes    | `crates/ampel-api/src/routes/mod.rs`                                     | Modify |
| Page      | `frontend/src/pages/Settings.tsx`                                        | Modify |
| Page      | `frontend/src/pages/Merge.tsx`                                           | Create |
| Component | `frontend/src/components/settings/NotificationsSettings.tsx`             | Create |
| Component | `frontend/src/components/settings/BehaviorSettings.tsx`                  | Create |
| Component | `frontend/src/components/merge/*.tsx`                                    | Create |
| API       | `frontend/src/api/merge.ts`                                              | Create |
| API       | `frontend/src/api/settings.ts`                                           | Create |
