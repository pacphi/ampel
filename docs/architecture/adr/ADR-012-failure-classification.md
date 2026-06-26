# ADR-012: Failure Classification Approach for the Agentic Remediation Tier

**Status**: Accepted
**Date**: 2026-06-24
**Deciders**: Architecture Team
**Technical Story**: When Phase 4 (agentic tier) activates after mechanical remediation, CI failures on the consolidated branch must be classified before routing to a fixer model, in order to select the correct Playbook task and minimize inference cost.

---

## Context

### Problem Statement

The Fleet PR Remediation Loop activates its agentic tier (Phase 4) when CI fails on the consolidated branch after mechanical remediation has been applied. Before a fixer model (Claude, Gemini, Ollama) is invoked, the harness must know what kind of failure it is dealing with. Without classification, the harness cannot select the right Playbook task, cannot route to the cheapest appropriate model, and cannot produce useful metrics for reflexion learning.

CI log output is structured text of variable length. Common failure modes follow recognizable surface patterns — a Rust compile error always contains `error[E`, a TypeScript type error always starts with `error TS` — yet a non-trivial minority of failures are ambiguous: flaky network, unrecognized assertion messages, novel linter output. Any classification strategy must handle both ends of this spectrum cheaply.

The classification result, `failure_class`, is written to `remediation_agent_session` for every run. Its values feed the router strategy (ADR-009), the Playbook task selector, and the downstream reflexion/learning pipeline. A wrong classification wastes tokens and may trigger the wrong automated fix; an `unknown` classification forces the most expensive model path but is always safe.

The time budget for classification is tight: it runs on the hot path between CI result receipt and the first model call. Any approach that introduces cold-start latency on the first classification of a session is a risk to the perceived responsiveness of the feature.

### Technical Context

- **Failure classes (v1 enum)**: `build_error`, `test_failure`, `type_error`, `lint`, `lockfile_conflict`, `flaky_test`, `missing_dependency`, `unknown`.
- **CI log input**: first 2 000 tokens of the failing job's combined stdout/stderr.
- **ONNX model provider**: already in the v1 provider set with `output_contract = classify_only`; runs in-process via `ort`/`candle`; carries no API key (`auth_type = none`).
- **Router strategy (ADR-009)**: routes to the cheapest account whose capability covers the `failure_class`; expects a populated `failure_class` before routing begins.
- **`remediation_agent_session`**: extended with `failure_class`, `classifier_source` (`heuristic` | `onnx` | `model`), and `classifier_confidence` fields for observability.
- **Air-gapped mode**: egress gate permits only `local_only` accounts; the classifier itself must never egress.
- **Apalis 0.6 job context**: classification runs synchronously inside the `RemediationJob` worker step before the first async model call.
- **Playbooks**: YAML task selectors filter on `when: { failure_class: [...] }`; the correct match requires an accurate class.

---

## Decision

**We will classify CI failures using a two-level cascade: a zero-cost heuristic fast-path (Level 1) followed by a local ONNX classifier fallback (Level 2). Only when both levels fail to reach the 0.7 confidence threshold is `failure_class` set to `unknown`, triggering escalation to the most capable configured inference model with raw logs as context.**

This approach eliminates inference cost on the 70–80 % of failures that are unambiguous, uses the already-budgeted ONNX provider for the remainder, and degrades gracefully to the full model path for genuinely novel cases — all without any network egress.

### Implementation Notes

**Level 1 — Heuristic pattern matching (synchronous, no cost)**

Implemented as a pure function in `crates/ampel-worker/src/remediation/classifier.rs`. Patterns are matched in priority order; the first match wins and returns immediately.

```rust
pub fn classify_heuristic(log: &str) -> Option<FailureClass> {
    let log_lower = log.to_ascii_lowercase();
    // Build errors: Rust compiler diagnostics
    if log.contains("error[E") || log.contains("failed to compile") || log.contains("cannot find") {
        return Some(FailureClass::BuildError);
    }
    // Type errors: TypeScript compiler output
    if log.contains("error TS") || log.contains("Type error") {
        return Some(FailureClass::TypeError);
    }
    // Lint: ESLint or Clippy deny-level
    if log_lower.contains("eslint") || log.contains("clippy::deny") {
        return Some(FailureClass::Lint);
    }
    // Lockfile conflicts: Cargo.lock or pnpm-lock.yaml skew
    if log_lower.contains("lock file") || log_lower.contains("lockfile")
        || log.contains("Cargo.lock") || log_lower.contains("pnpm-lock") {
        return Some(FailureClass::LockfileConflict);
    }
    // Test failures: nextest or Vitest output
    if log.contains("FAILED") && (log.contains("test ") || log.contains("tests/")) {
        return Some(FailureClass::TestFailure);
    }
    // Missing dependency: crate or npm package not found
    if log_lower.contains("no such crate") || log_lower.contains("cannot find crate")
        || log_lower.contains("module not found") || log_lower.contains("package not found") {
        return Some(FailureClass::MissingDependency);
    }
    None
}
```

