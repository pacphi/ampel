# ADR-009: Model Provider V1 Scope Selection

**Status**: Accepted
**Date**: 2026-06-24
**Deciders**: Architecture Team
**Technical Story**: Define the minimum viable set of model providers for the Phase 4 agentic tier of the Fleet PR Remediation Loops feature, ensuring coverage across hosted inference, local inference, and the router classifier from day one.

---

## Context

### Problem Statement

The Fleet PR Remediation Loops feature (triggered when a repo accumulates > 3 open PRs) requires an agentic tier capable of autonomously triaging, consolidating, and remediating PRs. The `ModelProvider` trait defined in ADR-007 was designed to support multiple backends behind a uniform interface; however, shipping all conceivable providers at once is not practical. A scope decision was needed to define exactly which providers constitute v1.

The core tension is between speed-to-ship and day-one capability breadth. Shipping a single provider validates the harness architecture fastest but leaves gaps: no local fallback, no air-gapped path, and no cheap classifier for the router strategy. Shipping too many providers increases integration surface and delays the initial release. The team needed a principled cut that covers the three functional roles — capable hosted fixer, local/air-gapped fixer, and cheap failure classifier — without including providers whose integration complexity exceeds their near-term value.

A secondary driver is the router model-selection strategy (also ADR-007): when a failure is classified by a lightweight in-process model, the router decides whether to dispatch to a hosted provider (when egress is permitted) or to a local server (when the org policy sets `air_gapped = true`). This pattern cannot be exercised at all without at least one classifier and at least one local inference backend present in v1.

The decision to support air-gapped deployments from day one (ADR-008) further constrains the scope: any org with `air_gapped = true` must have a functional remediation path that does not touch the public internet. That path requires both a local inference provider and a local classifier.

### Technical Context

- **Trait surface**: `ModelProvider` in `ampel-core` exposes `async fn infer(...)` and `async fn classify(...)` behind `#[async_trait]` for `dyn`-compatibility.
- **Provider metadata fields**: `kind` (`inference` | `agent`), `egress` (`External` | `LocalOnly`), `deployment` (`HostedApi` | `LocalServer` | `InProcess`), `output_contract` (`tool_use` | `unified_diff` | `classify_only`).
- **Router strategy**: classifies `failure_class` via a cheap model, then dispatches to a full inference provider based on `policy.air_gapped` and provider egress.
- **Air-gapped policy**: org-level hard ceiling; per-policy opt-in; blocks any provider with `Egress::External`.
- **Playbook rendering**: `minijinja` renders YAML playbooks; providers receive rendered prompt strings.
- **CI verification**: re-verify CI status immediately before merge (TOCTOU guard) — provider output drives, but does not replace, the verification step.
- **Sandbox**: each agentic run executes inside a rootless Podman/Docker container; provider HTTP calls originate from within that sandbox.
- **Apalis 0.6**: background job infrastructure; provider calls happen inside job handlers.
- **Existing crates available**: `reqwest` (HTTP), `ort` (ONNX Runtime), `serde_json`.

---

## Decision

**V1 ships four providers — Claude, Gemini, Ollama, and ONNX — covering all three functional roles (hosted capable fixer, local/air-gapped fixer, failure classifier) and enabling the air-gapped router pattern from day one. Codex CLI and a generic OpenAI-compatible endpoint are explicitly deferred to v2.**

The four providers map to functional roles as follows:

| Provider | Role | Kind | Deployment | Egress | Output Contract |
|----------|------|------|------------|--------|-----------------|
| Claude | Primary capable fixer | inference | HostedApi | External | tool\_use |
| Gemini | Alternate capable fixer | inference | HostedApi | External | tool\_use |
| Ollama | Local fixer | inference | LocalServer | LocalOnly | unified\_diff |
| ONNX | Failure classifier | inference | InProcess | LocalOnly | classify\_only |

### Implementation Notes

**Claude provider** authenticates via the `ANTHROPIC_API_KEY` environment variable. The default model is `claude-sonnet-4-6`; it is overridable via provider config. Requests use the Anthropic Messages API with tool use enabled. The provider implements `output_contract: tool_use`, returning structured JSON that the harness interprets as patch operations.

```rust
// Sketch: Claude provider config in provider_accounts / config layer
pub struct ClaudeProviderConfig {
    pub api_key: EncryptedSecret,       // AES-256-GCM via EncryptionService
    pub model: String,                  // default: "claude-sonnet-4-6"
    pub max_tokens: u32,                // default: 8192
    pub timeout_secs: u64,              // default: 120
}
```

