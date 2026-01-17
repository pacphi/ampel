# Ampel - Implementation Plan

## Project Structure

```text
ampel/
├── Cargo.toml                 # Workspace root
├── README.md
├── docs/
│   ├── PRODUCT_SPEC.md
│   ├── ARCHITECTURE.md
│   └── IMPLEMENTATION_PLAN.md
│
├── crates/
│   ├── ampel-api/            # Axum HTTP server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   ├── oauth.rs
│   │       │   ├── repositories.rs
│   │       │   ├── pull_requests.rs
│   │       │   └── dashboard.rs
│   │       ├── handlers/
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   └── rate_limit.rs
│   │       └── extractors/
│   │
│   ├── ampel-core/           # Business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── user.rs
│   │       │   ├── repository.rs
│   │       │   ├── pull_request.rs
│   │       │   └── ampel_status.rs
│   │       ├── services/
│   │       │   ├── mod.rs
│   │       │   ├── auth_service.rs
│   │       │   ├── repo_service.rs
│   │       │   └── pr_service.rs
│   │       └── errors.rs
│   │
│   ├── ampel-db/             # Database layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entities/
│   │       │   ├── mod.rs
│   │       │   ├── user.rs
│   │       │   ├── organization.rs
│   │       │   ├── git_provider.rs
│   │       │   ├── repository.rs
│   │       │   ├── pull_request.rs
│   │       │   └── ci_status.rs
│   │       ├── migrations/
│   │       │   ├── mod.rs
│   │       │   └── m20250101_000001_initial.rs
│   │       └── queries/
│   │
│   ├── ampel-providers/      # Git provider clients
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs
│   │       ├── github.rs
│   │       ├── gitlab.rs
│   │       ├── bitbucket.rs
│   │       ├── factory.rs
│   │       └── error.rs
│   │
│   └── ampel-worker/         # Background job processor
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── jobs/
│               ├── mod.rs
│               ├── poll_repository.rs
│               ├── refresh_token.rs
│               └── cleanup.rs
│
├── frontend/                  # React application
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── tailwind.config.js
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── components/
│       │   ├── ui/           # shadcn/ui components
│       │   ├── layout/
│       │   │   ├── Sidebar.tsx
│       │   │   ├── Header.tsx
│       │   │   └── Layout.tsx
│       │   ├── dashboard/
│       │   │   ├── GridView.tsx
│       │   │   ├── ListView.tsx
│       │   │   ├── RepoCard.tsx
│       │   │   └── StatusBadge.tsx
│       │   └── pr/
│       │       ├── PRDetailPanel.tsx
│       │       └── CIStatusList.tsx
│       ├── pages/
│       │   ├── Login.tsx
│       │   ├── Register.tsx
│       │   ├── Dashboard.tsx
│       │   ├── Repositories.tsx
│       │   └── Settings.tsx
│       ├── hooks/
│       │   ├── useAuth.ts
│       │   ├── usePullRequests.ts
│       │   └── useRepositories.ts
│       ├── api/
│       │   ├── client.ts
│       │   ├── auth.ts
│       │   ├── repositories.ts
│       │   └── pullRequests.ts
│       └── types/
│           ├── index.ts
│           ├── auth.ts
│           ├── repository.ts
│           └── pullRequest.ts
│
├── docker/
│   ├── Dockerfile.api
│   ├── Dockerfile.frontend
│   └── docker-compose.yml
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── deploy.yml
│
└── .env.example
```

---

## Phase 1: MVP (Foundation)

### Milestone 1.1: Project Setup

**Goal**: Establish project foundation and development environment.

- [ ] Initialize Cargo workspace with `Cargo.toml`
- [ ] Create crate directory structure
- [ ] Set up workspace dependencies
- [ ] Configure CI/CD pipeline (GitHub Actions)
- [ ] Create Docker Compose for local development
- [ ] Initialize React frontend with Vite + TypeScript
- [ ] Set up Tailwind CSS and shadcn/ui
- [ ] Create `.env.example` with required variables
- [ ] Add pre-commit hooks (rustfmt, clippy, eslint)

**Deliverables**:

- Working `cargo build` for all crates
- Working `npm run dev` for frontend
- `docker compose up` starts all services

