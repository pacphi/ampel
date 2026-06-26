# ADR-014: Air-Gapped Governance (Org Ceiling + Per-Policy Opt-In)

**Status**: Accepted  
**Date**: 2026-06-24  
**Deciders**: Architecture Team  
**Technical Story**: The agentic remediation tier (Phase 4) sends code and CI logs to
hosted AI providers (Claude, Gemini) when configured. Regulated fleets (financial
services, government, healthcare) require that code never leave the host perimeter.
A governance model must enforce this at the fleet level, not just per-policy.

---

## Context

### Problem Statement

The `ModelProvider` trait (ADR-007) classifies providers by `Egress`:
- `Egress::External` — Claude, Gemini; data leaves the host
- `Egress::LocalOnly` — Ollama (local server), ONNX (in-process); data stays on host

An operator managing a regulated org could accidentally configure an `External` provider
in a `remediation_policy` for a sensitive repository. A per-policy flag alone provides
no fleet-wide backstop — a misconfigured policy reaches the model provider before the
error is noticed.

The governance model must:
1. Allow the org administrator to declare the entire org air-gapped (no external
   provider calls, ever)
2. Allow individual teams or repos within a non-air-gapped org to still opt into
   local-only enforcement
3. Block External model provider account registration at the point of creation for
   air-gapped orgs
4. Surface the effective air-gapped status in the `/preview` response so operators
   can audit before enabling `auto_merge`

### Technical Context

- `RemediationPolicy` already uses a hierarchical scope model (repo → team → org →
  user default) resolved by `PolicyResolver` (ADR-002-adjacent).
- `ModelProviderAccount` is scoped to user/org/team; accounts with `egress_class =
  External` are the entities that must be blocked.
- The Ampel settings model already has `org_settings` and `user_settings` tables with
  JSON config blobs; `air_gapped` is a new boolean column on `org_settings`.
- Multi-tenant Ampel instances serve multiple orgs with different compliance requirements;
  a global single flag cannot serve this.

---

## Decision

**Two-level enforcement: an org-level hard ceiling (`org_settings.air_gapped`) and a
per-policy opt-in (`remediation_policy.air_gapped`). The org ceiling is enforced by
`RemediationService` before every provider dispatch and cannot be bypassed by policy
configuration.**

### Schema

```sql
-- Migration 1: org ceiling
ALTER TABLE org_settings ADD COLUMN air_gapped BOOLEAN NOT NULL DEFAULT FALSE;

-- Migration 2: per-policy opt-in
ALTER TABLE remediation_policy ADD COLUMN air_gapped BOOLEAN NOT NULL DEFAULT FALSE;
```

### PolicyResolver Ceiling Application

```rust
// After resolving the effective policy from the scope hierarchy:
if org.settings.air_gapped {
    // Org ceiling overrides per-policy; policy.air_gapped=false is silently corrected
    effective_policy.air_gapped = true;
}
// effective_policy.air_gapped is now authoritative
```

### Dispatch Guard in `RemediationService`

```rust
fn assert_egress_allowed(
    provider: &dyn ModelProvider,
    policy: &RemediationPolicy,
) -> Result<()> {
    if policy.air_gapped && provider.capabilities().egress == Egress::External {
        return Err(RemediationError::EgressBlocked {
            provider: provider.id().to_string(),
            reason: "air_gapped_policy",
        });
    }
    Ok(())
}

// Called before every infer() / run_agent() dispatch
assert_egress_allowed(provider.as_ref(), &effective_policy)?;
```

On `EgressBlocked`, the run transitions to `RemediationOutcome::Blocked` (not
`handoff_human` — no human action needed, operator must update config).

### Account Registration Guard

```rust
// In POST /api/orgs/{org_id}/model-provider-accounts handler
if org.settings.air_gapped
    && payload.egress_class == EgressClass::External
{
    return Err(ApiError::UnprocessableEntity(
        "Cannot add External model provider to an air-gapped org"
    ));
}
```

### Preview Enrichment

`GET /api/remediation/repositories/{repo_id}/preview` response includes:

