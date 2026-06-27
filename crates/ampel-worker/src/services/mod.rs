//! Worker-side services for autonomous PR remediation (Phase 2).
//!
//! - [`provider_adapter`]: adapts a `RemediationCapable` provider into the
//!   `ampel_core` `RemediationProvider` seam, capability-gated.
//! - [`sandbox_runner`]: Podman/Docker [`SandboxRunner`] + pure consolidation
//!   logic (lockfile/regen/merge-sequence/runtime detection).
//! - [`remediation_executor`]: drives one run through the state machine.

pub mod provider_adapter;
pub mod remediation_executor;
pub mod sandbox_runner;

pub use provider_adapter::{remediation_capable_provider, ProviderAdapter};
pub use remediation_executor::{RemediationExecutor, RunOutcome};
pub use sandbox_runner::{PodmanSandboxRunner, SandboxConfig};