### Milestone 1.2: Database & Models

**Goal**: Implement data layer with SeaORM.

- [ ] Design SeaORM entities for all tables
- [ ] Create initial migration (`m20250101_000001_initial.rs`)
- [ ] Implement repository pattern for data access
- [ ] Add database connection pool configuration
- [ ] Create seed data script for development
- [ ] Write unit tests for entity relationships

**Deliverables**:

- `sea-orm-cli migrate` runs successfully
- CRUD operations work for all entities
- Test coverage for database queries

### Milestone 1.3: Authentication

**Goal**: Implement user authentication with JWT.

- [ ] User registration endpoint with validation
- [ ] User login endpoint returning JWT
- [ ] JWT token generation (access + refresh)
- [ ] Password hashing with Argon2id
- [ ] Auth middleware for protected routes
- [ ] Refresh token rotation
- [ ] Logout endpoint (token invalidation)

**Deliverables**:

- Users can register and login
- Protected endpoints require valid JWT
- Refresh tokens work correctly

### Milestone 1.4: OAuth Integration

**Goal**: Connect to GitHub, GitLab, and Bitbucket via OAuth.

- [ ] GitHub OAuth flow implementation
- [ ] GitLab OAuth flow implementation
- [ ] Bitbucket OAuth flow implementation
- [ ] Token encryption (AES-256-GCM) for storage
- [ ] Token refresh mechanism for each provider
- [ ] Frontend OAuth connection UI
- [ ] Provider disconnection endpoint

**Deliverables**:

- Users can connect all three providers
- Tokens are encrypted in database
- Token refresh works automatically

### Milestone 1.5: Provider Abstraction

**Goal**: Create unified interface for Git providers.

- [ ] Define `GitProvider` trait with all methods
- [ ] Implement `GitHubProvider` using octocrab
- [ ] Implement `GitLabProvider` using gitlab crate
- [ ] Implement `BitbucketProvider` using reqwest
- [ ] Create `ProviderFactory` for instantiation
- [ ] Implement rate limit tracking per provider
- [ ] Write integration tests (mocked responses)

**Deliverables**:

- All three providers implement same trait
- Rate limits are tracked and respected
- Provider-specific errors are normalized

### Milestone 1.6: Core API

**Goal**: Implement REST API endpoints.

- [ ] Repository management endpoints (CRUD)
- [ ] Repository discovery endpoint
- [ ] Pull request listing with filters
- [ ] Pull request detail endpoint
- [ ] Dashboard summary endpoint
- [ ] Dashboard grid/list endpoints
- [ ] OpenAPI documentation with utoipa
- [ ] Request validation with validator crate

**Deliverables**:

- All endpoints documented in OpenAPI
- Filtering works correctly
- Pagination implemented

### Milestone 1.7: Background Jobs

**Goal**: Set up scheduled polling with apalis.

- [ ] Configure apalis with PostgreSQL storage
- [ ] Implement `RepositoryPollJob`
- [ ] Implement `TokenRefreshJob`
- [ ] Create cron scheduler for periodic polls
- [ ] Add job retry logic with backoff
- [ ] Implement rate limit awareness
- [ ] Create worker binary (`ampel-worker`)

**Deliverables**:

- Polling runs on schedule
- Token refresh prevents expiration
- Jobs retry on failure

### Milestone 1.8: Frontend - Core

**Goal**: Build authentication and navigation UI.

- [ ] Login page with form validation
- [ ] Registration page
- [ ] OAuth provider connection page
- [ ] Main navigation layout
- [ ] Sidebar component
- [ ] Header with user menu
- [ ] Dark/light theme toggle
- [ ] Protected route wrapper

**Deliverables**:

- Users can login/register
- OAuth connections work from UI
- Theme persists across sessions

### Milestone 1.9: Frontend - Dashboard

**Goal**: Build main dashboard views.

- [ ] Grid view with repository cards
- [ ] Status badge component (traffic light)
- [ ] List view with sortable table
- [ ] View toggle (grid/list)
- [ ] Filter sidebar component
- [ ] Search input with debounce
- [ ] Saved filter presets
- [ ] Auto-refresh with configurable interval

