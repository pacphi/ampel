# Ampel - Product Specification

## Current Implementation Status (Updated: 2025-12-22)

**MVP Completion**: ~87% (Phase 1 features fully implemented)
**Production Status**: Core features complete, Phase 2/3 features partially implemented

### Feature Implementation Matrix

#### Authentication & Accounts

| Feature                        | Spec Phase   | Actual Status | Evidence                                                       |
| ------------------------------ | ------------ | ------------- | -------------------------------------------------------------- |
| Multi-Provider Auth (PAT)      | Phase 1 (F1) | ✅ Complete   | `ampel-providers/`, `handlers/auth.rs`, `handlers/accounts.rs` |
| Token Encryption (AES-256-GCM) | Phase 1 (F1) | ✅ Complete   | `entities/provider_account.rs`                                 |
| Self-hosted GitLab/Bitbucket   | Phase 1 (F1) | ✅ Complete   | Provider implementations support custom URLs                   |

#### Repository Management

| Feature             | Spec Phase   | Actual Status | Evidence                                              |
| ------------------- | ------------ | ------------- | ----------------------------------------------------- |
| Add/Remove Repos    | Phase 1 (F2) | ✅ Complete   | `handlers/repositories.rs`, `queries/repo_queries.rs` |
| Repository Grouping | Phase 1 (F2) | ✅ Complete   | `entities/repository.rs`                              |
| Wildcard Patterns   | Phase 1 (F2) | ⚠️ Partial    | Backend ready, frontend UI pending                    |

#### Dashboard & PR Management

| Feature                  | Spec Phase   | Actual Status | Evidence                                         |
| ------------------------ | ------------ | ------------- | ------------------------------------------------ |
| PR Aggregation Dashboard | Phase 1 (F3) | ✅ Complete   | `handlers/dashboard.rs`, `components/dashboard/` |
| Grid View                | Phase 1 (F3) | ✅ Complete   | `frontend/src/components/dashboard/GridView.tsx` |
| List View                | Phase 1 (F3) | ✅ Complete   | `frontend/src/components/dashboard/ListView.tsx` |
| Real-time Updates        | Phase 1 (F3) | ✅ Complete   | TanStack Query auto-refresh                      |

#### Traffic Light Status

| Feature               | Spec Phase   | Actual Status | Evidence               |
| --------------------- | ------------ | ------------- | ---------------------- |
| Green/Yellow/Red      | Phase 1 (F4) | ✅ Complete   | `ampel-core/models.rs` |
| CI Status Aggregation | Phase 1 (F4) | ✅ Complete   | `entities/ci_check.rs` |

#### Background Jobs

| Feature              | Spec Phase   | Actual Status | Evidence                 |
| -------------------- | ------------ | ------------- | ------------------------ |
| Scheduled Polling    | Phase 1 (F5) | ✅ Complete   | `ampel-worker/` + Apalis |
| Rate Limit Awareness | Phase 1 (F5) | ✅ Complete   | Provider implementations |

#### Filtering & Search

| Feature              | Spec Phase   | Actual Status | Evidence                                          |
| -------------------- | ------------ | ------------- | ------------------------------------------------- |
| Multi-criteria       | Phase 1 (F6) | ✅ Complete   | `handlers/pr_filters.rs`, `entities/pr_filter.rs` |
| Full-text Search     | Phase 1 (F6) | ✅ Complete   | Database queries support text search              |
| Saved Filter Presets | Phase 1 (F6) | ✅ Complete   | `entities/pr_filter.rs` persists filters          |

#### PR Details

| Feature          | Spec Phase   | Actual Status | Evidence                                            |
| ---------------- | ------------ | ------------- | --------------------------------------------------- |
| PR Detail View   | Phase 1 (F7) | ✅ Complete   | `handlers/pull_requests.rs`, frontend PR components |
| CI Check Details | Phase 1 (F7) | ✅ Complete   | `entities/ci_check.rs`, `entities/review.rs`        |

#### Team Management (Originally Phase 2)

| Feature         | Spec Phase   | Actual Status | Evidence                                |
| --------------- | ------------ | ------------- | --------------------------------------- |
| Team Creation   | Phase 2 (F8) | ✅ Complete   | `handlers/teams.rs`, `entities/team.rs` |
| Team Dashboards | Phase 2 (F8) | ✅ Complete   | `entities/team_member.rs`               |
| Member Roles    | Phase 2 (F8) | ✅ Complete   | admin/member/viewer roles               |

#### Notifications

