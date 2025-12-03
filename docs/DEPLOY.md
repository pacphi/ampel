# Deploying to Fly.io

This guide covers deploying Ampel to [Fly.io](https://fly.io) using their VM-based infrastructure.

## Prerequisites

- [Fly CLI](https://fly.io/docs/getting-started/installing-flyctl/) installed
- Fly.io account (sign up at https://fly.io)
- Docker installed locally

## Initial Setup

### 1. Authenticate with Fly

```bash
fly auth login
```

### 2. Create Fly Applications

Create three apps: one for API, one for worker, and one for frontend.

```bash
# API server
fly apps create ampel-api

# Background worker
fly apps create ampel-worker

# Frontend
fly apps create ampel-frontend
```

## Database Setup

### Create PostgreSQL Database

```bash
fly postgres create --name ampel-db
```

When prompted, choose:

- Region: Your preferred region (e.g., `iad` for Virginia)
- Configuration: Start with `Development` for testing, `Production` for live

### Attach Database to Apps

```bash
fly postgres attach ampel-db --app ampel-api
fly postgres attach ampel-db --app ampel-worker
```

This automatically sets `DATABASE_URL` as a secret on both apps.

## Configuration Files

### API (`fly.api.toml`)

Create `fly.api.toml` in the project root:

```toml
app = "ampel-api"
primary_region = "iad"

[build]
  dockerfile = "docker/Dockerfile.api"

[env]
  HOST = "0.0.0.0"
  PORT = "8080"
  RUST_LOG = "info,ampel=debug"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 1

[[http_service.checks]]
  grace_period = "10s"
  interval = "30s"
  method = "GET"
  path = "/health"
  timeout = "5s"

[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 512
```

### Worker (`fly.worker.toml`)

Create `fly.worker.toml`:

```toml
app = "ampel-worker"
primary_region = "iad"

[build]
  dockerfile = "docker/Dockerfile.worker"

[env]
  RUST_LOG = "info,ampel=debug"

[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 512

[processes]
  worker = "/usr/local/bin/ampel-worker"
```

### Frontend (`fly.frontend.toml`)

Create `fly.frontend.toml`:

```toml
app = "ampel-frontend"
primary_region = "iad"

[build]
  dockerfile = "docker/Dockerfile.frontend"
  [build.args]
    VITE_API_URL = "https://ampel-api.fly.dev/api"

[http_service]
  internal_port = 80
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 1

[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 256
```

## Secrets Configuration

Set required secrets for the API:

```bash
# JWT secret (generate a secure key)
fly secrets set JWT_SECRET="$(openssl rand -hex 32)" --app ampel-api

# Encryption key for OAuth tokens
fly secrets set ENCRYPTION_KEY="$(openssl rand -base64 32)" --app ampel-api

# GitHub OAuth (if using)
fly secrets set GITHUB_CLIENT_ID="your-client-id" --app ampel-api
fly secrets set GITHUB_CLIENT_SECRET="your-client-secret" --app ampel-api
fly secrets set GITHUB_REDIRECT_URI="https://ampel-api.fly.dev/api/oauth/github/callback" --app ampel-api

# GitLab OAuth (if using)
fly secrets set GITLAB_CLIENT_ID="your-client-id" --app ampel-api
fly secrets set GITLAB_CLIENT_SECRET="your-client-secret" --app ampel-api
fly secrets set GITLAB_REDIRECT_URI="https://ampel-api.fly.dev/api/oauth/gitlab/callback" --app ampel-api

# Bitbucket OAuth (if using)
fly secrets set BITBUCKET_CLIENT_ID="your-client-id" --app ampel-api
fly secrets set BITBUCKET_CLIENT_SECRET="your-client-secret" --app ampel-api
fly secrets set BITBUCKET_REDIRECT_URI="https://ampel-api.fly.dev/api/oauth/bitbucket/callback" --app ampel-api

# CORS origins
fly secrets set CORS_ORIGINS="https://ampel-frontend.fly.dev" --app ampel-api
```

Set secrets for the worker:

```bash
fly secrets set ENCRYPTION_KEY="$(fly secrets list --app ampel-api | grep ENCRYPTION_KEY)" --app ampel-worker

# Copy OAuth secrets if worker needs them
fly secrets set GITHUB_CLIENT_ID="your-client-id" --app ampel-worker
fly secrets set GITHUB_CLIENT_SECRET="your-client-secret" --app ampel-worker
# ... repeat for other providers
```

## Deployment

### Deploy All Services

```bash
# Deploy API
fly deploy --config fly.api.toml

# Deploy Worker
fly deploy --config fly.worker.toml

# Deploy Frontend
fly deploy --config fly.frontend.toml
```

### Deployment Script

Create `scripts/deploy-fly.sh`:

```bash
#!/bin/bash
set -e

echo "Deploying Ampel to Fly.io..."

echo "Deploying API..."
fly deploy --config fly.api.toml

echo "Deploying Worker..."
fly deploy --config fly.worker.toml

echo "Deploying Frontend..."
fly deploy --config fly.frontend.toml

echo "Deployment complete!"
echo "API: https://ampel-api.fly.dev"
echo "Frontend: https://ampel-frontend.fly.dev"
```

Make it executable:

```bash
chmod +x scripts/deploy-fly.sh
```

## Custom Domain Setup

### Add Custom Domain

```bash
# For frontend
fly certs add www.your-domain.com --app ampel-frontend
fly certs add your-domain.com --app ampel-frontend

# For API
fly certs add api.your-domain.com --app ampel-api
```

### DNS Configuration

Add these DNS records at your domain registrar:

| Type  | Name | Value                                         |
| ----- | ---- | --------------------------------------------- |
| CNAME | www  | ampel-frontend.fly.dev                        |
| CNAME | api  | ampel-api.fly.dev                             |
| A     | @    | (IP from `fly ips list --app ampel-frontend`) |

### Update CORS and OAuth

After setting up custom domains:

```bash
# Update CORS
fly secrets set CORS_ORIGINS="https://your-domain.com,https://www.your-domain.com" --app ampel-api

# Update OAuth redirect URIs
fly secrets set GITHUB_REDIRECT_URI="https://api.your-domain.com/api/oauth/github/callback" --app ampel-api
# ... repeat for other providers
```

## Monitoring

### View Logs

```bash
# API logs
fly logs --app ampel-api

# Worker logs
fly logs --app ampel-worker

# Frontend logs
fly logs --app ampel-frontend
```

### Check Status

```bash
fly status --app ampel-api
fly status --app ampel-worker
fly status --app ampel-frontend
```

### SSH into Machine

```bash
fly ssh console --app ampel-api
```

## Scaling

### Vertical Scaling (VM Size)

Edit the `[[vm]]` section in your `fly.*.toml`:

```toml
[[vm]]
  cpu_kind = "shared"
  cpus = 2
  memory_mb = 1024
```

Then redeploy.

### Horizontal Scaling (Replicas)

```bash
# Scale API to 3 instances
fly scale count 3 --app ampel-api

# Scale to specific regions
fly regions set iad lax --app ampel-api
```

## Database Management

### Connect to Database

```bash
fly postgres connect --app ampel-db
```

### Run Migrations

Migrations run automatically on API startup. For manual runs:

```bash
# SSH into API machine
fly ssh console --app ampel-api

# Run migrations
./ampel-api migrate
```

### Backup Database

```bash
fly postgres backup create --app ampel-db
```

## Troubleshooting

### Check Machine Health

```bash
fly checks list --app ampel-api
```

### Restart Machines

```bash
fly machines restart --app ampel-api
```

### View Recent Deploys

```bash
fly releases --app ampel-api
```

### Rollback

```bash
fly releases rollback --app ampel-api
```

## Cost Optimization

- Use `auto_stop_machines = true` for low-traffic apps
- Start with `shared` CPU for development
- Use `min_machines_running = 0` for non-critical services
- Monitor usage with `fly dashboard`

## Production Checklist

- [ ] All secrets configured
- [ ] Database properly sized
- [ ] Custom domain with SSL
- [ ] OAuth redirect URIs updated
- [ ] CORS origins configured
- [ ] Health checks passing
- [ ] Monitoring/alerting set up
- [ ] Backup strategy in place
