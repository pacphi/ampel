//! CI failure classification (Phase 4, ADR-012).
//!
//! The remediation harness routes a red CI run through a classification cascade
//! (L1 heuristic → L2 ONNX → model escalation). This module owns the pure,
//! always-available **L1 heuristic** layer plus the shared result types and the
//! [`FailureClassifier`] trait the worker implements for the full cascade.
//!
//! [`classify_heuristic`] is a pure function over CI log text: no network, no
//! ONNX runtime, sub-millisecond. A recognized marker yields a [`FailureClass`]
//! with `confidence == 1.0`; no match yields [`FailureClass::Unknown`] with
//! `confidence == 0.0`, which the cascade escalates to the next layer.
//!
//! Enums follow the Phase-1/2 conventions: `serde` snake_case plus matching
//! [`std::fmt::Display`] / [`std::str::FromStr`] for the DB string columns.

use crate::errors::{AmpelError, AmpelResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The category of a failing CI run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    BuildError,
    TestFailure,
    TypeError,
    Lint,
    LockfileConflict,
    FlakyTest,
    MissingDependency,
    Unknown,
}

impl fmt::Display for FailureClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::BuildError => "build_error",
            Self::TestFailure => "test_failure",
            Self::TypeError => "type_error",
            Self::Lint => "lint",
            Self::LockfileConflict => "lockfile_conflict",
            Self::FlakyTest => "flaky_test",
            Self::MissingDependency => "missing_dependency",
            Self::Unknown => "unknown",
        };
        f.write_str(s)
    }
}

impl FromStr for FailureClass {
    type Err = AmpelError;

    fn from_str(s: &str) -> AmpelResult<Self> {
        match s {
            "build_error" => Ok(Self::BuildError),
            "test_failure" => Ok(Self::TestFailure),
            "type_error" => Ok(Self::TypeError),
            "lint" => Ok(Self::Lint),
            "lockfile_conflict" => Ok(Self::LockfileConflict),
            "flaky_test" => Ok(Self::FlakyTest),
            "missing_dependency" => Ok(Self::MissingDependency),
            "unknown" => Ok(Self::Unknown),
            other => Err(AmpelError::ValidationError(format!(
                "unknown failure_class: {other}"
            ))),
        }
    }
}

/// Which layer of the classification cascade produced a result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassifierSource {
    /// L1 pure regex/marker heuristic (this module).
    Heuristic,
    /// L2 local ONNX inference (ampel-worker, feature-gated).
    Onnx,
    /// L3 model escalation (ampel-worker).
    Model,
}

impl fmt::Display for ClassifierSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Heuristic => "heuristic",
            Self::Onnx => "onnx",
            Self::Model => "model",
        };
        f.write_str(s)
    }
}

impl FromStr for ClassifierSource {
    type Err = AmpelError;

    fn from_str(s: &str) -> AmpelResult<Self> {
        match s {
            "heuristic" => Ok(Self::Heuristic),
            "onnx" => Ok(Self::Onnx),
            "model" => Ok(Self::Model),
            other => Err(AmpelError::ValidationError(format!(
                "unknown classifier_source: {other}"
            ))),
        }
    }
}

/// The outcome of classifying a CI log: the class, the layer that produced it,
/// and a confidence in `[0.0, 1.0]`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub class: FailureClass,
    pub source: ClassifierSource,
    pub confidence: f32,
}

impl ClassificationResult {
    /// An unrecognized log: `Unknown`, heuristic source, zero confidence. The
    /// cascade escalates this to the next layer.
    pub fn unknown_heuristic() -> Self {
        Self {
            class: FailureClass::Unknown,
            source: ClassifierSource::Heuristic,
            confidence: 0.0,
        }
    }
}

