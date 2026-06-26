# ADR-006: Remediation Playbook Format and Storage

**Status**: Accepted
**Date**: 2026-06-24
**Deciders**: Architecture Team
**Technical Story**: The Remediation Agent Harness (Phase 4) externalises prompts and loop logic into versioned Playbooks that drive autonomous triage, consolidation, and remediation of open PRs across GitHub, GitLab, and Bitbucket.

---

## Context

### Problem Statement

The Fleet PR Remediation Loops feature introduces an agent harness that must execute multi-step remediation workflows autonomously. Each workflow is governed by a Playbook — a structured bundle that specifies the agent's role prompt, task templates keyed by failure class, context assembly rules, output contract (tool_use vs unified_diff), sandbox tools policy, loop configuration (max iterations, completion condition, reflexion hooks, token/time budgets), and optional per-model-provider overlays.

Hard-coding Playbooks in Rust source has two critical drawbacks: every prompt or policy change requires a binary rebuild and redeployment, and there is no mechanism for org or team administrators to tailor behaviour without forking the codebase. The remediation harness therefore needs an externalisation strategy that keeps fleet-wide defaults in the binary for zero-config deployments, allows privileged users to override them at the org and team scope without a deployment, and permits engineering teams to commit repo-local Playbooks alongside code for GitOps workflows and A/B testing.

Beyond editability, the format must be safe to load from untrusted sources. Repo-local files arrive from third-party repositories inside the sandbox. The system must prevent a malicious Playbook from escalating the sandbox's tool-use permissions beyond what the org administrator has authorised at deployment time.

Finally, the system must be testable in isolation. The GET `.../preview` endpoint must render a fully assembled prompt (with minijinja variables substituted) without invoking any model, so that prompt engineers can iterate quickly and CI pipelines can regression-test prompt output.

### Technical Context

- **Runtime**: Rust 1.95 + Tokio; worker binary (`ampel-worker`) runs Apalis background jobs.
- **Embedding**: `rust-embed` (`RustEmbed` derive macro) supports directory embedding; assets are included at compile time and accessible via `Asset::get("filename")` at runtime.
- **Templating**: `minijinja` (Jinja2-compatible, non-HTML, maintained by Armin Ronacher) is already selected for Playbook variable rendering (ADR-005).
- **Database**: SeaORM 1.1 on PostgreSQL 16+; JSONB columns are natively supported and queryable.
- **Async traits**: `#[async_trait]` for dyn-compatible traits; AFIT for non-dyn contexts.
- **Sandbox**: Rootless Podman/Docker container per remediation run; model providers: Claude, Gemini, Ollama, ONNX classifier.
- **Existing patterns**: `auto_merge_rule`, `merge_operation`, `provider_account` as reference entities for DB-backed configuration with `created_at` / `updated_at` timestamps.
- **API layer**: Axum 0.8; an SSE pattern for live updates already exists for the bulk-merge flow.
- **Scoping hierarchy**: repo → team → org → user (matches existing Ampel multitenancy model).

---

## Decision

**Playbooks are versioned YAML files. Built-in defaults are embedded in the worker binary via `rust-embed`. Org and team administrators may create or replace Playbooks through a `remediation_playbook` database table. Repo-local `.ampel/remediation.yaml` files, read from inside the sandbox at runtime, take highest precedence. Resolution order is: repo-local > DB scope override > built-in embedded default. The `minijinja` engine renders all template variables before the Playbook is handed to the harness. Repo-local files cannot grant permissions beyond the org-level tools-policy ceiling.**

This decision was reached because it is the only option that simultaneously satisfies zero-config deployment (embedded defaults), fleet-wide policy management (DB overrides), GitOps-friendly team workflows (repo-local files), and air-gapped security (org ceiling enforced server-side regardless of what any repo-local file requests).

### Implementation Notes

#### Playbook YAML Schema

