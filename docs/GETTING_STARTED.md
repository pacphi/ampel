# Getting Started with Ampel

Ampel is a PR dashboard application that provides traffic light status indicators
for your pull requests across GitHub, GitLab, and Bitbucket.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+ (for backend development)
- [Node.js](https://nodejs.org/) 20+ with [pnpm](https://pnpm.io/) (for frontend development)
- [Docker](https://www.docker.com/) and Docker Compose (for containerized deployment)
- PostgreSQL 16+ (or use Docker)
- [Make](https://www.gnu.org/software/make/) (optional, for unified commands)

### Option 1: Run with Docker (Recommended)

The fastest way to get Ampel running:

```bash
# Clone the repository
git clone https://github.com/pacphi/ampel.git
cd ampel

# Copy environment template
cp .env.example .env

# Start all services
make docker-up
# Or without Make:
cd docker && docker compose up -d
```

Access the application at `http://localhost:3000`

See [RUN.md](RUN.md) for detailed Docker instructions.

### Option 2: Local Development

For active development:

```bash
# Clone and setup
git clone https://github.com/pacphi/ampel.git
cd ampel
cp .env.example .env

# Install dependencies
make install

# Start PostgreSQL (via Docker or locally)
docker run -d --name ampel-postgres \
  -e POSTGRES_USER=ampel \
  -e POSTGRES_PASSWORD=ampel \
  -e POSTGRES_DB=ampel \
  -p 5432:5432 \
  postgres:16-alpine

# Start all services (run in separate terminals)
make dev-api      # Backend API
make dev-worker   # Background worker
make dev-frontend # Frontend dev server
```

See [DEVELOPMENT.md](DEVELOPMENT.md) for complete development setup.

## Using Make Commands

Ampel provides a unified `Makefile` interface for common operations:

```bash
make help         # Show all available commands
make install      # Install all dependencies
make build        # Build everything
make test         # Run all tests
make lint         # Run all linters
make format       # Format all code
make docker-up    # Start with Docker
```

See `make help` for the full list of available commands.

## Configuration

### Required Environment Variables

| Variable         | Description                                           |
| ---------------- | ----------------------------------------------------- |
| `DATABASE_URL`   | PostgreSQL connection string                          |
| `JWT_SECRET`     | Secret key for JWT tokens (min 32 chars)              |
| `ENCRYPTION_KEY` | Base64-encoded 32-byte key for OAuth token encryption |

### OAuth Providers (Optional)

Configure one or more Git providers:

| Provider  | Variables                                                                  |
| --------- | -------------------------------------------------------------------------- |
| GitHub    | `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GITHUB_REDIRECT_URI`          |
| GitLab    | `GITLAB_CLIENT_ID`, `GITLAB_CLIENT_SECRET`, `GITLAB_REDIRECT_URI`          |
| Bitbucket | `BITBUCKET_CLIENT_ID`, `BITBUCKET_CLIENT_SECRET`, `BITBUCKET_REDIRECT_URI` |

See `.env.example` for all configuration options.

## Project Structure

```text
ampel/
├── crates/                 # Rust backend
│   ├── ampel-api/          # REST API server (Axum)
│   ├── ampel-core/         # Business logic & domain models
│   ├── ampel-db/           # Database layer (SeaORM)
│   ├── ampel-providers/    # Git provider integrations
│   └── ampel-worker/       # Background job processor
├── frontend/               # React SPA
├── docker/                 # Docker configuration
├── docs/                   # Documentation
└── Makefile                # Unified build commands
```

## Next Steps

- [DEVELOPMENT.md](DEVELOPMENT.md) - Set up your development environment
- [CONTRIBUTING.md](CONTRIBUTING.md) - Learn how to contribute
- [DEPLOY.md](DEPLOY.md) - Deploy to Fly.io
- [RUN.md](RUN.md) - Run with Docker
