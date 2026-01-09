# CI/CD Setup for i18n Validation

This document explains the CI/CD pipeline for translation validation in the Ampel project.

## Overview

The i18n validation system ensures translation quality and coverage across all 20 supported languages through:

1. **GitHub Actions Workflow** - Automated validation on PR and push
2. **Pre-commit Hooks** - Local validation before commits
3. **Coverage Reporting** - Detailed coverage metrics and PR comments

## Quick Start

### Install Git Hooks

```bash
# Install pre-commit and commit-msg hooks
./scripts/install-git-hooks.sh
```

This installs:

- **pre-commit**: Validates translation files before commit (<5s)
- **commit-msg**: Enforces i18n commit message conventions

### Manual Validation

```bash
# Validate all translations
./scripts/i18n-validate.sh --all

# Validate backend only
./scripts/i18n-validate.sh --backend

# Validate frontend only
./scripts/i18n-validate.sh --frontend

# Auto-fix issues where possible
./scripts/i18n-validate.sh --all --fix
```

### Generate Coverage Report

```bash
# Text format (default)
node frontend/scripts/i18n-coverage-report.js

# JSON format
node frontend/scripts/i18n-coverage-report.js --format json

# Markdown format
node frontend/scripts/i18n-coverage-report.js --format markdown

# Check coverage threshold
node frontend/scripts/i18n-coverage-report.js --check --min-coverage 95
```

## GitHub Actions Workflow

The workflow runs on:

- Pull requests that modify translation files
- Pushes to the main branch

### Workflow Jobs

#### 1. validate-backend

Validates Rust translations in `crates/ampel-api/locales/`

**Steps:**

- Build ampel-i18n-builder
- Check coverage (≥95%)
- Validate YAML schema
- Check for missing translations
- Generate coverage report

**Artifacts:**

- `backend-coverage.json` - JSON coverage data
- `backend-coverage.md` - Markdown report

#### 2. validate-frontend

Validates React translations in `frontend/public/locales/`

**Steps:**

- Install dependencies (pnpm)
- Check coverage (≥95%)
- Validate JSON schema
- Check for missing keys
- Generate TypeScript types
- Verify types are up-to-date

**Artifacts:**

- `frontend-coverage.json` - JSON coverage data
- `frontend-coverage.md` - Markdown report

#### 3. test-rtl

Tests RTL (Right-to-Left) language support

**Languages:** Arabic (ar), Hebrew (he)

**Steps:**

- Run Playwright visual regression tests
- Upload visual diffs on failure

#### 4. test-complex-scripts

Tests complex script rendering

**Languages:** Arabic (ar), Thai (th), Korean (ko)

**Steps:**

- Run rendering tests for each language
- Verify proper text shaping and layout

#### 5. test-pluralization

Tests complex pluralization rules

**Languages:** Finnish (fi), Czech (cs), Russian (ru), Polish (pl)

**Steps:**

- Run pluralization tests
- Verify correct plural forms

#### 6. coverage-report

Generates combined coverage report and posts to PR

**Steps:**

- Download backend and frontend coverage
- Calculate overall coverage
- Post formatted report as PR comment
- Fail if coverage below threshold

#### 7. translation-api

Tests DeepL API integration (main branch only)

**Steps:**

- Run dry-run translation update
- Create automated PR if translations updated

## Coverage Thresholds

| Component | Threshold | Status      |
| --------- | --------- | ----------- |
| Backend   | 95%       | ✅ Required |
| Frontend  | 95%       | ✅ Required |
| Overall   | 95%       | ✅ Required |

**Coverage calculation:**

```
Coverage = (Translated Keys / Total Keys) × 100
```

**Exclusions:**

- Empty values (null, "", "TODO", "[TODO ...]")
- Missing keys
- Commented-out translations

## Pre-commit Hook

Fast local validation (<5 seconds) that runs before each commit.

### What it checks:

1. **Changed files detection**
   - Only validates if i18n files changed
   - Skips validation for unrelated commits

2. **Backend validation**
   - Coverage check (≥95%)
   - YAML syntax validation
   - Missing translations

3. **Frontend validation**
   - JSON syntax validation
   - Coverage check (≥95%)
   - Missing keys detection

### Bypass hook (emergency only)

```bash
git commit --no-verify
```

**⚠️ Warning:** Only bypass for urgent fixes. CI will still validate on PR.

## Commit Message Conventions

For i18n-related commits:

```
feat(i18n): add Japanese translations
fix(i18n): correct Arabic RTL layout
chore(i18n): update translation coverage
docs(i18n): improve localization guide
```

The commit-msg hook enforces:

- i18n commits only modify i18n files
- Proper commit message format

## Coverage Report Format

### Text Format

