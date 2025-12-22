# Ampel Fly.io Operations Runbook

**Version**: 1.0
**Last Updated**: 2025-12-22

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Deployment Procedures](#deployment-procedures)
3. [Scaling Operations](#scaling-operations)
4. [Monitoring and Health Checks](#monitoring-and-health-checks)
5. [Database Operations](#database-operations)
6. [Troubleshooting](#troubleshooting)
7. [Emergency Procedures](#emergency-procedures)

---

## Quick Reference

### Essential Commands

```bash
# Check app status
fly status --app ampel-api
fly status --app ampel-worker
fly status --app ampel-frontend

# View logs (real-time)
fly logs --app ampel-api -f

# Check health
fly checks list --app ampel-api

# SSH into machine
fly ssh console --app ampel-api

# Scale machines
fly scale count 2 --app ampel-api

# Restart app
fly apps restart ampel-api
```

### Application URLs

- **Frontend**: https://ampel-frontend.fly.dev
- **API**: https://ampel-api.fly.dev
- **API Health**: https://ampel-api.fly.dev/health
- **API Docs**: https://ampel-api.fly.dev/api/docs

### Dashboard Links

- **Fly.io Dashboard**: https://fly.io/dashboard
- **API App**: https://fly.io/apps/ampel-api
- **Worker App**: https://fly.io/apps/ampel-worker
- **Frontend App**: https://fly.io/apps/ampel-frontend
- **Database**: https://fly.io/apps/ampel-db
- **Redis**: https://fly.io/apps/ampel-redis

---

## Deployment Procedures

### Standard Deployment (via CI/CD)

1. Push code to `main` branch
2. GitHub Actions automatically runs tests
3. If tests pass, deploys to Fly.io
4. Monitor deployment:
   ```bash
   fly logs --app ampel-api -f
   ```

### Manual Deployment

```bash
# Deploy API
fly deploy --app ampel-api --config fly.api.toml --remote-only

# Deploy Worker
fly deploy --app ampel-worker --config fly.worker.toml --remote-only

# Deploy Frontend
fly deploy --app ampel-frontend --config fly.frontend.toml --remote-only
```

### Staged Deployment (with --stage flag)

```bash
# Deploy without restart
fly deploy --app ampel-api --stage

# Later, restart to apply
fly apps restart ampel-api
```

### Rollback Deployment

```bash
# List recent releases
fly releases --app ampel-api

# Rollback to previous version
fly releases rollback --app ampel-api --version <previous-version>

# Verify rollback
fly status --app ampel-api
fly logs --app ampel-api
```

### Database Migrations

```bash
# Run migrations after API deployment
fly ssh console --app ampel-api -C "/app/ampel-api migrate run"

# Check migration status
fly ssh console --app ampel-api -C "/app/ampel-api migrate status"

# Rollback migration (if needed)
fly ssh console --app ampel-api -C "/app/ampel-api migrate down"
```

---

## Scaling Operations

### Horizontal Scaling (Machine Count)

```bash
# Scale API to 3 machines
fly scale count 3 --app ampel-api

# Scale Worker to 2 machines
fly scale count 2 --app ampel-worker

# Scale Frontend to 2 machines
fly scale count 2 --app ampel-frontend

# Check current count
fly status --app ampel-api
```

### Vertical Scaling (Machine Size)

```bash
# Upgrade to performance-1x (1 dedicated vCPU, 2GB RAM)
fly scale vm performance-1x --app ampel-api

# Downgrade to shared-cpu-1x (1 shared vCPU, 256MB RAM)
fly scale vm shared-cpu-1x --app ampel-api

# Increase memory only
fly scale memory 512 --app ampel-api

# Check available VM sizes
fly platform vm-sizes
```

### Auto-scaling Configuration

Edit `fly.toml`:

```toml
[scaling]
  min_count = 1
  max_count = 5

[http_service]
  auto_stop_machines = true
  auto_start_machines = true
```

Then redeploy:

```bash
fly deploy --app ampel-api
```

### Regional Scaling

```bash
# Add a machine in Frankfurt
fly scale count 1 --region fra --app ampel-api

# List machines by region
fly machines list --app ampel-api

# Remove machine by ID
fly machine destroy <machine-id> --app ampel-api
```

---

## Monitoring and Health Checks

### Check Application Health

```bash
# Check all health checks
fly checks list --app ampel-api

# Watch health checks (every 10s)
watch -n 10 'fly checks list --app ampel-api'

# Test health endpoint manually
curl https://ampel-api.fly.dev/health
```

### View Logs

```bash
# Real-time logs
fly logs --app ampel-api -f

# Last 100 lines
fly logs --app ampel-api

# Filter by machine
fly logs --app ampel-api --machine <machine-id>

# Search logs
fly logs --app ampel-api | grep ERROR
```

### Monitor Resource Usage

```bash
# Check VM metrics
fly vm status --app ampel-api

# Monitor metrics (if configured)
curl https://ampel-api.fly.dev/metrics
```

### Database Monitoring

```bash
# Connect to database
fly postgres connect --app ampel-db

# Check database metrics
fly postgres db list --app ampel-db

# Check slow queries
fly ssh console --app ampel-db
# Then in psql:
# SELECT * FROM pg_stat_activity WHERE state = 'active';
```

### Redis Monitoring

```bash
# Connect to Redis
fly redis connect --app ampel-redis

# Check Redis info
fly ssh console --app ampel-api -C "redis-cli -u \$REDIS_URL INFO"

# Monitor Redis
fly ssh console --app ampel-api -C "redis-cli -u \$REDIS_URL MONITOR"
```

---

## Database Operations

### Backup Database

```bash
# List available backups (Managed Postgres has automatic backups)
fly postgres db list --app ampel-db

# Create manual backup
fly postgres db backup --app ampel-db

# Download backup
fly ssh console --app ampel-db -C "pg_dump ampel" > ampel_backup_$(date +%Y%m%d).sql
```

### Restore Database

```bash
# Restore from backup
fly postgres db restore --app ampel-db --backup <backup-id>

# Restore from local file
fly ssh console --app ampel-db < ampel_backup_20251222.sql
```

### Run SQL Queries

```bash
# Connect to database
fly postgres connect --app ampel-db

# Run single query
fly ssh console --app ampel-db -C "psql ampel -c 'SELECT COUNT(*) FROM users;'"

# Execute SQL file
fly ssh console --app ampel-db < migration.sql
```

### Database Maintenance

```bash
# Connect to database
fly postgres connect --app ampel-db

# Vacuum database
VACUUM ANALYZE;

# Reindex
REINDEX DATABASE ampel;

# Check database size
SELECT pg_size_pretty(pg_database_size('ampel'));

# Check table sizes
SELECT
  schemaname || '.' || tablename AS table,
  pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS size
FROM pg_tables
ORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC
LIMIT 10;
```

---

## Troubleshooting

### App Won't Start

**Symptoms**: App shows as unhealthy, logs show startup errors

**Steps**:

1. Check logs for errors:

   ```bash
   fly logs --app ampel-api
   ```

2. Verify environment variables:

   ```bash
   fly secrets list --app ampel-api
   ```

3. Test locally with same environment:

   ```bash
   docker build -f Dockerfile.api -t ampel-api .
   docker run -e DATABASE_URL=... ampel-api
   ```

4. SSH into machine and debug:
   ```bash
   fly ssh console --app ampel-api
   /app/ampel-api --version
   ```

### Database Connection Errors

**Symptoms**: Logs show "connection refused" or "timeout"

**Steps**:

1. Verify DATABASE_URL is set:

   ```bash
   fly secrets list --app ampel-api | grep DATABASE
   ```

2. Check database is running:

   ```bash
   fly status --app ampel-db
   ```

3. Test connection from API machine:

   ```bash
   fly ssh console --app ampel-api -C "psql \$DATABASE_URL -c 'SELECT 1;'"
   ```

4. Verify Flycast address:
   ```bash
   fly postgres db list --app ampel-db
   # Should show .flycast address
   ```

### High Memory Usage

**Symptoms**: App crashes with OOM errors

**Steps**:

1. Check current memory usage:

   ```bash
   fly vm status --app ampel-api
   ```

2. Review logs for memory leaks:

   ```bash
   fly logs --app ampel-api | grep -i "out of memory"
   ```

3. Scale up memory:

   ```bash
   fly scale memory 512 --app ampel-api
   ```

4. Profile application locally for memory leaks

### Slow Response Times

**Symptoms**: API responds slowly, timeouts

**Steps**:

1. Check database query performance:

   ```bash
   fly postgres connect --app ampel-db
   # Run EXPLAIN ANALYZE on slow queries
   ```

2. Check Redis hit rate:

   ```bash
   fly redis connect --app ampel-redis
   INFO stats
   ```

3. Review API logs for slow endpoints:

   ```bash
   fly logs --app ampel-api | grep -i "slow"
   ```

4. Consider scaling:
   ```bash
   fly scale count 2 --app ampel-api
   ```

### Health Checks Failing

**Symptoms**: Machines marked unhealthy, traffic not routed

**Steps**:

1. Check health endpoint manually:

   ```bash
   curl -v https://ampel-api.fly.dev/health
   ```

2. Review health check configuration in `fly.toml`:

   ```toml
   [[http_service.checks]]
     grace_period = "10s"  # Increase if app is slow to start
     interval = "30s"
     timeout = "5s"        # Increase if endpoint is slow
   ```

3. Check logs during health check failures:

   ```bash
   fly logs --app ampel-api -f
   ```

4. Test health endpoint from within machine:
   ```bash
   fly ssh console --app ampel-api -C "curl http://localhost:8080/health"
   ```

---

## Emergency Procedures

### Complete System Down

**Severity**: Critical

**Steps**:

1. Check all app statuses:

   ```bash
   fly status --app ampel-api
   fly status --app ampel-worker
   fly status --app ampel-frontend
   fly status --app ampel-db
   ```

2. Review logs for all apps:

   ```bash
   fly logs --app ampel-api
   fly logs --app ampel-worker
   fly logs --app ampel-frontend
   ```

3. Restart all apps:

   ```bash
   fly apps restart ampel-api
   fly apps restart ampel-worker
   fly apps restart ampel-frontend
   ```

4. If database is down, contact Fly.io support:

   ```bash
   fly postgres db list --app ampel-db
   # If down, create support ticket
   ```

5. If restart fails, rollback to last known good version:
   ```bash
   fly releases rollback --app ampel-api
   ```

### Database Corruption

**Severity**: Critical

**Steps**:

1. Stop all database writes:

   ```bash
   fly scale count 0 --app ampel-api
   fly scale count 0 --app ampel-worker
   ```

2. Restore from latest backup:

   ```bash
   fly postgres db list --app ampel-db
   fly postgres db restore --app ampel-db --backup <latest-backup-id>
   ```

3. Verify data integrity:

   ```bash
   fly postgres connect --app ampel-db
   # Run validation queries
   ```

4. Restart apps:
   ```bash
   fly scale count 1 --app ampel-api
   fly scale count 1 --app ampel-worker
   ```

### Security Incident

**Severity**: Critical

**Steps**:

1. Rotate all secrets immediately:

   ```bash
   # Generate new secrets
   NEW_JWT=$(openssl rand -hex 32)
   NEW_ENCRYPTION=$(openssl rand -hex 32)

   # Set new secrets
   fly secrets set --app ampel-api JWT_SECRET="$NEW_JWT"
   fly secrets set --app ampel-api ENCRYPTION_KEY="$NEW_ENCRYPTION"
   fly secrets set --app ampel-worker ENCRYPTION_KEY="$NEW_ENCRYPTION"
   ```

2. Review access logs:

   ```bash
   fly logs --app ampel-api | grep -E "(POST|PUT|DELETE)"
   ```

3. Check database for suspicious activity:

   ```bash
   fly postgres connect --app ampel-db
   # Review audit tables
   ```

4. Notify users to rotate their Personal Access Tokens (PATs) in provider settings

5. Force logout all users (clear sessions in Redis):

   ```bash
   fly redis connect --app ampel-redis
   FLUSHDB
   ```

6. Document incident and notify stakeholders

### Performance Degradation

**Severity**: High

**Steps**:

1. Scale up immediately:

   ```bash
   fly scale count 3 --app ampel-api
   fly scale vm performance-1x --app ampel-api
   ```

2. Identify bottleneck:

   ```bash
   # Check database
   fly postgres connect --app ampel-db
   SELECT * FROM pg_stat_activity;

   # Check Redis
   fly redis connect --app ampel-redis
   INFO stats

   # Check API logs
   fly logs --app ampel-api | grep -i "slow\|timeout\|error"
   ```

3. Apply immediate fixes:
   - Add database indexes for slow queries
   - Increase cache TTL
   - Enable rate limiting

4. Monitor improvement:
   ```bash
   watch -n 5 'fly checks list --app ampel-api'
   ```

---

## Contact Information

### On-Call Rotation

- **Primary**: [Team Lead Name] - [Phone/Email]
- **Secondary**: [Senior Engineer Name] - [Phone/Email]
- **Escalation**: [CTO/VP Engineering] - [Phone/Email]

### External Support

- **Fly.io Support**: https://fly.io/docs/about/support/
- **Fly.io Community**: https://community.fly.io/
- **Emergency Contact**: Email support@fly.io with "URGENT" in subject

---

**Runbook Version**: 1.0
**Next Review Date**: 2025-03-22
