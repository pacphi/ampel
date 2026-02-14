# Release Process

This document describes how to create releases for Ampel using Git and GitHub automation.

## Versioning

Ampel follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.0.1): Bug fixes, backwards compatible

## Release Workflow

### 1. Prepare the Release

```bash
# Ensure you're on main with latest changes
git checkout main
git pull origin main

# Create a release branch
git checkout -b release/v0.1.0
```

### 2. Update Version Numbers

Three files must be kept in sync:

- `Cargo.toml` — `[workspace.package] version` (used by all workspace crates)
- `crates/ampel-i18n-builder/Cargo.toml` — `[package] version` (standalone crate, not in workspace)
- `frontend/package.json` — `version`

Use the bump script to update all three at once:

```bash
# Explicit version
./scripts/bump-version.sh 0.4.0

# Or use symbolic bump types
./scripts/bump-version.sh patch   # 0.3.0 -> 0.3.1
./scripts/bump-version.sh minor   # 0.3.0 -> 0.4.0
./scripts/bump-version.sh major   # 0.3.0 -> 1.0.0
```

You can also edit the files manually if you prefer.

### 3. Update Changelog

Create or update `CHANGELOG.md`:

```markdown
# Changelog

## [0.1.0] - 2025-01-15

### Added

- Initial release
- GitHub, GitLab, Bitbucket PAT authentication
- PR dashboard with traffic light status
- Repository health scoring
- Team management

### Fixed

- N/A

### Changed

- N/A
```

### 4. Create Release Commit

```bash
git add -A
git commit -m "chore: prepare release v0.1.0"
git push origin release/v0.1.0
```

### 5. Create Pull Request

Open a PR from `release/v0.1.0` to `main` for final review.

### 6. Tag and Release

After merging to main:

```bash
git checkout main
git pull origin main

# Create annotated tag
git tag -a v0.1.0 -m "Release v0.1.0"

# Push tag
git push origin v0.1.0
```

## GitHub Release Automation

When you push a tag, the GitHub Actions workflow automatically:

1. Builds release binaries for multiple platforms
2. Builds and pushes Docker images
3. Creates a GitHub Release with artifacts

### Release Workflow (`.github/workflows/release.yml`)

The release workflow is triggered on version tags:

```yaml
on:
  push:
    tags:
      - 'v*'
```

### Release Artifacts

Each release includes:

- **Docker Images**: Published to GitHub Container Registry
  - `ghcr.io/pacphi/ampel-api:v0.1.0`
  - `ghcr.io/pacphi/ampel-worker:v0.1.0`
  - `ghcr.io/pacphi/ampel-frontend:v0.1.0`

- **Binary Releases** (optional): Pre-compiled binaries for:
  - Linux x86_64
  - macOS x86_64
  - macOS ARM64

### Configuring Container Registry

1. Create a GitHub Personal Access Token with `write:packages` scope
2. Add as repository secret: `GHCR_TOKEN`
3. The workflow will automatically authenticate and push images

## Hotfix Process

For urgent fixes to production:

```bash
# Create hotfix branch from the release tag
git checkout v0.1.0
git checkout -b hotfix/v0.1.1

# Make fixes
# ... edit files ...

# Commit and push
git add -A
git commit -m "fix: critical security patch"
git push origin hotfix/v0.1.1

# After PR approval and merge
git checkout main
git pull origin main
git tag -a v0.1.1 -m "Hotfix v0.1.1"
git push origin v0.1.1
```

## Pre-release Versions

For beta or release candidates:

```bash
# Beta release
git tag -a v0.2.0-beta.1 -m "Beta release v0.2.0-beta.1"

# Release candidate
git tag -a v0.2.0-rc.1 -m "Release candidate v0.2.0-rc.1"
```

Pre-release tags trigger the same workflow but are marked as pre-release in GitHub.

## Rollback Process

If a release has critical issues:

### Immediate Rollback

```bash
# Revert to previous Docker image
docker pull ghcr.io/pacphi/ampel-api:v0.0.9
docker compose up -d

# Or on Fly.io
fly deploy --image ghcr.io/pacphi/ampel-api:v0.0.9
```

### Git Revert

```bash
# Create revert commit
git revert HEAD
git push origin main

# Create patch release
git tag -a v0.1.1 -m "Revert v0.1.0 changes"
git push origin v0.1.1
```

## Release Checklist

Before releasing:

- [ ] All CI checks pass on main
- [ ] Version numbers updated (run `./scripts/bump-version.sh` or manually update all 3 files)
- [ ] Changelog updated
- [ ] Documentation updated
- [ ] Database migrations tested
- [ ] Manual QA completed
- [ ] Release notes drafted

After releasing:

- [ ] Verify GitHub Release created
- [ ] Verify Docker images published
- [ ] Deploy to staging environment
- [ ] Run smoke tests
- [ ] Deploy to production
- [ ] Announce release (if applicable)