Patterns should be expanded as new cases are observed in production. Adding a pattern is a single-line change with no model dependency.

**Level 2 — ONNX classifier (in-process, local, no egress)**

Invoked only when `classify_heuristic` returns `None`. The ONNX model is loaded once per worker process and held in an `Arc<OnnxClassifier>` on `AppState` (lazy-initialized on first use to avoid cold-start on startup).

```rust
pub async fn classify(
    log: &str,
    onnx: &OnnxClassifier,
) -> ClassificationResult {
    // Level 1
    if let Some(class) = classify_heuristic(log) {
        return ClassificationResult {
            class,
            source: ClassifierSource::Heuristic,
            confidence: 1.0,
        };
    }
    // Level 2: truncate to 2 000 tokens, run ONNX
    let tokens = truncate_to_tokens(log, 2_000);
    match onnx.classify(&tokens).await {
        Ok(dist) if dist.top_confidence() >= 0.7 => ClassificationResult {
            class: dist.top_class(),
            source: ClassifierSource::Onnx,
            confidence: dist.top_confidence(),
        },
        _ => ClassificationResult {
            class: FailureClass::Unknown,
            source: ClassifierSource::Onnx,
            confidence: 0.0,
        },
    }
}
```

**`unknown` escalation path**

When `failure_class = unknown`, the router selects the most capable configured model (Claude or Gemini in the default account ordering) and passes the raw log as additional context alongside the Playbook task prompt. The model is expected to both classify (implicitly, in its reasoning) and attempt a fix in a single pass. This is the most expensive path and should represent a small minority of runs.

**`remediation_agent_session` fields added**

| Column | Type | Notes |
|---|---|---|
| `failure_class` | `TEXT NOT NULL` | enum value |
| `classifier_source` | `TEXT NOT NULL` | `heuristic`, `onnx`, or `model` |
| `classifier_confidence` | `REAL` | 1.0 for heuristic, probability for ONNX, NULL for model path |

**ONNX model lifecycle**

- The ONNX model file is referenced via `model_provider_account.model_path` (not embedded in the binary).
- On worker startup the path is validated; a missing file emits a warning and disables Level 2 (falls through to `unknown` directly).
- The `OnnxClassifier` is wrapped in `once_cell::sync::OnceCell` for lazy load.
- The model is a lightweight text-classification head (e.g., a distilled BERT fine-tuned on CI log snippets); full code-generation models are never used at this layer.

**Metrics emitted**

- `ampel_classifier_source_total{source="heuristic"|"onnx"|"model"}` — counter
- `ampel_classifier_confidence_histogram{source="onnx"}` — histogram
- `ampel_classifier_unknown_total` — counter (the number that fell all the way through)

These feed the reflexion pipeline (planned Phase 5) and can surface patterns that should be promoted to Level 1 heuristics.

---

## Alternatives Considered

### Option A: Heuristic Only (Rejected)

**Pros**: Zero runtime dependencies beyond the pattern file; negligible latency; no model warm-up; trivially testable with unit tests.

**Cons**: Real-world CI logs produce 20–30 % `unknown` classifications where the failure message is tool-specific, locale-dependent, or wrapped in CI runner boilerplate. Each `unknown` routes to the most expensive model even for cases where an ONNX classifier would have succeeded. Reflexion metrics are noisy because `unknown` masks the true class distribution. Router precision degrades over time as novel tooling is introduced.

**Verdict**: Rejected. Acceptable as a temporary bootstrap but insufficient at scale. The 20–30 % unknown rate directly translates to unnecessary inference spend.

---

### Option B: ONNX Classifier Only (Rejected)

**Pros**: Single classification mechanism; consistent confidence scores; covers both obvious and ambiguous cases.

**Cons**: Cold-start latency on first load (model file I/O + tokenizer initialization) adds 200–600 ms to the first classification of a worker session, which is observable in the SSE stream. For the 70–80 % of failures that have a deterministic textual signature, running the ONNX model adds latency and CPU cost that provides no accuracy benefit over a regex match. Requires a trained model artifact to be provisioned alongside the binary, adding operational complexity that heuristics avoid entirely.

**Verdict**: Rejected as the sole mechanism. Retained as Level 2 fallback where it adds genuine value.

---

### Option C: Two-Level Cascade — Heuristic + ONNX Fallback (Accepted)

**Pros**: Zero cost on the common case; improved accuracy on the edge cases where heuristics return `None`; ONNX is already in the v1 provider set so no new operational surface is added; fully local and never egresses, satisfying air-gapped mode; `unknown` rate in practice should fall below 5 % rather than 20–30 %; classifier source is recorded for observability and reflexion.

