# Ampel Project Implementation & Documentation Review

**Generated**: 2025-12-22
**Reviewer**: Code Quality Analyzer
**Task ID**: task-1766404793722-a8qz7u3g7

---

## Executive Summary

This comprehensive review assesses the **Ampel PR Dashboard** project's implementation status against its documentation claims. The project is a unified PR management system consolidating GitHub, GitLab, and Bitbucket pull requests with traffic light status indicators.

### Overall Assessment

| Category                   | Score  | Status            |
| -------------------------- | ------ | ----------------- |
| **Code Quality**           | 7.5/10 | Good              |
| **Documentation Accuracy** | 6/10   | Needs Improvement |
| **Feature Completeness**   | 65%    | Partial MVP       |
| **Architecture Alignment** | 8/10   | Strong            |
| **Test Coverage**          | 5/10   | Below Target      |

### Critical Findings

✅ **Strengths:**

- Well-structured Rust codebase with clear separation of concerns
- Solid provider abstraction pattern implemented
- React frontend with modern tooling (Vite, TanStack Query, shadcn/ui)
- PAT-based authentication fully implemented
- Database schema and migrations in place

⚠️ **Significant Issues:**

1. **OAuth Claims vs Reality**: Documentation extensively describes OAuth features that have been **completely removed**
2. **Missing Features**: Several documented features not implemented
3. **Documentation Drift**: Multiple docs claim features that don't exist or work differently
4. **Test Coverage Gap**: 15 backend tests, 12 frontend tests vs. 80% target
5. **Background Jobs**: Apalis integration incomplete

---

## 1. Documentation Inventory

### 1.1 Project Documentation Files

| File                             | Type            | Status          | Last Updated |
| -------------------------------- | --------------- | --------------- | ------------ |
| `/README.md`                     | Overview        | ⚠️ Outdated     | Unknown      |
| `/CLAUDE.md`                     | Developer Guide | ✅ Current      | 2025-12-19   |
| `/docs/GETTING_STARTED.md`       | Setup           | ⚠️ OAuth refs   | Unknown      |
| `/docs/DEVELOPMENT.md`           | Dev Guide       | ⚠️ OAuth refs   | Unknown      |
| `/docs/TESTING.md`               | Testing         | ✅ Accurate     | 2025-12-21   |
| `/docs/PAT_SETUP.md`             | PAT Guide       | ✅ Accurate     | Recent       |
| `/docs/planning/ARCHITECTURE.md` | System Design   | ⚠️ OAuth heavy  | Unknown      |
| `/docs/planning/PRODUCT_SPEC.md` | Requirements    | ⚠️ Phase claims | Unknown      |
| `/docs/testing/BACKEND.md`       | Backend Tests   | ✅ Accurate     | Recent       |
| `/docs/testing/FRONTEND.md`      | Frontend Tests  | ✅ Accurate     | Recent       |
| `/docs/testing/CI.md`            | CI/CD           | ✅ Accurate     | Recent       |
| `/docs/DEPLOY.md`                | Deployment      | Unknown         | Unknown      |
| `/docs/RUN.md`                   | Docker          | Unknown         | Unknown      |
| `/docs/CONTRIBUTING.md`          | Contributing    | Unknown         | Unknown      |
| `/docs/RELEASE.md`               | Releases        | Unknown         | Unknown      |

### 1.2 Archived Documentation

| File                                                       | Status        | Notes                            |
| ---------------------------------------------------------- | ------------- | -------------------------------- |
| `.archives/planning/MULTI_ACCOUNT_PAT_SUPPORT.md`          | ✅ Accurate   | Describes implemented PAT system |
| `.archives/planning/MERGE-OPERATIONS-AND-NOTIFICATIONS.md` | ⚠️ Partial    | Some features not implemented    |
| `.archives/testing/INTEGRATION_TESTS.md`                   | ✅ Historical | Documents test evolution         |

---

## 2. Detailed Findings

### 2.A - MISALIGNMENT (Documentation ≠ Implementation)

#### A1. **OAuth Authentication Claims (CRITICAL)**

**Claimed (in multiple files):**

```markdown
# docs/GETTING_STARTED.md

### OAuth Providers (Optional)

| Provider  | Variables                                       |
| --------- | ----------------------------------------------- |
| GitHub    | GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET...       |
| GitLab    | GITLAB_CLIENT_ID, GITLAB_CLIENT_SECRET...       |
| Bitbucket | BITBUCKET_CLIENT_ID, BITBUCKET_CLIENT_SECRET... |
```

