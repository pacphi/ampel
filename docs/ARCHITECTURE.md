# Ampel Architecture Documentation

**Last Updated**: 2025-12-22
**Version**: 1.0
**Status**: Reflects actual implementation

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Tech Stack](#2-tech-stack)
3. [Architecture Principles](#3-architecture-principles)
4. [Authentication & Security](#4-authentication--security)
5. [Database Schema](#5-database-schema)
6. [Backend Architecture](#6-backend-architecture)
7. [Frontend Architecture](#7-frontend-architecture)
8. [Provider Abstraction](#8-provider-abstraction)
9. [Background Job System](#9-background-job-system)
10. [API Endpoints](#10-api-endpoints)
11. [Deployment](#11-deployment)
12. [Monitoring & Observability](#12-monitoring--observability)

---

## 1. System Overview

### 1.1 Purpose

Ampel is a unified pull request management dashboard that consolidates PRs from GitHub, GitLab, and Bitbucket into a single interface. It uses a traffic light system to indicate PR readiness:

- **Green**: Ready to merge (all checks pass, approved)
- **Yellow**: In progress (CI running, needs review)
- **Red**: Blocked (CI failed, conflicts, changes requested)

### 1.2 Core Features

- **Multi-Provider Support**: Connect multiple accounts across GitHub, GitLab, and Bitbucket
- **Personal Access Token (PAT) Authentication**: Secure token-based authentication (no OAuth)
- **Multitenancy**: Organizations and teams for collaborative PR management
- **Traffic Light Status**: Quick visual indication of PR health
- **Bulk Merge**: Merge multiple ready PRs in batch operations
- **PR Filtering**: Customizable filters for PR organization
- **Health Scores**: Calculated metrics for repository and PR health
- **Background Polling**: Automatic PR data synchronization

### 1.3 Key Design Decisions

- **PAT-Only Authentication**: OAuth support was removed in favor of simpler, more secure PAT-based authentication
- **Encrypted Token Storage**: All provider tokens encrypted with AES-256-GCM
- **Provider Abstraction**: Unified trait-based interface for all Git providers
- **Multitenancy**: Built-in support for organizations and teams
- **Async-First**: Fully asynchronous Rust backend with Tokio runtime

---

## 2. Tech Stack

### 2.1 Backend

| Component            | Technology   | Version | Purpose                       |
| -------------------- | ------------ | ------- | ----------------------------- |
| **Language**         | Rust         | 1.92.0  | Type-safe, performant backend |
| **Web Framework**    | Axum         | Latest  | HTTP server and routing       |
| **Database ORM**     | SeaORM       | Latest  | Type-safe database operations |
| **Database**         | PostgreSQL   | 16+     | Primary data store            |
| **Caching**          | Redis        | 7+      | Session and cache storage     |
| **Async Runtime**    | Tokio        | Latest  | Asynchronous execution        |
| **Background Jobs**  | Apalis       | Latest  | Job queue and processing      |
| **Serialization**    | Serde        | Latest  | JSON serialization            |
| **HTTP Client**      | Reqwest      | Latest  | Provider API calls            |
| **Password Hashing** | Argon2       | Latest  | Secure password hashing       |
| **JWT**              | jsonwebtoken | Latest  | Token-based authentication    |
| **Encryption**       | AES-256-GCM  | -       | Provider token encryption     |

### 2.2 Frontend

| Component            | Technology      | Version | Purpose                       |
| -------------------- | --------------- | ------- | ----------------------------- |
| **Language**         | TypeScript      | Latest  | Type-safe frontend code       |
| **Framework**        | React           | 19      | UI framework                  |
| **Build Tool**       | Vite            | 5       | Fast development and bundling |
| **State Management** | TanStack Query  | 5       | Server state caching          |
| **Forms**            | React Hook Form | Latest  | Form handling                 |
| **Validation**       | Zod             | Latest  | Schema validation             |
| **UI Components**    | shadcn/ui       | Latest  | Component library             |
| **Styling**          | Tailwind CSS    | Latest  | Utility-first CSS             |
| **HTTP Client**      | Axios           | Latest  | API communication             |
| **Routing**          | React Router    | Latest  | Client-side routing           |

### 2.3 Development Tools

- **Package Manager**: pnpm 10.24.0 (frontend), Cargo (backend)
- **Testing**: Vitest (frontend), cargo test (backend)
- **Linting**: ESLint (frontend), Clippy (backend)
- **Formatting**: Prettier (frontend), rustfmt (backend)

---

## 3. Architecture Principles

### 3.1 Separation of Concerns

The backend is organized into distinct crates with clear responsibilities:

```
crates/
├── ampel-api/          # HTTP handlers, routes, middleware
├── ampel-core/         # Business logic, domain models
├── ampel-db/           # Database entities, migrations, queries
├── ampel-providers/    # Git provider abstractions
└── ampel-worker/       # Background job processing
```

### 3.2 Clean Architecture Layers

1. **Presentation Layer** (`ampel-api`): HTTP handlers, request/response models
2. **Business Logic Layer** (`ampel-core`): Services, domain models, business rules
3. **Data Access Layer** (`ampel-db`): Database entities, queries, migrations
4. **External Integrations** (`ampel-providers`): Git provider implementations
5. **Background Processing** (`ampel-worker`): Asynchronous jobs

### 3.3 Error Handling

- Custom error types using `thiserror` crate
- Consistent error responses with HTTP status codes
- Provider-specific errors mapped to common types
- Detailed error logging for debugging

### 3.4 Security Principles

- **Zero Trust**: All API endpoints require authentication (except login/register)
- **Encryption at Rest**: Provider tokens encrypted in database
- **Secure Password Storage**: Argon2id password hashing
- **JWT Tokens**: Short-lived access tokens (15 min), refresh tokens in httpOnly cookies (7 days)
- **Input Validation**: All inputs validated with type-safe schemas

---

## 4. Authentication & Security

### 4.1 Authentication Flow

#### User Authentication

1. **Registration**:
   - User provides email and password
   - Password hashed with Argon2id
   - User record created in database
   - JWT access token (15 min) and refresh token (7 days) issued

2. **Login**:
   - User provides email and password
   - Password verified against hash
   - JWT access token and refresh token issued
   - Refresh token stored in httpOnly cookie

3. **Token Refresh**:
   - Client sends refresh token
   - New access token issued
   - Refresh token rotated (new refresh token issued)

#### Provider Account Authentication

1. **Add Provider Account**:
   - User provides PAT token for GitHub/GitLab/Bitbucket
   - Token validated against provider API
   - Token encrypted with AES-256-GCM
   - Encrypted token stored in `provider_accounts` table

2. **Token Validation**:
   - Backend decrypts token
   - Calls provider API to validate token
   - Returns user info, scopes, and expiry

### 4.2 Token Storage

| Token Type              | Storage Location | Encryption    | Expiry             |
| ----------------------- | ---------------- | ------------- | ------------------ |
| **JWT Access Token**    | Client memory    | None (signed) | 15 minutes         |
| **JWT Refresh Token**   | httpOnly cookie  | None (signed) | 7 days             |
| **Provider PAT Tokens** | Database         | AES-256-GCM   | Provider-dependent |

### 4.3 Encryption

**Provider Token Encryption**:

- Algorithm: AES-256-GCM
- Key: 32-byte base64-encoded key (set in `ENCRYPTION_KEY` env var)
- Nonce: 96-bit random nonce per encryption
- Storage: `{nonce}:{ciphertext}` format

**Implementation**: `crates/ampel-db/src/encryption.rs`

### 4.4 Authorization

- **User-scoped Resources**: Users can only access their own PRs, repos, and settings
- **Organization Access**: Users must be organization members to access org resources
- **Team Access**: Team membership controls access to team resources

---

## 5. Database Schema

### 5.1 Schema Overview

The database schema supports multitenancy, multiple provider accounts, and comprehensive PR tracking.

### 5.2 Repository Visibility

Ampel tracks repository visibility through two boolean fields on the `repositories` table:

- **`is_private`**: Indicates if the repository is private (requires authentication to access)
- **`is_archived`**: Indicates if the repository has been archived (read-only, usually inactive)

**Visibility Classification**:

| Type         | Condition                                    | Description                               |
| ------------ | -------------------------------------------- | ----------------------------------------- |
| **Public**   | `is_private = false AND is_archived = false` | Publicly accessible, active repository    |
| **Private**  | `is_private = true AND is_archived = false`  | Private, active repository                |
| **Archived** | `is_archived = true`                         | Archived repository (may also be private) |

**Provider Support**:

- **GitHub**: Fully supports public, private, and archived
- **GitLab**: Fully supports public, private, and archived
- **Bitbucket**: Supports public and private, but NOT archived (always `false`)

### 5.3 Core Tables

#### Users

Primary user accounts for Ampel authentication.

```sql
users (
  id UUID PRIMARY KEY,
  email VARCHAR NOT NULL UNIQUE,
  password_hash VARCHAR NOT NULL,
  display_name VARCHAR,
  avatar_url VARCHAR,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
```

#### Organizations

Multi-user organizations for team collaboration.

```sql
organizations (
  id UUID PRIMARY KEY,
  owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name VARCHAR NOT NULL,
  slug VARCHAR NOT NULL UNIQUE,
  description VARCHAR,
  logo_url VARCHAR,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
```

#### Teams

Sub-groups within organizations.

```sql
teams (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  name VARCHAR NOT NULL,
  description VARCHAR,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)

team_members (
  team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role VARCHAR NOT NULL,
  joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (team_id, user_id)
)
```

#### Provider Accounts

PAT-based provider account credentials.

```sql
provider_accounts (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider VARCHAR NOT NULL, -- 'github', 'gitlab', 'bitbucket'
  instance_url VARCHAR,      -- For self-hosted instances
  username VARCHAR NOT NULL,
  email VARCHAR,
  avatar_url VARCHAR,
  access_token_encrypted VARCHAR NOT NULL,  -- AES-256-GCM encrypted PAT
  scopes VARCHAR,
  is_default BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(user_id, provider, instance_url)
)
```

#### Repositories

Tracked repositories from providers.

```sql
repositories (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider VARCHAR NOT NULL,
  provider_id VARCHAR NOT NULL,
  owner VARCHAR NOT NULL,
  name VARCHAR NOT NULL,
  full_name VARCHAR NOT NULL,
  description VARCHAR,
  url VARCHAR NOT NULL,
  default_branch VARCHAR NOT NULL,
  is_private BOOLEAN NOT NULL DEFAULT false,
  is_archived BOOLEAN NOT NULL DEFAULT false,
  poll_interval_seconds INT NOT NULL DEFAULT 300,
  last_polled_at TIMESTAMPTZ,
  group_id UUID,  -- For organization/grouping
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
```

#### Pull Requests

Pull request data synced from providers.

```sql
pull_requests (
  id UUID PRIMARY KEY,
  repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  provider_id VARCHAR NOT NULL,
  number INT NOT NULL,
  title VARCHAR NOT NULL,
  description TEXT,
  url VARCHAR NOT NULL,
  state VARCHAR NOT NULL,
  source_branch VARCHAR NOT NULL,
  target_branch VARCHAR NOT NULL,
  author VARCHAR NOT NULL,
  author_avatar_url VARCHAR,
  is_draft BOOLEAN NOT NULL DEFAULT false,
  is_mergeable BOOLEAN,
  has_conflicts BOOLEAN NOT NULL DEFAULT false,
  ampel_status VARCHAR NOT NULL DEFAULT 'yellow', -- 'green', 'yellow', 'red'
  additions INT NOT NULL DEFAULT 0,
  deletions INT NOT NULL DEFAULT 0,
  changed_files INT NOT NULL DEFAULT 0,
  commits_count INT NOT NULL DEFAULT 0,
  comments_count INT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  merged_at TIMESTAMPTZ,
  closed_at TIMESTAMPTZ,
  UNIQUE(repository_id, number)
)
```

#### CI Checks

CI/CD status checks for PRs.

```sql
ci_checks (
  id UUID PRIMARY KEY,
  pull_request_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  name VARCHAR NOT NULL,
  status VARCHAR NOT NULL,      -- 'queued', 'in_progress', 'completed'
  conclusion VARCHAR,            -- 'success', 'failure', 'neutral', 'cancelled', 'timed_out'
  url VARCHAR,
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
```

#### Reviews

Code review status for PRs.

```sql
reviews (
  id UUID PRIMARY KEY,
  pull_request_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  provider_id VARCHAR NOT NULL,
  reviewer VARCHAR NOT NULL,
  reviewer_avatar_url VARCHAR,
  state VARCHAR NOT NULL,      -- 'approved', 'changes_requested', 'commented'
  body TEXT,
  submitted_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
```

### 5.3 Feature-Specific Tables

#### PR Filters

User-specific PR filter preferences.

```sql
pr_filter (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  show_draft BOOLEAN NOT NULL DEFAULT true,
  show_mergeable BOOLEAN NOT NULL DEFAULT true,
  show_conflicted BOOLEAN NOT NULL DEFAULT true,
  show_approved BOOLEAN NOT NULL DEFAULT true,
  show_changes_requested BOOLEAN NOT NULL DEFAULT true,
  show_pending_review BOOLEAN NOT NULL DEFAULT true,
  show_ci_passing BOOLEAN NOT NULL DEFAULT true,
  show_ci_failing BOOLEAN NOT NULL DEFAULT true,
  show_ci_pending BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(user_id)
)
```

#### User Settings

Behavioral settings for users.

```sql
user_settings (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  skip_review_requirement BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(user_id)
)
```

#### Merge Operations

Bulk merge operation tracking.

```sql
merge_operations (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  status VARCHAR NOT NULL,      -- 'pending', 'in_progress', 'completed', 'failed'
  total INT NOT NULL DEFAULT 0,
  success INT NOT NULL DEFAULT 0,
  failed INT NOT NULL DEFAULT 0,
  skipped INT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)

merge_operation_items (
  id UUID PRIMARY KEY,
  merge_operation_id UUID NOT NULL REFERENCES merge_operations(id) ON DELETE CASCADE,
  pull_request_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  status VARCHAR NOT NULL,      -- 'pending', 'success', 'failed', 'skipped'
  error_message TEXT,
  merged_sha VARCHAR,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
```

#### Health Scores

Repository and PR health metrics.

```sql
health_scores (
  id UUID PRIMARY KEY,
  repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  score INT NOT NULL,          -- 0-100
  pr_velocity FLOAT,
  avg_time_to_merge INT,       -- seconds
  open_pr_count INT NOT NULL DEFAULT 0,
  stale_pr_count INT NOT NULL DEFAULT 0,
  calculated_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
```

#### PR Metrics

Detailed PR performance metrics.

```sql
pr_metrics (
  id UUID PRIMARY KEY,
  pull_request_id UUID NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  time_to_first_review INT,    -- seconds
  time_to_approval INT,         -- seconds
  time_to_merge INT,            -- seconds
  review_cycles INT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(pull_request_id)
)
```

#### Auto-Merge Rules

Bot PR auto-merge configuration.

```sql
auto_merge_rule (
  id UUID PRIMARY KEY,
  repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  enabled BOOLEAN NOT NULL DEFAULT false,
  require_ci_pass BOOLEAN NOT NULL DEFAULT true,
  require_reviews BOOLEAN NOT NULL DEFAULT false,
  min_reviews INT NOT NULL DEFAULT 0,
  bot_patterns JSONB,           -- Array of regex patterns for bot PR detection
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(repository_id)
)
```

#### Notification Preferences

User notification settings (infrastructure ready, features pending).

```sql
notification_preferences (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  slack_enabled BOOLEAN NOT NULL DEFAULT false,
  slack_webhook_url VARCHAR,
  email_enabled BOOLEAN NOT NULL DEFAULT false,
  email_address VARCHAR,
  notify_on_ready BOOLEAN NOT NULL DEFAULT true,
  notify_on_blocked BOOLEAN NOT NULL DEFAULT true,
  notify_on_approved BOOLEAN NOT NULL DEFAULT false,
  digest_enabled BOOLEAN NOT NULL DEFAULT false,
  digest_frequency VARCHAR,     -- 'daily', 'weekly'
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(user_id)
)
```

### 5.4 Relationships

```
users
  ├── organizations (owner)
  ├── provider_accounts
  ├── repositories
  ├── pull_requests (via repositories)
  ├── merge_operations
  ├── user_settings
  ├── pr_filter
  └── notification_preferences

organizations
  ├── teams
  └── team_members (via teams)

repositories
  ├── pull_requests
  ├── health_scores
  └── auto_merge_rule

pull_requests
  ├── ci_checks
  ├── reviews
  ├── pr_metrics
  └── merge_operation_items
```

---

## 6. Backend Architecture

### 6.1 Crate Structure

#### `ampel-api` - HTTP API Layer

**Purpose**: HTTP server, routing, request handling, middleware

**Key Components**:

- `main.rs`: Server initialization, Axum app setup
- `routes/mod.rs`: API route definitions
- `handlers/`: HTTP request handlers for each resource
  - `auth.rs`: Login, register, logout, token refresh
  - `accounts.rs`: Provider account management
  - `repositories.rs`: Repository CRUD operations
  - `pull_requests.rs`: PR listing, details, merge
  - `bulk_merge.rs`: Batch merge operations
  - `dashboard.rs`: Dashboard summary and grid views
  - `teams.rs`: Team management
  - `analytics.rs`: Health scores and analytics
  - `notifications.rs`: Notification preferences
  - `user_settings.rs`: User behavior settings
  - `pr_filters.rs`: PR filter preferences
  - `bot_rules.rs`: Auto-merge rules
- `extractors/`: Custom Axum extractors
  - `auth.rs`: `AuthUser` extractor for JWT validation
  - `validated.rs`: Request validation
- `middleware/`: HTTP middleware
  - `rate_limit.rs`: Rate limiting (future)
- `state.rs`: Shared application state
- `config.rs`: Configuration management

**Dependencies**: `axum`, `tower`, `tower-http`

#### `ampel-core` - Business Logic Layer

**Purpose**: Domain models, business logic, services

**Key Components**:

- `models/`: Domain models
  - `user.rs`: User domain model
  - `repository.rs`: Repository domain model
  - `pull_request.rs`: PR domain model
  - `ampel_status.rs`: Traffic light status calculation logic
- `services/`: Business logic services
  - `auth_service.rs`: Authentication logic, password hashing
  - `repo_service.rs`: Repository business logic
  - `pr_service.rs`: PR status calculation, merge logic
  - `notification_service.rs`: Notification sending (infrastructure only)
- `errors.rs`: Custom error types

**Dependencies**: `thiserror`, `argon2`, `jsonwebtoken`

#### `ampel-db` - Data Access Layer

**Purpose**: Database entities, queries, migrations

**Key Components**:

- `entities/`: SeaORM entity models (18 tables)
  - `user.rs`, `organization.rs`, `team.rs`, `team_member.rs`
  - `provider_account.rs`, `repository.rs`, `pull_request.rs`
  - `ci_check.rs`, `review.rs`, `pr_metrics.rs`, `health_score.rs`
  - `merge_operation.rs`, `merge_operation_item.rs`
  - `auto_merge_rule.rs`, `pr_filter.rs`, `user_settings.rs`
  - `notification_preferences.rs`
- `queries/`: Query helper functions
  - `user_queries.rs`, `repo_queries.rs`, `pr_queries.rs`
  - `provider_account_queries.rs`, `merge_operation_queries.rs`
  - `ci_check_queries.rs`, `review_queries.rs`
  - `pr_filter_queries.rs`, `user_settings_queries.rs`
- `migrations/`: Database migration files
  - `m20250101_000001_initial.rs`: Core schema
  - `m20250102_000002_teams.rs`: Multitenancy support
  - `m20250103_000003_pr_filters.rs`: Filter preferences
  - `m20250104_000004_merge_notifications.rs`: Merge tracking
  - `m20250105_000005_skip_review_setting.rs`: User settings
  - `m20250120_000001_provider_accounts.rs`: PAT multi-account
- `encryption.rs`: AES-256-GCM token encryption
- `lib.rs`: Database connection management

**Dependencies**: `sea-orm`, `sea-orm-migration`, `aes-gcm`, `base64`

#### `ampel-providers` - Provider Abstraction

**Purpose**: Git provider integrations (GitHub, GitLab, Bitbucket)

**Key Components**:

- `traits.rs`: `GitProvider` trait definition
- `github.rs`: GitHub API implementation
- `gitlab.rs`: GitLab API implementation
- `bitbucket.rs`: Bitbucket API implementation
- `factory.rs`: Provider factory for instantiation
- `mock.rs`: Mock provider for testing
- `error.rs`: Provider-specific errors

**Provider Trait Methods**:

```rust
async fn validate_credentials(&self, ...) -> Result<TokenValidation>;
async fn get_user(&self, ...) -> Result<ProviderUser>;
async fn list_repositories(&self, ...) -> Result<Vec<DiscoveredRepository>>;
async fn get_repository(&self, ...) -> Result<DiscoveredRepository>;
async fn list_pull_requests(&self, ...) -> Result<Vec<ProviderPullRequest>>;
async fn get_pull_request(&self, ...) -> Result<ProviderPullRequest>;
async fn get_ci_checks(&self, ...) -> Result<Vec<ProviderCICheck>>;
async fn get_reviews(&self, ...) -> Result<Vec<ProviderReview>>;
async fn merge_pull_request(&self, ...) -> Result<MergeResult>;
async fn get_rate_limit(&self, ...) -> Result<RateLimitInfo>;
```

**Dependencies**: `async-trait`, `reqwest`, `serde_json`

#### `ampel-worker` - Background Jobs

**Purpose**: Asynchronous job processing with Apalis

**Key Components**:

- `main.rs`: Worker initialization, job registration
- `jobs/`:
  - `poll_repository.rs`: Sync PR data from providers
  - `cleanup.rs`: Database cleanup, stale data removal
  - `health_score.rs`: Calculate repository health scores
  - `metrics_collection.rs`: Collect PR performance metrics

**Job Types**:

- **RepositoryPollJob**: Fetch latest PRs, CI checks, reviews from provider
- **CleanupJob**: Remove old closed PRs, orphaned data
- **HealthScoreJob**: Calculate health scores for repositories
- **MetricsCollectionJob**: Calculate PR metrics (time to merge, review cycles)

**Dependencies**: `apalis`, `apalis-sql`, `tokio`

### 6.2 Request Flow

```
HTTP Request
    ↓
Axum Router (routes/mod.rs)
    ↓
Middleware (auth, rate limit, CORS)
    ↓
Handler (handlers/*.rs)
    ↓
Service (ampel-core/services/*.rs)
    ↓
Database Queries (ampel-db/queries/*.rs)
    |
    ├── SeaORM Entities (ampel-db/entities/*.rs)
    |       ↓
    |   PostgreSQL Database
    |
    └── Provider API (ampel-providers/*.rs)
            ↓
        GitHub/GitLab/Bitbucket API
    ↓
Response (JSON)
```

### 6.3 Error Handling

Custom error types with `thiserror`:

```rust
// ampel-core/src/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Provider error: {0}")]
    Provider(#[from] ampel_providers::error::ProviderError),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Resource not found: {0}")]
    NotFound(String),
}
```

Handlers convert errors to HTTP responses:

```rust
impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::NotFound(msg) => ApiError::NotFound(msg),
            CoreError::AuthenticationFailed(msg) => ApiError::Unauthorized(msg),
            // ...
        }
    }
}
```

---

## 7. Frontend Architecture

### 7.1 Directory Structure

```
frontend/src/
├── api/                # API client functions
│   ├── client.ts       # Axios instance, interceptors
│   ├── auth.ts         # Auth API calls
│   ├── accounts.ts     # Provider account API
│   ├── repositories.ts # Repository API
│   ├── pullRequests.ts # PR API
│   ├── dashboard.ts    # Dashboard API
│   ├── settings.ts     # Settings API
│   └── merge.ts        # Bulk merge API
├── components/
│   ├── ui/             # shadcn/ui components
│   ├── layout/         # Header, Sidebar, Layout wrappers
│   ├── dashboard/      # PRCard, RepoCard, GridView, ListView
│   ├── merge/          # Merge operation components
│   ├── settings/       # Settings page components
│   └── icons/          # Custom icons
├── hooks/              # React hooks
│   ├── useAuth.tsx     # Auth context and hook
│   ├── useTheme.tsx    # Theme management
│   └── usePullRequests.tsx  # PR data hook
├── pages/              # Route pages
│   ├── Login.tsx
│   ├── Register.tsx
│   ├── Dashboard.tsx
│   ├── Repositories.tsx
│   ├── Merge.tsx
│   ├── Analytics.tsx
│   └── Settings.tsx
├── types/              # TypeScript type definitions
│   └── index.ts
├── lib/                # Utilities
│   └── utils.ts
├── App.tsx             # Root component, routing
└── main.tsx            # React entry point
```

### 7.2 State Management

**TanStack Query** for server state:

```typescript
// Example: Dashboard summary query
const { data: summary, isLoading } = useQuery({
  queryKey: ['dashboard', 'summary'],
  queryFn: () => dashboardApi.getSummary(),
  staleTime: 60000, // Cache for 1 minute
});
```

**React Context** for global client state:

- `AuthContext`: User authentication state, login/logout functions

### 7.3 API Client

**Axios instance with interceptors**:

```typescript
// frontend/src/api/client.ts
const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_URL,
  headers: { 'Content-Type': 'application/json' },
  withCredentials: true, // Send cookies (refresh token)
});

// Request interceptor: Add JWT access token
apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Response interceptor: Handle 401, refresh token
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      // Attempt token refresh
      // If refresh fails, redirect to login
    }
    return Promise.reject(error);
  }
);
```

### 7.4 Routing

**React Router v6**:

```typescript
<Routes>
  <Route path="/login" element={<Login />} />
  <Route path="/register" element={<Register />} />

  <Route element={<ProtectedRoute />}>
    <Route path="/" element={<Dashboard />} />
    <Route path="/repositories" element={<Repositories />} />
    <Route path="/merge" element={<Merge />} />
    <Route path="/analytics" element={<Analytics />} />
    <Route path="/settings/*" element={<Settings />} />
  </Route>
</Routes>
```

### 7.5 Component Patterns

**Compound Components** (shadcn/ui):

```typescript
<Card>
  <CardHeader>
    <CardTitle>Repository Name</CardTitle>
  </CardHeader>
  <CardContent>
    {/* Content */}
  </CardContent>
</Card>
```

**Controlled Components** (forms):

```typescript
const form = useForm<FormData>({
  resolver: zodResolver(schema),
});

<FormField
  control={form.control}
  name="email"
  render={({ field }) => <Input {...field} />}
/>
```

### 7.6 Type Safety

**Shared types** between frontend and backend:

```typescript
// frontend/src/types/index.ts
export interface PullRequestWithDetails {
  id: string;
  repositoryId: string;
  number: number;
  title: string;
  status: 'green' | 'yellow' | 'red';
  isDraft: boolean;
  hasConflicts: boolean;
  ciChecks?: CICheck[];
  reviews?: Review[];
  // ... 20+ fields
}
```

---

## 8. Provider Abstraction

### 8.1 GitProvider Trait

All providers implement a unified trait:

```rust
#[async_trait]
pub trait GitProvider: Send + Sync {
    fn provider_type(&self) -> Provider;
    fn instance_url(&self) -> Option<&str>;

    async fn validate_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> ProviderResult<TokenValidation>;

    async fn list_pull_requests(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> ProviderResult<Vec<ProviderPullRequest>>;

    async fn merge_pull_request(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
        merge_request: &MergeRequest,
    ) -> ProviderResult<MergeResult>;

    // ... 10 total methods
}
```

### 8.2 Provider Implementations

#### GitHub

- **API Base URL**: `https://api.github.com`
- **Authentication**: `Authorization: token <PAT>`
- **Endpoints**:
  - `GET /user` - User info
  - `GET /user/repos` - Repository list
  - `GET /repos/{owner}/{repo}/pulls` - PR list
  - `GET /repos/{owner}/{repo}/pulls/{number}` - PR details
  - `GET /repos/{owner}/{repo}/commits/{ref}/status` - CI checks
  - `GET /repos/{owner}/{repo}/pulls/{number}/reviews` - Reviews
  - `PUT /repos/{owner}/{repo}/pulls/{number}/merge` - Merge PR
- **Rate Limit**: 5000 requests/hour (authenticated)

#### GitLab

- **API Base URL**: `https://gitlab.com/api/v4` (configurable for self-hosted)
- **Authentication**: `Authorization: Bearer <PAT>` or `PRIVATE-TOKEN: <PAT>`
- **Endpoints**:
  - `GET /user` - User info
  - `GET /projects` - Project list
  - `GET /projects/{id}/merge_requests` - MR list
  - `GET /projects/{id}/merge_requests/{iid}` - MR details
  - `GET /projects/{id}/merge_requests/{iid}/pipelines` - CI pipelines
  - `GET /projects/{id}/merge_requests/{iid}/approvals` - Approvals
  - `PUT /projects/{id}/merge_requests/{iid}/merge` - Merge MR
- **Rate Limit**: 300 requests/minute (authenticated)

#### Bitbucket

- **API Base URL**: `https://api.bitbucket.org/2.0`
- **Authentication**: `Authorization: Basic <base64(username:PAT)>`
- **Endpoints**:
  - `GET /user` - User info
  - `GET /repositories/{workspace}` - Repository list
  - `GET /repositories/{workspace}/{repo_slug}/pullrequests` - PR list
  - `GET /repositories/{workspace}/{repo_slug}/pullrequests/{id}` - PR details
  - `GET /repositories/{workspace}/{repo_slug}/commit/{commit}/statuses` - CI statuses
  - `POST /repositories/{workspace}/{repo_slug}/pullrequests/{id}/merge` - Merge PR
- **Rate Limit**: 1000 requests/hour (authenticated)

### 8.3 Provider Factory

```rust
pub fn create_provider(
    provider: Provider,
    instance_url: Option<&str>,
) -> Box<dyn GitProvider> {
    match provider {
        Provider::GitHub => Box::new(GitHubProvider::new(instance_url)),
        Provider::GitLab => Box::new(GitLabProvider::new(instance_url)),
        Provider::Bitbucket => Box::new(BitbucketProvider::new()),
    }
}
```

### 8.4 Provider Data Mapping

Providers return unified data structures:

```rust
pub struct ProviderPullRequest {
    pub provider_id: String,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub is_draft: bool,
    pub has_conflicts: bool,
    pub is_mergeable: Option<bool>,
    // ... 15 fields
}

// Converted to internal model
impl From<ProviderPullRequest> for pull_request::ActiveModel {
    fn from(pr: ProviderPullRequest) -> Self {
        // Mapping logic
    }
}
```

---

## 9. Background Job System

### 9.1 Apalis Integration

**Architecture**:

- PostgreSQL-backed job queue
- Multiple worker instances supported
- Job retries with exponential backoff
- Job status tracking

**Configuration**:

```rust
// ampel-worker/src/main.rs
let storage = PostgresStorage::new(pool.clone());

let worker = WorkerBuilder::new("ampel-worker")
    .register(poll_repository_job)
    .register(cleanup_job)
    .register(health_score_job)
    .register(metrics_collection_job)
    .build(storage);

worker.run().await?;
```

### 9.2 Job Types

#### RepositoryPollJob

**Purpose**: Sync PR data from provider API

**Trigger**: On-demand (user clicks refresh) or scheduled (future)

**Process**:

1. Fetch repository from database
2. Decrypt provider PAT token
3. Call provider API to list PRs
4. For each PR:
   - Fetch CI checks
   - Fetch reviews
   - Calculate ampel_status (green/yellow/red)
   - Upsert to database
5. Update `last_polled_at` timestamp

**Implementation**: `crates/ampel-worker/src/jobs/poll_repository.rs`

#### CleanupJob

**Purpose**: Remove stale data, clean up closed PRs

**Trigger**: Daily cron (future)

**Process**:

1. Delete PRs closed > 90 days ago
2. Remove orphaned CI checks and reviews
3. Clean up failed merge operations > 30 days old

**Implementation**: `crates/ampel-worker/src/jobs/cleanup.rs`

#### HealthScoreJob

**Purpose**: Calculate repository health metrics

**Trigger**: Daily or on-demand

**Process**:

1. For each repository:
   - Count open PRs
   - Count stale PRs (> 7 days old)
   - Calculate PR velocity (PRs merged per week)
   - Calculate average time to merge
2. Compute health score (0-100)
3. Store in `health_scores` table

**Implementation**: `crates/ampel-worker/src/jobs/health_score.rs`

#### MetricsCollectionJob

**Purpose**: Calculate PR-level performance metrics

**Trigger**: When PR is closed/merged

**Process**:

1. Calculate time to first review
2. Calculate time to approval
3. Calculate time to merge
4. Count review cycles
5. Store in `pr_metrics` table

**Implementation**: `crates/ampel-worker/src/jobs/metrics_collection.rs`

### 9.3 Job Scheduling

**Current State**: Jobs triggered manually or on API events

**Future**: Cron-based scheduling with Apalis:

```rust
let scheduler = Scheduler::new()
    .schedule(Schedule::every(5.minutes()), poll_all_repositories)
    .schedule(Schedule::daily(), cleanup_job)
    .schedule(Schedule::daily(), health_score_job);
```

---

## 10. API Endpoints

### 10.1 Authentication Endpoints

| Method | Endpoint             | Description              | Auth Required       |
| ------ | -------------------- | ------------------------ | ------------------- |
| POST   | `/api/auth/register` | Create new user account  | No                  |
| POST   | `/api/auth/login`    | Login and receive tokens | No                  |
| POST   | `/api/auth/refresh`  | Refresh access token     | Yes (refresh token) |
| POST   | `/api/auth/logout`   | Invalidate refresh token | Yes                 |
| GET    | `/api/auth/me`       | Get current user info    | Yes                 |
| PUT    | `/api/auth/me`       | Update current user info | Yes                 |

### 10.2 Provider Account Endpoints

| Method | Endpoint                        | Description                     | Auth Required |
| ------ | ------------------------------- | ------------------------------- | ------------- |
| GET    | `/api/accounts`                 | List user's provider accounts   | Yes           |
| POST   | `/api/accounts`                 | Add new provider account        | Yes           |
| GET    | `/api/accounts/:id`             | Get account details             | Yes           |
| PATCH  | `/api/accounts/:id`             | Update account (token rotation) | Yes           |
| DELETE | `/api/accounts/:id`             | Remove account                  | Yes           |
| POST   | `/api/accounts/:id/validate`    | Validate PAT token              | Yes           |
| POST   | `/api/accounts/:id/set-default` | Set default account             | Yes           |

### 10.3 Repository Endpoints

| Method | Endpoint                     | Description                | Auth Required |
| ------ | ---------------------------- | -------------------------- | ------------- |
| GET    | `/api/repositories`          | List tracked repositories  | Yes           |
| POST   | `/api/repositories`          | Add repository to track    | Yes           |
| GET    | `/api/repositories/discover` | Discover available repos   | Yes           |
| GET    | `/api/repositories/:id`      | Get repository details     | Yes           |
| PUT    | `/api/repositories/:id`      | Update repository settings | Yes           |
| DELETE | `/api/repositories/:id`      | Stop tracking repository   | Yes           |

### 10.4 Pull Request Endpoints

| Method | Endpoint                                                  | Description                 | Auth Required |
| ------ | --------------------------------------------------------- | --------------------------- | ------------- |
| GET    | `/api/pull-requests`                                      | List all PRs (with filters) | Yes           |
| GET    | `/api/repositories/:repo_id/pull-requests`                | List repo PRs               | Yes           |
| GET    | `/api/repositories/:repo_id/pull-requests/:pr_id`         | Get PR details              | Yes           |
| POST   | `/api/repositories/:repo_id/pull-requests/:pr_id/merge`   | Merge single PR             | Yes           |
| POST   | `/api/repositories/:repo_id/pull-requests/:pr_id/refresh` | Refresh PR data             | Yes           |

### 10.5 Dashboard Endpoints

| Method | Endpoint                 | Description                                          | Auth Required |
| ------ | ------------------------ | ---------------------------------------------------- | ------------- |
| GET    | `/api/dashboard/summary` | Get traffic light summary with visibility breakdowns | Yes           |
| GET    | `/api/dashboard/grid`    | Get repository grid view                             | Yes           |

**Dashboard Summary Response** includes:

- Total repositories and open PRs
- Status counts (green/yellow/red)
- Provider counts (GitHub/GitLab/Bitbucket)
- **Visibility breakdowns**: Repository, Open PRs, Ready to Merge, Needs Attention counts broken down by public/private/archived

See [Dashboard Visibility Breakdown API Documentation](/docs/api/DASHBOARD-VISIBILITY-BREAKDOWN.md) for complete details.

### 10.6 Bulk Merge Endpoints

| Method | Endpoint                    | Description                 | Auth Required |
| ------ | --------------------------- | --------------------------- | ------------- |
| POST   | `/api/merge/bulk`           | Create bulk merge operation | Yes           |
| GET    | `/api/merge/operations`     | List merge operations       | Yes           |
| GET    | `/api/merge/operations/:id` | Get operation details       | Yes           |

### 10.7 Team Endpoints

| Method | Endpoint                               | Description       | Auth Required |
| ------ | -------------------------------------- | ----------------- | ------------- |
| GET    | `/api/teams`                           | List user's teams | Yes           |
| POST   | `/api/teams`                           | Create new team   | Yes           |
| GET    | `/api/teams/:team_id`                  | Get team details  | Yes           |
| POST   | `/api/teams/:team_id/members`          | Add team member   | Yes           |
| DELETE | `/api/teams/:team_id/members/:user_id` | Remove member     | Yes           |

### 10.8 Settings Endpoints

| Method | Endpoint                         | Description               | Auth Required |
| ------ | -------------------------------- | ------------------------- | ------------- |
| GET    | `/api/settings/behavior`         | Get user settings         | Yes           |
| PUT    | `/api/settings/behavior`         | Update user settings      | Yes           |
| GET    | `/api/pr-filters`                | Get PR filter preferences | Yes           |
| PUT    | `/api/pr-filters`                | Update PR filters         | Yes           |
| POST   | `/api/pr-filters/reset`          | Reset to defaults         | Yes           |
| GET    | `/api/notifications/preferences` | Get notification prefs    | Yes           |
| PUT    | `/api/notifications/preferences` | Update notification prefs | Yes           |

### 10.9 Analytics Endpoints

| Method | Endpoint                            | Description            | Auth Required |
| ------ | ----------------------------------- | ---------------------- | ------------- |
| GET    | `/api/analytics/summary`            | Get analytics overview | Yes           |
| GET    | `/api/analytics/health`             | Get overall health     | Yes           |
| GET    | `/api/repositories/:repo_id/health` | Get repository health  | Yes           |

### 10.10 Bot/Auto-Merge Endpoints

| Method | Endpoint                                | Description         | Auth Required |
| ------ | --------------------------------------- | ------------------- | ------------- |
| GET    | `/api/repositories/:repo_id/auto-merge` | Get auto-merge rule | Yes           |
| PUT    | `/api/repositories/:repo_id/auto-merge` | Create/update rule  | Yes           |
| DELETE | `/api/repositories/:repo_id/auto-merge` | Delete rule         | Yes           |

---

## 11. Deployment

### 11.1 Development Setup

**Prerequisites**:

- Rust 1.92.0
- PostgreSQL 16+
- Redis 7+
- Node.js 20+ / pnpm 10.24.0

**Environment Configuration**:

```bash
cp .env.example .env
# Edit .env with database credentials, JWT secret, encryption key
```

**Start Services**:

```bash
# Terminal 1: API server
make dev-api

# Terminal 2: Background worker
make dev-worker

# Terminal 3: Frontend dev server
make dev-frontend
```

### 11.2 Production Build

**Backend**:

```bash
make build-release
# Outputs: target/release/ampel-api, target/release/ampel-worker
```

**Frontend**:

```bash
cd frontend && pnpm build
# Outputs: frontend/dist/
```

### 11.3 Docker Deployment (Future)

**Planned Docker Compose setup**:

```yaml
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: ampel
      POSTGRES_USER: ampel
      POSTGRES_PASSWORD: ampel
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

  api:
    build: .
    dockerfile: Dockerfile.api
    depends_on:
      - postgres
      - redis
    environment:
      DATABASE_URL: postgres://ampel:ampel@postgres:5432/ampel
      REDIS_URL: redis://redis:6379
    ports:
      - '8080:8080'

  worker:
    build: .
    dockerfile: Dockerfile.worker
    depends_on:
      - postgres
      - redis
    environment:
      DATABASE_URL: postgres://ampel:ampel@postgres:5432/ampel

  frontend:
    build: frontend
    ports:
      - '3000:80'
```

### 11.4 Fly.io Deployment (Future)

**Planned deployment targets**:

- **API**: Fly.io app with Postgres database
- **Worker**: Fly.io background worker machine
- **Frontend**: Fly.io static site or CDN

**Configuration**:

```toml
# fly.toml
[build]
  builder = "rust"

[env]
  RUST_LOG = "info,ampel=debug"

[[services]]
  http_checks = []
  internal_port = 8080
  protocol = "tcp"

  [[services.ports]]
    handlers = ["http"]
    port = 80

  [[services.ports]]
    handlers = ["tls", "http"]
    port = 443
```

### 11.5 Database Migrations

**Run migrations**:

```bash
# Development
make migrate

# Production
cargo run --bin ampel-api -- migrate
```

**Create new migration**:

```bash
cd crates/ampel-db
sea-orm-cli migrate generate <migration_name>
```

---

## 12. Monitoring & Observability

### 12.1 Current State

**Logging**:

- Rust: `tracing` crate with structured logging
- Log levels: `RUST_LOG` environment variable
- Output: JSON-formatted logs (future)

**Health Check**:

- Endpoint: `GET /health` returns `200 OK`

### 12.2 Planned Features

**Metrics** (Prometheus):

- API request latency
- Provider API call rates
- Job processing times
- Database query performance

**Distributed Tracing** (OpenTelemetry):

- Request tracing across services
- Provider API call tracing
- Job execution tracing

**Error Tracking** (Sentry):

- Automatic error reporting
- User context and breadcrumbs
- Performance monitoring

**Dashboards** (Grafana):

- API health dashboard
- Provider API status
- Job queue metrics
- Database performance

### 12.3 Alerting (Future)

**Planned Alerts**:

- API downtime
- High error rates
- Provider API rate limit approaching
- Job failures
- Database connection issues

---

## Appendix A: Traffic Light Status Calculation

The `ampel_status` field indicates PR readiness:

```rust
pub fn calculate_ampel_status(
    pr: &PullRequest,
    ci_checks: &[CICheck],
    reviews: &[Review],
) -> AmpelStatus {
    // Red conditions (blockers)
    if pr.has_conflicts {
        return AmpelStatus::Red;
    }

    if ci_checks.iter().any(|c| {
        c.status == "completed" &&
        (c.conclusion == Some("failure") || c.conclusion == Some("timed_out"))
    }) {
        return AmpelStatus::Red;
    }

    if reviews.iter().any(|r| r.state == "changes_requested") {
        return AmpelStatus::Red;
    }

    // Green conditions (ready to merge)
    let ci_passing = ci_checks.iter().all(|c| {
        c.status != "completed" || c.conclusion == Some("success")
    });

    let has_approval = reviews.iter().any(|r| r.state == "approved");

    if ci_passing && has_approval && !pr.is_draft {
        return AmpelStatus::Green;
    }

    // Yellow (in progress)
    AmpelStatus::Yellow
}
```

**Summary**:

- **Red**: Conflicts, CI failures, changes requested
- **Green**: CI passing, approved, not draft, no conflicts
- **Yellow**: Everything else (pending review, CI running)

---

## Appendix B: Configuration Reference

### Environment Variables

| Variable                    | Type   | Default | Description                               |
| --------------------------- | ------ | ------- | ----------------------------------------- |
| `DATABASE_URL`              | String | -       | PostgreSQL connection URL                 |
| `REDIS_URL`                 | String | -       | Redis connection URL                      |
| `JWT_SECRET`                | String | -       | Secret key for JWT signing (min 32 chars) |
| `JWT_ACCESS_EXPIRY_MINUTES` | Number | 15      | Access token lifetime                     |
| `JWT_REFRESH_EXPIRY_DAYS`   | Number | 7       | Refresh token lifetime                    |
| `ENCRYPTION_KEY`            | String | -       | Base64-encoded 32-byte AES key            |
| `HOST`                      | String | 0.0.0.0 | API server bind address                   |
| `PORT`                      | Number | 8080    | API server port                           |
| `RUST_LOG`                  | String | info    | Logging level                             |
| `CORS_ORIGINS`              | String | -       | Comma-separated allowed origins           |
| `VITE_API_URL`              | String | -       | Frontend API base URL                     |

---

## Appendix C: Glossary

- **PAT**: Personal Access Token - Provider-specific token for API access
- **JWT**: JSON Web Token - Stateless authentication token
- **Ampel**: German for "traffic light" - the PR status system
- **SeaORM**: Object-Relational Mapping library for Rust
- **Axum**: Web framework built on Tokio
- **Apalis**: Background job processing library
- **TanStack Query**: React data synchronization library (formerly React Query)
- **shadcn/ui**: Collection of re-usable React components

---

**End of Architecture Documentation**

_This document reflects the actual implementation state as of 2025-12-22. No OAuth features, only implemented functionality documented._
