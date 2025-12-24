# Running Ampel with Docker

This guide covers running Ampel using Docker and Docker Compose.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) 20.10+
- [Docker Compose](https://docs.docker.com/compose/install/) v2+

## Quick Start

### 1. Clone and Configure

```bash
git clone https://github.com/pacphi/ampel.git
cd ampel

# Copy environment template
cp .env.example .env
```

### 2. Configure Environment

Edit `.env` with required values:

```bash
# Required: Generate secure keys
JWT_SECRET=$(openssl rand -hex 32)
ENCRYPTION_KEY=$(openssl rand -base64 32)

# Personal Access Tokens (PATs) are configured via UI after login
# See docs/PAT_SETUP.md for instructions
```

### 3. Start All Services

```bash
# Using Make (recommended)
make docker-up

# Or directly
cd docker
docker compose up -d
```

### 4. Access the Application

- **Frontend**: http://localhost:3000
- **API**: http://localhost:8080/api
- **API Docs**: http://localhost:8080/swagger-ui

> **Note**: The frontend container runs nginx on port 8080 internally (security requirement for non-root user), which Docker maps to localhost:3000. See [Port Configuration](#port-configuration) for details.

## Docker Compose Services

The `docker-compose.yml` includes:

| Service    | Port (Host:Container) | Description              |
| ---------- | --------------------- | ------------------------ |
| `postgres` | 5432:5432             | PostgreSQL database      |
| `redis`    | 6379:6379             | Redis (for job queues)   |
| `api`      | 8080:8080             | Ampel REST API           |
| `worker`   | -                     | Background job processor |
| `frontend` | 3000:8080             | React web application    |

### Port Configuration

The frontend service maps **host port 3000** to **container port 8080**. This configuration is required for security:

**Why port 8080 internally?**

- The frontend container runs nginx as a **non-root user** (`USER nginx` in Dockerfile.frontend:66)
- Non-root users cannot bind to privileged ports below 1024 (including port 80)
- Port 8080 is the industry standard for unprivileged nginx containers
- This aligns with Fly.io's default port expectations for production deployment

**References:**

- [Fly.io nginx port configuration](https://fly.io/docs/app-guides/global-nginx-proxy/) - Fly.io expects port 8080 by default
- [nginx-unprivileged official image](https://github.com/nginx/docker-nginx-unprivileged) - Uses port 8080 as standard
- [Running nginx as non-root](https://support.tools/nginx-root-non-root/) - Security best practices

## Common Operations

### Start Services

```bash
# Start all services
docker compose up -d

# Start specific service
docker compose up -d api

# Start with logs
docker compose up
```

### Stop Services

```bash
# Stop all services
docker compose down

# Stop and remove volumes (data loss!)
docker compose down -v
```

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f api

# Last 100 lines
docker compose logs --tail=100 api
```

### Rebuild Images

```bash
# Rebuild all images
docker compose build

# Rebuild specific image
docker compose build api

# Rebuild without cache
docker compose build --no-cache
```

### Execute Commands

```bash
# Shell into container
docker compose exec api sh

# Run one-off command
docker compose exec api /usr/local/bin/ampel-api --version
```

## Environment Configuration

### Required Variables

| Variable         | Description                    | Example                   |
| ---------------- | ------------------------------ | ------------------------- |
| `JWT_SECRET`     | JWT signing key (min 32 chars) | `openssl rand -hex 32`    |
| `ENCRYPTION_KEY` | Token encryption key           | `openssl rand -base64 32` |

### Optional Variables

| Variable       | Default | Description          |
| -------------- | ------- | -------------------- |
| `RUST_LOG`     | `info`  | Log level            |
| `CORS_ORIGINS` | -       | Allowed CORS origins |

**Note**: Personal Access Tokens (PATs) are managed per-user via the UI. See [PAT_SETUP.md](PAT_SETUP.md) for setup instructions.

## Production Configuration

### docker-compose.prod.yml

For production, create `docker-compose.prod.yml`:

```yaml
services:
  postgres:
    restart: always
    volumes:
      - /data/ampel/postgres:/var/lib/postgresql/data

  redis:
    restart: always
    volumes:
      - /data/ampel/redis:/data

  api:
    restart: always
    environment:
      RUST_LOG: info

  worker:
    restart: always
    environment:
      RUST_LOG: info

  frontend:
    restart: always
```

Run with:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Resource Limits

Add resource constraints in production:

```yaml
services:
  api:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M
```

## Health Checks

All services include health checks:

```bash
# Check service health
docker compose ps

# Check specific container
docker inspect ampel-api --format='{{.State.Health.Status}}'
```

## Database Management

### Connect to PostgreSQL

```bash
docker compose exec postgres psql -U ampel -d ampel
```

### Backup Database

```bash
docker compose exec postgres pg_dump -U ampel ampel > backup.sql
```

### Restore Database

```bash
cat backup.sql | docker compose exec -T postgres psql -U ampel -d ampel
```

## Running Individual Images

### API Only

```bash
docker run -d \
  --name ampel-api \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pass@host:5432/ampel \
  -e JWT_SECRET=your-secret \
  -e ENCRYPTION_KEY=your-key \
  ampel-api:latest
```

### Frontend Only

```bash
docker run -d \
  --name ampel-frontend \
  -p 3000:8080 \
  -e VITE_API_URL=http://api-host:8080/api \
  ampel-frontend:latest
```

**Note**: The frontend uses port 8080 internally (not 80) because it runs as a non-root user for security.

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker compose logs api

# Check container status
docker compose ps

# Inspect container
docker inspect ampel-api
```

### Database Connection Issues

```bash
# Verify postgres is healthy
docker compose exec postgres pg_isready -U ampel

# Check network connectivity
docker compose exec api ping postgres
```

### Port Conflicts

```bash
# Check what's using the port
lsof -i :8080

# Use different ports
docker compose up -d -e API_PORT=8081
```

### Disk Space

```bash
# Check Docker disk usage
docker system df

# Clean up unused resources
docker system prune -a
```

### Reset Everything

```bash
# Stop containers and remove volumes
docker compose down -v

# Remove images
docker compose down --rmi all

# Fresh start
docker compose up -d --build
```

## Updating

### Pull Latest Images

```bash
docker compose pull
docker compose up -d
```

### Rebuild from Source

```bash
git pull
docker compose build --no-cache
docker compose up -d
```

## Monitoring

### Basic Monitoring

```bash
# Watch container stats
docker stats

# Check resource usage
docker compose top
```

### Log Aggregation

For production, consider:

- Docker logging drivers (json-file, syslog, journald)
- Log aggregation tools (Loki, ELK Stack)
- Metrics with Prometheus

### Example Prometheus Setup

Add to `docker-compose.yml`:

```yaml
prometheus:
  image: prom/prometheus:latest
  volumes:
    - ./prometheus.yml:/etc/prometheus/prometheus.yml
  ports:
    - '9090:9090'
```

## Security Considerations

1. **Never commit `.env` files** - Use `.env.example` as template
2. **Use secrets management** - Consider Docker secrets for sensitive data
3. **Network isolation** - Only expose necessary ports
4. **Regular updates** - Keep base images updated
5. **Non-root users** - All Ampel containers run as non-root

## Quick Reference

```bash
# Using Make
make docker-up      # Start all services
make docker-down    # Stop all services
make docker-build   # Build images
make docker-logs    # View logs

# Or directly with docker compose
docker compose up -d      # Start
docker compose down       # Stop
docker compose logs -f    # Logs
docker compose build      # Rebuild
docker compose ps         # Status
docker compose exec api sh # Shell

# Clean up
docker system prune -a
```
