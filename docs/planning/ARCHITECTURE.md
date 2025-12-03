# Ampel - Technical Architecture

## System Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Load Balancer                            │
└─────────────────────────────┬───────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│  React SPA    │    │  Axum API     │    │  Axum API     │
│  (Frontend)   │───▶│  Server 1     │    │  Server 2     │
└───────────────┘    └───────┬───────┘    └───────┬───────┘
                             │                     │
                             └──────────┬──────────┘
                                        │
        ┌───────────────────────────────┼───────────────────────────┐
        │                               │                           │
        ▼                               ▼                           ▼
┌───────────────┐              ┌───────────────┐           ┌───────────────┐
│  PostgreSQL   │              │    Redis      │           │  Background   │
│  (Primary)    │              │   (Cache)     │           │   Workers     │
└───────┬───────┘              └───────────────┘           │   (apalis)    │
        │                                                  └───────────────┘
        ▼
┌───────────────┐
│  PostgreSQL   │
│  (Replica)    │
└───────────────┘
```

---

## Technology Stack

### Backend (Rust)

| Component       | Library           | Version  | Purpose                    |
| --------------- | ----------------- | -------- | -------------------------- |
| Web Framework   | axum              | 0.7.x    | HTTP routing, middleware   |
| Async Runtime   | tokio             | 1.x      | Async I/O, task scheduling |
| Database ORM    | sea-orm           | 1.x      | Async database access      |
| Migrations      | sea-orm-migration | 1.x      | Schema migrations          |
| Background Jobs | apalis            | 0.5.x    | Job queue, cron scheduling |
| GitHub API      | octocrab          | 0.47.x   | GitHub REST/GraphQL client |
| GitLab API      | gitlab            | 0.1900.x | GitLab API v4 client       |
| Bitbucket API   | Custom            | -        | reqwest-based client       |
| Auth            | axum-extra        | 0.9.x    | JWT, cookies               |
| Validation      | validator         | 0.18.x   | Input validation           |
| Serialization   | serde             | 1.x      | JSON serialization         |
| Config          | config            | 0.14.x   | Configuration management   |
| Logging         | tracing           | 0.1.x    | Structured logging         |
| Error Handling  | thiserror         | 1.x      | Error types                |

### Frontend (React + TypeScript)

| Component     | Library         | Version | Purpose                   |
| ------------- | --------------- | ------- | ------------------------- |
| Framework     | React           | 18.x    | UI framework              |
| Build Tool    | Vite            | 5.x     | Fast dev server, bundling |
| Routing       | React Router    | 6.x     | Client-side routing       |
| State         | TanStack Query  | 5.x     | Server state management   |
| UI Components | shadcn/ui       | latest  | Accessible components     |
| Styling       | Tailwind CSS    | 3.x     | Utility-first CSS         |
| Forms         | React Hook Form | 7.x     | Form handling             |
| Icons         | Lucide React    | latest  | Icon library              |
| HTTP Client   | axios           | 1.x     | API requests              |

### Infrastructure

| Component     | Technology              | Purpose                       |
| ------------- | ----------------------- | ----------------------------- |
| Database      | PostgreSQL 16           | Primary data store            |
| Cache         | Redis 7                 | Session cache, rate limiting  |
| Container     | Docker                  | Containerization              |
| Orchestration | Docker Compose / Fly.io | Deployment                    |
| CI/CD         | GitHub Actions          | Automated testing, deployment |

---

## Data Model

### Entity Relationship Diagram

```text
┌─────────────────┐       ┌─────────────────┐
│  organizations  │       │     users       │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ name            │       │ email           │
│ slug            │       │ password_hash   │
│ created_at      │       │ created_at      │
└────────┬────────┘       └────────┬────────┘
         │                         │
         │    ┌────────────────────┘
         │    │
         ▼    ▼
┌─────────────────────┐
│ organization_members│
├─────────────────────┤
│ org_id (FK)         │
│ user_id (FK)        │
│ role                │
└─────────────────────┘

┌─────────────────┐       ┌─────────────────┐
│     teams       │       │  team_members   │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ team_id (FK)    │
│ org_id (FK)     │◀──────│ user_id (FK)    │
│ name            │       │ role            │
│ created_at      │       └─────────────────┘
└─────────────────┘