**Deliverables**:

- Both views display PR data
- Filtering reduces results
- Search works across repos

### Milestone 1.10: Frontend - Details

**Goal**: Build PR detail and repository views.

- [ ] Repository detail page
- [ ] PR detail side panel
- [ ] CI status list component
- [ ] Review status display
- [ ] Link to provider (GitHub/GitLab/Bitbucket)
- [ ] Manual refresh button
- [ ] Loading and error states

**Deliverables**:

- PR details display in panel
- CI checks show with status
- Links open in new tab

---

## Phase 2: Team Features

### Milestone 2.1: Multi-tenancy

- [ ] Organization entity and endpoints
- [ ] Team entity and endpoints
- [ ] Member management (invite, remove)
- [ ] Role-based access control (RBAC)
- [ ] Organization settings page
- [ ] Team dashboard view
- [ ] Member permissions UI

### Milestone 2.2: Notifications

- [ ] Slack webhook integration
- [ ] Email notification service (SMTP)
- [ ] Notification preferences API
- [ ] Notification preferences UI
- [ ] Browser push notifications
- [ ] Notification history

### Milestone 2.3: Merge Actions

- [ ] Merge endpoint per provider
- [ ] Merge strategy selection (squash, merge, rebase)
- [ ] Batch merge support
- [ ] Merge confirmation dialog
- [ ] Post-merge status update
- [ ] Merge history tracking

### Milestone 2.4: Bot Handling

- [ ] Bot author detection (Dependabot, Renovate, etc.)
- [ ] Bot PR filtering in UI
- [ ] Auto-merge rules configuration
- [ ] Auto-merge execution
- [ ] Bot-specific dashboard view

---

## Phase 3: Analytics & Intelligence

### Milestone 3.1: Metrics Collection

- [ ] PR cycle time tracking
- [ ] Review turnaround metrics
- [ ] Historical data storage (time series)
- [ ] Data retention policies

### Milestone 3.2: Reporting

- [ ] Analytics dashboard page
- [ ] PR throughput charts
- [ ] Review time distribution
- [ ] Team comparison views
- [ ] Export to CSV/PDF

### Milestone 3.3: Health Scoring

- [ ] Repository health algorithm
- [ ] Health score display per repo
- [ ] Health trend visualization
- [ ] Degradation alerting
- [ ] Health improvement suggestions

---

## Risk Assessment

| Risk                  | Impact | Likelihood | Mitigation                                                         |
| --------------------- | ------ | ---------- | ------------------------------------------------------------------ |
| API rate limits       | High   | Medium     | Intelligent caching, request batching, rate limit tracking         |
| Provider API changes  | Medium | Low        | Abstract provider interface, version pinning, integration tests    |
| Token expiration      | Medium | Medium     | Proactive refresh jobs, expiry alerts, graceful degradation        |
| Scale with many repos | High   | Medium     | Pagination, lazy loading, efficient queries, background processing |
| OAuth security        | High   | Low        | Follow OAuth 2.0 best practices, secure token storage, HTTPS only  |
| Database performance  | Medium | Medium     | Proper indexing, query optimization, connection pooling            |
| Frontend bundle size  | Low    | Medium     | Code splitting, lazy loading, tree shaking                         |

---

## Development Workflow

### Branching Strategy

```text
main ──────────────────────────────────────────▶ Production
  │
  └─── develop ─────────────────────────────────▶ Integration
         │
         ├─── feature/auth ─────────────────────▶ Feature branch
         ├─── feature/github-provider ──────────▶ Feature branch
         └─── fix/rate-limit-handling ──────────▶ Bug fix branch
```

### Pull Request Process

1. Create feature branch from `develop`
2. Implement changes with tests
3. Run `cargo fmt` and `cargo clippy`
4. Open PR with description and screenshots
5. CI must pass (tests, lint, build)
6. Code review required (1 approval)
7. Squash merge to `develop`

### Testing Strategy

