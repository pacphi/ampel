//! Remediation policy value objects and enums.
//!
//! The DB layer (`ampel-db`) stores these as plain `String`/text columns; this
//! module owns the (de)serialization so the rest of the system works with typed
//! values. Enums round-trip the DB string columns via [`std::fmt::Display`] /
//! [`std::str::FromStr`]; the composite value objects round-trip the JSON text
//! columns via `serde`.

use crate::errors::{AmpelError, AmpelResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// How much autonomy an operator has granted the remediation engine.
///
/// These variants are also the feature gate for the remediation surface: only
/// `DryRunOnly`/`SuggestOnly` are exercised in Phase 1; the higher tiers unlock
/// write behavior in later phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    DryRunOnly,
    SuggestOnly,
    AutoWithApproval,
    FullyAutonomous,
}

impl AutonomyLevel {
    /// Phase 1 ceiling: anything above `SuggestOnly` may perform repository
    /// writes and is therefore gated off until later phases.
    pub fn allows_writes(self) -> bool {
        matches!(self, Self::AutoWithApproval | Self::FullyAutonomous)
    }
}

impl fmt::Display for AutonomyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::DryRunOnly => "dry_run_only",
            Self::SuggestOnly => "suggest_only",
            Self::AutoWithApproval => "auto_with_approval",
            Self::FullyAutonomous => "fully_autonomous",
        };
        f.write_str(s)
    }
}

impl FromStr for AutonomyLevel {
    type Err = AmpelError;

    fn from_str(s: &str) -> AmpelResult<Self> {
        match s {
            "dry_run_only" => Ok(Self::DryRunOnly),
            "suggest_only" => Ok(Self::SuggestOnly),
            "auto_with_approval" => Ok(Self::AutoWithApproval),
            "fully_autonomous" => Ok(Self::FullyAutonomous),
            other => Err(AmpelError::ValidationError(format!(
                "unknown autonomy_level: {other}"
            ))),
        }
    }
}

/// How aggressive a remediation run is allowed to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationTier {
    ConsolidateOnly,
    FixAndConsolidate,
    FullRemediation,
}

impl fmt::Display for RemediationTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ConsolidateOnly => "consolidate_only",
            Self::FixAndConsolidate => "fix_and_consolidate",
            Self::FullRemediation => "full_remediation",
        };
        f.write_str(s)
    }
}

impl FromStr for RemediationTier {
    type Err = AmpelError;

    fn from_str(s: &str) -> AmpelResult<Self> {
        match s {
            "consolidate_only" => Ok(Self::ConsolidateOnly),
            "fix_and_consolidate" => Ok(Self::FixAndConsolidate),
            "full_remediation" => Ok(Self::FullRemediation),
            other => Err(AmpelError::ValidationError(format!(
                "unknown remediation_tier: {other}"
            ))),
        }
    }
}

/// The scope a policy is attached to. Resolution is most-specific-wins:
/// `Repository` > `Team` > `Org` > `User`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeType {
    Repository,
    Team,
    Org,
    User,
}

impl fmt::Display for ScopeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Repository => "repository",
            Self::Team => "team",
            Self::Org => "org",
            Self::User => "user",
        };
        f.write_str(s)
    }
}

impl FromStr for ScopeType {
    type Err = AmpelError;

    fn from_str(s: &str) -> AmpelResult<Self> {
        match s {
            "repository" => Ok(Self::Repository),
            "team" => Ok(Self::Team),
            "org" => Ok(Self::Org),
            "user" => Ok(Self::User),
            other => Err(AmpelError::ValidationError(format!(
                "unknown scope_type: {other}"
            ))),
        }
    }
}

/// Strategy for choosing which open PRs a run operates on.
///
/// Stored as JSON text in `remediation_policy.pr_selection`. Externally tagged
/// so each variant round-trips unambiguously.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrSelectionStrategy {
    /// Every open PR (subject to the other criteria filters).
    #[default]
    AllOpen,
    /// The N oldest open PRs by creation time.
    OldestFirst { max: u32 },
    /// PRs carrying any of the given labels.
    ///
    /// Phase 1 note: PR labels are not yet persisted on the `pull_requests`
    /// table, so this resolves against an empty label set (selects nothing).
    /// Retained in the value object for forward compatibility.
    ByLabel { labels: Vec<String> },
    /// An explicit allow-list of PR numbers.
    ExplicitIds { ids: Vec<i64> },
}

