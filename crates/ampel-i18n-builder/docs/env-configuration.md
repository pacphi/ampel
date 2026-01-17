# Environment Variable Configuration

The `ampel-i18n-builder` CLI supports loading configuration from environment variables with a clear precedence order.

## Configuration Precedence

Configuration values are resolved in the following order (highest to lowest priority):

1. **System environment variables** (`export VAR=value`)
2. **`.env` file** (project root)
3. **`config.toml`** file
4. **Default values** (hardcoded in application)

This means:

- System environment variables always win
- `.env` file provides convenient defaults for development
- `config.toml` provides project-specific configuration
- Built-in defaults ensure the application always works

## .env File Setup

### 1. Create .env file

```bash
cd crates/ampel-i18n-builder
cp .env.example .env
```

### 2. Add your API keys

Edit `.env` and replace placeholder values:

```bash
# Required: At least one translation provider
SYSTRAN_API_KEY=your_actual_systran_key
DEEPL_API_KEY=your_actual_deepl_key
GOOGLE_API_KEY=your_actual_google_key
OPENAI_API_KEY=your_actual_openai_key

# Optional: Override defaults
AMPEL_I18N_TIMEOUT_SECS=45
AMPEL_I18N_BATCH_SIZE=50
```

### 3. Verify loading

Run the CLI with debug logging to see loaded configuration:

```bash
RUST_LOG=debug cargo run -- translate --help
```

## Supported Environment Variables

### Translation Provider API Keys

| Variable          | Purpose                      | Required     |
| ----------------- | ---------------------------- | ------------ |
| `SYSTRAN_API_KEY` | SYSTRAN translation service  | One of these |
| `DEEPL_API_KEY`   | DeepL translation service    | One of these |
| `GOOGLE_API_KEY`  | Google Cloud Translation     | One of these |
| `OPENAI_API_KEY`  | OpenAI GPT-based translation | One of these |

### Configuration Overrides

| Variable                  | Type   | Default | Description                                                    |
| ------------------------- | ------ | ------- | -------------------------------------------------------------- |
| `AMPEL_I18N_TIMEOUT_SECS` | u64    | 30      | HTTP request timeout in seconds                                |
| `AMPEL_I18N_BATCH_SIZE`   | usize  | 20      | Number of translations per batch                               |
| `AMPEL_I18N_MAX_RETRIES`  | u32    | 3       | Maximum retry attempts for failed requests                     |
| `REDIS_URL`               | String | -       | Redis cache connection string (if using `redis-cache` feature) |

### Logging

| Variable   | Values                          | Default | Description      |
| ---------- | ------------------------------- | ------- | ---------------- |
| `RUST_LOG` | trace, debug, info, warn, error | info    | Log level filter |

## Usage Examples

### Development with .env file

```bash
# .env file contains all keys
cargo run -- translate en --target-locale fr --provider deepl
```

### CI/CD with system environment

```bash
# GitHub Actions, GitLab CI, etc.
export DEEPL_API_KEY="${{ secrets.DEEPL_API_KEY }}"
cargo run -- translate en --target-locale fr --provider deepl
```

### Override .env with system env

```bash
# .env has DEEPL_API_KEY=dev_key
# Override with production key for one command
DEEPL_API_KEY=prod_key cargo run -- translate en --target-locale fr
```

### Mix .env and CLI arguments

```bash
# .env provides API keys
# CLI provides operation-specific config
cargo run -- translate en \
  --target-locale fr,de,es \
  --provider deepl \
  --batch-size 100 \
  --timeout 60
```

## Security Best Practices

### 1. Never commit .env files

The `.gitignore` file already excludes `.env`:

```gitignore
.env
.env.local
.env.*.local
```

### 2. Use different keys per environment

```bash
# Development
.env.development → SYSTRAN_API_KEY=dev_key_with_low_quota

# Production
.env.production → SYSTRAN_API_KEY=prod_key_with_high_quota
```

### 3. Rotate keys regularly

Update `.env` when rotating API keys:

```bash
# Old key expires 2024-01-01
DEEPL_API_KEY=old_key_expires_soon

# New key
DEEPL_API_KEY=new_key_valid_until_2025
```

### 4. Use CI/CD secrets

Never hardcode production keys in `.env`. Use secret management:

**GitHub Actions:**

```yaml
env:
  DEEPL_API_KEY: ${{ secrets.DEEPL_API_KEY }}
```

**GitLab CI:**

```yaml
variables:
  DEEPL_API_KEY: $DEEPL_API_KEY # From GitLab CI/CD Settings
```

## Troubleshooting

### .env file not loaded

**Problem:** API keys in `.env` not recognized

**Solution:**

1. Ensure `.env` is in the project root or current working directory
2. Check file permissions: `chmod 600 .env`
3. Run with debug logging: `RUST_LOG=debug cargo run`

### System env not overriding .env

**Problem:** System environment variable ignored

**Solution:**
This should never happen - system env always takes precedence. If it does:

1. Verify export: `echo $DEEPL_API_KEY`
2. Check for typos in variable name
3. Ensure no quotes around value: `export KEY=value` not `export KEY="value"`

### Wrong API key used

**Problem:** Application uses unexpected API key

**Check precedence:**

```bash
# 1. Check system env
echo $DEEPL_API_KEY

# 2. Check .env file
grep DEEPL_API_KEY .env

# 3. Run with debug logging
RUST_LOG=debug cargo run -- translate --help 2>&1 | grep -i "api"
```

## Example Workflows

### Local Development

```bash
# Setup
cp .env.example .env
# Edit .env with development API keys

# Use for all commands
cargo run -- translate en --target-locale fr
cargo run -- sync --locale-dir locales/
```

### CI/CD Pipeline

```bash
# GitHub Actions workflow
name: Translation Tests
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    env:
      DEEPL_API_KEY: ${{ secrets.DEEPL_API_KEY }}
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --features integration-tests
```

### Multi-Environment Setup

```bash
# Development environment
cp .env.development .env
cargo run -- translate en --target-locale fr

# Staging environment
cp .env.staging .env
cargo run -- translate en --target-locale fr,de,es

# Production (use system env only, no .env file)
export DEEPL_API_KEY=$PROD_KEY
cargo run -- translate en --target-locale fr,de,es,it,pt
```

## Testing .env Loading

Run this command to verify environment variables are loaded correctly:

```bash
# Create test .env
cat > .env << 'EOF'
TEST_VAR=from_dotenv
EOF

# System env takes precedence
TEST_VAR=from_system cargo run -- --help

# Should use .env value
cargo run -- --help

# Cleanup
rm .env
```

## References

- [dotenv-rs documentation](https://docs.rs/dotenv/)
- [12-factor app: Config](https://12factor.net/config)
- [OWASP: Secrets Management](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