/// Pure L1 heuristic classifier over CI log text.
///
/// Matching is case-insensitive and marker-based (no external regex engine, no
/// allocations beyond a single lowercase copy). Checks run in a fixed priority
/// order so that more-specific classes win over generic ones (e.g. a `flaky`
/// marker beats a generic test failure; a TypeScript `error TS####` beats a
/// generic build error). A match returns confidence `1.0`; no match returns
/// [`FailureClass::Unknown`] with confidence `0.0`.
pub fn classify_heuristic(log_text: &str) -> ClassificationResult {
    let log = log_text.to_lowercase();

    let class = if is_lockfile_conflict(&log) {
        FailureClass::LockfileConflict
    } else if is_type_error(&log) {
        FailureClass::TypeError
    } else if is_missing_dependency(&log) {
        FailureClass::MissingDependency
    } else if is_flaky(&log) {
        FailureClass::FlakyTest
    } else if is_lint(&log) {
        FailureClass::Lint
    } else if is_build_error(&log) {
        FailureClass::BuildError
    } else if is_test_failure(&log) {
        FailureClass::TestFailure
    } else {
        return ClassificationResult::unknown_heuristic();
    };

    ClassificationResult {
        class,
        source: ClassifierSource::Heuristic,
        confidence: 1.0,
    }
}

fn is_lockfile_conflict(log: &str) -> bool {
    log.contains("<<<<<<<")
        || log.contains(">>>>>>>")
        || log.contains("merge conflict")
        || (log.contains("conflict")
            && (log.contains("cargo.lock")
                || log.contains("package-lock.json")
                || log.contains("pnpm-lock.yaml")
                || log.contains("yarn.lock")))
}

fn is_type_error(log: &str) -> bool {
    log.contains("error ts")
        || log.contains("is not assignable to type")
        || log.contains("mismatched types")
        || log.contains("type mismatch")
        || log.contains("expected type")
}

fn is_missing_dependency(log: &str) -> bool {
    log.contains("npm err! 404")
        || log.contains("cannot find module")
        || log.contains("module not found")
        || log.contains("could not resolve dependency")
        || log.contains("no matching package named")
        || log.contains("no matching version found")
        || log.contains("failed to resolve dependencies")
}

fn is_flaky(log: &str) -> bool {
    log.contains("flaky")
        || log.contains("flake")
        || log.contains("intermittent")
        || log.contains("nondeterministic")
        || log.contains("retrying test")
}

fn is_lint(log: &str) -> bool {
    log.contains("clippy")
        || log.contains("eslint")
        || log.contains("stylelint")
        || log.contains("prettier")
        || log.contains("rustfmt")
        || log.contains("no-unused-vars")
        || log.contains("lint error")
}

fn is_build_error(log: &str) -> bool {
    log.contains("error[e")
        || log.contains("could not compile")
        || log.contains("failed to compile")
        || log.contains("build failed")
        || log.contains("cannot find value")
        || log.contains("cannot find function")
        || log.contains("cannot find macro")
        || log.contains("cannot find type")
        || log.contains("unresolved import")
        || log.contains("linker `cc` failed")
}

fn is_test_failure(log: &str) -> bool {
    log.contains("test result: failed")
        || log.contains("assertion failed")
        || log.contains("assertion `left")
        || log.contains("tests failed")
        || log.contains("fail")
}

/// The classification cascade contract. `ampel-core` ships the pure
/// [`HeuristicClassifier`]; `ampel-worker` implements the full L1→L2→L3 cascade
/// behind the same trait so callers stay agnostic of the layers.
#[async_trait]
pub trait FailureClassifier: Send + Sync {
    async fn classify(&self, log_text: &str) -> ClassificationResult;
}

/// Always-available, pure L1 classifier. Used directly in tests and reused as
/// the first stage of the worker's cascade.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicClassifier;