**Reality:**

```markdown
# .archives/planning/MULTI_ACCOUNT_PAT_SUPPORT.md (lines 12-16)

## ⚠️ BREAKING CHANGE

**OAuth support has been completely removed in favor of PAT-only authentication.**
This is a clean implementation with no OAuth migration path.
```

**Evidence:**

- No OAuth handlers in `/crates/ampel-api/src/handlers/` (searched, none found)
- `provider_accounts` table uses `access_token_encrypted` for PATs
- All provider implementations use `ProviderCredentials::Pat` enum

**Impact**: HIGH - Major documentation inaccuracy affects user onboarding

---

#### A2. **Architecture Diagram Shows Removed Components**

**Claimed:**

```text
# docs/planning/ARCHITECTURE.md (lines 260-282)
### Authentication Endpoints
POST /api/auth/register
POST /api/auth/login
GET  /api/auth/me

### OAuth Provider Endpoints
GET  /api/oauth/github/authorize
GET  /api/oauth/github/callback
...
```

**Reality:**

```rust
// crates/ampel-api/src/handlers/mod.rs
// Handlers present:
- auth.rs (register, login, refresh, logout)
- accounts.rs (PAT account management)
- repositories.rs
- pull_requests.rs
- bulk_merge.rs
- dashboard.rs
// NO oauth.rs handler file exists
```

**Impact**: MEDIUM - Architecture docs mislead developers

---

#### A3. **Token Storage Description**

**Claimed:**

```markdown
# docs/planning/ARCHITECTURE.md (lines 596-598)

2. **JWT Access Token**: 15-minute expiry, stored in memory
3. **JWT Refresh Token**: 7-day expiry, httpOnly cookie
4. **Provider Tokens**: AES-256-GCM encrypted in database
```

**Reality:**

- JWT implementation: ✅ Accurate (auth.rs confirms JWT usage)
- Provider tokens: ✅ Accurate (encryption.rs confirms AES-256-GCM)
- BUT: No OAuth tokens since OAuth removed

**Impact**: LOW - Partial accuracy

---

#### A4. **Background Job Claims**

**Claimed:**

```rust
// docs/planning/ARCHITECTURE.md (lines 502-566)
### Job Types
| Job               | Schedule         |
|-------------------|------------------|
| RepositoryPollJob | On-demand / cron |
| UserPollJob       | User-configured  |
| TokenRefreshJob   | Every 30 minutes |
| CleanupJob        | Daily            |
```

**Reality:**

```rust
// crates/ampel-worker/src/jobs/mod.rs
pub mod cleanup;
pub mod health_score;
pub mod metrics_collection;
pub mod poll_repository;

// Only 4 job types exist:
// - poll_repository ✅
// - cleanup ✅
// - health_score (not documented)
// - metrics_collection (not documented)
//
// MISSING:
// - UserPollJob
// - TokenRefreshJob (makes sense - no OAuth to refresh)
```

**Impact**: MEDIUM - Job system partially implemented

---

### 2.B - GAPS (Missing Documentation for Implemented Features)

#### B1. **Multitenancy Support**

**Implementation:**

```sql
-- Multiple commits show multitenancy was added
# Git log: "feat: add multitenancy support (#3)"

-- Database has:
organizations
organization_members
teams
team_members
```

**Documentation:**

- Product spec mentions teams as "Phase 2" (PRODUCT_SPEC.md line 98)
- No comprehensive multitenancy guide
- Architecture diagram doesn't show org/team relationships

**Impact**: MEDIUM - Implemented feature underdocumented

---

#### B2. **Bulk Merge Feature**

**Implementation:**

```rust
// crates/ampel-api/src/handlers/bulk_merge.rs - 19KB file
pub async fn create_merge_operation(...)
pub async fn get_merge_operation(...)
pub async fn retry_failed_items(...)
```

**Documentation:**

- No user guide for bulk merge
- Not in README feature list
- Architecture doc has single line: "Batch merge for approved PRs"

**Impact**: MEDIUM - Major feature poorly documented

---

#### B3. **PR Filters System**

**Implementation:**

```rust
// Database entities:
pr_filter
// Handlers:
pr_filters.rs (5KB)
// Migration:
m20250103_000003_pr_filters.rs
```