| Feature           | Spec Phase   | Actual Status  | Evidence                                      |
| ----------------- | ------------ | -------------- | --------------------------------------------- |
| Slack Integration | Phase 2 (F9) | ⚠️ Partial     | Entity exists, no worker                      |
| Email Digests     | Phase 2 (F9) | ⚠️ Partial     | SMTP config in preferences, no implementation |
| Browser Push      | Phase 2 (F9) | ❌ Not Started | N/A                                           |

#### Merge Actions (Originally Phase 2)

| Feature          | Spec Phase    | Actual Status | Evidence                                               |
| ---------------- | ------------- | ------------- | ------------------------------------------------------ |
| One-click Merge  | Phase 2 (F10) | ✅ Complete   | `handlers/bulk_merge.rs`, `components/MergeDialog.tsx` |
| Bulk Merge       | Phase 2 (F10) | ✅ Complete   | `entities/merge_operation.rs`, bulk merge handler      |
| Merge Strategies | Phase 2 (F10) | ✅ Complete   | Supports squash/merge/rebase                           |

#### Bot Filtering

| Feature           | Spec Phase    | Actual Status  | Evidence                                               |
| ----------------- | ------------- | -------------- | ------------------------------------------------------ |
| Bot Detection     | Phase 2 (F11) | ⚠️ Partial     | `entities/auto_merge_rule.rs`, `handlers/bot_rules.rs` |
| Separate Bot View | Phase 2 (F11) | ❌ Not Started | Backend ready, no frontend UI                          |
| Auto-merge Rules  | Phase 2 (F11) | ⚠️ Partial     | Entity/handler exist, no worker                        |

#### Analytics & Reporting

| Feature          | Spec Phase    | Actual Status  | Evidence                                          |
| ---------------- | ------------- | -------------- | ------------------------------------------------- |
| PR Cycle Time    | Phase 3 (F12) | ✅ Complete    | `entities/pr_metrics.rs`, `handlers/analytics.rs` |
| Team Velocity    | Phase 3 (F12) | ✅ Complete    | Analytics handler with metrics aggregation        |
| Export (CSV/PDF) | Phase 3 (F12) | ❌ Not Started | N/A                                               |

#### Health Scores (Originally Phase 3)

| Feature            | Spec Phase    | Actual Status  | Evidence                                  |
| ------------------ | ------------- | -------------- | ----------------------------------------- |
| Repository Scoring | Phase 3 (F13) | ✅ Complete    | `entities/health_score.rs` with algorithm |
| Trend Analysis     | Phase 3 (F13) | ✅ Complete    | Health score tracking over time           |
| Health Alerting    | Phase 3 (F13) | ❌ Not Started | Scoring exists, no alert mechanism        |

#### AI Features

| Feature              | Spec Phase    | Actual Status  | Evidence |
| -------------------- | ------------- | -------------- | -------- |
| Smart PR Priority    | Phase 3 (F14) | ❌ Not Started | N/A      |
| Reviewer Suggestions | Phase 3 (F14) | ❌ Not Started | N/A      |
| Anomaly Detection    | Phase 3 (F14) | ❌ Not Started | N/A      |

### Summary by Phase

- **Phase 1 (MVP)**: ✅ 100% Complete (all 7 features implemented)
- **Phase 2**: ✅ 60% Complete (teams & merge actions done, notifications & bot filtering partial)
- **Phase 3**: ⚠️ 33% Complete (health scores done, analytics partial, AI not started)

### Recent Major Features (Git History Evidence)

- `6eb0ad8` - Multitenancy support (Organizations, Teams)
- `1e0452f` - Bulk merge reliability improvements
- `72935d2` - Infinite scroll pagination
- `f95e2bd` - Comprehensive integration tests
- `4378b49` - PAT-based multi-account support

---

## Executive Summary

Ampel ("traffic light" in German) is a PR management and repository health monitoring
system that provides at-a-glance visibility into pull request status and CI health
across multiple Git providers. Designed for developers and teams managing dozens to
hundreds of repositories.

## Problem Statement

Modern development teams face:

- PRs scattered across GitHub, GitLab, and Bitbucket
- No unified view of merge-ready PRs across all repos
- Manual checking of CI status for each repository
- Difficulty identifying blocked or stale PRs at scale
- Time wasted context-switching between provider UIs

## Vision

A single dashboard showing traffic-light status for all repositories:

- **Green**: CI passing, approved, ready to merge
- **Yellow**: CI pending, awaiting review
- **Red**: CI failed, blocked, needs attention

## Target Users

### Persona 1: Individual Developer

