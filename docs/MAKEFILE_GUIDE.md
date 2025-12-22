<!-- markdownlint-disable MD010 -->

# Ampel Makefile Guide

A beginner-friendly guide to using the Ampel Makefile for development, testing, and deployment.

## Table of Contents

- [What is a Makefile?](#what-is-a-makefile)
- [Quick Reference](#quick-reference)
- [Development Commands](#development-commands)
- [Build Commands](#build-commands)
- [Testing Commands](#testing-commands)
- [Code Quality Commands](#code-quality-commands)
- [Security & Dependencies](#security--dependencies)
- [Docker Commands](#docker-commands)
- [Deployment Commands](#deployment-commands)
- [GitHub Actions Integration](#github-actions-integration)
- [Customizing the Makefile](#customizing-the-makefile)
- [Troubleshooting](#troubleshooting)

## What is a Makefile?

A Makefile is a build automation tool that simplifies complex command sequences into simple, memorable commands. Instead of typing long commands like:

```bash
cd frontend && pnpm install && cd .. && cargo build --release
```

You can just run:

```bash
make install build-release
```

### Why We Use Make in Ampel

- **Unified Interface**: Single command structure for backend (Rust) and frontend (React/TypeScript)
- **Reduced Complexity**: Hides implementation details behind intuitive commands
- **Consistency**: Same commands work for all developers regardless of environment
- **Documentation**: Self-documenting through the `make help` command
- **Automation**: Chains multiple operations together (build → test → deploy)

## Quick Reference

### Most Common Commands

| Command             | Description                    | When to Use                                  |
| ------------------- | ------------------------------ | -------------------------------------------- |
| `make help`         | Show all available commands    | When you forget a command                    |
| `make install`      | Install all dependencies       | First time setup, after pulling changes      |
| `make dev-api`      | Start API server               | Backend development                          |
| `make dev-frontend` | Start frontend dev server      | Frontend development                         |
| `make test`         | Run all tests                  | Before committing code                       |
| `make lint-fix`     | Auto-fix code issues           | Before committing, during development        |
| `make docker-up`    | Start all services with Docker | Full-stack development, testing integrations |

### Common Workflows

**First-time setup:**

```bash
make install          # Install dependencies
make docker-up        # Start database and Redis
make dev-api          # In terminal 1: Start API
make dev-frontend     # In terminal 2: Start frontend
```

**Daily development:**

```bash
make test             # Run tests before starting work
make dev-api          # Start backend
make lint-fix         # Fix any linting issues
make test             # Run tests before committing
```

**Before committing:**

```bash
make format           # Format all code
make lint             # Check for issues
make test             # Run all tests
```

## Development Commands

### `make install`

Installs all project dependencies (backend Rust toolchain + frontend npm packages).

**What it does:**

- Verifies Rust toolchain installation
- Runs `pnpm install` in the frontend directory

**When to use:**

- First time cloning the repository
- After pulling changes that modify dependencies
- When dependency errors occur

**Example output:**

```bash
$ make install
==> Checking Rust toolchain...
active toolchain
----------------
1.91.1-x86_64-unknown-linux-gnu (default)
rustc 1.91.1 (c4616da40 2024-10-17)

==> Installing frontend dependencies...
Packages: +243
Progress: resolved 243, reused 243, downloaded 0, added 243, done
```

**Troubleshooting:**

- **"rustup: command not found"**: Install Rust from [rustup.rs](https://rustup.rs/)
- **"pnpm: command not found"**: Install pnpm with `npm install -g pnpm`

---

### `make dev-api`

Starts the backend API server in development mode.

**What it does:**

- Runs `cargo run --bin ampel-api`
- Starts API server on `http://localhost:8080`
- Enables hot-reloading with cargo watch (if installed)

**When to use:**

- Backend development
- Testing API endpoints
- Debugging backend issues

**Prerequisites:**

- Database running (use `make docker-up` to start PostgreSQL)
- `.env` file configured with `DATABASE_URL`

**Example:**

```bash
$ make dev-api
==> Starting API server...
2025-12-22T12:00:00.000Z INFO  [ampel_api] Starting Ampel API server
2025-12-22T12:00:00.001Z INFO  [ampel_api] Listening on 0.0.0.0:8080
2025-12-22T12:00:00.002Z INFO  [ampel_api] API documentation at http://localhost:8080/api/docs
```

**Access:**

- API: `http://localhost:8080`
- Swagger docs: `http://localhost:8080/api/docs`

**Troubleshooting:**

- **"Connection refused"**: Ensure database is running with `make docker-up`
- **"Port already in use"**: Stop existing API process or change port in `.env`
- **Compilation errors**: Run `make clean build` to rebuild

---

### `make dev-worker`

Starts the background job worker in development mode.

**What it does:**

- Runs `cargo run --bin ampel-worker`
- Processes background jobs (PR syncing, notifications, etc.)
- Connects to PostgreSQL and Redis

**When to use:**

- Developing background job features
- Testing async task processing
- Full-stack development requiring job processing

**Prerequisites:**

- Database and Redis running (`make docker-up`)
- `.env` configured with `DATABASE_URL` and `REDIS_URL`

**Example:**

```bash
$ make dev-worker
==> Starting background worker...
2025-12-22T12:00:00.000Z INFO  [ampel_worker] Starting Apalis worker
2025-12-22T12:00:00.001Z INFO  [ampel_worker] Connected to PostgreSQL
2025-12-22T12:00:00.002Z INFO  [ampel_worker] Worker ready for jobs
```

**Troubleshooting:**

- **"Failed to connect to Redis"**: Ensure Redis is running (`docker ps`)
- **Jobs not processing**: Check database connection and job queue tables

---

### `make dev-frontend`

Starts the frontend development server with hot-reloading.

**What it does:**

- Runs `cd frontend && pnpm run dev`
- Starts Vite dev server on `http://localhost:5173`
- Enables hot module replacement (HMR)

**When to use:**

- Frontend development
- UI/UX work
- Testing React components

**Prerequisites:**

- Frontend dependencies installed (`make install`)
- API server running (`make dev-api`) for full functionality

**Example:**

```bash
$ make dev-frontend
==> Starting frontend dev server...

  VITE v5.0.0  ready in 324 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
  ➜  press h + enter to show help
```

**Access:**

- Frontend: `http://localhost:5173`

**Troubleshooting:**

- **Blank page**: Check browser console for API connection errors
- **"Module not found"**: Run `make install` to install dependencies
- **Slow HMR**: Clear Vite cache with `rm -rf frontend/node_modules/.vite`

---

### `make dev`

Reminder command that shows how to start all services for full-stack development.

**What it does:**

- Prints instructions for starting API, worker, and frontend
- Suggests using Docker Compose as an alternative

**Output:**

```bash
$ make dev
Starting all services...
Run these in separate terminals:
  make dev-api
  make dev-worker
  make dev-frontend

Or use: docker compose up
```

**Recommended workflow:**

Use 3 separate terminal windows/tabs:

```bash
# Terminal 1
make dev-api

# Terminal 2
make dev-worker

# Terminal 3
make dev-frontend
```

Or use Docker for simplicity:

```bash
make docker-up
```

## Build Commands

### `make build`

Builds the entire project in **debug** mode (faster compilation, slower runtime).

**What it does:**

- Runs `cargo build` for backend
- Runs `pnpm run build` for frontend

**When to use:**

- Testing build process locally
- Debugging build issues
- Verifying all code compiles

**Example:**

```bash
$ make build
==> Building backend...
   Compiling ampel-api v0.1.0
   Compiling ampel-worker v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 13s

==> Building frontend...
vite v5.0.0 building for production...
✓ 1234 modules transformed.
dist/index.html                   0.45 kB │ gzip:  0.30 kB
dist/assets/index-abc123.js     543.21 kB │ gzip: 123.45 kB
✓ built in 12.34s
```

**Output locations:**

- Backend: `target/debug/ampel-api`, `target/debug/ampel-worker`
- Frontend: `frontend/dist/`

**Troubleshooting:**

- **Compilation errors**: Check error messages, ensure dependencies are installed
- **Disk space issues**: Run `make clean` to free space

---

### `make build-release`

Builds the project in **release** mode (slower compilation, optimized runtime).

**What it does:**

- Runs `cargo build --release` for backend
- Runs `pnpm run build` for frontend (production mode)

**When to use:**

- Before deployment
- Performance testing
- Creating production binaries

**Example:**

```bash
$ make build-release
==> Building backend (release)...
   Compiling ampel-api v0.1.0
   Compiling ampel-worker v0.1.0
    Finished `release` profile [optimized] target(s) in 5m 34s

==> Building frontend...
[Production build output]
```

**Output locations:**

- Backend: `target/release/ampel-api`, `target/release/ampel-worker`
- Frontend: `frontend/dist/`

**Performance:**

- Release builds are ~10-100x faster at runtime
- Compilation takes 2-3x longer
- Binary size may be larger (contains optimizations)

---

### `make clean`

Removes all build artifacts to free disk space or fix build issues.

**What it does:**

- Runs `cargo clean` (removes `target/` directory)
- Removes `frontend/dist` and Vite cache

**When to use:**

- Build errors that persist after code changes
- Freeing disk space (Rust builds can use 1-5 GB)
- Switching between debug and release builds

**Example:**

```bash
$ make clean
==> Cleaning backend...
==> Cleaning frontend...
```

**Note:** After cleaning, you'll need to rebuild with `make build`.

## Testing Commands

### `make test`

Runs **all tests** (backend + frontend) in one command.

**What it does:**

- Runs `cargo test --all-features` for Rust tests
- Runs `pnpm run test -- --run` for Vitest tests

**When to use:**

- Before committing code
- Verifying changes don't break existing functionality
- CI/CD validation locally

**Example:**

```bash
$ make test
==> Running backend tests...
   Compiling ampel-api v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 23s
     Running unittests src/lib.rs (target/debug/deps/ampel_api-...)

running 47 tests
test auth::tests::test_jwt_creation ... ok
test db::tests::test_connection_pool ... ok
...
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

==> Running frontend tests...

 ✓ src/components/Dashboard.test.tsx (3 tests) 234ms
 ✓ src/api/auth.test.ts (5 tests) 123ms
...

 Test Files  12 passed (12)
      Tests  43 passed (43)
   Start at  12:34:56
   Duration  2.34s
```

**Exit codes:**

- `0`: All tests passed
- `101`: Tests failed

**Troubleshooting:**

- **Database connection errors**: Use SQLite in-memory for tests (see [TESTING.md](./TESTING.md))
- **Flaky tests**: Run individual test with `cargo test test_name` or `pnpm test path/to/test.tsx`
- **Timeout errors**: Increase timeout in test configuration

---

### `make test-backend`

Runs **only backend tests** (faster than full test suite).

**What it does:**

- Runs `cargo test --all-features`
- Includes unit tests and integration tests

**When to use:**

- Backend-only changes
- Faster iteration during backend development
- Debugging backend test failures

**Example:**

```bash
$ make test-backend
==> Running backend tests...
running 47 tests
test result: ok. 47 passed; 0 failed
```

**Test filtering:**

Run specific tests:

```bash
cargo test auth          # Tests with "auth" in name
cargo test test_login    # Specific test
cargo test --lib         # Only unit tests
cargo test --test integration  # Only integration tests
```

---

### `make test-frontend`

Runs **only frontend tests** with Vitest.

**What it does:**

- Runs `cd frontend && pnpm run test -- --run`
- Executes all `.test.ts` and `.test.tsx` files

**When to use:**

- Frontend-only changes
- UI component testing
- Faster iteration during frontend development

**Example:**

```bash
$ make test-frontend
==> Running frontend tests...
 Test Files  12 passed (12)
      Tests  43 passed (43)
```

**Watch mode** (for development):

```bash
cd frontend && pnpm test
```

---

### `make test-coverage`

Generates **code coverage reports** for both backend and frontend.

**What it does:**

- Installs `cargo-tarpaulin` if not present
- Runs backend tests with coverage instrumentation
- Runs frontend tests with coverage
- Generates HTML and XML reports

**When to use:**

- Verifying test coverage meets standards (target: 80%)
- Finding untested code paths
- Before major releases

**Example:**

```bash
$ make test-coverage
==> Running backend tests with coverage...
Installing cargo-tarpaulin...  # First time only
[Progress...]

==> Backend coverage report: coverage/tarpaulin-report.html

==> Running frontend tests with coverage...

==> Coverage reports generated:
    Backend:  coverage/cobertura.xml, coverage/tarpaulin-report.html
    Frontend: frontend/coverage/
```

**Viewing reports:**

```bash
# Backend
open coverage/tarpaulin-report.html

# Frontend
open frontend/coverage/index.html
```

**Interpreting coverage:**

- **Green**: Well-tested code (80%+ coverage)
- **Yellow**: Moderate coverage (50-80%)
- **Red**: Under-tested code (<50%)

**Troubleshooting:**

- **Slow execution**: Coverage runs are 2-5x slower than normal tests
- **Missing coverage**: Ensure tests actually execute the code paths

---

### `make test-backend-coverage`

Backend-only coverage report (faster than full coverage).

**Output:** `coverage/tarpaulin-report.html` and `coverage/cobertura.xml`

---

### `make test-frontend-coverage`

Frontend-only coverage report.

**Output:** `frontend/coverage/index.html`

## Code Quality Commands

### `make lint`

Runs **all linters** (backend Clippy, frontend ESLint, markdown linter).

**What it does:**

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cd frontend && pnpm run lint && pnpm run type-check`
- `pnpm run lint:md`

**When to use:**

- Before committing code
- Catching common mistakes and anti-patterns
- Enforcing code style consistency

**Example:**

```bash
$ make lint
==> Linting backend...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
    Checking ampel-api v0.1.0

==> Linting frontend...
✓ No linting errors found

==> Linting markdown files...
✓ All markdown files are valid
```

**Common issues:**

- **Clippy warnings**: Follow suggestions to improve code quality
- **ESLint errors**: Run `make lint-fix` to auto-fix
- **Type errors**: Fix TypeScript type mismatches

**Troubleshooting:**

- **Too strict**: Clippy is configured with `-D warnings` (all warnings are errors)
- **False positives**: Use `#[allow(clippy::specific_lint)]` sparingly with justification

---

### `make lint-fix`

**Auto-fixes** linting issues where possible.

**What it does:**

- `cargo clippy --fix --allow-dirty --allow-staged`
- `cd frontend && pnpm run lint --fix`
- `pnpm run lint:md:fix`

**When to use:**

- After writing code with minor style issues
- Before committing to clean up code
- Bulk formatting fixes

**Example:**

```bash
$ make lint-fix
==> Auto-fixing backend lint issues...
Fixed 12 issues automatically

==> Auto-fixing frontend lint issues...
✔ 8 problems fixed

==> Auto-fixing markdown lint issues...
Fixed 3 formatting issues
```

**Note:** Not all issues can be auto-fixed. Review remaining warnings manually.

---

### `make format`

Formats **all code** (Rust with rustfmt, frontend with Prettier, markdown with Prettier).

**What it does:**

- `cargo fmt --all`
- `cd frontend && pnpm run format`
- `pnpm run format` (markdown)

**When to use:**

- Before every commit
- After bulk code changes
- Team collaboration (ensures consistent style)

**Example:**

```bash
$ make format
==> Formatting backend...
==> Formatting frontend...
==> Formatting markdown files...
```

**Configuration:**

- Backend: `rustfmt.toml`
- Frontend: `.prettierrc`
- Markdown: `.prettierrc`

---

### `make format-check`

Checks if code is formatted **without making changes**.

**What it does:**

- `cargo fmt --all -- --check`
- `cd frontend && pnpm run format:check`
- `pnpm run format:check`

**When to use:**

- CI/CD pipelines
- Pre-commit hooks
- Verifying code is ready to commit

**Example:**

```bash
$ make format-check
==> Checking backend formatting...
✓ All files are formatted correctly

==> Checking frontend formatting...
✓ All files are formatted correctly

==> Checking markdown formatting...
✓ All markdown files are formatted correctly
```

**Exit codes:**

- `0`: All files formatted correctly
- `1`: Some files need formatting (run `make format`)

## Security & Dependencies

### `make audit`

Runs **security audits** on all dependencies.

**What it does:**

- Installs `cargo-audit` if missing
- Runs `cargo audit` for Rust dependencies
- Runs `pnpm audit` for npm dependencies

**When to use:**

- Weekly security checks
- Before deployments
- After updating dependencies

**Example:**

```bash
$ make audit
==> Auditing backend dependencies...
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 567 security advisories (from /home/user/.cargo/advisory-db)
    Scanning Cargo.lock for vulnerabilities (234 crate dependencies)
✓ No vulnerabilities found

==> Auditing frontend dependencies...
found 0 vulnerabilities
```

**Handling vulnerabilities:**

```bash
# View details
cargo audit

# Update vulnerable dependencies
make upgrade

# If no fix available, assess risk and document exception
```

---

### `make license-check`

Verifies **license compliance** for all dependencies.

**What it does:**

- Installs `cargo-deny` if missing
- Runs `cargo deny check licenses`
- Runs `npx license-checker --summary` for frontend

**When to use:**

- Before adding new dependencies
- Legal compliance reviews
- Open-source releases

**Example:**

```bash
$ make license-check
==> Checking backend license compliance...
✓ All licenses are approved

==> Checking frontend license compliance...
├─ MIT: 189
├─ ISC: 34
├─ Apache-2.0: 23
└─ BSD-3-Clause: 12
```

**Configuration:**

- Backend: `deny.toml` (allowed licenses)
- Review licenses before accepting new dependencies

---

### `make outdated`

Lists **outdated dependencies** (but doesn't update them).

**What it does:**

- Installs `cargo-outdated` if missing
- Runs `cargo outdated -R` (root dependencies only)
- Runs `pnpm outdated` for frontend

**When to use:**

- Monthly dependency reviews
- Before planning updates
- Security maintenance

**Example:**

```bash
$ make outdated
==> Checking for outdated backend dependencies...
Name       Project  Compat  Latest   Kind
----       -------  ------  ------   ----
axum       0.6.0    0.6.20  0.7.0    Normal
tokio      1.32.0   1.35.1  1.35.1   Normal

==> Checking for outdated frontend dependencies...
Package         Current  Wanted  Latest
react           19.0.0   19.0.0  19.0.1
vite            5.0.0    5.0.0   5.1.0
```

**Interpretation:**

- **Project**: Current version
- **Compat**: Latest compatible with semver
- **Latest**: Newest version (may require breaking changes)

---

### `make upgrade`

Updates dependencies to **latest compatible versions** (respects semver).

**What it does:**

- Runs `cargo update` (updates `Cargo.lock`)
- Runs `cd frontend && pnpm update` (updates `pnpm-lock.yaml`)

**When to use:**

- Monthly maintenance updates
- After reviewing `make outdated`
- Security patches

**Example:**

```bash
$ make upgrade
==> Upgrading backend dependencies...
    Updating crates.io index
    Updating axum v0.6.0 -> v0.6.20
    Updating tokio v1.32.0 -> v1.35.1
Dependencies updated in Cargo.lock

==> Upgrading frontend dependencies...
Progress: resolved 243, reused 241, downloaded 2, added 0, done
Dependencies updated in pnpm-lock.yaml
```

**After upgrading:**

```bash
make test           # Verify nothing broke
make audit          # Check for new vulnerabilities
git add Cargo.lock pnpm-lock.yaml
git commit -m "chore: update dependencies"
```

---

### `make upgrade-latest`

Updates to **latest versions** (may break semver, use cautiously).

**What it does:**

- Installs `cargo-edit` if missing
- Runs `cargo upgrade` then `cargo update`
- Runs `cd frontend && pnpm update --latest`

**When to use:**

- Major version upgrades
- Migration to new library versions
- After thorough testing in a branch

**Example:**

```bash
$ make upgrade-latest
==> Upgrading backend to latest versions...
    Updating axum v0.6.0 -> v0.7.0 (BREAKING)
    Updating tokio v1.32.0 -> v1.35.1

==> Upgrading frontend to latest versions...
    Updating react v19.0.0 -> v19.0.1
```

**Warning:** This may introduce breaking changes. Test thoroughly.

## Docker Commands

### `make docker-build`

Builds all Docker images locally.

**What it does:**

- Runs `cd docker && docker compose build`
- Builds images for API, worker, frontend, database, Redis

**When to use:**

- After changing Dockerfiles
- Testing production builds locally
- Preparing for deployment

**Example:**

```bash
$ make docker-build
==> Building Docker images...
[+] Building 234.5s (45/45) FINISHED
 => [api internal] load build definition from Dockerfile.api
 => [worker internal] load build definition from Dockerfile.worker
 => [frontend internal] load build definition from Dockerfile.frontend
```

---

### `make docker-up`

Starts **all services** with Docker Compose in detached mode.

**What it does:**

- Runs `cd docker && docker compose up -d`
- Starts PostgreSQL, Redis, API, worker, frontend

**When to use:**

- Full-stack development
- Testing integrations
- Simplifying local setup (no need for 3 terminals)

**Example:**

```bash
$ make docker-up
==> Starting Docker services...
[+] Running 5/5
 ✔ Container ampel-postgres  Started
 ✔ Container ampel-redis     Started
 ✔ Container ampel-api       Started
 ✔ Container ampel-worker    Started
 ✔ Container ampel-frontend  Started
```

**Access:**

- Frontend: `http://localhost:5173`
- API: `http://localhost:8080`
- API Docs: `http://localhost:8080/api/docs`
- PostgreSQL: `localhost:5432` (user: `ampel`, password: `ampel`)
- Redis: `localhost:6379`

**Viewing logs:**

```bash
make docker-logs
```

---

### `make docker-down`

Stops all Docker services.

**What it does:**

- Runs `cd docker && docker compose down`
- Stops containers but **preserves volumes** (database data)

**When to use:**

- Ending development session
- Freeing system resources
- Before rebuilding images

**Example:**

```bash
$ make docker-down
==> Stopping Docker services...
[+] Running 5/5
 ✔ Container ampel-frontend  Stopped
 ✔ Container ampel-worker    Stopped
 ✔ Container ampel-api       Stopped
 ✔ Container ampel-redis     Stopped
 ✔ Container ampel-postgres  Stopped
```

---

### `make docker-restart`

Stops services, rebuilds images, and starts them again.

**What it does:**

- Runs `cd docker && docker compose down && docker compose up -d --build`

**When to use:**

- After changing Dockerfiles
- Applying code changes to running containers
- Troubleshooting container issues

**Example:**

```bash
$ make docker-restart
==> Restarting Docker services with rebuild...
[Rebuilding and restarting all services]
```

---

### `make docker-logs`

Shows **live logs** from all Docker containers.

**What it does:**

- Runs `cd docker && docker compose logs -f`
- Streams logs with color coding by service

**When to use:**

- Debugging issues
- Monitoring application behavior
- Viewing startup messages

**Example:**

```bash
$ make docker-logs
ampel-api      | 2025-12-22T12:00:00Z INFO [ampel_api] Starting API server
ampel-worker   | 2025-12-22T12:00:00Z INFO [ampel_worker] Processing job: sync_prs
ampel-frontend | VITE v5.0.0  ready in 324 ms
```

**Filtering logs:**

```bash
# Single service
cd docker && docker compose logs -f api

# Last 100 lines
cd docker && docker compose logs --tail 100
```

---

### `make docker-clean`

**Removes all Docker resources** including volumes (deletes database data).

**What it does:**

- Runs `cd docker && docker compose down -v --rmi local`
- Deletes containers, volumes, networks, and local images

**When to use:**

- Complete reset of Docker environment
- Freeing disk space
- Troubleshooting persistent issues

**Example:**

```bash
$ make docker-clean
==> Cleaning Docker resources...
[+] Running 8/8
 ✔ Container ampel-frontend  Removed
 ✔ Container ampel-api       Removed
 ✔ Volume ampel_postgres_data Removed
 ✔ Network ampel_default     Removed
```

**Warning:** This deletes all database data. Backup first if needed.

## Deployment Commands

### `make deploy-fly`

Deploys **all services** to Fly.io (API, worker, frontend).

**What it does:**

- Runs `fly deploy` for each service with respective configs
- Deploys API, worker, and frontend in sequence

**Prerequisites:**

- Fly.io CLI installed: `brew install flyctl`
- Authenticated: `fly auth login`
- Fly.io apps created (see deployment docs)

**When to use:**

- Production deployments
- Staging environment updates

**Example:**

```bash
$ make deploy-fly
==> Deploying API to Fly.io...
==> Building image
--> Building image done
==> Pushing image to fly
==> Creating release
--> Release v5 created

==> Deploying worker to Fly.io...
[Worker deployment output]

==> Deploying frontend to Fly.io...
[Frontend deployment output]

==> Deployment complete!
```

**Individual deployments:**

```bash
make deploy-fly-api       # API only
make deploy-fly-worker    # Worker only
make deploy-fly-frontend  # Frontend only
```

**Troubleshooting:**

- **Build failures**: Check Docker configuration and logs
- **Health check failures**: Verify service starts correctly locally first
- **Database connection errors**: Ensure database secrets are set in Fly.io

## GitHub Actions Integration

These commands require the [GitHub CLI](https://cli.github.com/) (`gh`).

### `make gh-ci`

Triggers the CI workflow on your **current branch**.

**What it does:**

- Gets current branch name
- Runs `gh workflow run ci.yml --ref <branch>`

**When to use:**

- Testing CI before pushing
- Re-running failed CI builds
- Manual CI triggers

**Example:**

```bash
$ make gh-ci
==> Triggering CI workflow...
CI workflow triggered on branch: feature/new-auth
Run 'make gh-watch' to monitor progress
```

**Monitoring:**

```bash
make gh-watch   # Watch live progress
make gh-status  # View recent CI runs
```

---

### `make gh-release V=x.y.z`

Creates and pushes a **release tag**, triggering the release workflow.

**What it does:**

- Creates annotated git tag `v<version>`
- Pushes tag to GitHub
- Triggers automated release workflow

**When to use:**

- Creating new releases
- Deploying to production

**Example:**

```bash
$ make gh-release V=1.2.3
==> Creating release v1.2.3...
Release v1.2.3 tag pushed. Release workflow will start automatically.
Run 'make gh-watch' to monitor progress
```

**Requirements:**

- Version must follow semver (e.g., `1.2.3`, not `v1.2.3`)
- Tag must not already exist
- You must be on the correct branch (usually `main`)

**Troubleshooting:**

- **"Tag already exists"**: Use `git tag -d v1.2.3` to delete locally, `git push origin :refs/tags/v1.2.3` to delete remotely
- **Failed workflow**: Check GitHub Actions tab for errors

---

### `make gh-watch`

Watches the **latest workflow run** with live updates.

**What it does:**

- Runs `gh run watch`
- Shows real-time progress of CI/CD jobs

**When to use:**

- After triggering CI with `make gh-ci`
- After creating a release
- Monitoring deploy progress

**Example:**

```bash
$ make gh-watch
==> Watching latest workflow run...
✓ backend-tests    3s
✓ frontend-tests   2s
* docker-build     Running...
  deploy           Pending
```

---

### `make gh-runs`

Lists the **10 most recent workflow runs**.

**What it does:**

- Runs `gh run list --limit 10`

**Example:**

```bash
$ make gh-runs
STATUS  TITLE              WORKFLOW  BRANCH         EVENT       ID
✓       CI                 CI        main           push        123456789
✗       CI                 CI        feat/auth      pull_request 123456788
✓       Release v1.2.2     Release   main           push        123456787
```

---

### `make gh-status`

Shows status of the **5 most recent CI workflow runs**.

**What it does:**

- Runs `gh run list --workflow=ci.yml --limit 5`

**Example:**

```bash
$ make gh-status
STATUS  TITLE  WORKFLOW  BRANCH      EVENT  ID
✓       CI     CI        main        push   123456789
✓       CI     CI        feat/auth   push   123456788
```

## Customizing the Makefile

### Adding a New Command

Edit the Makefile and add a new target:

```makefile
.PHONY: my-command
my-command:
	@echo "==> Running my custom command..."
	cargo run --example my_example
```

Then run:

```bash
make my-command
```

### Using Variables

Define variables at the top of the Makefile:

```makefile
PORT ?= 8080
DATABASE_URL ?= postgres://localhost/ampel

dev-api:
	DATABASE_URL=$(DATABASE_URL) cargo run --bin ampel-api
```

Override from command line:

```bash
make dev-api DATABASE_URL=postgres://prod-db/ampel
```

### Chaining Commands

Combine multiple targets:

```makefile
deploy: test build-release deploy-fly
	@echo "==> Deployed successfully!"
```

Run with:

```bash
make deploy   # Runs test → build-release → deploy-fly
```

### Conditional Execution

Check for prerequisites:

```makefile
check-env:
	@test -f .env || { echo "Error: .env not found"; exit 1; }

dev-api: check-env
	cargo run --bin ampel-api
```

## Troubleshooting

### Common Issues

**"make: command not found"**

Install Make:

```bash
# macOS
brew install make

# Ubuntu/Debian
sudo apt install make

# Windows (Git Bash)
# Make is included with Git for Windows
```

---

**"No rule to make target"**

You may have a typo in the command name. Run `make help` to see available commands.

---

**"Permission denied"**

Some commands require Docker or system permissions:

```bash
# Add user to docker group (Linux)
sudo usermod -aG docker $USER
newgrp docker
```

---

**Database connection errors**

Ensure services are running:

```bash
make docker-up
docker ps   # Verify containers are running
```

Check `.env` file has correct `DATABASE_URL`:

```bash
DATABASE_URL=postgres://ampel:ampel@localhost:5432/ampel
```

---

**Port already in use**

Find and kill the process:

```bash
# Find process on port 8080
lsof -i :8080

# Kill process
kill -9 <PID>

# Or change port in .env
PORT=8081
```

---

**Rust compilation errors**

Clean and rebuild:

```bash
make clean
make build
```

Ensure Rust toolchain is up to date:

```bash
rustup update
```

---

**Frontend build errors**

Clear caches:

```bash
cd frontend
rm -rf node_modules/.vite dist
pnpm install
```

---

**Docker build failures**

Check Docker daemon is running:

```bash
docker ps
```

Free up disk space:

```bash
docker system prune -a
```

---

**Tests failing unexpectedly**

Run tests individually to isolate failures:

```bash
# Backend
cargo test specific_test_name -- --nocapture

# Frontend
cd frontend && pnpm test src/components/MyComponent.test.tsx
```

Check for environment issues:

```bash
# Use in-memory SQLite for tests
export DATABASE_URL="sqlite::memory:"
cargo test
```

---

### Getting Help

1. **Check the help**: `make help`
2. **Read documentation**: [TESTING.md](./TESTING.md), [ARCHITECTURE.md](./ARCHITECTURE.md)
3. **View logs**: `make docker-logs`
4. **Ask the team**: Open an issue or discuss in team chat

---

## Quick Command Cheatsheet

```bash
# First-time setup
make install
make docker-up

# Daily development
make dev-api          # Terminal 1
make dev-frontend     # Terminal 2
make dev-worker       # Terminal 3 (if needed)

# Before committing
make format
make lint-fix
make test

# Deployment
make build-release
make deploy-fly

# Maintenance
make audit
make outdated
make upgrade
```

---

**Happy developing with Ampel!** 🚦