┌─────────────────┐       ┌─────────────────┐
│  git_providers  │       │  repositories   │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ user_id (FK)    │       │ provider_id (FK)│
│ provider_type   │◀──────│ external_id     │
│ access_token    │       │ name            │
│ refresh_token   │       │ full_name       │
│ token_expires   │       │ default_branch  │
│ instance_url    │       │ is_active       │
└─────────────────┘       │ last_polled_at  │
                          └────────┬────────┘
                                   │
                                   ▼
┌─────────────────┐       ┌─────────────────┐
│  pull_requests  │       │   ci_statuses   │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ repo_id (FK)    │       │ pr_id (FK)      │
│ external_id     │◀──────│ context         │
│ number          │       │ state           │
│ title           │       │ target_url      │
│ author          │       │ description     │
│ state           │       │ created_at      │
│ draft           │       └─────────────────┘
│ mergeable       │
│ review_status   │
│ ampel_status    │  ← (green/yellow/red)
│ created_at      │
│ updated_at      │
└─────────────────┘

┌─────────────────┐       ┌─────────────────┐
│ poll_schedules  │       │  poll_results   │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ id (PK)         │
│ user_id (FK)    │       │ schedule_id (FK)│
│ cron_expression │       │ started_at      │
│ is_active       │       │ completed_at    │
│ last_run_at     │       │ status          │
│ next_run_at     │       │ repos_polled    │
└─────────────────┘       │ prs_found       │
                          │ error_message   │
                          └─────────────────┘
```

### Database Schema (Key Tables)

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Organizations table
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Organization members (many-to-many)
CREATE TABLE organization_members (
    org_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    PRIMARY KEY (org_id, user_id)
);

-- Git providers (OAuth connections)
CREATE TABLE git_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    provider_type VARCHAR(50) NOT NULL, -- github, gitlab, bitbucket
    access_token_encrypted BYTEA NOT NULL,
    refresh_token_encrypted BYTEA,
    token_expires_at TIMESTAMP WITH TIME ZONE,
    instance_url VARCHAR(255), -- for self-hosted
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Repositories being monitored
CREATE TABLE repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID REFERENCES git_providers(id) ON DELETE CASCADE,
    external_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    default_branch VARCHAR(255) DEFAULT 'main',
    is_active BOOLEAN DEFAULT true,
    last_polled_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Pull requests
CREATE TABLE pull_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID REFERENCES repositories(id) ON DELETE CASCADE,
    external_id VARCHAR(255) NOT NULL,
    number INTEGER NOT NULL,
    title VARCHAR(500) NOT NULL,
    author VARCHAR(255) NOT NULL,
    state VARCHAR(50) NOT NULL, -- open, closed, merged
    draft BOOLEAN DEFAULT false,
    mergeable BOOLEAN,
    review_status VARCHAR(50), -- approved, changes_requested, pending
    ampel_status VARCHAR(20) NOT NULL, -- green, yellow, red
    html_url VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE,
    UNIQUE (repo_id, number)
);

-- CI status checks
CREATE TABLE ci_statuses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pr_id UUID REFERENCES pull_requests(id) ON DELETE CASCADE,
    context VARCHAR(255) NOT NULL,
    state VARCHAR(50) NOT NULL, -- pending, success, failure, error
    target_url VARCHAR(500),
    description VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Performance indexes
CREATE INDEX idx_pull_requests_repo ON pull_requests(repo_id);
CREATE INDEX idx_pull_requests_status ON pull_requests(ampel_status);
CREATE INDEX idx_pull_requests_state ON pull_requests(state);
CREATE INDEX idx_ci_statuses_pr ON ci_statuses(pr_id);
CREATE INDEX idx_repositories_provider ON repositories(provider_id);
CREATE INDEX idx_git_providers_user ON git_providers(user_id);
```

---

## API Design

### Authentication Endpoints

```text
POST /api/auth/register     - Register new user
POST /api/auth/login        - Login, returns JWT
POST /api/auth/refresh      - Refresh JWT token
POST /api/auth/logout       - Invalidate token
GET  /api/auth/me           - Get current user
```

### OAuth Provider Endpoints

```text
GET  /api/oauth/github/authorize    - Start GitHub OAuth
GET  /api/oauth/github/callback     - GitHub OAuth callback
GET  /api/oauth/gitlab/authorize    - Start GitLab OAuth
GET  /api/oauth/gitlab/callback     - GitLab OAuth callback
GET  /api/oauth/bitbucket/authorize - Start Bitbucket OAuth
GET  /api/oauth/bitbucket/callback  - Bitbucket OAuth callback
DELETE /api/oauth/:provider         - Disconnect provider
GET  /api/oauth/connections         - List connected providers
```

