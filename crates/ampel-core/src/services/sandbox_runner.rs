//! Sandboxed consolidation abstraction (ADR-003 / ADR-005).
//!
//! The mechanical "merge N PR branches in an isolated environment" work is
//! expressed as the [`SandboxRunner`] trait. The real Podman-backed
//! implementation is a later slice that lives in `ampel-worker` (it shells out
//! to `git`/`podman`); `ampel-core` only owns the trait + value objects + a
//! deterministic [`FakeSandboxRunner`] so the orchestration brain is fully
//! unit-testable with no containers, subprocesses, or network.
//!
//! Credentials are carried in an opaque [`CredentialHandle`] whose `Debug` impl
//! redacts the secret — a PAT must never reach a log line.

use crate::errors::AmpelResult;
use crate::remediation::{MergeDisposition, PrRef};
use async_trait::async_trait;
use std::fmt;
use uuid::Uuid;

/// An opaque, log-safe credential carrier (e.g. a decrypted PAT).
///
/// Deliberately does **not** derive `Debug`/`Serialize`; the manual `Debug`
/// redacts the value so accidental `{:?}` formatting can never leak the secret.
#[derive(Clone)]
pub struct CredentialHandle(String);

impl CredentialHandle {
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// Explicit, greppable accessor for the few places that genuinely need the
    /// plaintext (e.g. building a git askpass env inside the sandbox).
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CredentialHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CredentialHandle(***redacted***)")
    }
}

/// Everything the sandbox needs to perform one consolidation.
#[derive(Clone, Debug)]
pub struct ConsolidationSpec {
    pub run_id: Uuid,
    pub clone_url: String,
    pub default_branch: String,
    /// Source PRs in deterministic merge order (oldest-first).
    pub prs: Vec<PrRef>,
    /// Deterministic target branch: `ampel/remediation/<run_id>`.
    pub branch_name: String,
    /// Opaque credential — never logged in plaintext.
    pub credential: CredentialHandle,
}

impl ConsolidationSpec {
    /// The deterministic branch name formula (ADR-005).
    pub fn branch_name_for(run_id: Uuid) -> String {
        format!("ampel/remediation/{run_id}")
    }

    /// Build a spec with the canonical branch name derived from `run_id`.
    pub fn new(
        run_id: Uuid,
        clone_url: impl Into<String>,
        default_branch: impl Into<String>,
        prs: Vec<PrRef>,
        credential: CredentialHandle,
    ) -> Self {
        Self {
            branch_name: Self::branch_name_for(run_id),
            run_id,
            clone_url: clone_url.into(),
            default_branch: default_branch.into(),
            prs,
            credential,
        }
    }
}

/// Result of a sandbox consolidation run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsolidationOutcome {
    pub branch_name: String,
    /// The consolidating PR number, if one was opened.
    pub consolidated_pr_number: Option<i64>,
    /// Per-source-PR disposition (pr_number -> outcome).
    pub dispositions: Vec<(i64, MergeDisposition)>,
    /// HEAD SHA of the consolidated branch after all merges.
    pub head_sha: String,
}

/// Runs a consolidation in an isolated environment.
#[async_trait]
pub trait SandboxRunner: Send + Sync {
    async fn run_consolidation(&self, spec: ConsolidationSpec)
        -> AmpelResult<ConsolidationOutcome>;
}

#[cfg(any(test, feature = "test-utils"))]
pub use fake::FakeSandboxRunner;

#[cfg(any(test, feature = "test-utils"))]
mod fake {
    //! Deterministic in-process fake — no subprocess, no container.

    use super::*;
    use crate::errors::AmpelError;
    use std::sync::Mutex;

    /// Returns a deterministic outcome (all PRs `Consolidated`) and records the
    /// spec it last received so tests can assert on it.
    pub struct FakeSandboxRunner {
        consolidated_pr_number: Option<i64>,
        head_sha: String,
        /// When set, `run_consolidation` returns this error instead of an outcome
        /// (simulates a sandbox/infra crash for chaos tests).
        error: Option<String>,
        last_spec: Mutex<Option<ConsolidationSpec>>,
    }

