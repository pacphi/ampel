# ADR-008: Model Provider Credential Storage

**Status**: Accepted
**Date**: 2026-06-24
**Deciders**: Architecture Team
**Technical Story**: Fleet PR Remediation Loops require inference calls to external and local model providers (Claude, Gemini, Ollama, ONNX); those providers need credentials stored securely and scoped to the org/team/user that owns the remediation policy.

---

## Context

### Problem Statement

The Fleet PR Remediation Loops feature introduces four categories of model provider: hosted API providers requiring API keys (Claude, Gemini, and any OpenAI-compatible endpoint), local server providers requiring only an endpoint URL (Ollama), and in-process providers requiring only a filesystem path (ONNX). Each remediation harness must resolve the correct credentials before issuing an `infer()` or `run_agent()` call.

Credential scope must follow Ampel's existing hierarchical config model: a repo-level remediation policy inherits model credentials from its team, then its organization, then the user who owns the job. A single deployment may serve multiple organizations, each with distinct API keys, spend caps, and egress restrictions. Flat environment-variable injection cannot satisfy per-org scoping or runtime rotation.

The existing `provider_accounts` table already stores Git provider PATs encrypted with AES-256-GCM via `EncryptionService` (in `crates/ampel-db/src/encryption.rs`). The binary format is a 12-byte random nonce prepended to the AEAD ciphertext, keyed by the `ENCRYPTION_KEY` environment variable (base64-encoded 32-byte key). Model provider credentials must carry the same confidentiality guarantee at rest and the same zero-plaintext-in-API-response guarantee already enforced for Git PATs.

Hosted provider API keys expire, can be revoked, and should be validated on write and periodically re-validated by background workers. Local providers (Ollama, ONNX) carry no secret at all — they need only a URL or a file path — so the credential record for those variants must tolerate a NULL encrypted column without special-casing the encryption layer.

Spend caps and egress class are first-class fields rather than freeform JSON because the remediation harness must check them synchronously before every inference call; parsing JSON on the hot path would be fragile and slow.

### Technical Context

- **ORM**: SeaORM 1.1 with PostgreSQL 16+ (production) and SQLite (tests); migrations use `sea_orm_migration` with `#[async_trait]`.
- **Encryption**: `EncryptionService` in `crates/ampel-db/src/encryption.rs` — AES-256-GCM, 12-byte random nonce prepended to ciphertext, stored as `VarBinary`. Keyed by `ENCRYPTION_KEY` (base64, 32 bytes).
- **Existing precedent**: `provider_accounts.access_token_encrypted` column (`VarBinary NOT NULL`) — same pattern, same service.
- **Scope hierarchy**: organization → team → user. Resources in `ampel-core` are already scoped this way; `scope` + `scope_id` is the established idiom.
- **Background jobs**: Apalis 0.6 on PostgreSQL; validation pings and spend-reset jobs fit the existing `RepositoryPollJob` / `CleanupJob` pattern in `ampel-worker`.
- **API responses**: `ProviderAccountResponse` DTO already omits `access_token_encrypted`; the same omission must apply to model credentials.
- **Air-gap policy**: org-level `egress_class` hard ceiling for local-only deployments; per-record `egress_class` field enforces this without re-parsing config at call time.
- **No new Cargo dependencies**: `aes-gcm`, `rand`, and `base64` are already present; SeaORM JSON column support is already enabled.

---

## Decision

**We will introduce a `model_provider_accounts` entity in `crates/ampel-db` that reuses `EncryptionService` for API key storage, mirrors the structural conventions of `provider_accounts`, and adds model-specific fields (egress class, spend cap, model path, extra config) absent from the Git provider model.**

This approach requires no new crates, no new infrastructure, and no changes to `EncryptionService` itself. It extends the encrypted-at-rest pattern that is already audited, tested, and operational.

### Implementation Notes

**SeaORM entity skeleton** (`crates/ampel-db/src/entities/model_provider_account.rs`):

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "model_provider_accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    // Scope — one of "user" | "org" | "team"
    pub scope: String,
    pub scope_id: Uuid,

    // Provider identification
    pub provider: String,         // "claude" | "gemini" | "ollama" | "onnx" | "openai_compatible"
    pub account_label: String,

    // Authentication
    pub auth_type: String,        // "api_key" | "bearer" | "custom_header" | "none"
    #[sea_orm(column_type = "VarBinary(StringLen::None)", nullable)]
    pub api_key_encrypted: Option<Vec<u8>>,  // NULL for local providers

    // Endpoint / model location
    pub endpoint_url: Option<String>,  // Ollama / self-hosted base URL
    pub model_id: String,              // default model string (e.g. "claude-opus-4-5")
    pub model_path: Option<String>,    // ONNX file reference; not a secret

    // Extra provider config (temperature, max_tokens, region, custom headers)
    #[sea_orm(column_type = "Json")]
    pub extra_config: serde_json::Value,

    // Egress policy
    pub egress_class: String,     // "external" | "local_only"

    // Spend tracking
    pub spend_cap_usd: Option<Decimal>,
    pub spend_used_usd: Decimal,

    // Validation
    pub validation_status: String,  // "pending" | "valid" | "invalid" | "expired"
    pub last_validated_at: Option<DateTimeUtc>,
    pub token_expires_at: Option<DateTimeUtc>,

    // Status
    pub is_active: bool,
    pub is_default: bool,

    // Timestamps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

