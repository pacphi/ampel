# ADR-013: Async Trait Implementation Strategy

**Status**: Accepted
**Date**: 2026-06-24
**Deciders**: Architecture Team
**Technical Story**: Establish a consistent async trait strategy for the two new traits (`RemediationCapable` and `ModelProvider`) introduced by the Fleet PR Remediation Loops feature, while remaining consistent with the existing `GitProvider` idiom.

---

## Context

### Problem Statement

The Fleet PR Remediation Loops feature introduces two new traits with async methods:

- `RemediationCapable` — a supertrait of `GitProvider` that adds PR triage, consolidation, and remediation operations. Instances are stored as `Arc<dyn RemediationCapable>` inside `AppState` and `WorkerState` to support the factory pattern used across handler and Apalis worker code.
- `ModelProvider` — an abstraction over inference backends (Claude, Gemini, Ollama, ONNX classifier). Instances are also stored as `Arc<dyn ModelProvider>` and selected at runtime based on org-level configuration.

Both traits require `dyn` dispatch. Rust's object-safety rules mean that a naive `async fn` in a trait definition (AFIT / RPITIT, stabilized in Rust 1.75) is **not** object-safe by default. Using `Box<dyn RemediationCapable>` or `Arc<dyn ModelProvider>` with bare AFIT requires additional machinery — either `DynCompatExt` adapters, manual `BoxFuture` return types, or the unstable `dyn*` feature.

At the same time, several purely internal helpers (consolidation pipeline steps, playbook-rendering helpers, sandbox command builders) are used only through concrete types or generic bounds and do not need `dyn` dispatch. For these, AFIT is the correct choice: it avoids a heap allocation per call and produces cleaner code.

The decision must be consistent with the existing `GitProvider` trait, which already uses `#[async_trait]` and is stored as `Arc<dyn GitProvider>`. Introducing a second idiom for dyn-compatible traits would fragment the codebase and confuse contributors.

### Technical Context

- **Rust version**: 1.95.0 (pinned via `rust-toolchain.toml`). AFIT is stable but `dyn`-compatible async traits are not.
- **Existing idiom**: `GitProvider` uses `#[async_trait]` from the `async-trait` crate and is stored as `Arc<dyn GitProvider + Send + Sync>`. All provider implementations, tests, and handler code follow this pattern.
- **`#[async_trait]` overhead**: one `Box::pin` heap allocation per async method call. For IO-bound operations (HTTP calls to GitHub/GitLab/Bitbucket, LLM inference, database queries) this overhead is immeasurable relative to network latency.
- **Internal helpers**: consolidation steps, playbook rendering (minijinja), sandbox command builders — all consumed through concrete types or `impl Trait` generics; no `dyn` involved.
- **Apalis 0.6 jobs** (`RepositoryPollJob`, `CleanupJob`, etc.) are concrete structs; their internal helpers are not exposed as trait objects.
- **`dyn_async_traits` / `async_fn_in_trait` workarounds** (e.g., `DynCompatExt` from the `dynosaur` crate) exist but are experimental, add API surface complexity, and are not yet idiomatic in the Rust ecosystem as of 1.95.

---

## Decision

**We adopt a hybrid strategy: `#[async_trait]` for every trait that requires `dyn` dispatch; native AFIT for all other async traits and for generic-bound-only usage.**

This is consistent with the existing `GitProvider` pattern and requires no changes to current handler or worker code. It keeps the rule simple enough to apply mechanically: if you intend to store the trait behind `Arc<dyn ...>` or `Box<dyn ...>`, annotate it with `#[async_trait]`; otherwise use native `async fn`.

### Implementation Notes

**Rule of thumb**

| Trait usage | Approach |
|---|---|
| Stored as `Arc<dyn Trait>` / `Box<dyn Trait>` | `#[async_trait]` |
| Used only as `impl Trait` bound or concrete type | AFIT (native `async fn`) |
| Internal helper structs with async methods | AFIT (native `async fn`) |

**`RemediationCapable` (dyn-compatible)**

