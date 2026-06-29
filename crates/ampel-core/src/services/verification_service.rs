//! CI verification logic (ADR-010 TOCTOU guard).
//!
//! Pure, side-effect-free logic that normalizes provider CI checks into a
//! traffic-light [`CiVerificationResult`] and answers the only question that
//! matters at the merge gate: **is it safe to merge right now?**
//!
//! Key safety rule (ADR-010): a *missing* required check is treated as **red**,
//! never yellow — we never merge on the assumption that an absent check will
//! eventually pass. The result also carries the `ref_sha` it was computed
//! against so a caller can detect a Time-Of-Check/Time-Of-Use race
//! ([`VerificationService::reverify_sha_matches`]).
//!
//! This module deliberately does **not** depend on `ampel-providers`. Callers
//! adapt provider CI payloads into [`RawCiCheck`] (which mirrors the
//! `ProviderCICheck` shape: `name` / `status` / `conclusion`).

use crate::models::AmpelStatus;
use serde::{Deserialize, Serialize};

/// A raw, provider-agnostic CI check as handed to the verifier.
///
/// Mirrors the `ampel-providers::ProviderCICheck` shape without creating a
/// dependency on that crate. `status` is the provider run state
/// (`queued` / `in_progress` / `completed`); `conclusion` is the terminal
/// outcome when `status == "completed"`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawCiCheck {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

impl RawCiCheck {
    pub fn new(
        name: impl Into<String>,
        status: impl Into<String>,
        conclusion: Option<&str>,
    ) -> Self {
        Self {
            name: name.into(),
            status: status.into(),
            conclusion: conclusion.map(str::to_string),
        }
    }
}

/// Normalized terminal/in-flight status for a single check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pending,
    Running,
    Green,
    Red,
    Skipped,
    Cancelled,
}

impl CheckStatus {
    /// Map a provider `(status, conclusion)` pair into a normalized status.
    ///
    /// Unknown / missing conclusions on a completed check resolve to `Red`
    /// (fail-closed) so verification never silently passes on bad data.
    fn from_provider(status: &str, conclusion: Option<&str>) -> Self {
        match status {
            "queued" | "pending" | "waiting" | "requested" => Self::Pending,
            "in_progress" | "running" => Self::Running,
            "completed" | "success" | "failure" | "neutral" => match conclusion {
                Some("success") | Some("neutral") => Self::Green,
                Some("skipped") => Self::Skipped,
                Some("cancelled") | Some("canceled") => Self::Cancelled,
                // failure, timed_out, action_required, stale, startup_failure,
                // unknown, or absent => fail-closed.
                _ => {
                    // A bare "success"/"failure" arriving in the status slot.
                    match status {
                        "success" => Self::Green,
                        "failure" => Self::Red,
                        _ => Self::Red,
                    }
                }
            },
            _ => Self::Pending,
        }
    }
}

/// A provider check collapsed into the common shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedCiCheck {
    pub name: String,
    pub status: CheckStatus,
    pub required: bool,
}

/// A point-in-time snapshot of CI state for a specific commit SHA.
///
/// Two snapshots must be compared by `ref_sha`; a changed SHA means the older
/// snapshot is stale and MUST be discarded (TOCTOU).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiVerificationResult {
    pub ref_sha: String,
    pub checks: Vec<NormalizedCiCheck>,
    pub all_required_green: bool,
    pub mergeable: bool,
    pub ampel_status: AmpelStatus,
}

impl CiVerificationResult {
    /// The single merge-gate predicate (ADR-010):
    /// green overall **and** every required check green **and** mergeable.
    pub fn is_safe_to_merge(&self) -> bool {
        self.ampel_status == AmpelStatus::Green && self.all_required_green && self.mergeable
    }
}

/// Stateless verifier. Holds no connections — pure logic.
#[derive(Clone, Copy, Debug, Default)]
pub struct VerificationService;

impl VerificationService {
    pub fn new() -> Self {
        Self
    }