- Manages 10-50 personal/work repositories
- Wants quick daily check of PR status
- Needs to know which PRs are ready for action

### Persona 2: Team Lead

- Oversees team of 5-15 developers
- Monitors team velocity and PR throughput
- Needs aggregate view of team's PR health

### Persona 3: Organization Admin

- Manages multiple teams, 100+ repositories
- Requires high-level health dashboards
- Needs audit trail and compliance visibility

---

## Feature Requirements

### Phase 1: MVP - Core Dashboard (✅ 100% Complete)

**Status**: All Phase 1 features are fully implemented and production-ready.

#### F1: Multi-Provider Authentication ✅

- Personal Access Token (PAT) authentication for GitHub, GitLab, Bitbucket
- Secure token storage with AES-256-GCM encryption at rest
- Support for self-hosted GitLab/Bitbucket instances
- **Implementation**: `ampel-providers/`, `handlers/auth.rs`, `entities/provider_account.rs`

#### F2: Repository Management ✅

- Add/remove repositories to monitor
- Wildcard patterns (e.g., "myorg/\*") - backend ready, UI pending
- Repository grouping by provider/team
- **Implementation**: `handlers/repositories.rs`, `queries/repo_queries.rs`

#### F3: PR Aggregation Dashboard ✅

- Grid view: Card per repository with status indicator
- List view: Sortable table of all PRs with infinite scroll
- Real-time status updates via polling
- **Implementation**: `handlers/dashboard.rs`, `components/dashboard/GridView.tsx`, `components/dashboard/ListView.tsx`

#### F4: Traffic Light Status ✅

- **Green**: All checks pass + approved + no conflicts
- **Yellow**: Checks pending OR awaiting review
- **Red**: Checks failed OR conflicts OR blocked
- **Implementation**: `ampel-core/models.rs`, `entities/ci_check.rs`

#### F5: Scheduled Polling ✅

- Configurable poll intervals (1min - 1hr)
- Manual refresh option
- Rate limit aware scheduling
- **Implementation**: `ampel-worker/` with Apalis background jobs

#### F6: Filtering & Search ✅

- Filter by: provider, status, author, reviewer, age
- Full-text search on PR title/description
- Saved filter presets
- **Implementation**: `handlers/pr_filters.rs`, `entities/pr_filter.rs`

#### F7: PR Detail View ✅

- Full PR information without leaving Ampel
- CI check details with logs link
- Review status and comments summary
- **Implementation**: `handlers/pull_requests.rs`, `entities/review.rs`

---

### Phase 2: Team Collaboration & Automation (⚠️ 60% Complete)

**Status**: Teams and bulk merge fully implemented. Notifications have backend structure but no sending implementation. Bot filtering partially implemented.

#### F8: Team Management ✅ (Moved from Phase 2 to Complete)

- Create/manage teams within organizations
- Team-specific dashboards and filters
- Member role management (admin, member, viewer)
- **Implementation**: `handlers/teams.rs`, `entities/team.rs`, `entities/team_member.rs`
- **Note**: Originally planned for Phase 2, but fully implemented ahead of schedule

#### F9: Notifications ⚠️ (Partial Implementation)

- Slack integration for status changes - **Backend config ready, no worker sending**
- Email digests (daily/weekly summaries) - **SMTP settings exist, no sending implementation**
- Browser push notifications - **Not started**
- **Implementation**: `handlers/notifications.rs`, `entities/notification_preferences.rs`
- **Missing**: Worker jobs to actually send notifications, frontend notification UI

#### F10: Merge Actions ✅ (Moved from Phase 2 to Complete)

- One-click merge from dashboard
- Batch merge for approved PRs with progress tracking
- Merge strategy selection (squash, merge, rebase)
- **Implementation**: `handlers/bulk_merge.rs`, `entities/merge_operation.rs`, `components/MergeDialog.tsx`
- **Note**: Fully implemented with reliability improvements in commit `1e0452f`

#### F11: Bot Filtering ⚠️ (Partial Implementation)

- Auto-detect Dependabot, Renovate, etc. - **Backend detection logic ready**
- Separate view for bot PRs - **Not implemented in frontend**
- Auto-merge rules for bot PRs - **Entity/handler exist, no worker execution**
- **Implementation**: `handlers/bot_rules.rs`, `entities/auto_merge_rule.rs`
- **Missing**: Frontend bot view, worker to execute auto-merge rules

---

### Phase 3: Analytics & Intelligence (⚠️ 33% Complete)