    impl Default for FakeSandboxRunner {
        fn default() -> Self {
            Self {
                consolidated_pr_number: Some(9001),
                head_sha: "fakehead0000000000000000000000000000cafe".to_string(),
                error: None,
                last_spec: Mutex::new(None),
            }
        }
    }

    impl FakeSandboxRunner {
        pub fn new() -> Self {
            Self::default()
        }

        /// Configure the consolidated PR number and head SHA the fake returns.
        pub fn with_outcome(
            consolidated_pr_number: Option<i64>,
            head_sha: impl Into<String>,
        ) -> Self {
            Self {
                consolidated_pr_number,
                head_sha: head_sha.into(),
                error: None,
                last_spec: Mutex::new(None),
            }
        }

        /// A fake that fails the consolidation with `message` (chaos/infra crash).
        pub fn failing(message: impl Into<String>) -> Self {
            Self {
                error: Some(message.into()),
                ..Self::default()
            }
        }

        /// The spec passed to the most recent `run_consolidation` call.
        pub fn last_spec(&self) -> Option<ConsolidationSpec> {
            self.last_spec.lock().unwrap().clone()
        }

        /// Whether the runner was ever invoked.
        pub fn was_invoked(&self) -> bool {
            self.last_spec.lock().unwrap().is_some()
        }
    }

    #[async_trait]
    impl SandboxRunner for FakeSandboxRunner {
        async fn run_consolidation(
            &self,
            spec: ConsolidationSpec,
        ) -> AmpelResult<ConsolidationOutcome> {
            if let Some(message) = &self.error {
                *self.last_spec.lock().unwrap() = Some(spec);
                return Err(AmpelError::InternalError(message.clone()));
            }
            let dispositions = spec
                .prs
                .iter()
                .map(|p| (p.number as i64, MergeDisposition::Consolidated))
                .collect();
            let outcome = ConsolidationOutcome {
                branch_name: spec.branch_name.clone(),
                consolidated_pr_number: self.consolidated_pr_number,
                dispositions,
                head_sha: self.head_sha.clone(),
            };
            *self.last_spec.lock().unwrap() = Some(spec);
            Ok(outcome)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr(n: i32) -> PrRef {
        PrRef {
            number: n,
            title: format!("PR {n}"),
            branch: format!("feature/{n}"),
        }
    }

    #[test]
    fn should_build_deterministic_branch_name() {
        let id = Uuid::nil();
        assert_eq!(
            ConsolidationSpec::branch_name_for(id),
            format!("ampel/remediation/{id}")
        );
    }

    #[test]
    fn should_redact_credential_in_debug() {
        let cred = CredentialHandle::new("super-secret-pat");
        let rendered = format!("{cred:?}");
        assert!(!rendered.contains("super-secret-pat"));
        assert!(rendered.contains("redacted"));
    }

    #[test]
    fn should_keep_credential_redacted_inside_spec_debug() {
        let spec = ConsolidationSpec::new(
            Uuid::nil(),
            "https://example.test/repo.git",
            "main",
            vec![pr(1)],
            CredentialHandle::new("ghp_secret_value"),
        );
        assert!(!format!("{spec:?}").contains("ghp_secret_value"));
    }

    #[tokio::test]
    async fn should_return_all_consolidated_and_record_spec() {
        // Arrange
        let runner = FakeSandboxRunner::new();
        let spec = ConsolidationSpec::new(
            Uuid::new_v4(),
            "https://example.test/repo.git",
            "main",
            vec![pr(1), pr(2)],
            CredentialHandle::new("pat"),
        );

        // Act
        let outcome = runner.run_consolidation(spec.clone()).await.unwrap();

        // Assert
        assert_eq!(outcome.dispositions.len(), 2);
        assert!(outcome
            .dispositions
            .iter()
            .all(|(_, d)| *d == MergeDisposition::Consolidated));
        assert!(runner.was_invoked());
        assert_eq!(runner.last_spec().unwrap().branch_name, spec.branch_name);
    }
}