    /// Normalize `checks` against the `required_check_names` branch-protection
    /// set and compute the aggregate traffic-light status.
    ///
    /// A required check that is absent from `checks` forces `ampel_status` to
    /// [`AmpelStatus::Red`] and `all_required_green` to `false`.
    pub fn verify(
        &self,
        checks: &[RawCiCheck],
        required_check_names: &[String],
        mergeable: bool,
        ref_sha: impl Into<String>,
    ) -> CiVerificationResult {
        let normalized: Vec<NormalizedCiCheck> = checks
            .iter()
            .map(|c| NormalizedCiCheck {
                name: c.name.clone(),
                status: CheckStatus::from_provider(&c.status, c.conclusion.as_deref()),
                required: required_check_names.contains(&c.name),
            })
            .collect();

        // A required check is "missing" if no observed check carries its name.
        let any_required_missing = required_check_names
            .iter()
            .any(|name| !normalized.iter().any(|c| &c.name == name));

        // Fail closed when no required checks are configured: an operator MUST
        // define the branch-protection required set before autonomous merge is
        // ever considered safe. An empty required set is therefore NOT green —
        // `.all()` over an empty iterator is vacuously true, so guard explicitly.
        let all_required_green = !required_check_names.is_empty()
            && !any_required_missing
            && normalized
                .iter()
                .filter(|c| c.required)
                .all(|c| c.status == CheckStatus::Green);

        let ampel_status = Self::aggregate_status(&normalized, any_required_missing);

        CiVerificationResult {
            ref_sha: ref_sha.into(),
            checks: normalized,
            all_required_green,
            mergeable,
            ampel_status,
        }
    }

    /// Aggregate the traffic-light status across all checks.
    ///
    /// Red dominates yellow dominates green (matching `AmpelStatus`). A missing
    /// required check is an unconditional red.
    fn aggregate_status(checks: &[NormalizedCiCheck], any_required_missing: bool) -> AmpelStatus {
        if any_required_missing {
            return AmpelStatus::Red;
        }
        let mut has_red = false;
        let mut has_yellow = false;
        for c in checks {
            match c.status {
                CheckStatus::Red => has_red = true,
                CheckStatus::Pending | CheckStatus::Running | CheckStatus::Cancelled => {
                    has_yellow = true
                }
                CheckStatus::Green | CheckStatus::Skipped => {}
            }
        }
        if has_red {
            AmpelStatus::Red
        } else if has_yellow {
            AmpelStatus::Yellow
        } else {
            AmpelStatus::Green
        }
    }