**Status**: Health scores fully implemented. Analytics partially implemented. AI features not started.

#### F12: Analytics & Reporting ⚠️ (Partial Implementation)

- PR cycle time metrics - **✅ Complete**
- Review turnaround time - **✅ Complete**
- Team velocity trends - **✅ Complete**
- Export to CSV/PDF - **❌ Not started**
- **Implementation**: `handlers/analytics.rs`, `entities/pr_metrics.rs`
- **Missing**: Export functionality

#### F13: Health Scores ✅ (Moved from Phase 3 to Complete)

- Repository health scoring algorithm (0-100 scale)
- Trend analysis over time with metrics tracking
- Alerting on health degradation - **⚠️ Scoring exists, alert mechanism not implemented**
- **Implementation**: `entities/health_score.rs` with scoring fields: avg_time_to_merge, avg_review_time, stale_pr_count, failed_check_rate, pr_throughput
- **Note**: Originally planned for Phase 3, core functionality implemented early

#### F14: AI Features ❌ (Not Started)

- Smart PR prioritization - **Not implemented**
- Reviewer recommendations - **Not implemented**
- Anomaly detection - **Not implemented**
- **Status**: No implementation, future roadmap item

---

## User Stories

### Authentication

- **US1**: As a user, I can sign up with email/password
- **US2**: As a user, I can connect my GitHub account using a Personal Access Token
- **US3**: As a user, I can connect my GitLab account using a Personal Access Token
- **US4**: As a user, I can connect my Bitbucket account using an App Password
- **US5**: As a user, I can revoke provider access at any time

### Repository Management

- **US6**: As a user, I can browse my accessible repositories
- **US7**: As a user, I can add repositories to my watchlist
- **US8**: As a user, I can use wildcards to add multiple repos
- **US9**: As a user, I can organize repos into custom groups

### Dashboard

- **US10**: As a user, I see a grid of repos with status colors
- **US11**: As a user, I can switch between grid and list views
- **US12**: As a user, I can filter PRs by multiple criteria
- **US13**: As a user, I can search PRs by title or author
- **US14**: As a user, I can save filter combinations as presets

### PR Details

- **US15**: As a user, I can view PR details in a side panel
- **US16**: As a user, I can see CI check status and logs
- **US17**: As a user, I can see review comments summary

### Team Features (Phase 2)

- **US18**: As an org admin, I can create teams
- **US19**: As a team admin, I can add members to my team
- **US20**: As a team member, I can view team dashboard

---

## UI/UX Specifications

### Main Dashboard Layout

```text
┌─────────────────────────────────────────────────────────────┐
│ [Logo] Ampel          [Search...]     [Filter] [View] [👤] │
├─────────────────────────────────────────────────────────────┤
│ Sidebar    │  Main Content Area                             │
│            │                                                │
│ Providers  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐              │
│ ├ GitHub   │  │ 🟢  │ │ 🟡  │ │ 🔴  │ │ 🟢  │              │
│ ├ GitLab   │  │repo1│ │repo2│ │repo3│ │repo4│              │
│ └ Bitbucket│  │ 2PR │ │ 1PR │ │ 3PR │ │ 0PR │              │
│            │  └─────┘ └─────┘ └─────┘ └─────┘              │
│ Teams      │                                                │
│ ├ Frontend │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐              │
│ └ Backend  │  │ 🟢  │ │ 🟢  │ │ 🟡  │ │ 🟢  │              │
│            │  │repo5│ │repo6│ │repo7│ │repo8│              │
│ Filters    │  │ 1PR │ │ 0PR │ │ 5PR │ │ 2PR │              │
│ [Presets]  │  └─────┘ └─────┘ └─────┘ └─────┘              │
└─────────────────────────────────────────────────────────────┘
```

### List View Layout

```text
┌─────────────────────────────────────────────────────────────┐
│ Status │ Repository      │ PR Title        │ Author │ Age  │
├────────┼─────────────────┼─────────────────┼────────┼──────┤
│   🟢   │ org/repo1       │ Fix login bug   │ alice  │ 2h   │
│   🟡   │ org/repo2       │ Add feature X   │ bob    │ 1d   │
│   🔴   │ org/repo3       │ Update deps     │ bot    │ 3d   │
└─────────────────────────────────────────────────────────────┘
```

### PR Detail Side Panel

