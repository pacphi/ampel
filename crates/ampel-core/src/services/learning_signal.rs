//! Strategy-learning persistence + read abstractions (Phase 5b).
//!
//! `ampel-core` cannot depend on `ampel-db` (dependency cycle), so — exactly as
//! with [`RemediationRunRepository`](super::RemediationRunRepository) — the write
//! and read sides of the `learning_signal` table are expressed as traits here and
//! implemented in the outer layer (`ampel-db`) via dependency injection:
//!
//! - [`LearningSignalRecorder`] — append one [`LearningSignal`] per completed
//!   agentic remediation session (driven by the worker's `DbAgenticTier`).
//! - [`LearningStatsReader`] — read AGGREGATE per-`(failure_class, provider)`
//!   pass-rate stats. The [`PolicyResolver`](super::PolicyResolver) consumes this
//!   (optionally) to bias the `fallback_chain` provider ordering toward the
//!   highest historical pass-rate.
//!
//! # Security
//! A signal carries the provider *kind* only — never an API key or endpoint.

use crate::errors::AmpelResult;
use crate::remediation::{FailureClass, ProviderKind};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::fmt;
use std::str::FromStr;

use crate::errors::AmpelError;

/// The terminal outcome of an agentic session, from the learning point of view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearningOutcome {
    /// CI turned green — the provider fixed the failure.
    Passed,
    /// Budget/iterations exhausted (or otherwise handed off) without a fix.
    Exhausted,
}

impl LearningOutcome {
    /// Map a boolean "did it pass" into the outcome.
    pub fn from_passed(passed: bool) -> Self {
        if passed {
            Self::Passed
        } else {
            Self::Exhausted
        }
    }

    /// True for [`LearningOutcome::Passed`].
    pub fn is_passed(self) -> bool {
        matches!(self, Self::Passed)
    }
}

impl fmt::Display for LearningOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Exhausted => "exhausted",
        })
    }
}

impl FromStr for LearningOutcome {
    type Err = AmpelError;

    fn from_str(s: &str) -> AmpelResult<Self> {
        match s {
            "passed" => Ok(Self::Passed),
            "exhausted" => Ok(Self::Exhausted),
            other => Err(AmpelError::ValidationError(format!(
                "unknown learning_outcome: {other}"
            ))),
        }
    }
}

/// One strategy-learning observation about a completed agentic session.
///
/// The typed `provider`/`failure_class`/`outcome`/`cost_usd` are flattened to DB
/// string columns at the `ampel-db` boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearningSignal {
    pub provider: ProviderKind,
    pub failure_class: FailureClass,
    pub playbook_id: String,
    pub playbook_version: i32,
    pub outcome: LearningOutcome,
    pub duration_secs: i64,
    pub cost_usd: Option<Decimal>,
}

/// Aggregate pass-rate stats for one provider on one failure class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderStats {
    pub provider: ProviderKind,
    /// Total recorded sessions for this `(failure_class, provider)`.
    pub total: u64,
    /// Of those, how many ended in [`LearningOutcome::Passed`].
    pub passed: u64,
}

impl ProviderStats {
    /// Historical pass-rate in `[0.0, 1.0]`. Zero observations → `0.0`.
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }
}

/// Write-side: append one learning signal per completed agentic session.
#[async_trait]
pub trait LearningSignalRecorder: Send + Sync {
    /// Persist a single observation. Append-only; never updates.
    async fn record(&self, signal: LearningSignal) -> AmpelResult<()>;
}

/// Read-side: aggregate per-provider pass-rate for a given failure class.
#[async_trait]
pub trait LearningStatsReader: Send + Sync {
    /// Aggregate stats for every provider that has at least one recorded signal
    /// for `failure_class`. Providers with no data are simply absent.
    async fn provider_stats(&self, failure_class: FailureClass) -> AmpelResult<Vec<ProviderStats>>;
}

/// Pure, deterministic provider ordering used by the `fallback_chain` selection
/// mode: providers with recorded data come first, ordered by pass-rate
/// descending; ties (and the no-data remainder) keep the stable `default_order`.
///
/// This function is the testable core of the bias and performs no I/O.
pub fn bias_provider_chain(
    default_order: &[ProviderKind],
    stats: &[ProviderStats],
) -> Vec<ProviderKind> {
    // The default-order index gives the deterministic tiebreak and the stable
    // ordering of the no-data remainder.
    let mut with_data: Vec<(usize, f64, ProviderKind)> = Vec::new();
    let mut without_data: Vec<ProviderKind> = Vec::new();

    for (idx, &provider) in default_order.iter().enumerate() {
        match stats.iter().find(|s| s.provider == provider && s.total > 0) {
            Some(s) => with_data.push((idx, s.pass_rate(), provider)),
            None => without_data.push(provider),
        }
    }

    // Highest pass-rate first; tiebreak by stable default-order index.
    with_data.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });

    with_data
        .into_iter()
        .map(|(_, _, p)| p)
        .chain(without_data)
        .collect()
}

#[cfg(any(test, feature = "test-utils"))]
pub use in_memory::{InMemoryLearningSignalRecorder, InMemoryLearningStatsReader};

#[cfg(any(test, feature = "test-utils"))]
mod in_memory {
    //! In-process fakes for unit tests — no DB, no network.

    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Records signals in memory; exposes them for assertions.
    #[derive(Default)]
    pub struct InMemoryLearningSignalRecorder {
        signals: Mutex<Vec<LearningSignal>>,
    }

    impl InMemoryLearningSignalRecorder {
        pub fn new() -> Self {
            Self::default()
        }

