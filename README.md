# Ampel

> **See your PRs at a glance.** Ampel brings traffic light simplicity to PR management.

Ampel (German for "traffic light") is a unified PR dashboard that gives you instant
visibility into pull request health across all your repositories—GitHub, GitLab,
and Bitbucket—in one place.

## Why Ampel?

**Stop context-switching.** No more jumping between GitHub tabs, GitLab windows,
and Bitbucket dashboards. Ampel consolidates everything into a single, beautiful interface.

**Know what needs attention.** Our traffic light system cuts through the noise:

- 🟢 **Green** — Ready to merge. CI passing, reviews approved.
- 🟡 **Yellow** — In progress. Waiting on CI or reviews.
- 🔴 **Red** — Blocked. Failed checks or requested changes.

**Stay in flow.** Automatic polling keeps your dashboard current. Notifications alert
you when PRs need attention. Health scores help you spot bottlenecks before they
slow your team down.

## Features

- **Unified Dashboard** — GitHub, GitLab, and Bitbucket in one view
- **Traffic Light Status** — Instant visual PR health indicators
- **Repository Health Scores** — Track team velocity and identify bottlenecks
- **Smart Filtering** — By provider, status, author, reviewer, labels
- **Team Organization** — Group repos by team within your org
- **Bot PR Handling** — Special treatment for Dependabot, Renovate, and more
- **Notifications** — Slack and email alerts when PRs need you
- **One-Click Merges** — Merge directly from the dashboard

## Quick Start

**With Docker (fastest):**

```bash
git clone https://github.com/pacphi/ampel.git
cd ampel && cp .env.example .env
cd docker && docker compose up -d
```

Open [http://localhost:3000](http://localhost:3000) and connect your first repository.

**For development setup, deployment options, and more:** See [Getting Started](docs/GETTING_STARTED.md)

## Documentation

| Guide                                      | Description                          |
| ------------------------------------------ | ------------------------------------ |
| [Getting Started](docs/GETTING_STARTED.md) | Quick start and configuration        |
| [PAT Setup](docs/PAT_SETUP.md)             | GitHub, GitLab, Bitbucket PAT tokens |
| [Development](docs/DEVELOPMENT.md)         | Build and run locally                |
| [Testing](docs/TESTING.md)                 | Testing strategy and guides          |
| [Contributing](docs/CONTRIBUTING.md)       | How to contribute                    |
| [Deployment](docs/DEPLOY.md)               | Deploy to Fly.io                     |
| [Docker](docs/RUN.md)                      | Run with Docker                      |
| [Releases](docs/RELEASE.md)                | Release process                      |

**Planning & Architecture:**

- [Product Spec](docs/planning/PRODUCT_SPEC.md) — Features and requirements
- [Architecture](docs/planning/ARCHITECTURE.md) — System design

## Built With

A modern, performant stack: **Rust** + **Axum** on the backend, **React** + **TypeScript**
on the frontend, **PostgreSQL** for data, and **Docker** for deployment.

[See full tech stack →](docs/DEVELOPMENT.md#project-architecture)

## Contributing

We welcome contributions! Please read our [Contributing Guide](docs/CONTRIBUTING.md) to get started.

## License

MIT License — see [LICENSE](LICENSE) for details.

---

_Built with 🚦 by the Ampel team_