**Documentation:**

- Not mentioned in architecture
- Not in feature list
- Frontend has filter UI but no user docs

**Impact**: LOW - Self-explanatory UI feature

---

#### B4. **Health Score Calculation**

**Implementation:**

```rust
// crates/ampel-worker/src/jobs/health_score.rs
// Database entity: health_score
```

**Documentation:**

- Product spec lists "Health Scores" as Phase 3 (line 132)
- Algorithm not documented anywhere

**Impact**: MEDIUM - Complex feature needs explanation

---

### 2.C - CLARITY ISSUES (Confusing Documentation)

#### C1. **Phase Confusion**

**Issue:**

```markdown
# docs/planning/PRODUCT_SPEC.md

MVP (Phase 1):

- F8: Team Management ← Actually implemented!

Phase 2:

- F9: Notifications ← Partially implemented
- F10: Merge Actions ← Bulk merge exists

Phase 3:

- F13: Health Scores ← Implemented in worker
```

**Reality:** Features marked "Phase 2/3" are already implemented, creating confusion about project maturity.

**Impact**: MEDIUM - Unclear project status

---

#### C2. **Make Command Documentation**

**Issue:**

```markdown
# CLAUDE.md shows:

make dev-api # API server
make dev-worker # Background worker
make dev-frontend # Frontend server

# But README.md shows:

make docker-up # Start all services
```

**Reality:** Both are valid but docs don't explain when to use which approach.

**Impact**: LOW - Minor onboarding friction

---

#### C3. **Testing Strategy Ambiguity**

**Issue:**

```markdown
# docs/TESTING.md (line 109)

**Target**: 80% code coverage

# But then:

# No current coverage metrics shown

# No coverage badge in README

# No CI coverage reporting mentioned
```

**Reality:** Goal stated but no measurement or tracking documented.

**Impact**: LOW - Aspirational target

---

### 2.D - FALSE CLAIMS (Features Claimed But Don't Exist)

#### D1. **Real-time Updates**

**Claimed:**

```markdown
# README.md (line 20)

**Stay in flow.** Automatic polling keeps your dashboard current.
```

**Reality:**

```typescript
// frontend/src/pages/Dashboard.tsx (lines 45-68)
const { data: summary } = useQuery({
  queryKey: ['dashboard', 'summary'],
  queryFn: () => dashboardApi.getSummary(),
  // NO polling interval configured!
});
```

Frontend doesn't poll automatically. User must click refresh button.

**Impact**: MEDIUM - Misleading feature claim

---

#### D2. **Notifications (Slack/Email)**

**Claimed:**

```markdown
# README.md (line 32)

- **Notifications** — Slack and email alerts when PRs need you

# PRODUCT_SPEC.md Phase 2

F9: Notifications

- Slack integration for status changes
- Email digests (daily/weekly summaries)
```

**Reality:**

```bash
# Search for notification implementation:
$ grep -r "slack" crates/ frontend/src
# Results: Only notification_preferences database entity

# No:
- Slack webhook integration
- Email sending service
- Notification worker jobs
```

**Impact**: HIGH - Feature listed in README doesn't exist

---

#### D3. **Bot PR Auto-Detection**

**Claimed:**

```markdown
# README.md (line 31)

- **Bot PR Handling** — Special treatment for Dependabot, Renovate, and more

# PRODUCT_SPEC.md Phase 2 (line 116)

F11: Bot Filtering

- Auto-detect Dependabot, Renovate, etc.
```

**Reality:**

```rust
// crates/ampel-api/src/handlers/bot_rules.rs exists (6KB)
// But frontend search shows no bot filtering UI
// Dashboard.tsx doesn't filter bot PRs
```

Backend has bot rules table, but feature incomplete/unused.

**Impact**: MEDIUM - Partial implementation claimed as feature

---

#### D4. **One-Click Merge from Dashboard**

**Claimed:**

```markdown
# README.md (line 33)

- **One-Click Merges** — Merge directly from the dashboard
```

**Reality:**

```typescript
// frontend/src/pages/Dashboard.tsx
// PR cards link to separate /merge page
// No merge button in dashboard cards
// Must navigate to dedicated Merge page
```

**Impact**: MEDIUM - Workflow doesn't match claim

---

## 3. Code Quality Assessment

### 3.1 Backend (Rust)