```
============================================================
Translation Coverage Report
============================================================
Generated: 2025-12-27T07:30:00.000Z
Base Locale: en
Total Keys: 150
Overall Coverage: 96.5%

Coverage by Language:
------------------------------------------------------------
✅ en     100.0% (150/150)
✅ es      98.0% (147/150)
   Missing: 3 keys
❌ fr      92.0% (138/150)
   Missing: 12 keys
============================================================
```

### JSON Format

```json
{
  "baseLocale": "en",
  "totalKeys": 150,
  "overallCoverage": 96.5,
  "locales": {
    "en": {
      "coverage": 100,
      "translatedKeys": 150,
      "missingKeys": [],
      "emptyKeys": []
    },
    "es": {
      "coverage": 98,
      "translatedKeys": 147,
      "missingKeys": ["key1", "key2", "key3"],
      "emptyKeys": []
    }
  },
  "generatedAt": "2025-12-27T07:30:00.000Z"
}
```

### Markdown Format

```markdown
# Translation Coverage Report

**Generated:** 2025-12-27T07:30:00.000Z
**Base Locale:** en
**Total Keys:** 150
**Overall Coverage:** 96.5%

## Coverage by Language

| Language | Coverage | Translated | Missing | Empty | Status |
| -------- | -------- | ---------- | ------- | ----- | ------ |
| en       | 100.0%   | 150        | 0       | 0     | ✅     |
| es       | 98.0%    | 147        | 3       | 0     | ✅     |
| fr       | 92.0%    | 138        | 12      | 0     | ❌     |
```

## CI Metrics Storage

Metrics are stored in memory namespace `aqe/swarm/ci-metrics`:

```javascript
{
  "lastRun": "2025-12-27T07:30:00.000Z",
  "backend": {
    "coverage": 96.5,
    "translatedKeys": 145,
    "totalKeys": 150
  },
  "frontend": {
    "coverage": 97.2,
    "translatedKeys": 146,
    "totalKeys": 150
  },
  "overall": {
    "coverage": 96.85,
    "passedThreshold": true,
    "threshold": 95
  },
  "languages": {
    "en": 100,
    "es": 98,
    "fr": 92
  }
}
```

## Troubleshooting

### Hook not executing

```bash
# Reinstall hooks
./scripts/install-git-hooks.sh

# Verify hook is executable
ls -la .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Coverage check timeout

The pre-commit hook has a 5-second timeout per check. If it times out:

1. Ensure dependencies are installed
2. Check for large translation files
3. Use `--no-verify` for urgent commits (CI will validate)

### YAML validation errors

```bash
# Install yamllint
pip install yamllint

# Check specific file
yamllint -c .yamllint.yml crates/ampel-api/locales/en/common.yml

# Auto-fix formatting
yamllint -c .yamllint.yml --format auto crates/ampel-api/locales/**/*.yml
```

### JSON validation errors

```bash
# Validate specific file
node -e "JSON.parse(require('fs').readFileSync('frontend/public/locales/en/common.json', 'utf8'))"

# Auto-format with jq
jq '.' frontend/public/locales/en/common.json > temp.json && mv temp.json frontend/public/locales/en/common.json
```

## Performance Optimization

### Pre-commit Hook

- Skips validation if no i18n files changed
- Only validates changed translation files
- 5-second timeout prevents blocking commits

### GitHub Actions

- Caches Rust and pnpm dependencies
- Runs validation jobs in parallel
- Only runs on relevant file changes

### Coverage Report

- Uses efficient key traversal algorithm
- Caches results for repeated runs
- Generates multiple formats in single pass

## Security Considerations

### Secrets Management

- DeepL API key stored in GitHub Secrets
- Never commit API keys to repository
- Automated translation PRs require human review

### Access Control

- Only maintainers can approve automated translation PRs
- PR comments require write permissions
- Coverage reports don't expose sensitive data

## Future Improvements

1. **Performance**
   - Add incremental coverage checks (only changed files)
   - Parallel validation for multiple languages
   - Caching of validation results

2. **Features**
   - Auto-fix for common translation issues
   - Machine translation quality scoring
   - Translation memory integration

3. **Reporting**
   - Historical coverage trends
   - Per-language quality metrics
   - Translation velocity tracking

## Related Documentation

- [IMPLEMENTATION_ROADMAP_V2.md](IMPLEMENTATION_ROADMAP_V2.md) - Full implementation plan
- [LOCALIZATION_IMPLEMENTATION_PLAN.md](LOCALIZATION_IMPLEMENTATION_PLAN.md) - Detailed localization guide
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [SPECIFICATION.md](SPECIFICATION.md) - Feature specifications

## Support

For issues or questions:

1. Check [Troubleshooting](#troubleshooting) section
2. Review GitHub Actions logs
3. Run local validation with `--fix` flag
4. Open an issue with validation output
