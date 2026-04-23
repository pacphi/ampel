# Release Process

Releases are fully automated by [release-please](https://github.com/googleapis/release-please). Maintainers do not bump versions, edit `CHANGELOG.md`, or create tags by hand.

## Versioning

Ampel follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.0.1): Bug fixes, backwards compatible

While Ampel is pre-1.0 (`0.x.y`), `feat:` commits bump the minor version and `fix:` commits bump the patch. Breaking changes (`feat!:` / `BREAKING CHANGE:`) also bump the minor until 1.0.

All three version-bearing files are kept in sync automatically:

- `Cargo.toml` — `[workspace.package] version`
- `crates/ampel-i18n-builder/Cargo.toml` — `[package] version`
- `frontend/package.json` — `version`

## How a Release Happens

### 1. Merge PRs to `main`

Use [Conventional Commit](https://www.conventionalcommits.org/) prefixes on PR titles (squash-merge titles become the commit on `main` and drive the changelog):

| Prefix      | Changelog section      | Version impact (pre-1.0) |
| ----------- | ---------------------- | ------------------------ |
| `feat:`     | Features               | minor                    |
| `fix:`      | Bug Fixes              | patch                    |
| `perf:`     | Performance            | patch                    |
| `docs:`     | Documentation          | none                     |
| `refactor:` | Code Refactoring       | none                     |
| `test:`     | Tests                  | none                     |
| `build:`    | Build System           | none                     |
| `ci:`       | Continuous Integration | none                     |
| `chore:`    | Miscellaneous Chores   | none                     |

Append `!` (e.g. `feat!:`) or include `BREAKING CHANGE:` in the body to flag a breaking change.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full convention.

### 2. Review the Release PR

After each merge to `main`, the `Release` workflow runs. release-please opens (or updates) a PR titled `chore(main): release X.Y.Z` containing:

- Version bumps in all three files listed above
- A new section prepended to `CHANGELOG.md`
- An updated `.release-please-manifest.json`

Review it like any other PR. The proposed version comes from the Conventional Commit types accumulated since the last release.

### 3. Merge the Release PR

Merging triggers release-please to:

1. Push the tag `vX.Y.Z` to `main`
2. Create a GitHub Release with the changelog section as the body

The same `Release` workflow run then executes the build/publish jobs, gated on `release_created`:

- **Docker Images** pushed to GHCR:
  - `ghcr.io/pacphi/ampel-api:vX.Y.Z`
  - `ghcr.io/pacphi/ampel-worker:vX.Y.Z`
  - `ghcr.io/pacphi/ampel-frontend:vX.Y.Z`
- **Binaries** attached to the GitHub Release:
  - Linux x86_64, macOS x86_64, macOS ARM64
- **Crate**: `ampel-i18n-builder` published to crates.io

## Pre-releases

To cut an alpha/beta/rc, include a `Release-As:` footer in a commit on `main`:

```text
chore: cut next beta

Release-As: 0.4.0-beta.1
```

release-please will propose that exact version in the next Release PR. Pre-releases are marked accordingly on GitHub.

## Hotfixes

1. Open a PR against `main` with a `fix:` commit.
2. Merge it. release-please will include it in the next Release PR as a patch bump.
3. If a hotfix must ship without unrelated in-flight work, either:
   - Merge only the `fix:` PR, then merge the Release PR immediately, or
   - Use a `Release-As: X.Y.Z` footer to force a specific patch version.

## Rollback

1. Open a PR reverting the offending commit(s) with a `fix:` or `revert:` title.
2. Merge it. release-please will open a Release PR bumping the patch version.
3. Merge the Release PR to ship the rollback.

For immediate operational rollback, redeploy the previous image tag:

```bash
# Docker Compose
docker pull ghcr.io/pacphi/ampel-api:vX.Y.(Z-1)
docker compose up -d

# Fly.io
fly deploy --image ghcr.io/pacphi/ampel-api:vX.Y.(Z-1)
```

## Required Secrets

- `GITHUB_TOKEN` — provided automatically by GitHub Actions.
- `CARGO_REGISTRY_TOKEN` — repository secret used to publish `ampel-i18n-builder` to crates.io.

## Configuration

- `.github/workflows/release.yml` — single workflow driving everything.
- `release-please-config.json` — release-please monorepo config with linked versions.
- `.release-please-manifest.json` — current version of each tracked package.
- `CHANGELOG.md` — auto-maintained; do not edit by hand.

## CLI Walkthrough

A concrete, terminal-only workflow using `git` and `gh`. Assumes `gh auth status` is green and the repo default is set (`gh repo set-default pacphi/ampel`).

### Sanity checks (one-time)

```bash
# Squash-merge must use PR title (release-please relies on this)
gh api repos/pacphi/ampel --jq '{title: .squash_merge_commit_title, msg: .squash_merge_commit_message}'
# Want: title=PR_TITLE or COMMIT_OR_PR_TITLE; msg=PR_BODY or COMMIT_MESSAGES
```

### Daily: merging work into `main`

```bash
git checkout -b feat/new-thing
# ...hack...
git commit -m "feat(api): add /metrics endpoint"
git push -u origin feat/new-thing

gh pr create --fill
# After review:
gh pr merge --squash --delete-branch
```

The squash-merge PR title is what release-please reads — it drives both the version bump and the changelog entry.

### After merging to `main`: the rolling Release PR

Each merge to `main` runs the `Release` workflow. The `release-please` job opens or updates a single rolling Release PR titled `chore(main): release X.Y.Z`:

```bash
# Watch the workflow run
gh run watch

# Find the Release PR
gh pr list --label "autorelease: pending"

RPR=$(gh pr list --label "autorelease: pending" --json number -q '.[0].number')
gh pr view $RPR
gh pr diff $RPR
```

The Release PR accumulates every subsequent merge until you decide to ship.

**Do not hand-edit the Release PR** — release-please force-pushes its branch on every run and will overwrite your changes. To force a regeneration:

```bash
gh pr close $RPR
git commit --allow-empty -m "chore: recompute release"
git push origin main
```

### Cutting the release

Merging the Release PR (squash) is the single action that ships the release:

```bash
RPR=$(gh pr list --label "autorelease: pending" --json number -q '.[0].number')
gh pr merge $RPR --squash

# Watch the full pipeline: tag creation, Docker build/push, binaries, crate publish
gh run watch
```

### Verifying the release

```bash
git fetch --tags
git tag --sort=-v:refname | head -3

gh release view vX.Y.Z
gh release view vX.Y.Z --json assets --jq '.assets[].name'

# Docker images
gh api /users/pacphi/packages/container/ampel-api/versions --jq '.[0].metadata.container.tags'

# crates.io (may lag ~1 min while indexing)
curl -s https://crates.io/api/v1/crates/ampel-i18n-builder | jq '.crate.max_version'

# Version files on main
git pull
grep '^version' Cargo.toml | head -1
grep '"version"' frontend/package.json | head -1
```

### Forcing a specific version

Push an empty commit with a `Release-As:` footer. The next workflow run updates the Release PR to use that exact version.

```bash
git commit --allow-empty -m "chore: cut beta" -m "Release-As: 0.4.0-beta.1"
git push origin main
```

### Re-running a failed release pipeline

The tag is cut by the `release-please` job; if a downstream job (Docker, binaries, crate) fails transiently, just re-run the failed jobs:

```bash
gh run list --workflow=release.yml --limit 5
gh run rerun <run-id> --failed
gh run watch
```

### Manual trigger

The workflow also responds to `workflow_dispatch`:

```bash
gh workflow run release.yml --ref main
gh run watch
```

### Mental model

- You never run `git tag`, edit version files, or edit `CHANGELOG.md` by hand.
- You only merge two kinds of things to `main`: normal PRs (with conventional titles) and the Release PR (when you want to ship).
- The workflow is idempotent — re-pushing just recomputes the Release PR; re-running on a tagged commit is a no-op.
