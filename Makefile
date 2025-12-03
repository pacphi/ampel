# Ampel Root Makefile
# Unified interface for build, test, and deployment operations

.PHONY: help
.PHONY: install build build-release clean
.PHONY: dev dev-api dev-worker dev-frontend
.PHONY: test test-backend test-frontend
.PHONY: lint lint-backend lint-frontend lint-docs lint-fix lint-fix-backend lint-fix-frontend lint-fix-docs
.PHONY: format format-backend format-frontend format-docs format-check format-check-docs
.PHONY: audit audit-backend audit-frontend
.PHONY: outdated outdated-backend outdated-frontend
.PHONY: upgrade upgrade-backend upgrade-frontend upgrade-latest
.PHONY: docker docker-build docker-up docker-down docker-logs
.PHONY: deploy deploy-fly
.PHONY: gh-ci gh-release gh-watch gh-runs gh-status

# =============================================================================
# Help
# =============================================================================

help:
	@echo "Ampel Makefile - Unified build interface"
	@echo ""
	@echo "Setup & Dependencies:"
	@echo "  install          - Install all dependencies (backend + frontend)"
	@echo ""
	@echo "Build:"
	@echo "  build            - Build everything (debug)"
	@echo "  build-release    - Build everything (release)"
	@echo "  clean            - Clean all build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  dev              - Start all services for development"
	@echo "  dev-api          - Start API server only"
	@echo "  dev-worker       - Start background worker only"
	@echo "  dev-frontend     - Start frontend dev server only"
	@echo ""
	@echo "Testing:"
	@echo "  test             - Run all tests"
	@echo "  test-backend     - Run backend tests only"
	@echo "  test-frontend    - Run frontend tests only"
	@echo ""
	@echo "Code Quality:"
	@echo "  lint             - Run all linters"
	@echo "  lint-fix         - Auto-fix lint issues"
	@echo "  format           - Format all code"
	@echo "  format-check     - Check formatting without changes"
	@echo ""
	@echo "Security & Dependencies:"
	@echo "  audit            - Run security audits"
	@echo "  outdated         - List outdated dependencies"
	@echo "  upgrade          - Upgrade dependencies to latest compatible"
	@echo ""
	@echo "Docker:"
	@echo "  docker-build     - Build all Docker images"
	@echo "  docker-up        - Start all services with Docker Compose"
	@echo "  docker-down      - Stop all Docker services"
	@echo "  docker-logs      - View Docker logs"
	@echo ""
	@echo "Deployment:"
	@echo "  deploy-fly       - Deploy to Fly.io"
	@echo ""
	@echo "GitHub Actions (requires gh CLI):"
	@echo "  gh-ci            - Trigger CI workflow on current branch"
	@echo "  gh-release V=x.y.z - Create and push a release tag"
	@echo "  gh-watch         - Watch the latest workflow run"
	@echo "  gh-runs          - List recent workflow runs"
	@echo "  gh-status        - View CI workflow status"

# =============================================================================
# Setup & Dependencies
# =============================================================================

install: install-backend install-frontend

install-backend:
	@echo "==> Checking Rust toolchain..."
	rustup show

install-frontend:
	@echo "==> Installing frontend dependencies..."
	cd frontend && pnpm install

# =============================================================================
# Build
# =============================================================================

build: build-backend build-frontend

build-backend:
	@echo "==> Building backend..."
	cargo build

build-release: build-release-backend build-frontend

build-release-backend:
	@echo "==> Building backend (release)..."
	cargo build --release

build-frontend:
	@echo "==> Building frontend..."
	cd frontend && pnpm run build

clean: clean-backend clean-frontend

clean-backend:
	@echo "==> Cleaning backend..."
	cargo clean

clean-frontend:
	@echo "==> Cleaning frontend..."
	cd frontend && rm -rf dist node_modules/.vite

# =============================================================================
# Development
# =============================================================================

dev:
	@echo "Starting all services..."
	@echo "Run these in separate terminals:"
	@echo "  make dev-api"
	@echo "  make dev-worker"
	@echo "  make dev-frontend"
	@echo ""
	@echo "Or use: docker compose up"

dev-api:
	@echo "==> Starting API server..."
	cargo run --bin ampel-api

dev-worker:
	@echo "==> Starting background worker..."
	cargo run --bin ampel-worker

dev-frontend:
	@echo "==> Starting frontend dev server..."
	cd frontend && pnpm run dev

# =============================================================================
# Testing
# =============================================================================

test: test-backend test-frontend

test-backend:
	@echo "==> Running backend tests..."
	cargo test --all-features

test-frontend:
	@echo "==> Running frontend tests..."
	cd frontend && pnpm run test -- --run

# =============================================================================
# Code Quality
# =============================================================================

lint: lint-backend lint-frontend lint-docs

lint-backend:
	@echo "==> Linting backend..."
	cargo clippy --all-targets --all-features -- -D warnings

lint-frontend:
	@echo "==> Linting frontend..."
	cd frontend && pnpm run lint

lint-docs:
	@echo "==> Linting markdown files..."
	pnpm run lint:md

lint-fix: lint-fix-backend lint-fix-frontend lint-fix-docs

lint-fix-backend:
	@echo "==> Auto-fixing backend lint issues..."
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

lint-fix-frontend:
	@echo "==> Auto-fixing frontend lint issues..."
	cd frontend && pnpm run lint --fix

lint-fix-docs:
	@echo "==> Auto-fixing markdown lint issues..."
	pnpm run lint:md:fix

