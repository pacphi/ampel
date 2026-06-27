//! Consolidation preview value objects.

use serde::{Deserialize, Serialize};

/// A lightweight reference to a pull request selected for a run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrRef {
    pub number: i32,
    pub title: String,
    pub branch: String,
}

/// Final decision recorded for a single source PR in a remediation run.
///
/// Set once per source PR and immutable thereafter (drives the audit log and
/// SSE progress events). `#[non_exhaustive]` guards against accidental addition
/// of mutable variants. Persisted to `remediation_run_pr` as JSON
/// (externally tagged via the `disposition` key, snake_case).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "disposition", rename_all = "snake_case")]
#[non_exhaustive]
pub enum MergeDisposition {
    /// Commits were included in the consolidated PR and the original closed.
    Consolidated,
    /// The PR was closed; the consolidating PR number is recorded for traceability.
    ClosedWithRef { consolidated_pr_number: u64 },
    /// Skipped because of an unresolved merge conflict.
    SkippedConflict { reason: String },
    /// No action taken; records why (e.g. `"draft"`, `"excluded by label"`).
    LeftOpen { reason: String },
}

impl MergeDisposition {
    /// True if the PR was acted upon (closed or consolidated).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Consolidated | Self::ClosedWithRef { .. })
    }

    /// The reason text, when the variant carries one.
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::SkippedConflict { reason } | Self::LeftOpen { reason } => Some(reason),
            _ => None,
        }
    }
}

/// The read-only result of a `preview` (dry-run). Building this performs zero
/// repository writes — it only reads the DB and projects what a run *would* do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsolidationPlan {
    /// PRs that would be selected, in selection order.
    pub would_select: Vec<PrRef>,
    pub pr_count: usize,
    /// Predicted merge conflicts. Empty stub in Phase 1 (no merge simulation yet).
    pub predicted_conflicts: Vec<String>,
    /// Naive duration heuristic for operator expectation-setting.
    pub estimated_duration_secs: u64,
    /// Effective air-gapped flag (after the ADR-014 org ceiling).
    pub air_gapped: bool,
    /// True when air-gapping blocks the external-provider portion of a run.
    /// In Phase 1 the preview still renders; this flags the constraint.
    pub blocked_by_air_gap: bool,
}

impl ConsolidationPlan {
    /// Per-PR base cost for the duration heuristic (seconds).
    const SECS_PER_PR: u64 = 30;
    /// Fixed setup/overhead cost for any run (seconds).
    const BASE_OVERHEAD_SECS: u64 = 15;

    /// Build a plan from the selected PRs and resolved air-gapped state.
    pub fn from_selection(selected: Vec<PrRef>, air_gapped: bool) -> Self {
        let pr_count = selected.len();
        let estimated_duration_secs = if pr_count == 0 {
            0
        } else {
            Self::BASE_OVERHEAD_SECS + (pr_count as u64) * Self::SECS_PER_PR
        };

        Self {
            would_select: selected,
            pr_count,
            predicted_conflicts: Vec::new(),
            estimated_duration_secs,
            air_gapped,
            blocked_by_air_gap: air_gapped,
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
    fn should_count_selected_prs() {
        let plan = ConsolidationPlan::from_selection(vec![pr(1), pr(2)], false);
        assert_eq!(plan.pr_count, 2);
    }

    #[test]
    fn should_estimate_zero_duration_for_empty_selection() {
        let plan = ConsolidationPlan::from_selection(vec![], false);
        assert_eq!(plan.estimated_duration_secs, 0);
    }

    #[test]
    fn should_estimate_duration_from_pr_count() {
        let plan = ConsolidationPlan::from_selection(vec![pr(1), pr(2)], false);
        assert_eq!(plan.estimated_duration_secs, 15 + 2 * 30);
    }

    #[test]
    fn should_flag_blocked_by_air_gap_when_air_gapped() {
        let plan = ConsolidationPlan::from_selection(vec![pr(1)], true);
        assert!(plan.air_gapped);
        assert!(plan.blocked_by_air_gap);
    }

    #[test]
    fn should_not_flag_air_gap_when_not_air_gapped() {
        let plan = ConsolidationPlan::from_selection(vec![pr(1)], false);
        assert!(!plan.blocked_by_air_gap);
    }

    #[test]
    fn should_round_trip_consolidation_plan_json() {
        let plan = ConsolidationPlan::from_selection(vec![pr(1)], true);
        let json = serde_json::to_string(&plan).unwrap();
        assert_eq!(
            serde_json::from_str::<ConsolidationPlan>(&json).unwrap(),
            plan
        );
    }

    #[test]
    fn should_round_trip_merge_disposition_json() {
        for d in [
            MergeDisposition::Consolidated,
            MergeDisposition::ClosedWithRef {
                consolidated_pr_number: 42,
            },
            MergeDisposition::SkippedConflict {
                reason: "lockfile".into(),
            },
            MergeDisposition::LeftOpen {
                reason: "draft".into(),
            },
        ] {
            let json = serde_json::to_string(&d).unwrap();
            assert_eq!(serde_json::from_str::<MergeDisposition>(&json).unwrap(), d);
        }
    }

    #[test]
    fn should_tag_merge_disposition_with_snake_case_key() {
        let json = serde_json::to_string(&MergeDisposition::Consolidated).unwrap();
        assert_eq!(json, "{\"disposition\":\"consolidated\"}");
    }

    #[test]
    fn should_report_terminal_only_for_acted_dispositions() {
        assert!(MergeDisposition::Consolidated.is_terminal());
        assert!(MergeDisposition::ClosedWithRef {
            consolidated_pr_number: 1
        }
        .is_terminal());
        assert!(!MergeDisposition::SkippedConflict { reason: "x".into() }.is_terminal());
        assert!(!MergeDisposition::LeftOpen { reason: "x".into() }.is_terminal());
    }

    #[test]
    fn should_expose_reason_only_for_reasoned_dispositions() {
        assert_eq!(
            MergeDisposition::SkippedConflict {
                reason: "conflict".into()
            }
            .reason(),
            Some("conflict")
        );
        assert_eq!(MergeDisposition::Consolidated.reason(), None);
    }
}