/// A resolved, flattened snapshot of the effective policy used to drive PR
/// selection and previews. Produced by the `PolicyResolver`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediationCriteria {
    pub min_open_prs: i32,
    pub pr_selection: PrSelectionStrategy,
    pub max_prs_per_run: i32,
    pub allowed_targets: Vec<String>,
    pub skip_draft: bool,
    pub require_green_before_merge: bool,
    /// Effective air-gapped flag after the ADR-014 org ceiling is applied.
    pub air_gapped: bool,
    pub autonomy_level: AutonomyLevel,
    pub remediation_tier: RemediationTier,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_round_trip_autonomy_level_through_db_string() {
        for v in [
            AutonomyLevel::DryRunOnly,
            AutonomyLevel::SuggestOnly,
            AutonomyLevel::AutoWithApproval,
            AutonomyLevel::FullyAutonomous,
        ] {
            assert_eq!(AutonomyLevel::from_str(&v.to_string()).unwrap(), v);
        }
    }

    #[test]
    fn should_reject_unknown_autonomy_level_string() {
        assert!(AutonomyLevel::from_str("nope").is_err());
    }

    #[test]
    fn should_round_trip_remediation_tier_through_db_string() {
        for v in [
            RemediationTier::ConsolidateOnly,
            RemediationTier::FixAndConsolidate,
            RemediationTier::FullRemediation,
        ] {
            assert_eq!(RemediationTier::from_str(&v.to_string()).unwrap(), v);
        }
    }

    #[test]
    fn should_round_trip_scope_type_through_db_string() {
        for v in [
            ScopeType::Repository,
            ScopeType::Team,
            ScopeType::Org,
            ScopeType::User,
        ] {
            assert_eq!(ScopeType::from_str(&v.to_string()).unwrap(), v);
        }
    }

    #[test]
    fn should_serialize_autonomy_level_as_snake_case_json() {
        let json = serde_json::to_string(&AutonomyLevel::AutoWithApproval).unwrap();
        assert_eq!(json, "\"auto_with_approval\"");
    }

    #[test]
    fn should_round_trip_pr_selection_all_open_json() {
        let s = PrSelectionStrategy::AllOpen;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(
            serde_json::from_str::<PrSelectionStrategy>(&json).unwrap(),
            s
        );
    }

    #[test]
    fn should_round_trip_pr_selection_oldest_first_json() {
        let s = PrSelectionStrategy::OldestFirst { max: 7 };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(
            serde_json::from_str::<PrSelectionStrategy>(&json).unwrap(),
            s
        );
    }

    #[test]
    fn should_round_trip_pr_selection_by_label_json() {
        let s = PrSelectionStrategy::ByLabel {
            labels: vec!["deps".into(), "security".into()],
        };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(
            serde_json::from_str::<PrSelectionStrategy>(&json).unwrap(),
            s
        );
    }

    #[test]
    fn should_round_trip_pr_selection_explicit_ids_json() {
        let s = PrSelectionStrategy::ExplicitIds {
            ids: vec![1, 2, 42],
        };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(
            serde_json::from_str::<PrSelectionStrategy>(&json).unwrap(),
            s
        );
    }

    #[test]
    fn should_round_trip_remediation_criteria_json() {
        let c = RemediationCriteria {
            min_open_prs: 2,
            pr_selection: PrSelectionStrategy::OldestFirst { max: 5 },
            max_prs_per_run: 10,
            allowed_targets: vec!["main".into(), "develop".into()],
            skip_draft: true,
            require_green_before_merge: true,
            air_gapped: true,
            autonomy_level: AutonomyLevel::DryRunOnly,
            remediation_tier: RemediationTier::ConsolidateOnly,
        };
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(
            serde_json::from_str::<RemediationCriteria>(&json).unwrap(),
            c
        );
    }

    #[test]
    fn should_treat_low_autonomy_as_read_only() {
        assert!(!AutonomyLevel::DryRunOnly.allows_writes());
        assert!(!AutonomyLevel::SuggestOnly.allows_writes());
        assert!(AutonomyLevel::AutoWithApproval.allows_writes());
        assert!(AutonomyLevel::FullyAutonomous.allows_writes());
    }
}