**Encryption on write** (service layer in `ampel-core` or `ampel-api`):

```rust
// Only encrypt when auth_type != "none"
let api_key_encrypted = if auth_type == AuthType::None {
    None
} else {
    Some(encryption_service.encrypt(&plaintext_api_key)?)
};
```

**Decryption on read** (before passing to inference harness — never to DTO):

```rust
let plaintext = account
    .api_key_encrypted
    .as_deref()
    .map(|enc| encryption_service.decrypt(enc))
    .transpose()?;
```

**DTO response** — omit `api_key_encrypted` entirely; expose only safe fields:

```rust
pub struct ModelProviderAccountResponse {
    pub id: Uuid,
    pub scope: String,
    pub scope_id: Uuid,
    pub provider: String,
    pub account_label: String,
    pub auth_type: String,
    pub endpoint_url: Option<String>,
    pub model_id: String,
    pub model_path: Option<String>,
    pub egress_class: String,
    pub spend_cap_usd: Option<Decimal>,
    pub spend_used_usd: Decimal,
    pub validation_status: String,
    pub last_validated_at: Option<DateTimeUtc>,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    // api_key_encrypted intentionally absent
}
```

**Spend cap enforcement** (remediation harness, before every `infer()` call):

```rust
if let Some(cap) = account.spend_cap_usd {
    if account.spend_used_usd >= cap {
        return Err(RemediationError::SpendCapExceeded { account_id: account.id });
    }
}
```

**Validation ping strategy per provider**:

| Provider | Validation method |
|----------|-------------------|
| Claude | `POST /v1/messages` with `max_tokens: 1` |
| Gemini | `GET /v1/models` — list models |
| Ollama | `GET /api/tags` — list local models |
| ONNX | Load model file + 1-token decode via ONNX Runtime |
| openai_compatible | `GET /models` against `endpoint_url` |

**SeaORM migration** (`crates/ampel-db/src/migrations/mYYYYMMDD_NNNNNN_model_provider_accounts.rs`) should create:
- Primary key index on `id`
- Index on `(scope, scope_id)` for hierarchical resolution
- Index on `provider` for filtering
- Partial unique index: one default per `(scope, scope_id, provider)` where `is_default = true` (raw SQL, same pattern as `provider_accounts`)

**Scope resolution order** in the remediation harness:
1. Repo-level policy specifies `preferred_provider` + optional `model_provider_account_id`
2. If explicit ID not set: query `model_provider_accounts` where `scope = 'team'` and `scope_id = repo.team_id` and `is_default = true` and `is_active = true`
3. Fall back to `scope = 'org'`, then `scope = 'user'`
4. If no record found: return `RemediationError::NoModelProviderConfigured`

---

## Alternatives Considered

### Option A: Separate Secret Vault Service — HashiCorp Vault or AWS Secrets Manager (Rejected)

**Pros**: Industry-standard secret lifecycle management; built-in rotation, audit log, and fine-grained ACLs; secrets never touch the application database.

**Cons**: Requires provisioning and operating an additional infrastructure component (Vault cluster or AWS dependency); adds a network call on every credential fetch (latency + availability coupling); complicates local development and air-gapped deployments; no Fly.io native integration — would require a sidecar or secrets-injection layer; significant new Cargo dependencies (`vaultrs` or `aws-sdk-secretsmanager`); onboarding overhead for teams that do not already operate Vault.

**Verdict**: Rejected. The operational burden and infrastructure dependency are disproportionate to the threat model of a self-hosted PR dashboard. Encrypted-at-rest in PostgreSQL with envelope encryption (ENCRYPTION_KEY in the execution environment) is sufficient for the current compliance posture. This decision should be revisited if Ampel targets regulated industries or multi-cloud SaaS.

### Option B: Extend EncryptionService with New Entity (Accepted)

**Pros**: Zero new crates or infrastructure; consistent with the audited `provider_accounts` pattern; per-scope credential isolation without platform dependency; `EncryptionService` is already tested, including nonce-uniqueness guarantees; PostgreSQL column-level encryption is sufficient for data-at-rest; Fly.io secrets already manage `ENCRYPTION_KEY`; local development works identically to production.

**Cons**: Envelope encryption means the application process holds the plaintext key in memory; key rotation requires re-encrypting every row (no transparent re-wrap); no built-in access audit log — must be implemented separately in application telemetry.

**Verdict**: Accepted. Matches existing security posture, requires no new dependencies, and ships faster.

### Option C: Plaintext Environment Variables (Rejected)

**Pros**: Zero implementation cost; trivially available to all processes.

**Cons**: Cannot be scoped per-org or per-team; rotation requires redeployment; visible in process env dumps and log leakage; incompatible with the multi-tenant model where different orgs may use different Claude projects or Gemini API keys; no spend tracking or validation status; Fly.io secrets are per-app, not per-tenant.

