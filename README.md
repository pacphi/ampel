# Ampel 🚦

[![CI](https://github.com/pacphi/ampel/actions/workflows/ci.yml/badge.svg)](https://github.com/pacphi/ampel/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/pacphi/ampel/branch/main/graph/badge.svg)](https://codecov.io/gh/pacphi/ampel)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **See your PRs at a glance.** Ampel brings traffic light simplicity to PR management across GitHub, GitLab, and Bitbucket.

Ampel (German for "traffic light") is a unified PR dashboard that consolidates pull requests from multiple Git providers into a single interface, using an intuitive traffic light system to show what's ready to merge, what's in progress, and what's blocked.

## Why Ampel?

- 🎯 **Stop Context-Switching** - No more jumping between GitHub, GitLab, and Bitbucket. Everything in one dashboard.
- ⚡ **Instant Visibility** - Traffic light status (🟢 Green = ready, 🟡 Yellow = in progress, 🔴 Red = blocked) cuts through the noise.
- 🚀 **Production-Ready** - Battle-tested observability stack with Prometheus metrics, Grafana dashboards, and comprehensive monitoring.
- 🔒 **Self-Hosted & Secure** - Deploy on your infrastructure with AES-256-GCM encrypted PAT storage. No data leaves your control.
- 🤝 **Team-First Design** - Organizations, teams, bulk merge operations, and health scores for managing repos at scale.

## Quick Start

**With Docker (fastest path to value):**

```bash
git clone https://github.com/pacphi/ampel.git
cd ampel && cp .env.example .env
cd docker && docker compose up -d
```

Open [http://localhost:3000](http://localhost:3000) and connect your first repository.

**That's it.** You'll have a fully functional PR dashboard with monitoring stack in under 5 minutes.

## ✨ Features

### Core Capabilities

- **Unified Dashboard** - GitHub, GitLab, and Bitbucket PRs in one view
- **Traffic Light Status** - Visual health indicators with CI integration
- **Smart Filtering** - Filter by provider, status, author, reviewer, labels, age
- **Repository Health Scores** - Track team velocity and identify bottlenecks
- **Multitenancy** - Organizations and teams with role-based access control
- **Multi-Language Support** - 27 languages with RTL support (Arabic, Hebrew) and automatic detection

### Advanced Features

- **Bulk Merge Operations** - Merge multiple approved PRs with progress tracking
- **Team Organization** - Group repos by team, customize dashboards
- **Bot PR Rules** - Separate Dependabot/Renovate PRs with auto-merge support
- **Analytics & Reporting** - PR cycle time, review turnaround, team velocity trends
- **Production Monitoring** - Prometheus metrics, Grafana dashboards, distributed tracing
- **Language Persistence** - User language preference saved to account and synced across devices

[View complete feature matrix →](docs/planning/PRODUCT_SPEC.md)

## 📚 Documentation

### 🚀 Getting Started

**First-time setup and quick wins:**

- [Getting Started Guide](docs/GETTING_STARTED.md) - Installation, configuration, first repository
- [PAT Setup Guide](docs/PAT_SETUP.md) - Configure GitHub, GitLab, Bitbucket Personal Access Tokens
- [Docker Quick Start](docs/RUN.md) - Run Ampel with Docker in 3 commands
- [Makefile Guide](docs/MAKEFILE_GUIDE.md) - Complete reference of all available commands

### ✨ Features & Capabilities

**Deep-dive into what Ampel can do:**

- [Product Specification](docs/planning/PRODUCT_SPEC.md) - Complete feature matrix with implementation status
- [Multitenancy](docs/features/MULTITENANCY.md) - Organizations, teams, and access control
- [Bulk Merge Operations](docs/features/BULK_MERGE.md) - Merge multiple PRs with progress tracking
- [Health Scores](docs/features/HEALTH_SCORES.md) - Repository health scoring and trend analysis

### 🌍 Internationalization (i18n)

**Multi-language support (27 languages with RTL support):**

- Multi-language UI with 27 supported languages
- RTL (right-to-left) support for Arabic, Hebrew, Persian, and Urdu
- Automatic locale detection from browser settings
- Language switcher in user settings

<!-- Documentation links - Coming soon:
- User Guide - Change language, supported languages, RTL support, troubleshooting
- Developer Guide - Add translatable strings, use i18n hooks/macros, test translations
-->

### 🏗️ Architecture & Development

**System design and contributing:**

- [Architecture Overview](docs/ARCHITECTURE.md) - System design, crate structure, database models
- [Development Guide](docs/DEVELOPMENT.md) - Local development setup and workflow
- [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute to Ampel

### 🧪 Testing

**Comprehensive testing documentation:**

- [Testing Overview](docs/TESTING.md) - Testing strategy and quick reference
- [Backend Testing](docs/testing/BACKEND.md) - Rust unit and integration tests (PostgreSQL/SQLite)
- [Frontend Testing](docs/testing/FRONTEND.md) - React component testing with Vitest
- [CI Workflows](docs/testing/CI.md) - GitHub Actions CI pipeline guide
- [Coverage Tracking](docs/testing/COVERAGE.md) - Code coverage goals and reporting
- [Worker Testing](docs/testing/WORKER-TEST-PATTERNS.md) - Background job test patterns

### 📊 Observability

**Production monitoring and troubleshooting:**

- [Observability Overview](docs/observability/README.md) - **Start here** - Complete observability guide
- [Quick Start (5 min)](docs/observability/QUICKSTART.md) - Get monitoring running locally
- [Monitoring Guide](docs/observability/MONITORING.md) - Prometheus, Grafana, alerting setup
- [Metrics Catalog](docs/observability/METRICS.md) - All available metrics with usage examples
- [Prometheus Guide](docs/observability/PROMETHEUS.md) - Prometheus configuration and PromQL queries
- [Grafana Dashboards](docs/observability/GRAFANA.md) - Dashboard creation and visualization
- [API Endpoints](docs/observability/API-ENDPOINTS.md) - Health checks and metrics endpoints
- [Troubleshooting](docs/observability/TROUBLESHOOTING.md) - Common issues and solutions

### 🚀 Deployment

**Production deployment guides:**

- [Fly.io Deployment](docs/DEPLOY.md) - Deploy to Fly.io with native monitoring
- [Docker Deployment](docs/RUN.md) - Self-hosted Docker setup
- [Operations Runbook](docs/deployment/RUNBOOK.md) - Production operations guide
- [Secrets Management](docs/deployment/SECRETS_TEMPLATE.md) - Environment variables and secrets
- [Release Process](docs/RELEASE.md) - How to cut a release

## 🛠️ Tech Stack

Built with a modern, performant stack designed for production workloads:

- **Backend**: Rust 1.91+ with Axum (REST API), SeaORM (database), Apalis (background jobs)
- **Frontend**: React 19 + TypeScript, Vite, TanStack Query, shadcn/ui, Tailwind CSS
- **Database**: PostgreSQL 16 for data, Redis 7 for caching
- **Observability**: Prometheus metrics, Grafana dashboards, OpenTelemetry tracing
- **Deployment**: Docker, Fly.io, self-hosted options

[See complete architecture →](docs/ARCHITECTURE.md)

## 📈 Project Status

**Current Release**: MVP Complete (87% of planned features)

**What's Working Today**:

- ✅ Multi-provider PR aggregation (GitHub, GitLab, Bitbucket)
- ✅ Traffic light status with CI integration
- ✅ Team management and collaboration
- ✅ Bulk merge operations with progress tracking
- ✅ Health scoring and analytics
- ✅ Production-ready observability stack

**In Progress**:

- 🚧 Notification workers (Slack, email)
- 🚧 Bot filtering frontend UI
- 🚧 Export functionality (CSV/PDF)

[View detailed implementation status →](docs/planning/PRODUCT_SPEC.md)

## Contributing

We welcome contributions! Whether you're fixing a bug, adding a feature, or improving documentation, your help makes Ampel better.

**Get Started**:

1. Read the [Contributing Guide](docs/CONTRIBUTING.md)
2. Check the [Development Guide](docs/DEVELOPMENT.md) for setup
3. Browse open issues or propose a new feature
4. Submit a pull request

## License

MIT License — see [LICENSE](LICENSE) for details.

---

_Built with 🚦 by the Ampel team_
