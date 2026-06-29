//! Remediation run state machine (Phase 2).
//!
//! [`RunState`] is the authoritative lifecycle for a single repository's
//! remediation run. The DB stores it as a lowercase snake-case text column;
//! this module owns the (de)serialization (via [`std::fmt::Display`] /
//! [`std::str::FromStr`]) and the legal-transition graph
//! ([`RunState::can_transition_to`]), mirroring the Phase-1 enum conventions in
//! [`crate::remediation::policy`].
//!
//! Transition graph (authoritative):
//! ```text
//! created ─► selecting ─► consolidating ─► verifying ─► merging ─► finalizing ─► completed
//!                                            │  ▲          ▲
//!                                            ▼  │          │
//!                                        agent_fixing      │
//!                                            │             │
//!                                            ▼             │
//!                                     awaiting_approval ───┘   (human gate)
//! created ──► no_op
//! <any non-terminal> ──► handoff_human | failed | cancelled
//! ```
//! The `awaiting_approval` gate is reached from `verifying` only for the
//! `auto_with_approval` autonomy tier (a safe verification parks the run there
//! until a human approves, which advances it to `merging`). The
//! `fully_autonomous` tier keeps the direct `verifying → merging` edge.
//! Terminal states (`completed`, `handoff_human`, `failed`, `cancelled`,
//! `no_op`) permit no outgoing transitions.

use crate::errors::{AmpelError, AmpelResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Lifecycle state of a remediation run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Created,
    Selecting,
    Consolidating,
    Verifying,
    AwaitingApproval,
    Merging,
    Finalizing,
    AgentFixing,
    Completed,
    HandoffHuman,
    Failed,
    Cancelled,
    NoOp,
}

impl RunState {
    /// Terminal states permit no further transitions.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::HandoffHuman | Self::Failed | Self::Cancelled | Self::NoOp
        )
    }

    /// A run is active while it is non-terminal (i.e. still in flight).
    pub fn is_active(self) -> bool {
        !self.is_terminal()
    }

    /// Whether a transition from `self` to `next` is legal.
    ///
    /// Any non-terminal state may bail out to `handoff_human`, `failed`, or
    /// `cancelled`. Terminal states are sinks.
    pub fn can_transition_to(self, next: RunState) -> bool {
        use RunState::*;

        // Terminal states never transition.
        if self.is_terminal() {
            return false;
        }

        // Universal bail-outs available from any active state.
        if matches!(next, HandoffHuman | Failed | Cancelled) {
            return true;
        }

        matches!(
            (self, next),
            (Created, Selecting)
                | (Created, NoOp)
                | (Selecting, Consolidating)
                | (Consolidating, Verifying)
                | (Verifying, Merging)
                | (Verifying, AwaitingApproval)
                | (AwaitingApproval, Merging)
                | (Verifying, AgentFixing)
                | (AgentFixing, Verifying)
                | (Merging, Finalizing)
                | (Finalizing, Completed)
        )
    }
}

impl fmt::Display for RunState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Created => "created",
            Self::Selecting => "selecting",
            Self::Consolidating => "consolidating",
            Self::Verifying => "verifying",
            Self::AwaitingApproval => "awaiting_approval",
            Self::Merging => "merging",
            Self::Finalizing => "finalizing",
            Self::AgentFixing => "agent_fixing",
            Self::Completed => "completed",
            Self::HandoffHuman => "handoff_human",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::NoOp => "no_op",
        };
        f.write_str(s)
    }
}

impl FromStr for RunState {
    type Err = AmpelError;

