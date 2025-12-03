# Ampel - Product Specification

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

### MVP (Phase 1)

#### F1: Multi-Provider Authentication

- OAuth flow for GitHub, GitLab, Bitbucket
- Secure token storage with encryption at rest
- Support for self-hosted GitLab/Bitbucket instances

#### F2: Repository Management

- Add/remove repositories to monitor
- Wildcard patterns (e.g., "myorg/\*")
- Repository grouping by provider/team

#### F3: PR Aggregation Dashboard

- Grid view: Card per repository with status indicator
- List view: Sortable table of all PRs
- Real-time status updates via polling

#### F4: Traffic Light Status

- **Green**: All checks pass + approved + no conflicts
- **Yellow**: Checks pending OR awaiting review
- **Red**: Checks failed OR conflicts OR blocked

#### F5: Scheduled Polling

- Configurable poll intervals (1min - 1hr)
- Manual refresh option
- Rate limit aware scheduling

#### F6: Filtering & Search

- Filter by: provider, status, author, reviewer, age
- Full-text search on PR title/description
- Saved filter presets

#### F7: PR Detail View

- Full PR information without leaving Ampel
- CI check details with logs link
- Review status and comments summary

### Phase 2

#### F8: Team Management

- Create/manage teams within organizations
- Team-specific dashboards and filters
- Member role management (admin, member, viewer)

#### F9: Notifications

- Slack integration for status changes
- Email digests (daily/weekly summaries)
- Browser push notifications

#### F10: Merge Actions

- One-click merge from dashboard
- Batch merge for approved PRs
- Merge strategy selection (squash, merge, rebase)

#### F11: Bot Filtering

- Auto-detect Dependabot, Renovate, etc.
- Separate view for bot PRs
- Auto-merge rules for bot PRs

### Phase 3

#### F12: Analytics & Reporting

- PR cycle time metrics
- Review turnaround time
- Team velocity trends
- Export to CSV/PDF

#### F13: Health Scores

- Repository health scoring algorithm
- Trend analysis over time
- Alerting on health degradation

#### F14: AI Features

- Smart PR prioritization
- Reviewer recommendations
- Anomaly detection

---

## User Stories

### Authentication

- **US1**: As a user, I can sign up with email/password
- **US2**: As a user, I can connect my GitHub account via OAuth
- **US3**: As a user, I can connect my GitLab account via OAuth
- **US4**: As a user, I can connect my Bitbucket account via OAuth
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

- OAuth 2.0 for all provider authentication
- Tokens encrypted at rest (AES-256)
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

| Feature              | Ampel   | Graphite | Mergify | GitHub Native |
| -------------------- | ------- | -------- | ------- | ------------- |
| Multi-provider       | ✅      | ❌       | ❌      | ❌            |
| Traffic light status | ✅      | ❌       | ❌      | ❌            |
| Scheduled polling    | ✅      | ✅       | ✅      | ❌            |
| Merge automation     | Phase 2 | ✅       | ✅      | ❌            |
| Team dashboards      | Phase 2 | ✅       | ✅      | ❌            |
| Self-hosted option   | ✅      | ❌       | ❌      | ❌            |
| Open source          | ✅      | ❌       | ❌      | N/A           |

---

## Glossary

| Term             | Definition                                               |
| ---------------- | -------------------------------------------------------- |
| **Ampel Status** | Traffic light indicator (green/yellow/red) for PR health |
| **Provider**     | Git hosting platform (GitHub, GitLab, Bitbucket)         |
| **Poll**         | Scheduled API call to fetch PR/CI status                 |
| **Watchlist**    | User's selected repositories to monitor                  |
| **Merge-ready**  | PR with passing CI, approvals, and no conflicts          |