#[async_trait]
impl FailureClassifier for HeuristicClassifier {
    async fn classify(&self, log_text: &str) -> ClassificationResult {
        classify_heuristic(log_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_CLASSES: [FailureClass; 8] = [
        FailureClass::BuildError,
        FailureClass::TestFailure,
        FailureClass::TypeError,
        FailureClass::Lint,
        FailureClass::LockfileConflict,
        FailureClass::FlakyTest,
        FailureClass::MissingDependency,
        FailureClass::Unknown,
    ];

    #[test]
    fn should_round_trip_failure_class_through_db_string() {
        for v in ALL_CLASSES {
            assert_eq!(FailureClass::from_str(&v.to_string()).unwrap(), v);
        }
    }

    #[test]
    fn should_reject_unknown_failure_class_string() {
        assert!(FailureClass::from_str("nope").is_err());
    }

    #[test]
    fn should_round_trip_classifier_source_through_db_string() {
        for v in [
            ClassifierSource::Heuristic,
            ClassifierSource::Onnx,
            ClassifierSource::Model,
        ] {
            assert_eq!(ClassifierSource::from_str(&v.to_string()).unwrap(), v);
        }
    }

    #[test]
    fn should_serialize_failure_class_as_snake_case_json() {
        assert_eq!(
            serde_json::to_string(&FailureClass::LockfileConflict).unwrap(),
            "\"lockfile_conflict\""
        );
    }

    fn assert_classified_as(log: &str, expected: FailureClass) {
        let result = classify_heuristic(log);
        assert_eq!(result.class, expected, "log: {log}");
        assert_eq!(result.source, ClassifierSource::Heuristic);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn should_classify_build_error_from_rustc_log() {
        assert_classified_as(
            "error[E0432]: unresolved import `crate::foo`\n  --> src/lib.rs:1:5",
            FailureClass::BuildError,
        );
    }

    #[test]
    fn should_classify_test_failure_from_nextest_log() {
        assert_classified_as(
            "test result: FAILED. 12 passed; 3 failed; 0 ignored",
            FailureClass::TestFailure,
        );
    }

    #[test]
    fn should_classify_type_error_from_tsc_log() {
        assert_classified_as(
            "src/app.ts(10,5): error TS2322: Type 'string' is not assignable to type 'number'.",
            FailureClass::TypeError,
        );
    }

    #[test]
    fn should_classify_lint_from_clippy_log() {
        assert_classified_as(
            "error: unused variable: `x`\n = note: `#[deny(clippy::all)]` on by default",
            FailureClass::Lint,
        );
    }

    #[test]
    fn should_classify_lockfile_conflict_from_git_markers() {
        assert_classified_as(
            "Auto-merging Cargo.lock\n<<<<<<< HEAD\nfoo = 1.0\n>>>>>>> branch",
            FailureClass::LockfileConflict,
        );
    }

    #[test]
    fn should_classify_flaky_test_from_retry_marker() {
        assert_classified_as(
            "test integration::login ... FAILED (flaky: passed on retry)",
            FailureClass::FlakyTest,
        );
    }

    #[test]
    fn should_classify_missing_dependency_from_npm_404() {
        assert_classified_as(
            "npm ERR! 404 Not Found - GET https://registry.npmjs.org/leftpadx - Not found",
            FailureClass::MissingDependency,
        );
    }

    #[test]
    fn should_prefer_lockfile_conflict_over_build_error_when_both_present() {
        // Co-occurring markers: a Cargo.lock conflict AND a `could not compile`
        // build error. Lockfile precedence must beat the build-error fallback.
        assert_classified_as(
            "Auto-merging Cargo.lock\n<<<<<<< HEAD\nfoo = 1.0\n=======\nfoo = 2.0\n>>>>>>> branch\n\
             error: could not compile `ampel` due to previous error",
            FailureClass::LockfileConflict,
        );
    }

    #[test]
    fn should_prefer_type_error_over_build_error_when_both_present() {
        // Co-occurring markers: a TypeScript `error TS####` AND a generic
        // `build failed`. Type-error precedence must beat the build-error check.
        assert_classified_as(
            "src/app.ts(10,5): error TS2322: Type 'string' is not assignable to type 'number'.\n\
             build failed",
            FailureClass::TypeError,
        );
    }

    #[test]
    fn should_classify_unrecognized_log_as_unknown_with_zero_confidence() {
        let result = classify_heuristic("Cloning into 'repo'... done.\nSetting up environment.");
        assert_eq!(result.class, FailureClass::Unknown);
        assert_eq!(result.source, ClassifierSource::Heuristic);
        assert_eq!(result.confidence, 0.0);
    }

    #[tokio::test]
    async fn should_classify_via_heuristic_classifier_trait() {
        let classifier = HeuristicClassifier;
        let result = classifier
            .classify("error[E0599]: no method named `foo`")
            .await;
        assert_eq!(result.class, FailureClass::BuildError);
    }
}
