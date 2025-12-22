# Fly.io Secrets Template for Ampel

This document provides templates for setting secrets in Fly.io for all Ampel applications.

## Security Warning

**NEVER commit actual secrets to version control!**

This file contains templates only. Generate actual secrets using secure methods (see below).

---

## Secret Generation

Generate secure random secrets using OpenSSL:

```bash
# Generate 256-bit (32-byte) random keys
openssl rand -hex 32

# Generate 128-bit (16-byte) random keys
openssl rand -hex 16

# Generate base64-encoded keys
openssl rand -base64 32
```

---

## API Server Secrets

Set secrets for the API server (`ampel-api`):

```bash
# Database connection (get from `fly postgres db list --app ampel-db`)
fly secrets set \
  --app ampel-api \
  DATABASE_URL="postgres://postgres:<PASSWORD>@ampel-db.flycast:5432/ampel"

# Redis connection (get from `fly redis status --app ampel-redis`)
fly secrets set \
  --app ampel-api \
  REDIS_URL="redis://default:<PASSWORD>@ampel-redis.flycast:6379"

# JWT secret (generate with `openssl rand -hex 32`)
fly secrets set \
  --app ampel-api \
  JWT_SECRET="<RANDOM_256_BIT_KEY>"

# Encryption key for provider PAT tokens (generate with `openssl rand -base64 32`)
fly secrets set \
  --app ampel-api \
  ENCRYPTION_KEY="<RANDOM_256_BIT_KEY>"

# CORS origins (frontend URL)
fly secrets set \
  --app ampel-api \
  CORS_ORIGINS="https://ampel-frontend.fly.dev"
```

### Bulk Import from .env file

Create a `.env.production` file (DO NOT COMMIT):

```env
DATABASE_URL=postgres://postgres:<PASSWORD>@ampel-db.flycast:5432/ampel
REDIS_URL=redis://default:<PASSWORD>@ampel-redis.flycast:6379
JWT_SECRET=<RANDOM_256_BIT_KEY>
ENCRYPTION_KEY=<RANDOM_256_BIT_KEY>
CORS_ORIGINS=https://ampel-frontend.fly.dev
```

Then import:

```bash
fly secrets import --app ampel-api < .env.production
```

**Note**: Personal Access Tokens (PATs) are configured per-user via the UI after deployment. See [PAT_SETUP.md](../PAT_SETUP.md) for details.

---

## Worker Secrets

Set secrets for the background worker (`ampel-worker`):

```bash
# Database connection (same as API)
fly secrets set \
  --app ampel-worker \
  DATABASE_URL="postgres://postgres:<PASSWORD>@ampel-db.flycast:5432/ampel"

# Redis connection (same as API)
fly secrets set \
  --app ampel-worker \
  REDIS_URL="redis://default:<PASSWORD>@ampel-redis.flycast:6379"

# Encryption key (MUST BE SAME AS API for token decryption)
fly secrets set \
  --app ampel-worker \
  ENCRYPTION_KEY="<SAME_KEY_AS_API>"
```

---

## Frontend Secrets

Set secrets for the frontend (`ampel-frontend`):

```bash
# API endpoint URL
fly secrets set \
  --app ampel-frontend \
  VITE_API_URL="https://ampel-api.fly.dev"
```

Note: For Vite apps, build-time environment variables must be prefixed with `VITE_`.

---

## GitHub Actions Secret

Add the Fly.io deploy token to GitHub repository secrets:

1. Generate deploy token:

   ```bash
   fly tokens create deploy -x 999999h
   ```

2. Copy the entire output (including `FlyV1` prefix)

3. Go to GitHub repository → Settings → Secrets and variables → Actions

4. Create new repository secret:
   - Name: `FLY_API_TOKEN`
   - Value: `<PASTE_TOKEN_HERE>`

---

## Secret Rotation Schedule

Rotate secrets on the following schedule:

- **JWT_SECRET**: Every 90 days
- **ENCRYPTION_KEY**: Every 180 days (requires re-encrypting provider PAT tokens)
- **Database Password**: Every 180 days
- **Deploy Tokens**: Every 365 days

**Note**: Provider Personal Access Tokens (PATs) are managed by users through the UI and should be rotated according to each provider's security recommendations.

### Rotation Procedure

1. Generate new secret value
2. Set new secret: `fly secrets set --app <app> <KEY>="<NEW_VALUE>"`
3. Verify deployment: `fly status --app <app>`
4. Update documentation
5. Revoke old secret (if applicable)

---

## Verification

Verify secrets are set correctly:

```bash
# List secret names (values are encrypted)
fly secrets list --app ampel-api
fly secrets list --app ampel-worker
fly secrets list --app ampel-frontend

# Check app logs for secret injection
fly logs --app ampel-api

# Test database connection
fly ssh console --app ampel-api -C "psql \$DATABASE_URL -c 'SELECT 1;'"

# Test Redis connection
fly ssh console --app ampel-api -C "redis-cli -u \$REDIS_URL PING"
```

---

## Provider Personal Access Tokens (PATs)

Ampel uses Personal Access Tokens (PATs) instead of OAuth for provider authentication. This provides:

- **Simpler setup**: No OAuth application registration required
- **Better security**: Users control their own tokens
- **Flexibility**: Per-user token management via UI

### Post-Deployment User Setup

After deploying Ampel:

1. Users register/login to Ampel
2. Navigate to Settings → Provider Accounts
3. Add provider account with PAT token
4. Token is encrypted with `ENCRYPTION_KEY` and stored securely

### Creating Provider PATs

See [PAT_SETUP.md](../PAT_SETUP.md) for detailed instructions on creating PATs for:

- **GitHub**: `https://github.com/settings/tokens`
- **GitLab**: `https://gitlab.com/-/profile/personal_access_tokens`
- **Bitbucket**: `https://bitbucket.org/account/settings/app-passwords/`

**Required Scopes:**

- **GitHub**: `repo` (full access) or `public_repo` (public repos only)
- **GitLab**: `read_api`, `read_repository`
- **Bitbucket**: `repository:read`, `pullrequest:read`

---

## Troubleshooting

### Secret not applied after setting

Secrets require app restart. Fly.io automatically restarts apps when secrets are set.

Verify restart:

```bash
fly status --app ampel-api
fly logs --app ampel-api
```

### Database connection errors

Verify Flycast address:

```bash
fly postgres db list --app ampel-db
```

Ensure connection string uses `.flycast` domain, not `.fly.dev`.

### PAT encryption errors

Ensure `ENCRYPTION_KEY` is:

- Base64-encoded 32-byte key
- Same across API and Worker
- Generated with: `openssl rand -base64 32`

Test decoding:

```bash
echo "YOUR_KEY" | base64 -d | wc -c
# Should output: 32
```

---

## Security Best Practices

1. **Never commit secrets** - Use `.gitignore` for `.env.*` files
2. **Use strong random secrets** - Minimum 256-bit (32 bytes)
3. **Rotate regularly** - Follow rotation schedule
4. **Store in password manager** - Use 1Password, LastPass, or similar
5. **Limit access** - Only deploy keys in CI/CD, not developer machines
6. **Audit regularly** - Review `fly secrets list` monthly
7. **Monitor logs** - Watch for secret exposure in logs
8. **Use separate keys per environment** - Dev, staging, production

---

**Document Version**: 1.0
**Last Updated**: 2025-12-22