    /// TOCTOU guard: the snapshot is still valid iff the SHA is unchanged.
    pub fn reverify_sha_matches(snapshot_sha: &str, fresh_sha: &str) -> bool {
        snapshot_sha == fresh_sha
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(name: &str, status: &str, conclusion: Option<&str>) -> RawCiCheck {
        RawCiCheck::new(name, status, conclusion)
    }

    fn required(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn should_be_red_when_required_check_is_missing() {
        // Arrange: "build" is required but not present among the checks.
        let svc = VerificationService::new();
        let checks = [check("lint", "completed", Some("success"))];

        // Act
        let r = svc.verify(&checks, &required(&["build"]), true, "sha1");

        // Assert: missing required => red, never yellow.
        assert_eq!(r.ampel_status, AmpelStatus::Red);
        assert!(!r.all_required_green);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_be_green_and_safe_when_required_green_and_mergeable() {
        // Arrange
        let svc = VerificationService::new();
        let checks = [
            check("build", "completed", Some("success")),
            check("test", "completed", Some("success")),
        ];

        // Act
        let r = svc.verify(&checks, &required(&["build", "test"]), true, "sha1");

        // Assert
        assert_eq!(r.ampel_status, AmpelStatus::Green);
        assert!(r.all_required_green);
        assert!(r.is_safe_to_merge());
    }

    #[test]
    fn should_reflect_non_required_red_in_status_but_keep_required_green() {
        // Arrange: a non-required check fails; the required one passes.
        let svc = VerificationService::new();
        let checks = [
            check("build", "completed", Some("success")),
            check("optional-fuzz", "completed", Some("failure")),
        ];

        // Act
        let r = svc.verify(&checks, &required(&["build"]), true, "sha1");

        // Assert: status reflects the red check (per AmpelStatus rules), so it
        // is NOT safe to merge even though every *required* check is green.
        assert!(r.all_required_green);
        assert_eq!(r.ampel_status, AmpelStatus::Red);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_be_yellow_when_required_check_pending() {
        // Arrange
        let svc = VerificationService::new();
        let checks = [check("build", "in_progress", None)];

        // Act
        let r = svc.verify(&checks, &required(&["build"]), true, "sha1");

        // Assert
        assert_eq!(r.ampel_status, AmpelStatus::Yellow);
        assert!(!r.all_required_green);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_not_be_safe_when_not_mergeable_even_if_green() {
        // Arrange
        let svc = VerificationService::new();
        let checks = [check("build", "completed", Some("success"))];

        // Act
        let r = svc.verify(&checks, &required(&["build"]), false, "sha1");

        // Assert
        assert_eq!(r.ampel_status, AmpelStatus::Green);
        assert!(r.all_required_green);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_fail_closed_when_required_set_is_empty() {
        // Arrange: green checks but NO required-check set configured.
        let svc = VerificationService::new();
        let checks = [check("build", "completed", Some("success"))];

        // Act
        let r = svc.verify(&checks, &required(&[]), true, "sha1");

        // Assert: empty required set is never safe (operator must define one).
        assert!(!r.all_required_green);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_fail_closed_when_multi_required_subset_present() {
        // Arrange: two required checks, only "build" present + green.
        let svc = VerificationService::new();
        let checks = [check("build", "completed", Some("success"))];

        // Act
        let r = svc.verify(&checks, &required(&["build", "test"]), true, "sha1");

        // Assert: missing "test" forces red and blocks merge.
        assert_eq!(r.ampel_status, AmpelStatus::Red);
        assert!(!r.all_required_green);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_be_red_when_required_check_completed_with_no_conclusion() {
        // Arrange: required check completed but conclusion absent (None).
        let svc = VerificationService::new();
        let checks = [check("build", "completed", None)];

        // Act
        let r = svc.verify(&checks, &required(&["build"]), true, "sha1");

        // Assert: unknown/None conclusion on a completed check => red.
        assert_eq!(r.ampel_status, AmpelStatus::Red);
        assert!(!r.all_required_green);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_be_red_when_required_check_timed_out() {
        // Arrange: required check completed with a non-success terminal outcome.
        let svc = VerificationService::new();
        let checks = [check("build", "completed", Some("timed_out"))];

        // Act
        let r = svc.verify(&checks, &required(&["build"]), true, "sha1");

        // Assert
        assert_eq!(r.ampel_status, AmpelStatus::Red);
        assert!(!r.all_required_green);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_map_cancelled_required_check_to_yellow_and_block() {
        // Arrange: required check was cancelled.
        let svc = VerificationService::new();
        let checks = [check("build", "completed", Some("cancelled"))];

        // Act
        let r = svc.verify(&checks, &required(&["build"]), true, "sha1");

        // Assert: cancelled => yellow (not green), so not safe.
        assert_eq!(r.ampel_status, AmpelStatus::Yellow);
        assert!(!r.all_required_green);
        assert!(!r.is_safe_to_merge());
    }

    #[test]
    fn should_treat_skipped_check_as_non_blocking() {
        // Arrange: required "build" green; a non-required "lint" skipped.
        let svc = VerificationService::new();
        let checks = [
            check("build", "completed", Some("success")),
            check("lint", "completed", Some("skipped")),
        ];

        // Act
        let r = svc.verify(&checks, &required(&["build"]), true, "sha1");

        // Assert: skipped does not block — overall green + safe.
        assert_eq!(r.ampel_status, AmpelStatus::Green);
        assert!(r.all_required_green);
        assert!(r.is_safe_to_merge());
    }

    #[test]
    fn should_detect_toctou_sha_mismatch() {
        assert!(VerificationService::reverify_sha_matches("abc", "abc"));
        assert!(!VerificationService::reverify_sha_matches("abc", "def"));
    }

    #[test]
    fn should_round_trip_verification_result_json() {
        let svc = VerificationService::new();
        let checks = [check("build", "completed", Some("success"))];
        let r = svc.verify(&checks, &required(&["build"]), true, "sha1");
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(
            serde_json::from_str::<CiVerificationResult>(&json).unwrap(),
            r
        );
    }
}
