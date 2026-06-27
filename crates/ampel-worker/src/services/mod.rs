//! Worker-side services for autonomous PR remediation (Phase 2).
//!
//! - [`provider_adapter`]: adapts a `RemediationCapable` provider into the
//!   `ampel_core` `RemediationProvider` seam, capability-gated.
//! - [`sandbox_runner`]: Podman/Docker [`SandboxRunner`] + pure consolidation
//!   logic (lockfile/regen/merge-sequence/runtime detection).
//! - [`remediation_executor`]: drives one run through the state machine.
//! - [`notifier`]: notification delivery seam (Slack via `ampel-core`, or noop).

pub mod agent_harness;
pub mod failure_classifier;
pub mod notifier;
pub mod playbook;
pub mod playbook_resolver;
pub mod provider_adapter;
pub mod remediation_executor;
pub mod sandbox_runner;

// Re-exported for library consumers (slice-3 wiring); the bin target does not
// use these yet, hence the allow.
#[allow(unused_imports)]
pub use agent_harness::{AgentWorktree, CiVerifier, RemediationAgentHarness, VerificationStatus};
#[allow(unused_imports)]
pub use failure_classifier::CascadeClassifier;
#[allow(unused_imports)]
pub use playbook::{clamp_tools, LoopConfig, Playbook, PlaybookTask, ToolsPolicy};
#[allow(unused_imports)]
pub use playbook_resolver::{
    build_system_instruction, embedded_default_yaml, render_instructions, resolve, PlaybookContext,
    PlaybookScope,
};
pub use provider_adapter::{remediation_capable_provider, ProviderAdapter};
pub use remediation_executor::{RemediationExecutor, RunOutcome};
pub use sandbox_runner::{PodmanSandboxRunner, SandboxConfig};