```rust
use async_trait::async_trait;
use crate::providers::GitProvider;

#[async_trait]
pub trait RemediationCapable: GitProvider {
    async fn triage_prs(&self, repo_id: Uuid) -> Result<TriageReport, RemediationError>;
    async fn consolidate_prs(&self, plan: &ConsolidationPlan) -> Result<MergeResult, RemediationError>;
    async fn apply_playbook(&self, playbook: &Playbook, ctx: &RemediationContext)
        -> Result<RemediationOutcome, RemediationError>;
}
```

Stored in `AppState`:

```rust
pub struct AppState {
    // ...existing fields...
    pub remediation: Arc<dyn RemediationCapable + Send + Sync>,
}
```

**`ModelProvider` (dyn-compatible)**

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn infer(&self, prompt: &str, opts: &InferenceOptions) -> Result<String, ModelError>;
    async fn classify(&self, input: &ClassifyInput) -> Result<ClassifyOutput, ModelError>;
    fn provider_name(&self) -> &str;
}
```

Factory selects implementation at startup based on org config:

```rust
pub fn build_model_provider(cfg: &ModelConfig) -> Arc<dyn ModelProvider + Send + Sync> {
    match cfg.backend {
        ModelBackend::Claude  => Arc::new(ClaudeProvider::new(cfg)),
        ModelBackend::Gemini  => Arc::new(GeminiProvider::new(cfg)),
        ModelBackend::Ollama  => Arc::new(OllamaProvider::new(cfg)),
        ModelBackend::Onnx    => Arc::new(OnnxProvider::new(cfg)),
    }
}
```

**Internal helpers (AFIT)**

Consolidation pipeline steps are generic over a concrete executor type, not stored as trait objects:

```rust
// Internal only — no dyn needed
trait ConsolidationStep {
    async fn execute(&self, state: &mut ConsolidationState) -> Result<(), ConsolidationError>;
}
```

Playbook rendering helpers are concrete structs:

```rust
impl PlaybookRenderer {
    async fn render(&self, template: &str, ctx: &serde_json::Value) -> Result<String, RenderError> {
        // minijinja rendering (CPU-bound, but async context needed for unified error path)
    }
}
```

**Contributing guidance**

Add the following note to `docs/architecture/README.md` or the crate-level `lib.rs` doc comment for `ampel-providers`:

> **Async trait rule**: Use `#[async_trait]` when the trait will be stored behind `Arc<dyn ...>` or `Box<dyn ...>`. Use native `async fn in trait` (AFIT) for all other async traits. This mirrors the existing `GitProvider` pattern and keeps `dyn` dispatch sound on stable Rust.

---

## Alternatives Considered

### Option A: AFIT everywhere with `DynCompatExt` workarounds (Rejected)

Use native `async fn` in all trait definitions and apply a `DynCompatExt` adapter (e.g., from the `dynosaur` crate or a hand-rolled `BoxFuture` wrapper) wherever `dyn` dispatch is needed.

**Pros**:
- Single trait-definition style across the codebase.
- Avoids the `async-trait` proc-macro dependency.
- Forward-compatible: when `dyn`-safe AFIT lands on stable, the adapters can be removed.

**Cons**:
- Every `dyn` call site requires an adapter type or manual `BoxFuture` return annotation.
- `dynosaur` and similar crates are not yet widely adopted and add an additional dependency.
- Incompatible with the existing `GitProvider` pattern without a large-scale refactor.
- Adds complexity for contributors unfamiliar with the workaround.

**Verdict**: Rejected. The complexity cost outweighs the consistency benefit given the IO-bound nature of the workload and the existing codebase idiom.

### Option B: `#[async_trait]` everywhere (Rejected)

Apply `#[async_trait]` to every async trait, including internal non-dyn helpers.

**Pros**:
- Single rule, zero cognitive overhead for contributors.
- Consistent with the existing `GitProvider` pattern.

**Cons**:
- Unnecessary heap allocation for internal code paths that never need `dyn` dispatch (e.g., consolidation steps called thousands of times per batch job).
- Discourages adoption of stable AFIT, which is the Rust idiom going forward.

**Verdict**: Rejected. The blanket rule is wasteful for internal hot paths and prevents natural migration toward AFIT over time.

### Option C: Hybrid — `#[async_trait]` for dyn-compatible; AFIT for non-dyn internal code (Accepted)