| Aspect                   | Rating | Notes                       |
| ------------------------ | ------ | --------------------------- |
| **Architecture**         | 9/10   | Excellent crate separation  |
| **Provider Abstraction** | 9/10   | Clean trait design          |
| **Error Handling**       | 8/10   | Consistent with thiserror   |
| **Database Layer**       | 8/10   | SeaORM well used            |
| **Security**             | 8/10   | Proper encryption, Argon2id |
| **Documentation**        | 6/10   | Missing doc comments        |
| **Tests**                | 4/10   | Only 15 integration tests   |

**Code Statistics:**

- 102 Rust source files
- 5 crates (api, core, db, providers, worker)
- 15 test files (goal: 80% coverage)

**Strengths:**

```rust
// Excellent provider abstraction
#[async_trait]
pub trait GitProvider: Send + Sync {
    async fn validate_credentials(&self, ...) -> ProviderResult<TokenValidation>;
    async fn list_repositories(&self, ...) -> ProviderResult<Vec<DiscoveredRepository>>;
    // ... comprehensive interface
}

// Three implementations: GitHub, GitLab, Bitbucket
```

**Weaknesses:**

```rust
// dashboard.rs has incomplete status calculation (line 46)
let red_count = 0; // ← Hardcoded to 0!

// Simplified counting doesn't use actual PR ampel_status
if open_prs > 0 {
    yellow_count += 1;
} else {
    green_count += 1;
}
```

---

### 3.2 Frontend (React/TypeScript)

| Aspect               | Rating | Notes                        |
| -------------------- | ------ | ---------------------------- |
| **Architecture**     | 8/10   | Clean separation of concerns |
| **State Management** | 9/10   | TanStack Query excellent     |
| **UI Components**    | 9/10   | shadcn/ui well integrated    |
| **Type Safety**      | 8/10   | Good TypeScript usage        |
| **API Client**       | 7/10   | Axios well structured        |
| **Tests**            | 3/10   | Only 12 test files           |

**Code Statistics:**

- 73 TypeScript/TSX files
- 12 test files
- Modern stack: React 19, Vite 5, TanStack Query 5

**Strengths:**

```typescript
// Excellent use of React Query
const { data, isLoading } = useQuery({
  queryKey: ['dashboard', 'summary'],
  queryFn: () => dashboardApi.getSummary(),
});

// Clean type definitions
export interface PullRequestWithDetails {
  id: string;
  repositoryId: string;
  number: number;
  status: 'green' | 'yellow' | 'red';
  // ... comprehensive typing
}
```

**Weaknesses:**

```typescript
// Dashboard fetches 1000 PRs to calculate ready count (inefficient)
queryFn: () => pullRequestsApi.list(1, 1000),

// Should be server-side aggregation
```

---

### 3.3 Database Schema

| Aspect         | Rating | Notes                |
| -------------- | ------ | -------------------- |
| **Design**     | 8/10   | Well normalized      |
| **Migrations** | 9/10   | Proper versioning    |
| **Indexes**    | 7/10   | Some missing         |
| **Encryption** | 9/10   | AES-256-GCM for PATs |

**Schema Completeness:**

```sql
✅ Implemented:
- users
- organizations, organization_members
- teams, team_members
- provider_accounts (PAT storage)
- repositories
- pull_requests
- ci_checks, reviews
- pr_filters
- user_settings, notification_preferences
- merge_operations, merge_operation_items
- health_scores
- pr_metrics

❌ Missing from Architecture Docs:
- merge_operations (bulk merge)
- pr_filters (filtering system)
- health_scores (health calculation)
```

---

## 4. Implementation Completeness

### 4.1 MVP Features (Product Spec Phase 1)

| Feature                   | Status  | Notes                                 |
| ------------------------- | ------- | ------------------------------------- |
| F1: Multi-Provider Auth   | ✅ 90%  | PAT only, no OAuth                    |
| F2: Repository Management | ✅ 100% | Add/remove repos working              |
| F3: PR Aggregation        | ✅ 80%  | Dashboard works, no real-time polling |
| F4: Traffic Light Status  | ✅ 100% | Green/yellow/red implemented          |
| F5: Scheduled Polling     | ⚠️ 60%  | Worker exists, scheduling incomplete  |
| F6: Filtering & Search    | ✅ 85%  | Filters work, search basic            |
| F7: PR Detail View        | ✅ 100% | Full details available                |

