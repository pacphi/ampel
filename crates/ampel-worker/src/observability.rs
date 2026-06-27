//! Worker-side Prometheus metrics for autonomous PR remediation (Phase 3).
//!
//! The metrics crate is kept out of `ampel-core` (which must stay
//! dependency-light); all remediation counters/histograms are emitted from the
//! worker layer — the [`crate::services::RemediationExecutor`], where run
//! outcomes are observed. `main.rs` installs the Prometheus scrape endpoint and
//! calls [`describe_metrics`] once at startup; the executor calls the
//! `record_*` helpers below at the relevant points in the run lifecycle.
//!
//! Cardinality: every label value is drawn from a small bounded set
//! (terminal run states, provider kinds, conflict classes, handoff reasons) —
//! never free-form text or anything secret-bearing.

use metrics::{counter, describe_counter, describe_histogram, histogram, Unit};

/// Terminal-run counter, labelled by `state`.
pub const RUNS_TOTAL: &str = "remediation_runs_total";
/// Successful merge counter, labelled by `provider`.
pub const MERGES_TOTAL: &str = "remediation_merges_total";
/// Skipped-conflict counter, labelled by `conflict_class`.
pub const CONFLICTS_TOTAL: &str = "remediation_conflicts_total";
/// Human-handoff counter, labelled by `reason`.
pub const HANDOFFS_TOTAL: &str = "remediation_handoffs_total";
/// Run-duration histogram, labelled by `phase`.
pub const DURATION_SECONDS: &str = "remediation_duration_seconds";

/// Describe every remediation metric. Safe to call once at worker startup
/// (mirrors the `ampel-api` describe pattern).
pub fn describe_metrics() {
    describe_counter!(
        RUNS_TOTAL,
        "Total remediation runs that reached a terminal state, by state"
    );
    describe_counter!(
        MERGES_TOTAL,
        "Total successful consolidated-PR merges, by provider kind"
    );
    describe_counter!(
        CONFLICTS_TOTAL,
        "Total per-PR skipped-conflict dispositions, by conflict class"
    );
    describe_counter!(
        HANDOFFS_TOTAL,
        "Total remediation runs handed off to a human, by reason"
    );
    describe_histogram!(
        DURATION_SECONDS,
        Unit::Seconds,
        "Remediation run duration in seconds, by terminal phase"
    );
}

/// Record a run reaching a terminal `state` and its total `duration_secs`.
pub fn record_run_terminal(state: &str, duration_secs: f64) {
    counter!(RUNS_TOTAL, "state" => state.to_string()).increment(1);
    histogram!(DURATION_SECONDS, "phase" => state.to_string()).record(duration_secs);
}

/// Record a successful merge performed against `provider`.
pub fn record_merge(provider: &str) {
    counter!(MERGES_TOTAL, "provider" => provider.to_string()).increment(1);
}

/// Record one skipped-conflict disposition under a bounded `conflict_class`.
pub fn record_conflict(conflict_class: &'static str) {
    counter!(CONFLICTS_TOTAL, "conflict_class" => conflict_class).increment(1);
}

/// Record a human handoff with a bounded `reason`.
pub fn record_handoff(reason: &'static str) {
    counter!(HANDOFFS_TOTAL, "reason" => reason).increment(1);
}

/// Map a free-form skipped-conflict reason onto a bounded, low-cardinality
/// class suitable for a Prometheus label. Keeps the `conflict_class` label from
/// exploding on per-PR reason strings.
pub fn classify_conflict(reason: &str) -> &'static str {
    let r = reason.to_ascii_lowercase();
    if r.contains("lock") {
        "lockfile"
    } else if r.contains("conflict") || r.contains("merge") {
        "merge"
    } else if r.contains("test") || r.contains("ci") {
        "ci"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_classify_lockfile_conflicts() {
        // Arrange / Act / Assert
        assert_eq!(classify_conflict("Cargo.lock conflict"), "lockfile");
        assert_eq!(classify_conflict("pnpm-lock.yaml diverged"), "lockfile");
    }

    #[test]
    fn should_classify_merge_conflicts() {
        assert_eq!(classify_conflict("unresolved merge conflict"), "merge");
    }

    #[test]
    fn should_classify_ci_conflicts() {
        assert_eq!(classify_conflict("required CI check failed"), "ci");
    }

    #[test]
    fn should_fall_back_to_other_for_unknown_reason() {
        assert_eq!(classify_conflict("something unexpected"), "other");
    }
}