| Type              | Coverage Target | Tools                           |
| ----------------- | --------------- | ------------------------------- |
| Unit tests        | 80%             | `cargo test`, Jest              |
| Integration tests | Critical paths  | `cargo test --test`, Playwright |
| E2E tests         | Happy paths     | Playwright                      |
| API tests         | All endpoints   | Bruno/Postman                   |

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  backend:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: test
        ports:
          - 5432:5432
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test

  frontend:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: frontend
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm ci
      - run: npm run lint
      - run: npm run build
      - run: npm test
```

---

## Dependencies

### Cargo.toml (Workspace Root)

```toml
[workspace]
resolver = "2"
members = [
    "crates/ampel-api",
    "crates/ampel-core",
    "crates/ampel-db",
    "crates/ampel-providers",
    "crates/ampel-worker",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/pacphi/ampel"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Web framework
axum = "0.7"
axum-extra = { version = "0.9", features = ["typed-header", "cookie"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "compression-gzip"] }

# Database
sea-orm = { version = "1.0", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
sea-orm-migration = "1.0"

# Background jobs
apalis = { version = "0.5", features = ["postgres"] }
apalis-cron = "0.5"

# Git providers
octocrab = "0.47"
gitlab = "0.1900"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Auth & Security
jsonwebtoken = "9"
argon2 = "0.5"
aes-gcm = "0.10"
rand = "0.8"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
config = "0.14"
dotenvy = "0.15"

# API Documentation
utoipa = { version = "4", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "7", features = ["axum"] }

# Internal crates
ampel-core = { path = "crates/ampel-core" }
ampel-db = { path = "crates/ampel-db" }
ampel-providers = { path = "crates/ampel-providers" }
```

### Frontend package.json

```json
{
  "name": "ampel-frontend",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext ts,tsx",
    "test": "vitest"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.22.0",
    "@tanstack/react-query": "^5.24.0",
    "axios": "^1.6.7",
    "react-hook-form": "^7.50.0",
    "@hookform/resolvers": "^3.3.4",
    "zod": "^3.22.4",
    "lucide-react": "^0.336.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.1"
  },
  "devDependencies": {
    "@types/react": "^18.2.56",
    "@types/react-dom": "^18.2.19",
    "@vitejs/plugin-react": "^4.2.1",
    "autoprefixer": "^10.4.17",
    "eslint": "^8.56.0",
    "postcss": "^8.4.35",
    "tailwindcss": "^3.4.1",
    "typescript": "^5.3.3",
    "vite": "^5.1.4",
    "vitest": "^1.3.1"
  }
}
```

---

## Environment Variables

```bash
# .env.example

# Database
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key-min-32-chars
JWT_ACCESS_EXPIRY=15m
JWT_REFRESH_EXPIRY=7d

# Encryption (for provider tokens)
ENCRYPTION_KEY=your-32-byte-encryption-key

# GitHub OAuth
GITHUB_CLIENT_ID=your-github-client-id
GITHUB_CLIENT_SECRET=your-github-client-secret
GITHUB_REDIRECT_URI=http://localhost:8080/api/oauth/github/callback

# GitLab OAuth
GITLAB_CLIENT_ID=your-gitlab-client-id
GITLAB_CLIENT_SECRET=your-gitlab-client-secret
GITLAB_REDIRECT_URI=http://localhost:8080/api/oauth/gitlab/callback

# Bitbucket OAuth
BITBUCKET_CLIENT_ID=your-bitbucket-client-id
BITBUCKET_CLIENT_SECRET=your-bitbucket-client-secret
BITBUCKET_REDIRECT_URI=http://localhost:8080/api/oauth/bitbucket/callback

# Server
HOST=0.0.0.0
PORT=8080
RUST_LOG=info,ampel=debug

# Frontend
VITE_API_URL=http://localhost:8080/api
```

---

## Getting Started (Development)

```bash
# Clone repository
git clone https://github.com/pacphi/ampel.git
cd ampel

# Copy environment file
cp .env.example .env
# Edit .env with your OAuth credentials

# Start infrastructure
docker compose up -d postgres redis

# Run database migrations
cd crates/ampel-db
sea-orm-cli migrate up
cd ../..

# Start backend API
cargo run --bin ampel-api

# In another terminal, start worker
cargo run --bin ampel-worker

# In another terminal, start frontend
cd frontend
npm install
npm run dev

# Access application
open http://localhost:3000
```