### Repository Endpoints

```text
GET    /api/repositories           - List monitored repos
POST   /api/repositories           - Add repository to monitor
DELETE /api/repositories/:id       - Remove repository
GET    /api/repositories/:id       - Get repository details
POST   /api/repositories/discover  - Discover accessible repos
PATCH  /api/repositories/:id       - Update repository settings
```

### Pull Request Endpoints

```text
GET  /api/pull-requests            - List PRs with filters
GET  /api/pull-requests/:id        - Get PR details
POST /api/pull-requests/:id/merge  - Merge PR (Phase 2)
POST /api/pull-requests/refresh    - Manual refresh all
POST /api/pull-requests/:id/refresh - Refresh single PR
```

### Dashboard Endpoints

```text
GET /api/dashboard/summary         - Aggregate status counts
GET /api/dashboard/grid            - Grid view data
GET /api/dashboard/list            - List view data (paginated)
```

### Organization Endpoints (Phase 2)

```text
GET    /api/organizations          - List user's orgs
POST   /api/organizations          - Create organization
GET    /api/organizations/:id      - Get org details
PUT    /api/organizations/:id      - Update organization
DELETE /api/organizations/:id      - Delete organization
POST   /api/organizations/:id/members - Add member
DELETE /api/organizations/:id/members/:user_id - Remove member
```

### Team Endpoints (Phase 2)

```text
GET    /api/organizations/:org_id/teams      - List teams
POST   /api/organizations/:org_id/teams      - Create team
GET    /api/teams/:id                        - Get team
PUT    /api/teams/:id                        - Update team
DELETE /api/teams/:id                        - Delete team
POST   /api/teams/:id/members                - Add member
DELETE /api/teams/:id/members/:user_id       - Remove member
```

### Request/Response Examples

#### List Pull Requests

```http
GET /api/pull-requests?status=green&provider=github&page=1&limit=20
Authorization: Bearer <jwt_token>
```

```json
{
  "data": [
    {
      "id": "uuid",
      "repository": {
        "id": "uuid",
        "full_name": "org/repo",
        "provider": "github"
      },
      "number": 123,
      "title": "Fix login bug",
      "author": "alice",
      "ampel_status": "green",
      "state": "open",
      "draft": false,
      "mergeable": true,
      "review_status": "approved",
      "ci_checks": [
        { "context": "build", "state": "success" },
        { "context": "test", "state": "success" }
      ],
      "html_url": "https://github.com/org/repo/pull/123",
      "created_at": "2025-01-15T10:30:00Z",
      "updated_at": "2025-01-15T12:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 45,
    "total_pages": 3
  }
}
```

---

## Provider Abstraction

### Trait Definition

```rust
use async_trait::async_trait;

#[async_trait]
pub trait GitProvider: Send + Sync {
    /// Authenticate and validate credentials
    async fn authenticate(&self) -> Result<(), ProviderError>;

    /// List accessible repositories
    async fn list_repositories(&self) -> Result<Vec<Repository>, ProviderError>;

    /// Get repository details
    async fn get_repository(&self, id: &str) -> Result<Repository, ProviderError>;

    /// List pull requests for a repository
    async fn list_pull_requests(
        &self,
        repo_id: &str,
    ) -> Result<Vec<PullRequest>, ProviderError>;

    /// Get pull request details including CI status
    async fn get_pull_request(
        &self,
        repo_id: &str,
        pr_number: u64,
    ) -> Result<PullRequest, ProviderError>;

    /// Get CI/CD status for a PR
    async fn get_ci_status(
        &self,
        repo_id: &str,
        ref_: &str,
    ) -> Result<Vec<CIStatus>, ProviderError>;

    /// Merge a pull request
    async fn merge_pull_request(
        &self,
        repo_id: &str,
        pr_number: u64,
        strategy: MergeStrategy,
    ) -> Result<(), ProviderError>;

    /// Get rate limit status
    async fn get_rate_limit(&self) -> Result<RateLimit, ProviderError>;
}
```

### Provider Factory