**Pros**:
- Consistent with existing `GitProvider` pattern — no refactoring required.
- Zero overhead for internal non-dyn code.
- Clear, mechanical rule that contributors can apply without deep Rust async knowledge.
- Forward-compatible: when stable `dyn`-safe AFIT ships, only the dyn-compatible traits need updating.

**Cons**:
- Two idioms in the codebase; contributors must know which to apply.
- `async-trait` proc-macro dependency retained.

**Verdict**: Accepted. The rule is simple, the existing precedent is clear, and the overhead is irrelevant for IO-bound operations.

---

## Trade-off Analysis

| Aspect | Option A: AFIT + DynCompatExt | Option B: #[async_trait] everywhere | Option C: Hybrid (chosen) |
|---|---|---|---|
| Object-safety complexity | High (adapter boilerplate at every dyn site) | None | Low (rule: dyn → #[async_trait]) |
| Per-call overhead (dyn paths) | Low (same Box::pin as #[async_trait]) | One Box::pin alloc per call | One Box::pin alloc per dyn call |
| Per-call overhead (internal paths) | None | One Box::pin alloc per call | None |
| Consistency with GitProvider | Breaks existing pattern | Maintains existing pattern | Maintains existing pattern |
| Refactor risk | High (must touch all existing GitProvider sites) | None | None |
| Forward-compatibility with stable dyn AFIT | Best (traits already AFIT) | Requires migration later | Requires migration for dyn traits later |
| Contributor clarity | Low (two concepts plus adapters) | High (one rule) | High (one simple rule) |
| Dependencies added | dynosaur or manual BoxFuture | None beyond existing async-trait | None beyond existing async-trait |

---

## Consequences

### Positive

- `RemediationCapable` and `ModelProvider` integrate cleanly with the factory and `AppState`/`WorkerState` patterns already established for `GitProvider`.
- Internal consolidation pipeline code avoids unnecessary heap allocations, which matters for batch jobs processing hundreds of PRs.
- The contributing rule is simple enough to document in a single sentence and apply without ambiguity.
- No changes required to existing `GitProvider` implementations, tests, or handler code.

### Negative

- The `async-trait` crate remains a dependency; contributors must be aware that annotating a dyn-compatible trait without `#[async_trait]` will produce a compile error.
- The codebase will carry two async idioms until Rust stabilizes dyn-safe AFIT, at which point a migration will be needed for `GitProvider`, `RemediationCapable`, and `ModelProvider`.

### Neutral

- The `async-trait` proc-macro generates `Pin<Box<dyn Future>>` return types; this is visible in IDE hover documentation, which can surprise contributors. A code comment on each annotated trait directing readers to this ADR mitigates confusion.
- ONNX classifier calls are synchronous internally but wrapped in `spawn_blocking`; the `ModelProvider::classify` method is still declared `async` for a uniform interface.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| Contributor adds AFIT to a dyn-compatible trait by mistake | Medium | Compile error at the `Arc<dyn ...>` declaration catches this immediately. Document the rule in the crate README and this ADR. |
| `async-trait` crate abandonment or incompatibility with a future Rust edition | Low | The crate is maintained by dtolnay and widely adopted. If it becomes unmaintained, the migration path to native dyn-safe AFIT is straightforward once that feature stabilizes. |
| Performance regression from Box::pin on hot paths | Low | All dyn-dispatched calls cross a network boundary or touch the database; the allocation overhead is orders of magnitude below IO latency. Internal hot paths use AFIT and are unaffected. |
| Supertrait constraint complexity (`RemediationCapable: GitProvider`) with `#[async_trait]` | Low | Both traits use `#[async_trait]`; the macro handles supertrait composition correctly. Covered by existing GitProvider test patterns. |
| Migration effort when dyn-safe AFIT stabilizes | Low | Confined to three traits (`GitProvider`, `RemediationCapable`, `ModelProvider`) and their implementations. The hybrid rule makes future migration straightforward. |

---

## Related ADRs

- ADR-001: Locale Middleware State Access Pattern — establishes the `AppState` extension pattern used to store `Arc<dyn ...>` trait objects in Axum handlers.