```yaml
version: "1"                         # semver string; enforced by loader
name: "default-remediation"
description: "Fleet-wide default remediation playbook"

role_prompt: |
  You are an expert software engineer performing automated PR remediation
  for the {{ repo_full_name }} repository. Current date: {{ now_utc }}.

task_templates:
  conflict:
    prompt: "Resolve merge conflicts in the following diff:\n\n{{ diff }}"
    output_contract: unified_diff
  test_failure:
    prompt: "Fix the failing tests identified below:\n\n{{ test_output }}"
    output_contract: tool_use
  lint_failure:
    prompt: "Correct the linting errors:\n\n{{ lint_output }}"
    output_contract: unified_diff

context_spec:
  include_diff: true
  include_ci_logs: true
  include_pr_description: true
  max_diff_bytes: 131072            # 128 KiB hard cap

output_contract:
  default: tool_use                 # tool_use | unified_diff
  require_reasoning: true

tools_policy:
  run_command_allowlist:
    - "cargo test"
    - "cargo clippy"
    - "pnpm test -- --run"
  network: none                     # none | egress_only | unrestricted

loop_config:
  max_iterations: 5
  completion_condition: "all_checks_green"
  on_iteration_reflexion: true
  budget:
    max_tokens: 200000
    max_wall_seconds: 600

provider_overlays:
  claude:
    model: "claude-sonnet-4-5"
    temperature: 0.2
  gemini:
    model: "gemini-2.0-flash"
    temperature: 0.2
  ollama:
    model: "qwen2.5-coder:7b"
    temperature: 0.1
```

#### Embedding Built-in Defaults

```rust
// crates/ampel-worker/src/playbook/embedded.rs
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "playbooks/"]         // relative to crate root; compiled into binary
struct EmbeddedPlaybooks;

pub fn load_builtin(name: &str) -> Option<String> {
    EmbeddedPlaybooks::get(&format!("{name}.yaml"))
        .map(|f| String::from_utf8_lossy(f.data.as_ref()).into_owned())
}
```

#### Database Table

```sql
-- migration: YYYYMMDDHHMMSS_create_remediation_playbook.sql
CREATE TABLE remediation_playbook (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    scope       TEXT        NOT NULL CHECK (scope IN ('org','team','repo')),
    scope_id    UUID        NOT NULL,
    name        TEXT        NOT NULL,
    version     TEXT        NOT NULL,
    body        TEXT        NOT NULL,   -- YAML text; validated on write
    source      TEXT        NOT NULL DEFAULT 'db'
                            CHECK (source IN ('builtin','db','repo')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (scope, scope_id, name)
);
CREATE INDEX idx_remediation_playbook_scope ON remediation_playbook (scope, scope_id);
```

#### Resolution Logic

```rust
// crates/ampel-worker/src/playbook/resolver.rs
pub async fn resolve(
    repo_path: &Path,
    scope: &ResolverScope,
    db: &DatabaseConnection,
    org_ceiling: &ToolsPolicy,
) -> Result<Playbook, PlaybookError> {
    // 1. repo-local (highest precedence)
    let repo_local = repo_path.join(".ampel/remediation.yaml");
    if repo_local.exists() {
        let raw = tokio::fs::read_to_string(&repo_local).await?;
        let mut pb = parse_and_validate(&raw)?;
        pb.tools_policy = pb.tools_policy.clamp_to_ceiling(org_ceiling);
        pb.source = PlaybookSource::Repo;
        return Ok(pb);
    }

    // 2. DB override (team then org)
    if let Some(row) = db_fetch_override(scope, db).await? {
        let mut pb = parse_and_validate(&row.body)?;
        pb.tools_policy = pb.tools_policy.clamp_to_ceiling(org_ceiling);
        pb.source = PlaybookSource::Db;
        return Ok(pb);
    }

    // 3. embedded default (lowest precedence)
    let raw = load_builtin("default-remediation")
        .ok_or(PlaybookError::EmbeddedNotFound)?;
    let mut pb = parse_and_validate(&raw)?;
    pb.tools_policy = pb.tools_policy.clamp_to_ceiling(org_ceiling);
    pb.source = PlaybookSource::Builtin;
    Ok(pb)
}
```