```rust
pub enum ProviderType {
    GitHub,
    GitLab,
    Bitbucket,
}

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(
        provider_type: ProviderType,
        credentials: ProviderCredentials,
    ) -> Arc<dyn GitProvider> {
        match provider_type {
            ProviderType::GitHub => Arc::new(GitHubProvider::new(credentials)),
            ProviderType::GitLab => Arc::new(GitLabProvider::new(credentials)),
            ProviderType::Bitbucket => Arc::new(BitbucketProvider::new(credentials)),
        }
    }
}
```

### Ampel Status Calculation

```rust
pub fn calculate_ampel_status(pr: &PullRequest, ci_statuses: &[CIStatus]) -> AmpelStatus {
    // Red conditions (highest priority)
    if ci_statuses.iter().any(|s| s.state == CIState::Failure || s.state == CIState::Error) {
        return AmpelStatus::Red;
    }
    if pr.mergeable == Some(false) {
        return AmpelStatus::Red; // Has conflicts
    }
    if pr.review_status == Some(ReviewStatus::ChangesRequested) {
        return AmpelStatus::Red;
    }

    // Yellow conditions
    if ci_statuses.iter().any(|s| s.state == CIState::Pending) {
        return AmpelStatus::Yellow;
    }
    if pr.review_status.is_none() || pr.review_status == Some(ReviewStatus::Pending) {
        return AmpelStatus::Yellow;
    }
    if pr.draft {
        return AmpelStatus::Yellow;
    }

    // Green: All checks pass, approved, no conflicts
    AmpelStatus::Green
}
```

---

## Background Job Architecture

### Job Types

| Job               | Purpose                               | Schedule         |
| ----------------- | ------------------------------------- | ---------------- |
| RepositoryPollJob | Poll single repository for PR updates | On-demand / cron |
| UserPollJob       | Poll all repositories for a user      | User-configured  |
| TokenRefreshJob   | Refresh expiring OAuth tokens         | Every 30 minutes |
| CleanupJob        | Remove stale data, old poll results   | Daily            |
| RateLimitCheckJob | Check and respect API rate limits     | Before each poll |

### apalis Configuration

```rust
use apalis::prelude::*;
use apalis_cron::CronStream;
use apalis::postgres::PostgresStorage;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryPollJob {
    pub repo_id: Uuid,
    pub user_id: Uuid,
}

impl Job for RepositoryPollJob {
    const NAME: &'static str = "repository_poll";
}

async fn poll_repository(
    job: RepositoryPollJob,
    ctx: JobContext,
    state: Data<AppState>,
) -> Result<(), JobError> {
    let provider = state.get_provider_for_repo(job.repo_id).await?;

    // Check rate limits before polling
    let rate_limit = provider.get_rate_limit().await?;
    if rate_limit.remaining < 10 {
        return Err(JobError::RateLimited(rate_limit.reset_at));
    }

    // Fetch and update PRs
    let prs = provider.list_pull_requests(&job.repo_id.to_string()).await?;
    state.update_pull_requests(job.repo_id, prs).await?;

    Ok(())
}

// Worker setup
pub async fn start_workers(pool: PgPool) -> Result<()> {
    let storage = PostgresStorage::new(pool);

    // Poll worker
    let poll_worker = WorkerBuilder::new("poll-worker")
        .layer(TraceLayer::new())
        .layer(RetryLayer::new(RetryPolicy::retries(3)))
        .with_storage(storage.clone())
        .build_fn(poll_repository);

    // Cron-based token refresh
    let refresh_schedule = Schedule::from_str("0 */30 * * * *")?;
    let refresh_worker = WorkerBuilder::new("token-refresh")
        .with_storage(CronStream::new(refresh_schedule))
        .build_fn(refresh_tokens);

    Monitor::new()
        .register(poll_worker)
        .register(refresh_worker)
        .run()
        .await?;

    Ok(())
}
```

---

## Security Architecture

### Authentication Flow

```text
┌─────────┐     ┌─────────┐     ┌─────────┐
│ Browser │     │   API   │     │   DB    │
└────┬────┘     └────┬────┘     └────┬────┘
     │               │               │
     │ POST /login   │               │
     │──────────────▶│               │
     │               │ Verify creds  │
     │               │──────────────▶│
     │               │◀──────────────│
     │               │               │
     │ JWT (access)  │               │
     │◀──────────────│               │
     │ Cookie (refresh)              │
     │◀──────────────│               │
     │               │               │
```