        /// All recorded signals, in insertion order.
        pub fn recorded(&self) -> Vec<LearningSignal> {
            self.signals.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl LearningSignalRecorder for InMemoryLearningSignalRecorder {
        async fn record(&self, signal: LearningSignal) -> AmpelResult<()> {
            self.signals.lock().unwrap().push(signal);
            Ok(())
        }
    }

    /// Serves canned per-`(failure_class, provider)` stats to the resolver.
    #[derive(Default)]
    pub struct InMemoryLearningStatsReader {
        // (failure_class, provider) -> (total, passed)
        stats: Mutex<HashMap<(FailureClass, ProviderKind), (u64, u64)>>,
    }

    impl InMemoryLearningStatsReader {
        pub fn new() -> Self {
            Self::default()
        }

        /// Seed an observation count for a `(failure_class, provider)` pairing.
        pub fn with_stats(
            self,
            failure_class: FailureClass,
            provider: ProviderKind,
            total: u64,
            passed: u64,
        ) -> Self {
            self.stats
                .lock()
                .unwrap()
                .insert((failure_class, provider), (total, passed));
            self
        }
    }

    #[async_trait]
    impl LearningStatsReader for InMemoryLearningStatsReader {
        async fn provider_stats(
            &self,
            failure_class: FailureClass,
        ) -> AmpelResult<Vec<ProviderStats>> {
            Ok(self
                .stats
                .lock()
                .unwrap()
                .iter()
                .filter(|((fc, _), _)| *fc == failure_class)
                .map(|((_, provider), (total, passed))| ProviderStats {
                    provider: *provider,
                    total: *total,
                    passed: *passed,
                })
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_ORDER: [ProviderKind; 4] = [
        ProviderKind::Claude,
        ProviderKind::Gemini,
        ProviderKind::Ollama,
        ProviderKind::Onnx,
    ];

    #[test]
    fn should_round_trip_learning_outcome_through_string() {
        for v in [LearningOutcome::Passed, LearningOutcome::Exhausted] {
            assert_eq!(LearningOutcome::from_str(&v.to_string()).unwrap(), v);
        }
    }

    #[test]
    fn should_compute_pass_rate() {
        let s = ProviderStats {
            provider: ProviderKind::Claude,
            total: 4,
            passed: 3,
        };
        assert_eq!(s.pass_rate(), 0.75);
    }

    #[test]
    fn should_report_zero_pass_rate_without_observations() {
        let s = ProviderStats {
            provider: ProviderKind::Claude,
            total: 0,
            passed: 0,
        };
        assert_eq!(s.pass_rate(), 0.0);
    }

    #[test]
    fn should_order_highest_pass_rate_provider_first() {
        // Arrange: Ollama has the best rate, Gemini next, others have no data.
        let stats = [
            ProviderStats {
                provider: ProviderKind::Gemini,
                total: 10,
                passed: 6,
            },
            ProviderStats {
                provider: ProviderKind::Ollama,
                total: 10,
                passed: 9,
            },
        ];

        // Act
        let order = bias_provider_chain(&DEFAULT_ORDER, &stats);

        // Assert: data-backed providers (by rate desc) lead; no-data keep default.
        assert_eq!(
            order,
            vec![
                ProviderKind::Ollama,
                ProviderKind::Gemini,
                ProviderKind::Claude,
                ProviderKind::Onnx,
            ]
        );
    }

    #[test]
    fn should_keep_default_order_when_no_stats() {
        // Arrange + Act
        let order = bias_provider_chain(&DEFAULT_ORDER, &[]);

        // Assert
        assert_eq!(order, DEFAULT_ORDER.to_vec());
    }

    #[test]
    fn should_break_ties_by_stable_default_order() {
        // Arrange: Gemini and Ollama tie on pass-rate; Claude/Onnx have no data.
        let stats = [
            ProviderStats {
                provider: ProviderKind::Ollama,
                total: 2,
                passed: 1,
            },
            ProviderStats {
                provider: ProviderKind::Gemini,
                total: 2,
                passed: 1,
            },
        ];

        // Act
        let order = bias_provider_chain(&DEFAULT_ORDER, &stats);

        // Assert: tie resolves to default order (Gemini before Ollama).
        assert_eq!(
            order,
            vec![
                ProviderKind::Gemini,
                ProviderKind::Ollama,
                ProviderKind::Claude,
                ProviderKind::Onnx,
            ]
        );
    }

    #[test]
    fn should_ignore_stats_with_zero_total() {
        // Arrange: a zero-observation stat must not jump ahead of no-data providers.
        let stats = [ProviderStats {
            provider: ProviderKind::Onnx,
            total: 0,
            passed: 0,
        }];

        // Act
        let order = bias_provider_chain(&DEFAULT_ORDER, &stats);

        // Assert: unchanged default order.
        assert_eq!(order, DEFAULT_ORDER.to_vec());
    }

    #[tokio::test]
    async fn should_record_and_expose_signals_via_in_memory_fake() {
        // Arrange
        let recorder = InMemoryLearningSignalRecorder::new();
        let signal = LearningSignal {
            provider: ProviderKind::Claude,
            failure_class: FailureClass::BuildError,
            playbook_id: "global".into(),
            playbook_version: 1,
            outcome: LearningOutcome::Passed,
            duration_secs: 12,
            cost_usd: Some(Decimal::new(50, 2)),
        };

        // Act
        recorder.record(signal.clone()).await.unwrap();

        // Assert
        assert_eq!(recorder.recorded(), vec![signal]);
    }
}
