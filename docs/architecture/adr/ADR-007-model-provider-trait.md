# ADR-007: ModelProvider Trait Design (Inference-Kind vs Agent-Kind)

**Status**: Accepted  
**Date**: 2026-06-24  
**Deciders**: Architecture Team  
**Technical Story**: The agentic remediation tier (Phase 4) integrates with multiple AI
providers that differ fundamentally in integration style. A single trait must accommodate
both raw inference APIs (which Ampel drives step-by-step) and self-driving agents (which
run their own inner loop) without coupling core harness logic to any specific provider.

---

## Context

### Problem Statement

The `RemediationAgentHarness` needs to call AI providers to fix CI failures.
Two integration kinds exist in the provider lineup:

**Inference providers** (Claude Messages API, Gemini API, Ollama): Ampel sends a prompt
with context (failing CI logs, diff, changed files), the model returns tool calls or a
unified diff, Ampel applies the edits, pushes, and re-checks CI. The harness owns the
iterate→apply→push loop.

**Agent providers** (Codex CLI, Claude Code headless — Phase 5): The provider is given a
working tree and a goal condition (`provider_ci_green`), then runs its own tool-calling
loop until the goal is met or budget is exhausted. The harness hands over the worktree
and waits.

A single `generate()` method cannot cleanly model agent-kind handoff: an agent's
response is not a single turn — it is the final outcome after N internal turns that may
take minutes. Budget enforcement, progress reporting, and the worktree handoff are
all structurally different.

### Technical Context

- Harness is in `ampel-core::services::remediation_agent_harness`
- Providers stored as `Arc<dyn ModelProvider>` in `WorkerState` (requires `dyn`
  compatibility; `#[async_trait]` required per ADR-013)
- V1 providers: Claude (inference, hosted), Gemini (inference, hosted), Ollama
  (inference, local), ONNX (inference, in-process, classifier only)
- V2 providers (future): Codex CLI (agent), generic OpenAI-compatible (inference)

---

## Decision

**Define a `ModelProvider` trait with two dispatch methods — `infer()` for
inference-kind and `run_agent()` for agent-kind — plus a `capabilities()` descriptor.
The harness checks `capabilities().kind` and routes accordingly. Default implementations
return `Err(ProviderError::NotSupported)` so providers need only implement one method.**

### Trait Definition

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn id(&self) -> &str;
    fn capabilities(&self) -> ModelCaps;
    async fn validate(&self, creds: &ModelCredentials) -> Result<ValidationResult>;

    // Inference-kind: one prompt-response turn; harness drives the loop.
    async fn infer(
        &self,
        creds: &ModelCredentials,
        req: InferenceRequest,
    ) -> Result<InferenceResponse> {
        Err(ProviderError::NotSupported("infer"))
    }

    // Agent-kind: delegate the entire inner loop; provider returns when done.
    async fn run_agent(
        &self,
        creds: &ModelCredentials,
        task: AgentTask,
        worktree: &Worktree,
        budget: &Budget,
    ) -> Result<AgentOutcome> {
        Err(ProviderError::NotSupported("run_agent"))
    }
}