**Gemini provider** authenticates via `GOOGLE_API_KEY`. Supported models are `gemini-2.0-flash` (fast, lower cost) and `gemini-2.5-pro` (higher capability); default is `gemini-2.0-flash`. The provider calls the Google Generative AI REST API with function-calling enabled (`output_contract: tool_use`). Because Gemini's function-calling response schema differs from Anthropic's, the harness normalises both to an internal `ToolCallResult` type before the playbook step handler processes them.

**Ollama provider** calls a local HTTP server (OpenAI-compatible `/api/chat` endpoint) at a configurable base URL (default `http://localhost:11434`). Recommended models are `qwen2.5-coder` and `deepseek-coder-v2`. Because tool-use availability varies by loaded model, the output contract is `unified_diff`: the provider prompts for a raw unified diff and the harness applies it via `git apply`. This is less structured than tool use but universally compatible with any Ollama-hosted model.

```rust
pub struct OllamaProviderConfig {
    pub base_url: Url,                  // default: http://localhost:11434
    pub model: String,                  // e.g. "qwen2.5-coder:7b"
    pub context_length: Option<u32>,
    pub timeout_secs: u64,              // default: 300 (local models are slower)
}
```

**ONNX provider** is not a full fixer. It runs in-process via the `ort` crate and is used exclusively by the router strategy to classify `failure_class` (e.g. `lint`, `type_error`, `test_failure`, `merge_conflict`) before dispatching to a real inference provider. The model file path is configured via `model_path`; the provider exposes only `classify(...)`, not `infer(...)`. Calling `infer(...)` on the ONNX provider returns `Err(ModelProviderError::OperationNotSupported)`.

```rust
pub struct OnnxProviderConfig {
    pub model_path: PathBuf,            // path to .onnx classifier model
    pub label_map: PathBuf,             // JSON label → failure_class mapping
    pub intra_op_threads: u32,          // default: 2
}
```

**Router wiring** — when `policy.air_gapped = true`, the router filters to providers with `Egress::LocalOnly` and selects Ollama for inference (ONNX for classification). When `air_gapped = false`, the router may dispatch to Claude or Gemini based on playbook-level preference or cost policy.

```
failure_class = onnx.classify(failure_log)
if policy.air_gapped:
    provider = ollama
else:
    provider = playbook.preferred_provider ?? claude
result = provider.infer(playbook_prompt)
```

All four provider configs are stored encrypted (AES-256-GCM via `EncryptionService`) in `provider_accounts.access_token_encrypted` where applicable. Ollama and ONNX do not require secrets but their configs are stored in the same `provider_accounts` table for uniformity.

---

## Alternatives Considered

### Option A: Claude Only (Rejected)

Ship only the Claude provider for v1. All other providers deferred to v2.

**Pros**:
- Smallest integration surface; validates the `ModelProvider` harness fastest.
- Minimal secrets-management burden in v1.
- Fastest time-to-first-working-demo.

**Cons**:
- No local fallback; air-gapped orgs cannot use the agentic tier at all in v1.
- Router strategy cannot be exercised (no classifier, no local provider to route to).
- Gemini as a redundancy/fallback also deferred, reducing reliability.
- Creates pressure to bolt on Ollama + ONNX hastily in an early v2 patch.

**Verdict**: Rejected. Leaves the air-gapped use case (a committed ADR-008 requirement) entirely unserved in v1, and the router pattern — a core architectural element — cannot be validated.

---

### Option B: Claude + Ollama (Rejected)

Ship Claude (hosted) and Ollama (local) only. Gemini and ONNX deferred to v2.

**Pros**:
- Covers hosted + local split; air-gapped orgs have a functional path.
- Smaller scope than Option C; two fewer provider integrations.
- Ollama's unified-diff output contract is simpler to implement than tool-use normalisation.

**Cons**:
- No cheap classifier: the router must use heuristics or a full inference call to determine `failure_class`, which is slow and expensive.
- Gemini deferred: no fallback if Claude is unavailable or rate-limited.
- Without ONNX, the router model-selection strategy documented in ADR-007 cannot be exercised in v1, leaving a core strategy untestable.

**Verdict**: Rejected. The absence of ONNX means the router strategy is untestable in v1 and `failure_class` classification either requires a costly full inference call or degrades to string matching. This is a poor foundation.

---

### Option C: Claude + Gemini + Ollama + ONNX (Accepted)

Ship all four providers as defined in the Decision section above.

**Pros**:
- All three functional roles covered: hosted fixer (Claude, Gemini), local fixer (Ollama), classifier (ONNX).
- Air-gapped router pattern fully exercisable from day one.
- Gemini provides a live fallback if Claude is rate-limited or unavailable.
- ONNX classifier is cheap (in-process, no network hop) and keeps router decisions fast.
- Sets a clean precedent for how v2 providers (Codex CLI, generic OpenAI endpoint) slot in.