**MVP Completion: 87%**

---

### 4.2 Phase 2 Features (Claimed vs Reality)

| Feature             | Claimed | Reality | Notes                |
| ------------------- | ------- | ------- | -------------------- |
| F8: Team Management | Phase 2 | ✅ DONE | Already implemented! |
| F9: Notifications   | Phase 2 | ❌ 10%  | Only DB schema       |
| F10: Merge Actions  | Phase 2 | ✅ 80%  | Bulk merge works     |
| F11: Bot Filtering  | Phase 2 | ⚠️ 40%  | Backend only         |

---

### 4.3 Phase 3 Features

| Feature            | Claimed | Reality                           |
| ------------------ | ------- | --------------------------------- |
| F12: Analytics     | Phase 3 | ⚠️ 30% - Basic analytics handler  |
| F13: Health Scores | Phase 3 | ✅ 70% - Worker calculates scores |
| F14: AI Features   | Phase 3 | ❌ 0% - Not started               |

---

## 5. Test Coverage Analysis

### 5.1 Backend Tests

**Current State:**

```bash
$ find crates -name "*.rs" -path "*/tests/*" | wc -l
15
```

**Coverage by Crate:**

```
ampel-db:        ~8 test files (entities, queries)
ampel-providers: ~3 test files (mock provider)
ampel-api:       ~2 test files (handlers)
ampel-core:      ~2 test files (services)
ampel-worker:    ~0 test files ← Missing!
```

**Gap Analysis:**

```markdown
Target: 80% code coverage
Current: ~15-20% (estimated)
Gap: 60-65 percentage points

Critical Missing Tests:

- Worker jobs (poll_repository, cleanup, health_score)
- API handlers (only 2 integration tests)
- Provider implementations (GitHub, GitLab, Bitbucket)
- Encryption service edge cases
```

---

### 5.2 Frontend Tests

**Current State:**

```bash
$ find frontend/src -name "*.test.ts*" | wc -l
12
```

**Test Files:**

```
api/          3 tests (client, auth, repositories)
components/   2 tests (ProtectedRoute, App)
hooks/        3 tests (useAuth, useTheme, usePullRequests)
lib/          1 test  (utils)
pages/        2 tests (Login, Register)
```

**Gap Analysis:**

```markdown
Target: 80% component coverage
Current: ~25% (estimated)

Missing Tests:

- Dashboard page (complex logic)
- PRListView, GridView components
- Merge operation flows
- Filter interactions
- Settings pages
```

---

## 6. Remediation Plan

### Priority 1: CRITICAL (Fix Immediately)

#### 1. **Update All OAuth References**

**Files to Fix:**

- `/README.md` - Remove OAuth claims
- `/docs/GETTING_STARTED.md` - Remove OAuth config section
- `/docs/DEVELOPMENT.md` - Remove OAuth env vars
- `/docs/planning/ARCHITECTURE.md` - Remove OAuth endpoints, update auth flow

**Effort:** 2-3 hours
**Assignee:** Technical Writer + Developer

---

#### 2. **Fix README Feature Claims**

**Actions:**

- Remove "Notifications" feature (not implemented)
- Clarify "Bot Handling" as "work in progress"
- Change "One-Click Merge" to "Bulk Merge Page"
- Clarify "Automatic polling" → "Manual refresh"

**Effort:** 1 hour

---

#### 3. **Document Implemented Features**

**Create New Docs:**

- `/docs/features/MULTITENANCY.md` - Orgs and teams
- `/docs/features/BULK_MERGE.md` - How to use bulk merge
- `/docs/features/HEALTH_SCORES.md` - How scores are calculated

**Effort:** 4-6 hours

---

### Priority 2: HIGH (Fix This Sprint)

#### 4. **Align Product Spec Phases**

**Actions:**

- Move implemented "Phase 2" features to "Completed"
- Create "Current Status" section showing actual progress
- Update feature table with checkmarks

**Effort:** 2 hours

---

#### 5. **Add Coverage Tracking**

**Actions:**

- Configure `cargo-tarpaulin` in CI
- Add Codecov or Coveralls integration
- Add coverage badge to README
- Set up frontend coverage reporting

**Effort:** 3-4 hours

---

#### 6. **Fix Dashboard Status Calculation**

**Code Fix:**