pub struct ModelCaps {
    pub kind: ProviderKind,        // Inference | Agent
    pub modality: Modality,        // HostedApi | LocalServer | InProcess
    pub tool_use: bool,
    pub code_edit: bool,
    pub max_context_tokens: u32,
    pub streaming: bool,
    pub cost: CostModel,           // PerToken { in_usd_per_1m, out_usd_per_1m } | Free
    pub egress: Egress,            // External | LocalOnly
}
```

### Harness Routing

```rust
match provider.capabilities().kind {
    ProviderKind::Inference => {
        // Harness drives iterate→apply→push→verify loop
        loop {
            let resp = provider.infer(&creds, build_request(&ctx, playbook)).await?;
            apply_edits(&resp, &worktree).await?;
            push_branch().await?;
            let result = verification_service.verify(&repo, &consolidated_ref).await?;
            if result.ampel_status == AmpelStatus::Green { break; }
            if budget.exhausted() { return Ok(AgentOutcome::BudgetExhausted); }
            ctx.feed_back_ci_logs(&result);
        }
    }
    ProviderKind::Agent => {
        // Provider drives its own loop; harness waits for outcome
        let outcome = provider.run_agent(&creds, task, &worktree, &budget).await?;
        // Re-verify regardless of agent's claim — agent cannot self-certify (ADR-010)
    }
}
```

### Output Contracts

| Contract | Used by | Description |
|----------|---------|-------------|
| `ToolUse` | Claude, Gemini | Structured tool calls (`read_file`, `write_file`, `run_command`) |
| `UnifiedDiff` | Ollama | Patch in unified diff format; harness applies via `patch -p1` |
| `ClassifyOnly` | ONNX | Returns `FailureClass` + confidence score; no file edits |

### V1 Provider Matrix

| Provider | Kind | Modality | Egress | Output contract |
|----------|------|----------|--------|-----------------|
| Claude (claude-sonnet-4-6) | Inference | HostedApi | External | ToolUse |
| Gemini (gemini-2.0-flash) | Inference | HostedApi | External | ToolUse |
| Ollama (qwen2.5-coder default) | Inference | LocalServer | LocalOnly | UnifiedDiff |
| ONNX (classifier model) | Inference | InProcess | LocalOnly | ClassifyOnly |

---

## Alternatives Considered

### Option A: Single uniform `generate()` method (Rejected)

**Approach**: One `async fn generate(prompt, context) -> Response`.

**Cons**:
- ❌ Cannot model agent-kind handoff — an agent runs for N turns; there is no
  "one response" to return until the agent is done
- ❌ Budget enforcement and worktree handoff must be special-cased per provider

**Verdict**: REJECTED.

### Option B: Two-kind trait with `capabilities()` dispatcher (ACCEPTED)

**Pros**:
- ✅ Clean separation between inference-kind and agent-kind integration patterns
- ✅ Harness routing is a single `match capabilities().kind`
- ✅ `capabilities()` drives egress enforcement, output-contract selection, and cost
  tracking automatically
- ✅ Extensible: new kinds add a method and variant; existing implementations unaffected

**Cons**:
- ⚠️ Providers must implement only one dispatch method; the other returns `NotSupported`
  (enforced by default impls)

**Verdict**: ACCEPTED.

### Option C: Enum dispatch (Rejected)

**Approach**: `enum ModelProvider { Claude(...), Gemini(...), ... }`

**Cons**:
- ❌ Violates open/closed principle — adding a provider requires modifying core enum
- ❌ Cannot store as `Arc<dyn ModelProvider>` in `WorkerState`

**Verdict**: REJECTED.

---

## Trade-off Analysis

| Aspect | Option A (single method) | Option B (two-kind) ⭐ | Option C (enum) |
|--------|--------------------------|----------------------|-----------------|
| **Agent-kind support** | ❌ Cannot model | ✅ Full | ✅ Full |
| **Extensibility** | ✅ Simple | ✅ Open/closed | ❌ Closed |
| **Harness clarity** | ❌ Mixed concerns | ✅ Clean routing | ⚠️ Match arms per variant |
| **`dyn` storage** | ✅ Yes | ✅ Yes | ❌ Awkward |
| **Egress enforcement** | ❌ Manual per-call | ✅ Via `capabilities()` | ⚠️ Per-variant |

---

## Consequences

### Positive

- Harness routing is explicit and testable — mocks implement only the relevant kind
- `capabilities()` enables automatic egress enforcement without provider-specific
  code in the harness
- Adding a v2 agent-kind provider requires only implementing `run_agent()` on a new struct

### Negative

- Two dispatch methods on the trait; providers implement one and stub the other via
  the default `NotSupported` implementation

### Neutral

- `#[async_trait]` required for `Arc<dyn ModelProvider>` storage (ADR-013)

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Provider implements wrong kind | Low | Integration tests assert `capabilities().kind` matches behaviour |
| `run_agent()` budget exhaustion not surfaced | Medium | `AgentOutcome::BudgetExhausted` variant; harness checks before calling verify |
| Egress bypass via misconfigured provider | Medium | `pub(crate)` visibility on concrete impls; egress checked at harness level |

---

## Related ADRs

- ADR-008: Model provider credential storage — `ModelCredentials` injected per call
- ADR-009: Model provider v1 scope — Claude + Gemini + Ollama + ONNX with their kinds
- ADR-012: Failure classification — `ClassifyOnly` contract from ONNX provider
- ADR-013: `#[async_trait]` strategy — required for `Arc<dyn ModelProvider>`
- ADR-014: Air-gapped governance — `capabilities().egress` is the enforcement point