```json
{
  "providers": [
    {
      "account_id": "...",
      "provider": "claude",
      "egress_class": "external",
      "blocked_by_air_gap": true,
      "air_gapped_source": "org"
    }
  ]
}
```

---

## Alternatives Considered

### Option A: Per-policy flag only (Rejected)

**Approach**: Only `remediation_policy.air_gapped` exists; no org-level setting.

**Pros**: Simpler schema; per-repo granularity from day one.

**Cons**:
- ❌ No fleet-wide backstop — a misconfigured policy can reach an external provider
  for any repo in the org
- ❌ An admin cannot enforce org-wide compliance without auditing every policy

**Verdict**: REJECTED — insufficient for regulated fleets.

### Option B: Org ceiling + per-policy opt-in (ACCEPTED)

**Pros**:
- ✅ Org admin has an irrevocable, non-bypassable ceiling
- ✅ Teams can still restrict individual repos further within a non-air-gapped org
- ✅ PolicyResolver already handles hierarchical overrides — adding a ceiling step
  is minimal code
- ✅ Consistent with the `auto_merge_rule` precedence model

**Cons**:
- ⚠️ Two booleans (`org_settings.air_gapped` + `remediation_policy.air_gapped`) require
  clear documentation to avoid operator confusion

**Verdict**: ACCEPTED.

### Option C: Global single flag (Rejected)

**Approach**: One `AMPEL_AIR_GAPPED=true` env var or global config.

**Pros**: Simplest possible enforcement.

**Cons**:
- ❌ Cannot serve multi-tenant Ampel instances with mixed compliance postures
- ❌ No per-org or per-team granularity

**Verdict**: REJECTED — incompatible with multi-tenant deployment model.

---

## Trade-off Analysis

| Aspect | Option A (per-policy) | Option B (org + policy) ⭐ | Option C (global) |
|--------|-----------------------|--------------------------|-------------------|
| **Fleet-wide backstop** | ❌ None | ✅ Org ceiling | ✅ Yes (too coarse) |
| **Per-org granularity** | ✅ Yes | ✅ Yes | ❌ No |
| **Multi-tenant support** | ✅ Yes | ✅ Yes | ❌ No |
| **Implementation complexity** | Low | Low–Medium | Minimal |
| **Misconfiguration risk** | High | Low | None (but too broad) |
| **Preview auditability** | ⚠️ No enrichment | ✅ `air_gapped_source` field | ⚠️ No enrichment |

---

## Consequences

### Positive

- Org admins can set `air_gapped=true` and be confident no code leaves the perimeter,
  regardless of how individual policies are configured
- Registration guard at the API layer surfaces the misconfiguration early, not at
  dispatch time
- `/preview` enrichment lets operators audit egress before enabling `auto_merge`

### Negative

- Two booleans must be documented clearly; `PolicyResolver` ceiling logic must be tested
  for all four combinations (`org × policy`)
- A policy with `air_gapped=false` in an air-gapped org silently becomes `air_gapped=true`;
  this is intentional but could surprise operators who query the raw policy

### Neutral

- The `EgressBlocked` outcome produces a `Blocked` run state distinct from `HandoffHuman`;
  operators must update model account config, not approve a run

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Org ceiling not applied due to PolicyResolver bug | High | Integration tests for all four org×policy combinations; separate unit test for ceiling logic |
| External account added before org is set air-gapped | Medium | Periodic validation job flags External accounts in air-gapped orgs; operator notified |
| Preview enrichment omitted, operator blindly enables auto_merge | Medium | Frontend requires preview before first auto_merge enable; `blocked_by_air_gap` shown prominently |

---

## Related ADRs

- ADR-007: `ModelProvider` trait — `capabilities().egress` is the value compared in
  `assert_egress_allowed`
- ADR-008: Model provider credential storage — `egress_class` is a field on
  `ModelProviderAccount`; registration guard is at the account creation API
- ADR-009: Model provider v1 scope — Ollama and ONNX are `LocalOnly`; Claude and Gemini
  are `External`; air-gapped orgs can only use the former two