**Cons**: Two code paths to maintain; ONNX model file must be provisioned and kept current; the 0.7 confidence threshold is a hyperparameter that may require tuning as the model and CI tooling evolve; lazy initialization means the first ONNX-path classification in a new worker process still incurs warm-up latency (mitigated by lazy load — it does not block startup).

**Verdict**: Accepted. The trade-off between accuracy and cost is the best achievable without network egress.

---

## Trade-off Analysis

| Aspect | Option A: Heuristic Only | Option B: ONNX Only | Option C: Two-Level (Chosen) |
|---|---|---|---|
| **Latency (common case)** | Sub-millisecond | 200–600 ms cold, ~5 ms warm | Sub-millisecond |
| **Latency (edge case)** | Sub-millisecond | ~5 ms warm | ~5 ms warm (after Level 1 miss) |
| **Unknown rate** | 20–30 % | ~5 % | < 5 % |
| **Inference cost** | Zero | Zero (local) | Zero (local) |
| **Egress risk** | None | None | None |
| **Air-gapped compatible** | Yes | Yes | Yes |
| **Operational complexity** | Low | Medium (model artifact) | Medium (model artifact) |
| **Maintainability** | Heuristics accumulate over time | Single model, retrain cycle | Heuristics + retrain cycle |
| **Reflexion signal quality** | Degraded (high unknown rate) | Good | Good |
| **Cold-start penalty** | None | Affects startup | Affects first ONNX-path call only |

---

## Consequences

### Positive

- Inference spend for classification is zero on the majority of failures.
- The `failure_class` value available to the router and Playbook selector is accurate enough to route correctly for > 95 % of runs.
- `classifier_source` and `classifier_confidence` in `remediation_agent_session` provide the observability signal needed to promote new patterns to Level 1 and to retrain the ONNX model over time.
- No network egress occurs at the classification stage, which is compatible with air-gapped deployments and requires no credential at this layer.
- The heuristic fast-path is fully unit-testable without any model dependency.

### Negative

- An ONNX model artifact must be provisioned alongside the worker binary. The model must be fine-tuned on CI log data and updated as new tooling is introduced.
- The 0.7 confidence threshold is a tuneable hyperparameter. Too low increases misclassification; too high increases `unknown` rate. Initial value is an informed estimate; production data will inform adjustment.
- Two code paths (heuristic and ONNX) must be maintained and kept consistent with each other and with the `FailureClass` enum.
- Lazy ONNX initialization means the first classification that falls through Level 1 in a fresh worker process incurs 200–600 ms of warm-up latency. This is one-time per process lifetime and does not affect steady-state throughput.

### Neutral

- The `FailureClass` enum is defined in `crates/ampel-core` and shared across the worker, API, and any future CLI tooling. Extending it with new values requires a SeaORM migration to update the stored string domain.
- Heuristic patterns should be reviewed and expanded as new CI failure modes are observed; this is low-effort but requires discipline to avoid pattern sprawl.
- The reflexion pipeline (planned Phase 5) will consume `classifier_source` and `classifier_confidence` to measure drift and trigger retraining; this ADR does not define that pipeline.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| ONNX model file missing at worker startup | Medium | Emit a startup warning and disable Level 2 gracefully; fall through to `unknown` rather than crashing |
| Confidence threshold set too low — misclassification | High | Start at 0.7; instrument `ampel_classifier_confidence_histogram` to observe distribution before tuning |
| Confidence threshold set too high — excess `unknown` rate | Medium | Monitor `ampel_classifier_unknown_total`; lower threshold if rate stays above 10 % in production |
| Heuristic pattern false-positive (e.g., log message contains "error[E" in a comment) | Low | Patterns are ordered and short-circuit; add negative anchors if false positives are observed in production |
| ONNX cold-start latency makes the first agentic run feel slow | Low | Lazy-load on first use; SSE stream communicates "classifying…" status to the UI during warm-up |
| `FailureClass::Unknown` rate stays above expectations | Medium | Review Level 1 patterns; retrain ONNX on production log samples; threshold tuning |
| Air-gapped mode operator does not provision ONNX model | Low | Documented as a prerequisite in the air-gapped deployment guide; startup check validates the path |

---

## Related ADRs

- ADR-009: Model Router Strategy — consumes `failure_class` to select the cheapest appropriate account from the configured provider list.
- ADR-010: Playbook Design and Execution — Playbook task `when:` selectors filter on `failure_class`; an accurate class is required for correct task dispatch.
- ADR-011: Sandbox Isolation for Agentic Tier — the classifier runs inside the same Apalis worker job as the harness, not inside the sandbox; the sandbox boundary begins when the fixer model's edits are applied.
