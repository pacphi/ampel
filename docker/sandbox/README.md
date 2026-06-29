# Ampel Remediation Sandbox (ADR-003)

Isolated, polyglot toolchain image in which the worker performs mechanical PR
consolidation (clone → sequential `git merge --no-ff` → lockfile regeneration).

## Build

```bash
./build.sh [tag]            # builds ghcr.io/ampel/remediation-sandbox:<tag>
AMPEL_SANDBOX_IMAGE=my/img ./build.sh v1
```

The worker selects the image and runtime via environment:

| Var | Default | Meaning |
|---|---|---|
| `AMPEL_SANDBOX_RUNTIME` | auto-detect (`podman` → `docker`) | Container runtime |
| `AMPEL_SANDBOX_IMAGE` | `ghcr.io/ampel/remediation-sandbox:latest` | OCI image (pin a digest in prod) |
| `AMPEL_CLONE_DEPTH` | `50` | `git clone --depth` |
| `AMPEL_SUBPROCESS_TIMEOUT_SECS` | `300` | Per-subprocess timeout |

## Security posture

- **Non-root**: runs as UID 1000; no build-time credentials.
- **Credential handling**: the decrypted PAT is injected at run time via a
  tmpfs-backed env-file (never an image layer, never a CLI argument, never
  logged). `GIT_TERMINAL_PROMPT=0` / `GIT_ASKPASS=/bin/echo` prevent interactive
  credential prompts. Subprocess output is scrubbed of the token before logging.
- **No force-push**: the toolchain and the worker code expose no force-push path.

## Egress allowlist

Run the container on a network that permits **only**:

1. **The configured Git provider host** — `github.com` / `api.github.com`, the
   GitLab instance host, or `bitbucket.org` / the Bitbucket Server host — for
   clone, push, and PR API calls.
2. **Package registries** required for lockfile regeneration:
   - npm / pnpm / yarn → `registry.npmjs.org`
   - Cargo → `crates.io`, `static.crates.io`, `index.crates.io`
   - Go modules → `proxy.golang.org`, `sum.golang.org`
   - Python / Poetry → `pypi.org`, `files.pythonhosted.org`
   - Ruby / Bundler → `rubygems.org`

Everything else should be denied. Air-gapped deployments (ADR-014) disable the
external-provider portion entirely; consolidation still runs but push/PR steps
are withheld.