#### minijinja Rendering

```rust
// crates/ampel-worker/src/playbook/render.rs
use minijinja::{Environment, context};

pub fn render_prompt(template: &str, vars: &PlaybookContext) -> Result<String, minijinja::Error> {
    let mut env = Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    env.add_template("prompt", template)?;
    let tmpl = env.get_template("prompt")?;
    tmpl.render(context! {
        repo_full_name => vars.repo_full_name,
        now_utc        => vars.now_utc.to_rfc3339(),
        diff           => vars.diff,
        test_output    => vars.test_output,
        lint_output    => vars.lint_output,
    })
}
```

#### API Surface

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/remediation/playbooks` | List all DB overrides visible to the caller |
| `POST` | `/api/remediation/playbooks` | Create a new DB override (org/team scope) |
| `PATCH` | `/api/remediation/playbooks/{id}` | Update body or version of an existing override |
| `DELETE` | `/api/remediation/playbooks/{id}` | Remove a DB override (falls back to embedded default) |
| `GET` | `/api/remediation/playbooks/{id}/preview` | Render assembled prompt; no model call |

The `/preview` endpoint accepts an optional JSON body containing mock context variables so prompt engineers can test template output deterministically.

#### Security: Tools-Policy Ceiling

The `ToolsPolicy::clamp_to_ceiling` method is called unconditionally after every resolution step — including repo-local files. A repo-local file can restrict the allowlist or remove network access; it cannot add commands that are absent from the org ceiling. This enforcement is done server-side inside the worker process, not inside the sandbox, so the sandbox itself cannot bypass it.

---

## Alternatives Considered

### Option A: Embedded YAML + DB Overrides + Repo-Local (Accepted)

**Pros**:
- Zero-config: ships with sensible defaults baked into the binary.
- GitOps: teams version Playbooks alongside code; diffs are reviewable in PRs.
- Fleet management: org/team admins override without a deployment.
- A/B testing: two DB rows with different names can be targeted per-repo.
- Air-gapped enforcement: org ceiling is applied server-side regardless of file source.

**Cons**:
- Three sources to reason about; resolution logic must be explicit and well-tested.
- Repo-local files arrive from untrusted sources; ceiling enforcement is a hard requirement.
- Binary rebuild is needed to update the embedded default (mitigated: DB overrides are live).

**Verdict**: Accepted. Satisfies all requirements; complexity is manageable and well-precedented in tools like Renovate and Dependabot.

---

### Option B: Database Only (Rejected)

**Pros**:
- Single source of truth; no resolution order to reason about.
- Live updates without binary changes.
- Simple UI: one table, one API.

**Cons**:
- No GitOps; Playbooks cannot be reviewed in code PRs or rolled back with `git revert`.
- Bootstrap problem: every fresh deployment requires manual DB seeding before any remediation can run.
- Harder to A/B test across repos without additional tooling.
- Outage risk: if the DB is unavailable during a remediation run, no fallback exists.

**Verdict**: Rejected. Eliminates GitOps capability and creates a bootstrap dependency that conflicts with the zero-config goal.

---

### Option C: Repo-Local Only (Rejected)

**Pros**:
- Maximum flexibility: each repo owns its Playbook entirely.
- Fully GitOps; changes are always code-reviewed.

**Cons**:
- Does not scale for large fleets: every repository requires a `.ampel/remediation.yaml` file before remediation can run.
- No fleet-wide defaults; org policy enforcement requires per-repo file audits.
- No mechanism for fleet-wide rollout of prompt improvements without opening thousands of PRs.
- A bootstrap Playbook is still needed, creating a circular dependency (Playbook needed to create the Playbook PR).

**Verdict**: Rejected. Operationally unscalable and incompatible with the fleet-wide defaults requirement.

---

## Trade-off Analysis

| Aspect | Option A: Embedded + DB + Repo | Option B: DB Only | Option C: Repo-Local Only |
|---|---|---|---|
| Zero-config deployment | Yes (embedded default) | No (requires DB seed) | No (requires per-repo file) |
| GitOps / PR-reviewable | Yes (repo-local tier) | No | Yes |
| Fleet-wide policy management | Yes (DB overrides) | Yes | No |
| Live updates without redeploy | Yes (DB tier) | Yes | No |
| A/B testing | Yes (named DB rows) | Yes | Difficult |
| Air-gapped security enforcement | Yes (ceiling applied server-side) | Yes | Hard to audit |
| Operational complexity | Medium (3 sources) | Low | Low (per repo) / High (fleet) |
| DB outage resilience | Yes (embedded fallback) | No | Yes |

---

## Consequences

### Positive

- Prompt engineers can iterate on Playbooks without a Rust build cycle, using the DB override tier or a local `.ampel/remediation.yaml` in a test repository.
- The `/preview` endpoint enables regression testing of prompt output in CI pipelines.
- Embedded defaults guarantee that the system functions in a fresh deployment with no additional configuration.
- The security ceiling model means org administrators retain control even when granting teams the ability to customise Playbooks.
- Version fields on DB rows enable audit trails and safe rollback to a previous body.

### Negative

- Three-tier resolution logic must be thoroughly tested; a bug in the resolver could silently apply the wrong Playbook.
- Repo-local files from untrusted repositories represent an attack surface; the ceiling enforcement path is security-critical and must be fuzz-tested.
- The embedded YAML files become part of the binary's release artifact; large or numerous Playbooks increase binary size.
- Org-level ceiling configuration must be defined and documented before any repo-local or DB override customisation is meaningful.

### Neutral

- The `remediation_playbook` table follows the same SeaORM entity + timestamped migration pattern used for `auto_merge_rule`, `merge_operation`, and `provider_account`; no new database patterns are introduced.
- minijinja is already decided (this ADR builds on that choice without revisiting it).
- YAML was chosen over TOML or JSON because it supports multi-line string literals (essential for role prompts) without escape sequences, and is already used in the project's CI workflow files.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| Resolver applies wrong tier due to logic bug | High | Unit tests covering all seven resolution paths (3 tiers × present/absent + ceiling clamp); property-based tests with arbitrary Playbooks |
| Repo-local file escalates tools\_policy beyond org ceiling | Critical | `clamp_to_ceiling` called unconditionally server-side; ceiling enforcement is covered by dedicated security tests and fuzz corpus |
| Malformed YAML in DB override breaks all remediation for a scope | High | Validate YAML + schema on `POST`/`PATCH`; return 422 with error details; embedded default always available as fallback |
| minijinja undefined variable causes runtime panic | Medium | `UndefinedBehavior::Strict` converts undefined vars to errors; `/preview` catches these before production runs |
| Embedded default becomes stale relative to harness API changes | Medium | Loader validates Playbook schema version on startup; version mismatch fails fast with a clear error message |
| DB outage during remediation run | Low | Resolver falls through to embedded default; remediation continues with reduced customisation, logs a warning |
| Binary size growth from large embedded Playbooks | Low | Monitor binary size in CI; compress embedded assets with `rust-embed`'s `deflate` feature if needed |

---

## Related ADRs

- ADR-001: Locale Middleware State Access Pattern — establishes the pattern of layered configuration with DB-backed overrides used throughout Ampel.
- ADR-005: minijinja as Playbook Templating Engine — selects the templating engine referenced in the Implementation Notes above.
- ADR-007 (planned): Remediation Sandbox Isolation — governs the Podman/Docker container model that enforces `tools_policy.network` and `run_command_allowlist` at the process level.
- ADR-008 (planned): Model Provider Routing for Remediation Agents — governs how `provider_overlays` in a Playbook are resolved to an actual model client.