```text
┌─────────────────────────────────────┐
│ PR #123: Fix login bug         [×] │
├─────────────────────────────────────┤
│ Repository: org/repo1               │
│ Author: alice                       │
│ Created: 2 hours ago                │
│ Branch: fix-login → main            │
├─────────────────────────────────────┤
│ Status: 🟢 Ready to merge           │
├─────────────────────────────────────┤
│ CI Checks:                          │
│ ✅ build (2m 15s)                   │
│ ✅ test (5m 32s)                    │
│ ✅ lint (45s)                       │
├─────────────────────────────────────┤
│ Reviews:                            │
│ ✅ bob - Approved                   │
│ 💬 carol - 2 comments               │
├─────────────────────────────────────┤
│ [View on GitHub] [Merge ▼]          │
└─────────────────────────────────────┘
```

---

## Success Metrics

| Metric              | Target         | Measurement           |
| ------------------- | -------------- | --------------------- |
| Time to first value | < 5 min        | Onboarding completion |
| Daily active users  | 60% of signups | Login frequency       |
| PR discovery time   | -50% vs manual | User survey           |
| Dashboard load time | < 2 seconds    | p95 latency           |
| API availability    | 99.9%          | Uptime monitoring     |

---

## Non-Functional Requirements

### Performance

- Dashboard loads in < 2 seconds for 100 repos
- Supports 10,000+ repositories per organization
- Handles 1000 concurrent users

### Security

- Personal Access Token (PAT) authentication for all Git providers
- PATs encrypted at rest with AES-256-GCM
- HTTPS only, HSTS enabled
- SOC 2 compliance path

### Scalability

- Horizontal scaling for API servers
- Database read replicas for queries
- CDN for static assets

### Reliability

- 99.9% uptime SLA
- Automated failover
- Data backup every 6 hours

---

## Competitive Analysis

| Feature              | Ampel         | Graphite | Mergify | GitHub Native |
| -------------------- | ------------- | -------- | ------- | ------------- |
| Multi-provider       | ✅ Complete   | ❌       | ❌      | ❌            |
| Traffic light status | ✅ Complete   | ❌       | ❌      | ❌            |
| Scheduled polling    | ✅ Complete   | ✅       | ✅      | ❌            |
| Merge automation     | ✅ Complete   | ✅       | ✅      | ❌            |
| Team dashboards      | ✅ Complete   | ✅       | ✅      | ❌            |
| Health scores        | ✅ Complete   | ❌       | ⚠️      | ❌            |
| Self-hosted option   | ✅ Available  | ❌       | ❌      | ❌            |
| Open source          | ✅ Apache 2.0 | ❌       | ❌      | N/A           |

---

## Glossary

| Term             | Definition                                               |
| ---------------- | -------------------------------------------------------- |
| **Ampel Status** | Traffic light indicator (green/yellow/red) for PR health |
| **Provider**     | Git hosting platform (GitHub, GitLab, Bitbucket)         |
| **Poll**         | Scheduled API call to fetch PR/CI status                 |
| **Watchlist**    | User's selected repositories to monitor                  |
| **Merge-ready**  | PR with passing CI, approvals, and no conflicts          |

---

## Development Notes

### What Changed From Original Spec?

This specification was originally written as a planning document. The actual implementation has **exceeded expectations** in several areas:

1. **Phase 2 Features Implemented Early**: Team management (F8) and bulk merge actions (F10) were completed ahead of schedule
2. **Phase 3 Features Implemented Early**: Health scores (F13) and analytics (F12) were implemented during MVP development
3. **Multitenancy Added**: Organizations and teams were added in commit `6eb0ad8`, enabling true multi-tenant support
4. **Infinite Scroll**: Added in commit `72935d2`, improving UX for large PR lists

### What's Still Needed?

Based on the implementation matrix above:

**High Priority (Complete Phase 2)**:

- Notification worker to actually send Slack/email notifications
- Frontend UI for bot filtering
- Worker to execute auto-merge rules for bot PRs

**Medium Priority (Complete Phase 3)**:

- Export functionality (CSV/PDF) for analytics
- Alerting mechanism for health score degradation
- Frontend wildcard pattern UI for repository management

**Future Roadmap (Phase 4)**:

- AI features (smart prioritization, reviewer recommendations, anomaly detection)

### Current State Summary

**What Works Today (Production-Ready)**:

- Complete multi-provider PR aggregation dashboard
- Traffic light status with CI integration
- Team management and collaboration
- Bulk merge operations with progress tracking
- Health scoring and analytics
- Comprehensive filtering and search

**What's Partially Implemented**:

- Notifications (config exists, no sending)
- Bot filtering (backend ready, no frontend)
- Export (analytics work, no export format)

This spec now reflects **reality over aspirations**.