    fn from_str(s: &str) -> AmpelResult<Self> {
        match s {
            "created" => Ok(Self::Created),
            "selecting" => Ok(Self::Selecting),
            "consolidating" => Ok(Self::Consolidating),
            "verifying" => Ok(Self::Verifying),
            "awaiting_approval" => Ok(Self::AwaitingApproval),
            "merging" => Ok(Self::Merging),
            "finalizing" => Ok(Self::Finalizing),
            "agent_fixing" => Ok(Self::AgentFixing),
            "completed" => Ok(Self::Completed),
            "handoff_human" => Ok(Self::HandoffHuman),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "no_op" => Ok(Self::NoOp),
            other => Err(AmpelError::ValidationError(format!(
                "unknown run_state: {other}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RunState::*;
    use super::*;

    const ALL: [RunState; 13] = [
        Created,
        Selecting,
        Consolidating,
        Verifying,
        AwaitingApproval,
        Merging,
        Finalizing,
        AgentFixing,
        Completed,
        HandoffHuman,
        Failed,
        Cancelled,
        NoOp,
    ];

    #[test]
    fn should_round_trip_run_state_through_db_string() {
        for v in ALL {
            assert_eq!(RunState::from_str(&v.to_string()).unwrap(), v);
        }
    }

    #[test]
    fn should_reject_unknown_run_state_string() {
        assert!(RunState::from_str("nope").is_err());
    }

    #[test]
    fn should_serialize_run_state_as_snake_case_json() {
        assert_eq!(
            serde_json::to_string(&RunState::AgentFixing).unwrap(),
            "\"agent_fixing\""
        );
        assert_eq!(
            serde_json::to_string(&RunState::HandoffHuman).unwrap(),
            "\"handoff_human\""
        );
    }

    #[test]
    fn should_allow_every_happy_path_transition() {
        assert!(Created.can_transition_to(Selecting));
        assert!(Selecting.can_transition_to(Consolidating));
        assert!(Consolidating.can_transition_to(Verifying));
        assert!(Verifying.can_transition_to(Merging));
        assert!(Merging.can_transition_to(Finalizing));
        assert!(Finalizing.can_transition_to(Completed));
    }

    #[test]
    fn should_allow_awaiting_approval_gate_path() {
        // Verifying parks at the human gate, then a human approves to merging.
        assert!(Verifying.can_transition_to(AwaitingApproval));
        assert!(AwaitingApproval.can_transition_to(Merging));
        // The no-approval (fully autonomous) path remains a direct edge.
        assert!(Verifying.can_transition_to(Merging));
    }

    #[test]
    fn should_allow_awaiting_approval_to_bail_out() {
        assert!(AwaitingApproval.can_transition_to(HandoffHuman));
        assert!(AwaitingApproval.can_transition_to(Cancelled));
        assert!(AwaitingApproval.can_transition_to(Failed));
    }

    #[test]
    fn should_reject_awaiting_approval_skipping_merge() {
        // The gate cannot jump straight to finalizing/completed.
        assert!(!AwaitingApproval.can_transition_to(Finalizing));
        assert!(!AwaitingApproval.can_transition_to(Completed));
    }

    #[test]
    fn should_serialize_awaiting_approval_as_snake_case_json() {
        assert_eq!(
            serde_json::to_string(&RunState::AwaitingApproval).unwrap(),
            "\"awaiting_approval\""
        );
    }

    #[test]
    fn should_allow_agent_fixing_loop() {
        assert!(Verifying.can_transition_to(AgentFixing));
        assert!(AgentFixing.can_transition_to(Verifying));
    }

    #[test]
    fn should_allow_created_to_no_op() {
        assert!(Created.can_transition_to(NoOp));
    }

    #[test]
    fn should_allow_any_active_state_to_bail_out() {
        for s in [
            Created,
            Selecting,
            Consolidating,
            Verifying,
            AwaitingApproval,
            Merging,
            Finalizing,
            AgentFixing,
        ] {
            assert!(s.can_transition_to(HandoffHuman), "{s} -> handoff_human");
            assert!(s.can_transition_to(Failed), "{s} -> failed");
            assert!(s.can_transition_to(Cancelled), "{s} -> cancelled");
        }
    }

    #[test]
    fn should_reject_representative_illegal_transitions() {
        // Skipping a stage.
        assert!(!Created.can_transition_to(Consolidating));
        assert!(!Selecting.can_transition_to(Verifying));
        assert!(!Consolidating.can_transition_to(Merging));
        assert!(!Verifying.can_transition_to(Finalizing));
        // Going backwards.
        assert!(!Merging.can_transition_to(Verifying));
        // Only `created` may shortcut to no_op.
        assert!(!Selecting.can_transition_to(NoOp));
        assert!(!Consolidating.can_transition_to(NoOp));
    }

    #[test]
    fn should_reject_all_transitions_from_terminal_states() {
        for term in [Completed, HandoffHuman, Failed, Cancelled, NoOp] {
            assert!(term.is_terminal());
            assert!(!term.is_active());
            for next in ALL {
                assert!(
                    !term.can_transition_to(next),
                    "{term} must not transition to {next}"
                );
            }
        }
    }

    #[test]
    fn should_report_active_for_non_terminal_states() {
        for s in [
            Created,
            Selecting,
            Consolidating,
            Verifying,
            AwaitingApproval,
            Merging,
            Finalizing,
            AgentFixing,
        ] {
            assert!(s.is_active());
            assert!(!s.is_terminal());
        }
    }
}