**Cons**:
- Four provider integrations in one phase; higher initial implementation effort.
- Two distinct `output_contract` normalisations needed (`tool_use` for Claude/Gemini, `unified_diff` for Ollama).
- ONNX model artifact must be bundled or fetched at startup; adds a deployment concern.
- Gemini function-calling response schema differs from Anthropic's; normalisation layer required.

**Verdict**: Accepted. The additional effort is bounded and the coverage across roles and egress modes justifies it. The router strategy, a core architectural commitment, becomes testable and shippable in v1.

---

## Trade-off Analysis

| Aspect | Option A (Claude only) | Option B (Claude + Ollama) | Option C (Chosen) |
|--------|------------------------|---------------------------|-------------------|
| Implementation effort | Low | Medium | High |
| Air-gapped support | None | Yes (Ollama) | Yes (Ollama + ONNX router) |
| Router strategy testable | No | No (no classifier) | Yes |
| Hosted fallback | No | No | Yes (Gemini) |
| Output contracts needed | 1 (tool\_use) | 2 (tool\_use, unified\_diff) | 2 (tool\_use, unified\_diff) |
| Response normalisation | Minimal | Minimal | Two hosted schemas to normalise |
| ONNX deployment burden | None | None | Model artifact required |
| v2 rework risk | High (bolt-on) | Medium (add ONNX + Gemini) | Low (clean extension points) |
| ADR-008 compliance | Fails | Partial (no router) | Full |

---

## Consequences

### Positive

- Air-gapped deployments are fully supported in v1; no org is blocked from using the agentic tier due to egress policy.
- The router model-selection strategy (ADR-007) is exercisable and testable in v1, reducing the risk of architectural drift in v2.
- Gemini as an alternate hosted fixer provides live redundancy without requiring operator intervention when Claude is unavailable or rate-limited.
- ONNX keeps `failure_class` classification cheap and local; it does not consume inference tokens or add network latency to the routing decision.
- The two-output-contract design (`tool_use` and `unified_diff`) establishes a normalisation pattern that v2 providers can adopt without harness changes.

### Negative

- Four provider integrations must ship together; any one delayed blocks the full v1 scope.
- Two distinct hosted response schemas (Anthropic Messages API vs Google Generative AI API) require a normalisation layer in the harness — additional code that must be maintained as both APIs evolve.
- The ONNX model artifact (classifier) must be distributed with the binary or fetched at job-worker startup; this adds a deployment step not present in the other options.
- `output_contract: classify_only` on the ONNX provider means callers must guard against calling `infer(...)` on it; this is a runtime contract, not a compile-time one.

### Neutral

- Codex CLI and the generic OpenAI-compatible endpoint are explicitly documented as v2 items, giving the team a clear backlog entry without scope creep into v1.
- Ollama's `unified_diff` output contract is less structured than `tool_use` but is intentional: it decouples Ampel from Ollama's evolving tool-use support across models.
- Provider configs are stored in `provider_accounts` for all four providers, including Ollama and ONNX which require no secrets; this keeps the config surface uniform at the cost of minor schema awkwardness for secret-free providers.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Gemini function-calling API changes between now and GA | Medium | Pin to a specific API version; add integration test against live API in CI nightly run. |
| ONNX classifier model artifact unavailable at job-worker startup | High | Fail-closed: if model cannot be loaded, disable router strategy and fall back to heuristic `failure_class` detection; log a structured warning. |
| Ollama tool-use gaps cause unified-diff patches to be malformed | Medium | Validate diff syntax before `git apply`; reject and surface error in SSE stream rather than applying a bad patch. |
| Claude or Gemini rate limits spike under load | Low | Implement per-provider exponential backoff with jitter in the `ModelProvider` trait adapter; Gemini serves as live fallback for Claude. |
| ONNX in-process execution contends with Tokio worker threads | Low | Run ONNX inference on a `tokio::task::spawn_blocking` thread; configure `intra_op_threads` conservatively (default 2). |
| Air-gapped orgs use an Ollama model that produces low-quality diffs | Medium | Document recommended models (`qwen2.5-coder`, `deepseek-coder-v2`); expose `min_confidence` threshold in playbook config to gate merge on diff quality score. |
| Two output contracts diverge in harness handling over time | Low | Define a single `NormalisedProviderOutput` internal type at the harness boundary; both `tool_use` and `unified_diff` paths must produce it before the step handler is invoked. |

---

## Related ADRs

- ADR-007: ModelProvider trait design and router model-selection strategy
- ADR-008: Air-gapped deployment policy and egress controls
- ADR-010: ONNX classifier model distribution and startup loading (planned)
- ADR-011: Output contract normalisation (`tool_use` vs `unified_diff`) (planned)