format: format-backend format-frontend format-docs

format-backend:
	@echo "==> Formatting backend..."
	cargo fmt --all

format-frontend:
	@echo "==> Formatting frontend..."
	cd frontend && pnpm run format

format-docs:
	@echo "==> Formatting markdown files..."
	pnpm run format

format-check: format-check-backend format-check-frontend format-check-docs

format-check-backend:
	@echo "==> Checking backend formatting..."
	cargo fmt --all -- --check

format-check-frontend:
	@echo "==> Checking frontend formatting..."
	cd frontend && pnpm run format:check

format-check-docs:
	@echo "==> Checking markdown formatting..."
	pnpm run format:check

# =============================================================================
# Security
# =============================================================================

audit: audit-backend audit-frontend

audit-backend:
	@echo "==> Auditing backend dependencies..."
	cargo audit

audit-frontend:
	@echo "==> Auditing frontend dependencies..."
	cd frontend && pnpm audit

# =============================================================================
# Dependency Management
# =============================================================================

outdated: outdated-backend outdated-frontend

outdated-backend:
	@echo "==> Checking for outdated backend dependencies..."
	@command -v cargo-outdated >/dev/null 2>&1 || { echo "Installing cargo-outdated..."; cargo install cargo-outdated; }
	cargo outdated -R

outdated-frontend:
	@echo "==> Checking for outdated frontend dependencies..."
	cd frontend && pnpm outdated

upgrade: upgrade-backend upgrade-frontend

upgrade-backend:
	@echo "==> Upgrading backend dependencies..."
	cargo update
	@echo "Dependencies updated in Cargo.lock"

upgrade-frontend:
	@echo "==> Upgrading frontend dependencies..."
	cd frontend && pnpm update
	@echo "Dependencies updated in pnpm-lock.yaml"

# Upgrade to latest versions (may break semver constraints)
upgrade-latest: upgrade-latest-backend upgrade-latest-frontend

upgrade-latest-backend:
	@echo "==> Upgrading backend to latest versions..."
	@command -v cargo-edit >/dev/null 2>&1 || { echo "Installing cargo-edit..."; cargo install cargo-edit; }
	cargo upgrade
	cargo update

upgrade-latest-frontend:
	@echo "==> Upgrading frontend to latest versions..."
	cd frontend && pnpm update --latest

# =============================================================================
# Docker
# =============================================================================

docker-build:
	@echo "==> Building Docker images..."
	cd docker && docker compose build

docker-up:
	@echo "==> Starting Docker services..."
	cd docker && docker compose up -d

docker-down:
	@echo "==> Stopping Docker services..."
	cd docker && docker compose down

docker-logs:
	cd docker && docker compose logs -f

docker-clean:
	@echo "==> Cleaning Docker resources..."
	cd docker && docker compose down -v --rmi local

# =============================================================================
# Deployment
# =============================================================================

deploy-fly: deploy-fly-api deploy-fly-worker deploy-fly-frontend
	@echo "==> Deployment complete!"

deploy-fly-api:
	@echo "==> Deploying API to Fly.io..."
	fly deploy --config fly.api.toml

deploy-fly-worker:
	@echo "==> Deploying worker to Fly.io..."
	fly deploy --config fly.worker.toml

deploy-fly-frontend:
	@echo "==> Deploying frontend to Fly.io..."
	fly deploy --config fly.frontend.toml

# =============================================================================
# CI helpers (used by GitHub Actions)
# =============================================================================

ci-backend: format-check-backend lint-backend test-backend build-release-backend

ci-frontend: lint-frontend test-frontend build-frontend

ci: ci-backend ci-frontend
	@echo "==> All CI checks passed!"

# =============================================================================
# GitHub Actions (requires gh CLI)
# =============================================================================

# Check if gh CLI is available
.PHONY: _check-gh
_check-gh:
	@command -v gh >/dev/null 2>&1 || { echo "Error: gh CLI is not installed. Install from https://cli.github.com/"; exit 1; }
	@gh auth status >/dev/null 2>&1 || { echo "Error: gh CLI is not authenticated. Run 'gh auth login'"; exit 1; }

# Trigger CI workflow on current branch
gh-ci: _check-gh
	@echo "==> Triggering CI workflow..."
	@BRANCH=$$(git rev-parse --abbrev-ref HEAD); \
	gh workflow run ci.yml --ref $$BRANCH; \
	echo "CI workflow triggered on branch: $$BRANCH"; \
	echo "Run 'make gh-watch' to monitor progress"

# Create a release tag and trigger release workflow
# Usage: make gh-release V=1.0.0
gh-release: _check-gh
ifndef V
	$(error Version not specified. Usage: make gh-release V=x.y.z)
endif
	@echo "==> Creating release v$(V)..."
	@if git rev-parse "v$(V)" >/dev/null 2>&1; then \
		echo "Error: Tag v$(V) already exists"; \
		exit 1; \
	fi
	@git tag -a "v$(V)" -m "Release v$(V)"
	@git push origin "v$(V)"
	@echo "Release v$(V) tag pushed. Release workflow will start automatically."
	@echo "Run 'make gh-watch' to monitor progress"

# Watch the latest workflow run
gh-watch: _check-gh
	@echo "==> Watching latest workflow run..."
	@gh run watch

# List recent workflow runs
gh-runs: _check-gh
	@gh run list --limit 10

# View CI workflow status
gh-status: _check-gh
	@gh run list --workflow=ci.yml --limit 5