### Token Security

1. **User Passwords**: Hashed with Argon2id
2. **JWT Access Token**: 15-minute expiry, stored in memory
3. **JWT Refresh Token**: 7-day expiry, httpOnly cookie
4. **Provider Tokens**: AES-256-GCM encrypted in database

```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

pub struct TokenEncryption {
    cipher: Aes256Gcm,
}

impl TokenEncryption {
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(&random_bytes(12));
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_bytes())?;
        Ok([nonce.as_slice(), &ciphertext].concat())
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String> {
        let (nonce, data) = ciphertext.split_at(12);
        let plaintext = self.cipher.decrypt(Nonce::from_slice(nonce), data)?;
        Ok(String::from_utf8(plaintext)?)
    }
}
```

### Authorization (RBAC)

| Role        | Scope        | Permissions                      |
| ----------- | ------------ | -------------------------------- |
| Owner       | Organization | Full admin, delete org, billing  |
| Admin       | Organization | Manage teams, members, settings  |
| Member      | Organization | View, add repos, manage own PRs  |
| Team Admin  | Team         | Manage team members and repos    |
| Team Member | Team         | View team dashboard, own actions |

### Rate Limiting

```rust
use tower_governor::{GovernorLayer, GovernorConfig};

// API rate limiting: 100 req/min per user
let governor_config = GovernorConfig::default()
    .per_second(100)
    .burst_size(20);

let app = Router::new()
    .route("/api/*", any(api_handler))
    .layer(GovernorLayer::new(governor_config));
```

---

## Deployment Architecture

### Docker Compose (Development/Self-hosted)

```yaml
version: '3.8'

services:
  api:
    build:
      context: .
      dockerfile: docker/Dockerfile.api
    ports:
      - '8080:8080'
    environment:
      DATABASE_URL: postgres://ampel:ampel@postgres:5432/ampel
      REDIS_URL: redis://redis:6379
      JWT_SECRET: ${JWT_SECRET}
      ENCRYPTION_KEY: ${ENCRYPTION_KEY}
    depends_on:
      - postgres
      - redis

  worker:
    build:
      context: .
      dockerfile: docker/Dockerfile.api
    command: ['./ampel-worker']
    environment:
      DATABASE_URL: postgres://ampel:ampel@postgres:5432/ampel
      REDIS_URL: redis://redis:6379
    depends_on:
      - postgres
      - redis

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - '3000:80'
    depends_on:
      - api

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: ampel
      POSTGRES_PASSWORD: ampel
      POSTGRES_DB: ampel
    volumes:
      - pgdata:/var/lib/postgresql/data
    ports:
      - '5432:5432'

  redis:
    image: redis:7-alpine
    ports:
      - '6379:6379'

volumes:
  pgdata:
```

### Fly.io (Production)

```toml
# fly.toml
app = "ampel-api"
primary_region = "sjc"

[build]
  dockerfile = "docker/Dockerfile.api"

[env]
  RUST_LOG = "info,ampel=debug"
  PORT = "8080"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 1

[[services]]
  protocol = "tcp"
  internal_port = 8080

  [[services.ports]]
    port = 80
    handlers = ["http"]

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  [[services.http_checks]]
    interval = "10s"
    timeout = "2s"
    path = "/health"

[mounts]
  source = "ampel_data"
  destination = "/data"
```

### Multi-stage Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-bookworm AS builder

WORKDIR /app
COPY . .

RUN cargo build --release --bin ampel-api

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ampel-api /usr/local/bin/

EXPOSE 8080

CMD ["ampel-api"]
```

---

## Monitoring & Observability

### Structured Logging

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

// Usage
tracing::info!(
    user_id = %user.id,
    repo_count = repos.len(),
    "Polling repositories for user"
);
```

### Health Check Endpoint

```rust
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_healthy = state.db.ping().await.is_ok();
    let redis_healthy = state.redis.ping().await.is_ok();

    if db_healthy && redis_healthy {
        (StatusCode::OK, Json(json!({ "status": "healthy" })))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
            "status": "unhealthy",
            "db": db_healthy,
            "redis": redis_healthy
        })))
    }
}
```

### Metrics (Future)

- Request latency (p50, p95, p99)
- Error rates by endpoint
- Background job success/failure rates
- Provider API rate limit usage
- Active users (DAU/MAU)
