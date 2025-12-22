# Deploying Ampel to Fly.io

This guide provides comprehensive instructions for deploying Ampel to [Fly.io](https://fly.io) using their global application platform with managed databases, private networking, and automated deployments.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Infrastructure Setup](#infrastructure-setup)
- [Application Deployment](#application-deployment)
- [Secrets Management](#secrets-management)
- [CI/CD with GitHub Actions](#cicd-with-github-actions)
- [Custom Domain Setup](#custom-domain-setup)
- [Monitoring and Operations](#monitoring-and-operations)
- [Scaling](#scaling)
- [Troubleshooting](#troubleshooting)
- [Cost Estimation](#cost-estimation)
- [References](#references)

---

## Architecture Overview

Ampel deploys as three separate Fly.io applications connected via private networking:

```text
┌─────────────────────────────────────────────────────────────────┐
│                      Fly.io Organization                        │
│                                                                 │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────────┐   │
│  │    Frontend    │  │   API Server   │  │     Worker       │   │
│  │    (nginx)     │  │  (Rust/Axum)   │  │  (Rust/Apalis)   │   │
│  │   Port: 8080   │  │   Port: 8080   │  │  No Public Port  │   │
│  │  Public HTTPS  │  │  Public HTTPS  │  │  Internal Only   │   │
│  └────────────────┘  └────────────────┘  └──────────────────┘   │
│          │                   │                    │             │
│          └───────────────────┼────────────────────┘             │
│                              │                                  │
│                   6PN Private Network (fdaa::/48)               │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐             │
│         │                    │                    │             │
│  ┌──────┴───────┐    ┌───────┴───────┐     ┌──────┴──────┐      │
│  │   Managed    │    │    Upstash    │     │ Fly Metrics │      │
│  │  PostgreSQL  │    │     Redis     │     │  Dashboard  │      │
│  └──────────────┘    └───────────────┘     └─────────────┘      │
└─────────────────────────────────────────────────────────────────┘
```

**Service Communication:**

| From     | To         | Network      | URL Pattern                        |
| -------- | ---------- | ------------ | ---------------------------------- |
| Frontend | API        | Public HTTPS | `https://ampel-api.fly.dev/api`    |
| API      | PostgreSQL | Private 6PN  | `postgres://ampel-db.flycast:5432` |
| API      | Redis      | Private 6PN  | `redis://ampel-redis.flycast:6379` |
| Worker   | PostgreSQL | Private 6PN  | `postgres://ampel-db.flycast:5432` |
| Worker   | Redis      | Private 6PN  | `redis://ampel-redis.flycast:6379` |

> **Reference**: [Fly.io Private Networking](https://fly.io/docs/networking/private-networking/)

---

## Prerequisites

### Required Tools

1. **Fly CLI (flyctl)** - Install from [fly.io/docs/flyctl/install](https://fly.io/docs/flyctl/install/):

   ```bash
   # macOS/Linux
   curl -L https://fly.io/install.sh | sh

   # Windows (PowerShell)
   pwsh -Command "iwr https://fly.io/install.ps1 -useb | iex"

   # Homebrew
   brew install flyctl
   ```

2. **Docker** - For local testing of deployment images

3. **Fly.io Account** - Sign up at [fly.io](https://fly.io)

### Authentication

```bash
# Login to Fly.io
fly auth login

# Verify authentication
fly auth whoami
```

---

## Quick Start

For experienced users, here's the minimal deployment sequence:

```bash
# 1. Create organization and apps
fly orgs create ampel-org
fly apps create ampel-api --org ampel-org
fly apps create ampel-worker --org ampel-org
fly apps create ampel-frontend --org ampel-org

# 2. Create PostgreSQL database
fly postgres create --name ampel-db --org ampel-org --region iad

# 3. Create Redis
fly redis create --name ampel-redis --org ampel-org --region iad

# 4. Set secrets (see Secrets Management section for full list)
fly secrets set --app ampel-api \
  DATABASE_URL="postgres://..." \
  REDIS_URL="redis://..." \
  JWT_SECRET="$(openssl rand -hex 32)"

# 5. Deploy all services
fly deploy --config fly/fly.api.toml --remote-only
fly deploy --config fly/fly.worker.toml --remote-only
fly deploy --config fly/fly.frontend.toml --remote-only
```

---

## Infrastructure Setup

### 1. Create Fly.io Organization

```bash
# Create organization
fly orgs create ampel-org

# List organizations
fly orgs list

# Set default organization
fly orgs select ampel-org
```

### 2. Provision Managed PostgreSQL

Fly.io Managed Postgres provides high-availability PostgreSQL with automatic backups.

```bash
fly postgres create \
  --name ampel-db \
  --org ampel-org \
  --region iad \
  --vm-size shared-cpu-1x \
  --volume-size 10
```

**Configuration options:**

| Option          | Recommended     | Notes                           |
| --------------- | --------------- | ------------------------------- |
| `--region`      | `iad`           | Ashburn, VA - good for US users |
| `--vm-size`     | `shared-cpu-1x` | Start small, scale as needed    |
| `--volume-size` | `10`            | 10GB storage, expandable        |

After creation, save the connection credentials displayed. Connect and set up:

```bash
# Connect to database
fly postgres connect --app ampel-db

# Create database and extensions
CREATE DATABASE ampel;
\c ampel
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
\q
```

> **Reference**: [Fly.io Managed Postgres](https://fly.io/docs/postgres/)

### 3. Provision Upstash Redis

Upstash Redis provides serverless Redis with per-request pricing.

```bash
fly redis create \
  --name ampel-redis \
  --org ampel-org \
  --region iad
```

Test the connection:

```bash
fly redis connect --app ampel-redis
PING
# Should return: PONG
```

> **Reference**: [Upstash for Redis on Fly.io](https://fly.io/docs/upstash/redis/)

### 4. Create Application Entries

Create Fly.io apps without deploying:

```bash
# API Server
fly apps create ampel-api --org ampel-org

# Background Worker
fly apps create ampel-worker --org ampel-org

# Frontend
fly apps create ampel-frontend --org ampel-org
```

---

## Application Deployment

### Deployment Files

All deployment configuration files are in the `fly/` directory:

```text
fly/
├── fly.api.toml        # API server configuration
├── fly.worker.toml     # Worker configuration
├── fly.frontend.toml   # Frontend configuration
├── fly/               # Fly.io deployment configs
│   ├── fly.api.toml
│   ├── fly.worker.toml
│   ├── fly.frontend.toml
│   └── nginx.conf
├── docker/            # Docker build assets
│   ├── Dockerfile.api
│   ├── Dockerfile.worker
│   ├── Dockerfile.frontend
│   ├── docker-compose.yml
│   └── nginx.conf
└── nginx.conf          # nginx configuration for SPA
```

### Deploy API Server

```bash
# Deploy API
fly deploy --config fly/fly.api.toml --remote-only

# Verify deployment
fly status --app ampel-api
fly logs --app ampel-api

# Test health endpoint
curl https://ampel-api.fly.dev/health
```

### Deploy Worker

```bash
# Deploy Worker
fly deploy --config fly/fly.worker.toml --remote-only

# Verify deployment
fly status --app ampel-worker
fly logs --app ampel-worker
```

### Deploy Frontend

```bash
# Deploy Frontend (set API URL at build time)
fly deploy --config fly/fly.frontend.toml \
  --build-arg VITE_API_URL=https://ampel-api.fly.dev \
  --remote-only

# Verify deployment
fly status --app ampel-frontend
curl https://ampel-frontend.fly.dev
```

### Run Database Migrations

```bash
# Run migrations via SSH
fly ssh console --app ampel-api -C "/app/ampel-api migrate run"

# Check migration status
fly ssh console --app ampel-api -C "/app/ampel-api migrate status"
```

---

## Secrets Management

Fly.io stores secrets encrypted and injects them as environment variables at runtime.

> **Reference**: [Fly.io Secrets](https://fly.io/docs/apps/secrets/)

### Generate Secure Keys

```bash
# Generate 256-bit random keys
openssl rand -hex 32
```

### API Server Secrets

```bash
fly secrets set --app ampel-api \
  DATABASE_URL="postgres://postgres:<PASSWORD>@ampel-db.flycast:5432/ampel" \
  REDIS_URL="redis://default:<PASSWORD>@ampel-redis.flycast:6379" \
  JWT_SECRET="<RANDOM_256_BIT_KEY>" \
  ENCRYPTION_KEY="<RANDOM_256_BIT_KEY>" \
  CORS_ORIGINS="https://ampel-frontend.fly.dev"
```

**Note**: Personal Access Tokens (PATs) are configured per-user through the UI after deployment. See [PAT_SETUP.md](PAT_SETUP.md) for instructions.

### Worker Secrets

```bash
fly secrets set --app ampel-worker \
  DATABASE_URL="postgres://postgres:<PASSWORD>@ampel-db.flycast:5432/ampel" \
  REDIS_URL="redis://default:<PASSWORD>@ampel-redis.flycast:6379" \
  ENCRYPTION_KEY="<SAME_AS_API>"
```

### Verify Secrets

```bash
# List secret names (values are encrypted)
fly secrets list --app ampel-api
```

### Bulk Import from File

Create `.env.production` (DO NOT COMMIT):

```env
DATABASE_URL=postgres://postgres:xxx@ampel-db.flycast:5432/ampel
REDIS_URL=redis://default:xxx@ampel-redis.flycast:6379
JWT_SECRET=xxx
ENCRYPTION_KEY=xxx
```

Import:

```bash
fly secrets import --app ampel-api < .env.production
```

---

## CI/CD with GitHub Actions

### Setup Deploy Token

1. Generate a Fly.io deploy token:

   ```bash
   fly tokens create deploy --name github-actions -x 999999h
   ```

2. Add to GitHub repository secrets:
   - Go to: Repository → Settings → Secrets and variables → Actions
   - Create secret: `FLY_API_TOKEN`
   - Paste the token value (including `FlyV1` prefix)

### GitHub Workflow

The workflow at `.github/workflows/deploy.yml` provides:

- **Test jobs** before deployment (backend + frontend)
- **Path-based triggers** - only deploys changed components
- **Rolling deployment strategy** for zero-downtime
- **Automatic database migrations** after API deployment
- **Deployment verification** with health checks

**Required GitHub Secrets:**

| Secret          | Description         |
| --------------- | ------------------- |
| `FLY_API_TOKEN` | Fly.io deploy token |

**Optional Secrets (for build-time injection):**

| Secret         | Description                 |
| -------------- | --------------------------- |
| `VITE_API_URL` | API URL for frontend builds |

> **Reference**: [Fly.io + GitHub Actions](https://fly.io/docs/launch/continuous-deployment-with-github-actions/)

### Manual Deployment Script

Create `scripts/deploy-fly.sh`:

```bash
#!/bin/bash
set -e

echo "Deploying Ampel to Fly.io..."

echo "Deploying API..."
fly deploy --config fly/fly.api.toml --remote-only

echo "Running migrations..."
fly ssh console --app ampel-api -C "/app/ampel-api migrate run"

echo "Deploying Worker..."
fly deploy --config fly/fly.worker.toml --remote-only

echo "Deploying Frontend..."
fly deploy --config fly/fly.frontend.toml \
  --build-arg VITE_API_URL=https://ampel-api.fly.dev \
  --remote-only

echo "Deployment complete!"
echo "Frontend: https://ampel-frontend.fly.dev"
echo "API: https://ampel-api.fly.dev"
```

---

## Custom Domain Setup

### Add Custom Domains

```bash
# Frontend domain
fly certs add ampel.example.com --app ampel-frontend

# API domain
fly certs add api.ampel.example.com --app ampel-api
```

### DNS Configuration

Add DNS records at your registrar:

| Type  | Name           | Value                    |
| ----- | -------------- | ------------------------ |
| CNAME | `@` or `ampel` | `ampel-frontend.fly.dev` |
| CNAME | `api`          | `ampel-api.fly.dev`      |

### Update Configuration After Domain Setup

```bash
# Update CORS origins
fly secrets set --app ampel-api \
  CORS_ORIGINS="https://ampel.example.com,https://www.ampel.example.com"
```

> **Reference**: [Fly.io Custom Domains](https://fly.io/docs/networking/custom-domain/)

---

## Monitoring and Operations

### View Logs

```bash
# Real-time logs
fly logs --app ampel-api -f

# Last 100 lines
fly logs --app ampel-api

# Filter by machine
fly logs --app ampel-api --machine <machine-id>
```

### Check Application Status

```bash
# Status overview
fly status --app ampel-api

# Health checks
fly checks list --app ampel-api

# Machine details
fly machines list --app ampel-api
```

### SSH Access

```bash
# Interactive shell
fly ssh console --app ampel-api

# Run single command
fly ssh console --app ampel-api -C "ls -la /app"
```

### Database Operations

```bash
# Connect to PostgreSQL
fly postgres connect --app ampel-db

# Backup database
fly postgres db backup --app ampel-db

# List backups
fly postgres db list --app ampel-db
```

### Restart Services

```bash
fly apps restart ampel-api
fly apps restart ampel-worker
fly apps restart ampel-frontend
```

---

## Scaling

### Horizontal Scaling (Instances)

```bash
# Scale API to 3 instances
fly scale count 3 --app ampel-api

# Scale Worker to 2 instances
fly scale count 2 --app ampel-worker
```

### Vertical Scaling (Resources)

```bash
# Upgrade VM size
fly scale vm shared-cpu-2x --app ampel-api

# Increase memory
fly scale memory 1024 --app ampel-api
```

### Regional Scaling

```bash
# Add instance in Frankfurt
fly scale count 1 --region fra --app ampel-api

# List available regions
fly platform regions
```

### Auto-scaling Configuration

Auto-scaling is configured in `fly.toml`:

```toml
[http_service]
  auto_stop_machines = "stop"   # Stop idle machines
  auto_start_machines = true    # Start on demand
  min_machines_running = 1      # Minimum instances
```

> **Reference**: [Fly.io Scaling](https://fly.io/docs/apps/scale-count/)

---

## Troubleshooting

### App Won't Start

```bash
# Check logs for errors
fly logs --app ampel-api

# Verify secrets are set
fly secrets list --app ampel-api

# SSH and check binary
fly ssh console --app ampel-api -C "ls -la /app"
```

### Database Connection Errors

```bash
# Verify DATABASE_URL uses .flycast address
fly secrets list --app ampel-api | grep DATABASE

# Test connection from app
fly ssh console --app ampel-api -C "psql \$DATABASE_URL -c 'SELECT 1;'"

# Check database status
fly status --app ampel-db
```

### Health Checks Failing

```bash
# Test health endpoint
curl -v https://ampel-api.fly.dev/health

# Check from inside machine
fly ssh console --app ampel-api -C "curl localhost:8080/health"

# Review health check config in fly.toml
```

### Rollback Deployment

```bash
# List releases
fly releases --app ampel-api

# Rollback to previous version
fly releases rollback --app ampel-api

# Rollback to specific version
fly releases rollback --app ampel-api --version 5
```

---

## Cost Estimation

### Monthly Costs (USD)

| Service            | Plan                 | Monthly Cost      |
| ------------------ | -------------------- | ----------------- |
| Managed PostgreSQL | Shared-1x, 1GB       | ~$15-20           |
| PostgreSQL Storage | 10GB                 | ~$2.50            |
| Upstash Redis      | Pay-per-request      | ~$0-10            |
| API VM             | shared-cpu-1x, 512MB | ~$5-7             |
| Worker VM          | shared-cpu-1x, 512MB | ~$5-7             |
| Frontend VM        | shared-cpu-1x, 256MB | ~$3-5             |
| **Total**          |                      | **~$30-50/month** |

**Notes:**

- Fly.io offers $5 free credit monthly for Hobby plan
- VMs billed per-second when running
- Auto-stop machines reduce costs significantly
- Prices vary by region and actual usage

> **Reference**: [Fly.io Pricing](https://fly.io/docs/about/pricing/)

---

## Security Checklist

Before going to production:

- [ ] All secrets stored in Fly.io vault (not in code)
- [ ] Database uses private `.flycast` address
- [ ] HTTPS enforced (`force_https = true`)
- [ ] CORS origins restricted to frontend domain
- [ ] Non-root user in Dockerfiles
- [ ] PAT encryption key securely generated and stored
- [ ] Security headers configured in nginx
- [ ] Database backups enabled
- [ ] `cargo audit` run for vulnerabilities

---

## References

### Official Fly.io Documentation

1. [Getting Started](https://fly.io/docs/getting-started/) - Initial setup guide
2. [Fly.io Configuration Reference](https://fly.io/docs/reference/configuration/) - fly.toml specification
3. [Managed Postgres](https://fly.io/docs/postgres/) - Database setup and management
4. [Upstash Redis](https://fly.io/docs/upstash/redis/) - Redis configuration
5. [Private Networking](https://fly.io/docs/networking/private-networking/) - 6PN setup
6. [Secrets Management](https://fly.io/docs/apps/secrets/) - Secure configuration
7. [Health Checks](https://fly.io/docs/reference/health-checks/) - Monitoring configuration
8. [GitHub Actions Deployment](https://fly.io/docs/launch/continuous-deployment-with-github-actions/) - CI/CD setup
9. [Rust on Fly.io](https://fly.io/docs/rust/) - Rust deployment patterns
10. [Static Sites](https://fly.io/docs/languages-and-frameworks/static/) - Frontend deployment

### Community Resources

11. [Deploying Rust Axum apps](https://fly.io/docs/rust/frameworks/axum/) - Framework-specific guide
12. [Fly.io Community Forum](https://community.fly.io/) - Community support

---

## File Reference

### Deployment Configuration

| File                         | Purpose                                       |
| ---------------------------- | --------------------------------------------- |
| `fly/fly.api.toml`           | API server Fly.io configuration               |
| `fly/fly.worker.toml`        | Worker Fly.io configuration                   |
| `fly/fly.frontend.toml`      | Frontend Fly.io configuration                 |
| `fly/nginx.conf`             | Fly.io-specific nginx configuration           |
| `docker/Dockerfile.api`      | API multi-stage build with cargo-chef         |
| `docker/Dockerfile.worker`   | Worker multi-stage build                      |
| `docker/Dockerfile.frontend` | Frontend build with nginx                     |
| `docker/nginx.conf`          | nginx SPA configuration with security headers |
| `docker/docker-compose.yml`  | Local development orchestration               |

### CI/CD

| File                           | Purpose                       |
| ------------------------------ | ----------------------------- |
| `.github/workflows/deploy.yml` | GitHub Actions CI/CD workflow |

### Supplementary Documentation

| File                                  | Purpose                         |
| ------------------------------------- | ------------------------------- |
| `docs/deployment/RUNBOOK.md`          | Detailed operations runbook     |
| `docs/deployment/SECRETS_TEMPLATE.md` | Secrets configuration templates |

### Local Development (`docker/`)

| File                         | Purpose                               |
| ---------------------------- | ------------------------------------- |
| `docker/docker-compose.yml`  | Local development environment         |
| `docker/Dockerfile.api`      | Local API build (with BuildKit cache) |
| `docker/Dockerfile.worker`   | Local worker build                    |
| `docker/Dockerfile.frontend` | Local frontend build                  |