**Verdict**: Rejected. Fundamentally incompatible with multi-tenancy and the per-scope credential requirement.

---

## Trade-off Analysis

| Aspect | Option A: Vault / Secrets Manager | Option B: EncryptionService + Entity (Chosen) | Option C: Env Vars |
|--------|----------------------------------|-----------------------------------------------|--------------------|
| Implementation effort | High — new infra, new Cargo deps | Low — extends existing pattern | Trivial |
| Per-scope isolation | Yes (path-based policies) | Yes (scope + scope_id columns) | No |
| Key rotation | Transparent (Vault) / semi-auto (ASM) | Manual row re-encryption | Redeployment |
| Audit log | Built-in | Application-level (tracing) | None |
| Air-gap compatibility | No (needs Vault/AWS reachable) | Yes | Yes |
| Local dev complexity | High (Docker Vault or localstack) | None (same as existing) | None |
| New dependencies | `vaultrs` or `aws-sdk-secretsmanager` | None | None |
| Spend cap & validation | Out of band | Native columns | Not possible |
| Data-at-rest encryption | Yes (Vault) / Yes (ASM) | Yes (AES-256-GCM) | No |
| Multi-tenant correctness | Yes | Yes | No |

---

## Consequences

### Positive

- No new Cargo dependencies or infrastructure components required.
- Credential scoping (user/team/org) integrates with the existing hierarchical config resolution pattern used throughout Ampel.
- `EncryptionService` test coverage and nonce-uniqueness guarantees already established by `provider_accounts`; model credentials inherit those guarantees.
- `api_key_encrypted` nullable column cleanly handles local providers (Ollama, ONNX) without branching in the encryption layer.
- Spend cap and egress class as first-class typed columns enable synchronous enforcement in the remediation harness without JSON parsing overhead.
- Validation status and `last_validated_at` enable an Apalis background job to periodically re-ping providers and surface stale credentials in the dashboard.
- The DTO pattern (omit ciphertext) is already established and reviewed; the same omission applies to model credentials with no new API surface to audit.

### Negative

- The application process holds the AES-256-GCM key in memory (loaded from `ENCRYPTION_KEY` at startup). A memory-dump attack on the API process would expose the key. Mitigated by Fly.io process isolation and the fact that the key is already trusted at the `provider_accounts` layer.
- Key rotation (changing `ENCRYPTION_KEY`) requires a migration job that decrypts all `api_key_encrypted` values with the old key and re-encrypts with the new key. This is a planned operational procedure, not implemented automatically.
- No built-in access audit trail. Credential reads for inference calls must be logged explicitly via the `tracing` instrumentation layer to satisfy audit requirements.
- Adding a new top-level entity increases migration complexity; the migration must be coordinated with existing migration ordering in `ampel-db/src/lib.rs`.

### Neutral

- `model_path` for ONNX is stored as a plain string (not encrypted) because it is a filesystem path, not a secret. Teams must ensure ONNX model files are protected by filesystem permissions separately.
- `extra_config` as a JSON column allows forward-compatible extension (custom headers, region hints, temperature defaults) without schema migrations for each new provider option. Schema validation is delegated to the service layer, not the database.
- `openai_compatible` as a provider variant allows any OpenAI-API-compatible self-hosted model (e.g., vLLM, LM Studio) to be configured without code changes, by providing `endpoint_url` and an `api_key`.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| `ENCRYPTION_KEY` exposure via process memory dump | Medium | Fly.io VM isolation; key never logged; secret injected via Fly secrets, not env file |
| Key rotation downtime | Medium | Implement a `rotate-model-keys` maintenance job in `ampel-worker` that processes rows in batches; schedule during low-traffic window |
| Spend cap bypass due to race condition | Low | Use a database-level transaction + `SELECT ... FOR UPDATE` on `spend_used_usd` before incrementing; reject if cap already met |
| Validation ping leaks API key via logs | Medium | Ensure inference client code never logs the plaintext key; use `tracing` with `%` formatter only for non-secret fields |
| Local provider record confusion (NULL api_key_encrypted) | Low | `auth_type = "none"` is the canonical signal; service layer asserts `api_key_encrypted IS NULL` when `auth_type = "none"` and returns an error if violated |
| Migration ordering conflict | Low | Add new migration with timestamp after the most recent existing migration; register in `ampel-db/src/lib.rs` migrator list |
| ONNX model path traversal | Low | Validate `model_path` against an allowlist of base directories at write time in the API layer |

---

## Related ADRs

- ADR-001: Locale Middleware State Access Pattern — establishes the convention of documenting non-obvious Axum/Rust architectural decisions in this format.
- ADR-005 (planned): Fleet PR Remediation Loops — sandbox isolation and playbook execution model; references model provider resolution as a dependency.
- ADR-006 (planned): Octopus Merge via subprocess git commands — sibling decision in the same remediation loops feature set.
- ADR-007 (planned): Playbook YAML embedding via rust-embed with DB overrides — establishes the playbook configuration layer that consumes model provider credentials resolved per this ADR.