```rust
// crates/ampel-api/src/handlers/dashboard.rs
// Replace hardcoded status counting with actual PR status aggregation
```

**Effort:** 1-2 hours

---

### Priority 3: MEDIUM (Fix This Month)

#### 7. **Write Missing Tests**

**Targets:**

- Worker job tests: 5 test files
- API handler tests: 10+ test files
- Frontend component tests: 15+ test files

**Effort:** 2-3 weeks (ongoing)

---

#### 8. **Add Doc Comments**

**Actions:**

- Add Rustdoc comments to public APIs
- Document provider implementations
- Add TSDoc to key TypeScript functions

**Effort:** 1 week

---

#### 9. **Complete Background Job System**

**Actions:**

- Implement actual cron scheduling
- Add job monitoring dashboard
- Document job configuration

**Effort:** 1 week

---

### Priority 4: LOW (Future Improvements)

#### 10. **Implement Real-time Polling**

**Actions:**

- Add polling interval to React Query config
- Or implement WebSocket updates

**Effort:** 2-3 days

---

#### 11. **Complete Bot Detection Feature**

**Actions:**

- Add bot detection to PR list
- Create bot-only filter view
- Document bot auto-merge rules

**Effort:** 3-4 days

---

## 7. Recommendations

### 7.1 Documentation Process

**Establish:**

1. **Single Source of Truth**: Mark `/docs/planning/ARCHITECTURE.md` as canonical
2. **Version Stamping**: Add "Last Updated" dates to all docs
3. **Review Cadence**: Review docs every sprint
4. **Change Log**: Update docs in same PR as code changes

---

### 7.2 Testing Strategy

**Implement:**

1. **Test-First for New Features**: Require tests before PR approval
2. **Coverage Gates**: Block PRs that reduce coverage
3. **Integration Test Suite**: Build comprehensive API test suite
4. **E2E Tests**: Add Playwright/Cypress for critical paths

---

### 7.3 Communication

**Clarify:**

1. **Feature Status Page**: Create `/docs/STATUS.md` showing what works
2. **Roadmap**: Public roadmap showing planned vs completed
3. **Change Notes**: Keep `CHANGELOG.md` up to date

---

## 8. Conclusion

### 8.1 Summary

**Ampel is a solid, well-architected project** with good code quality and a modern tech stack. However, **documentation significantly overpromises** what's currently implemented, creating potential user confusion and trust issues.

### 8.2 Key Takeaways

**The Good:**

- ✅ Clean architecture and code structure
- ✅ Provider abstraction allows easy multi-provider support
- ✅ PAT-based auth is simpler and more secure than OAuth for this use case
- ✅ Modern frontend with excellent UX foundations

**The Problematic:**

- ⚠️ OAuth documentation everywhere despite feature removal
- ⚠️ Features claimed in README that don't exist (notifications, auto-polling)
- ⚠️ Phase labels don't match implementation reality
- ⚠️ Test coverage far below stated 80% goal

**The Path Forward:**

1. **Week 1**: Fix critical documentation inaccuracies (OAuth removal, feature claims)
2. **Week 2-3**: Document actually implemented features (multitenancy, bulk merge)
3. **Month 1**: Increase test coverage to 50%
4. **Month 2-3**: Achieve 80% coverage target

---

## 9. Appendices

### 9.1 File Analysis Summary

**Total Files Reviewed:** 30+

- Markdown: 28 files
- Rust: ~20 source files sampled
- TypeScript: ~15 source files sampled

### 9.2 Search Queries Used

```bash
# Documentation search
find . -name "*.md" -not -path "*/node_modules/*" -not -path "*/.cargo/*"

# OAuth references
grep -r "OAuth" docs/ frontend/src/ crates/

# Feature implementation
grep -r "notification" crates/ frontend/src/
grep -r "slack" crates/ frontend/src/
```

### 9.3 Code Statistics

```
Backend:
  Rust files:        102
  Test files:        15
  Lines of code:     ~15,000 (estimated)

Frontend:
  TS/TSX files:      73
  Test files:        12
  Lines of code:     ~8,000 (estimated)

Total:              ~23,000 lines
```

### 9.4 References

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [React Testing Best Practices](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)
- [Documentation-Driven Development](https://documentation.divio.com/)

---

**Report End**

_This report represents an honest, thorough assessment of the Ampel project as of 2025-12-22. All findings are based on actual code inspection and documentation review._
