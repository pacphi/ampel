//! Tier-2 agentic remediation seam (Phase 4, ADR-008/009/012/014).
//!
//! When a consolidated run verifies RED and its resolved `remediation_tier`
//! permits autonomous fixes, the [`RemediationExecutor`](super::remediation_executor)
//! delegates a *recovery attempt* to an [`AgenticTier`]. This module owns:
//!
//! - the **tier gate** ([`tier_allows_agentic`]) — only `fix_and_consolidate`
//!   and `full_remediation` may invoke the model; `consolidate_only` never does;
//! - the **air-gapped egress gate** ([`assert_egress_allowed`], ADR-014) — an
//!   External-egress provider is refused in an air-gapped policy *before* any
//!   inference;
//! - the [`AgenticTier`] trait + its [`AgentTierOutcome`] (the executor only
//!   needs to know "recovered, re-verify" vs "exhausted, hand off");
//! - [`DbAgenticTier`], the concrete implementation that selects a model account,
//!   decrypts its credentials AT THE CALL SITE (never logged), resolves the
//!   playbook + budget, drives the [`RemediationAgentHarness`], and persists a
//!   `remediation_agent_session` row (iterations / tokens / cost / failure class
//!   / outcome).
//!
//! ## Credential safety (ADR-008)
//! Plaintext credentials live only inside [`DbAgenticTier::attempt`], decrypted
//! via [`EncryptionService`] for exactly the harness run. They are never
//! serialized, logged, or written to the session transcript/row.
//!
//! NOTE: `#![allow(dead_code)]` — the seam, helpers, and [`DbAgenticTier`] are
//! exercised by unit + executor integration tests and exported from the library,
//! but the worker *binary* does not yet construct the sandbox-backed
//! [`AgentWorktree`]/[`CiVerifier`] bridges; that production wiring is a
//! follow-up. The bin target would otherwise flag these as unused.
#![allow(dead_code)]

use std::sync::Arc;

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::remediation::{
    AgentBudget, AgentOutcome, AgentTerminalReason, Egress, FailureClassifier, ModelCredentials,
    ModelProvider, ProviderKind, RemediationTier,
};
use ampel_db::encryption::EncryptionService;
use ampel_db::entities::{model_provider_account, remediation_agent_session};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use super::agent_harness::{AgentWorktree, CiVerifier, RemediationAgentHarness};
use super::playbook::Playbook;
use super::playbook_resolver::{embedded_default_yaml, PlaybookContext};
use crate::providers::build_model_provider;

/// Whether a resolved tier may invoke the agentic (model-driven) path.
///
/// `consolidate_only` is purely mechanical and must NEVER reach a model.
pub fn tier_allows_agentic(tier: RemediationTier) -> bool {
    matches!(
        tier,
        RemediationTier::FixAndConsolidate | RemediationTier::FullRemediation
    )
}

/// ADR-014 egress gate: in an air-gapped policy an External-egress provider may
/// not be dispatched. Returns an error (the caller hands off to a human) rather
/// than silently leaking to the public internet.
pub fn assert_egress_allowed(air_gapped: bool, egress: Egress) -> AmpelResult<()> {
    if air_gapped && egress == Egress::External {
        return Err(AmpelError::ValidationError(
            "egress blocked: air-gapped policy forbids an external-egress model provider"
                .to_string(),
        ));
    }
    Ok(())
}

/// What the agentic attempt tells the executor to do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentTierOutcome {
    /// The agent pushed fixes; the executor should re-verify and (if green) merge.
    Recovered,
    /// Budget/iteration exhausted, egress-blocked, or errored — hand off to human.
    Exhausted,
}

impl AgentTierOutcome {
    /// Map a harness [`AgentOutcome`] onto the executor-facing decision.
    pub fn from_agent_outcome(outcome: &AgentOutcome) -> Self {
        if outcome.passed && outcome.terminal_reason == AgentTerminalReason::CiGreen {
            Self::Recovered
        } else {
            Self::Exhausted
        }
    }
}

/// The Tier-2 seam the executor depends on. `Arc<dyn>` so the executor stays
/// agnostic of the concrete DB/harness wiring (and so tests can inject a fake).
#[async_trait]
pub trait AgenticTier: Send + Sync {
    /// Attempt an autonomous recovery for `run_id`. Implementations persist their
    /// own `remediation_agent_session` row; the executor only consumes the
    /// returned [`AgentTierOutcome`].
    async fn attempt(&self, run_id: Uuid) -> AmpelResult<AgentTierOutcome>;
}

/// Map a terminal reason onto a bounded metric/label string.
pub fn terminal_label(reason: AgentTerminalReason) -> &'static str {
    match reason {
        AgentTerminalReason::CiGreen => "ci_green",
        AgentTerminalReason::BudgetExhausted => "budget_exhausted",
        AgentTerminalReason::MaxIterations => "max_iterations",
        AgentTerminalReason::Error => "error",
    }
}

/// Concrete DB- + harness-backed [`AgenticTier`].
///
/// Collaborators that touch the sandbox/CI (the [`AgentWorktree`] and
/// [`CiVerifier`]) are injected as traits so this is unit-testable with fakes and
/// a `MockModelProvider`; production wires the sandbox-backed implementations.
pub struct DbAgenticTier {
    db: DatabaseConnection,
    encryption: Arc<EncryptionService>,
    classifier: Arc<dyn FailureClassifier>,
    worktree: Arc<dyn AgentWorktree>,
    verifier: Arc<dyn CiVerifier>,
    /// Whether the resolved policy is air-gapped (ADR-014 ceiling).
    air_gapped: bool,
    /// Trusted run metadata for prompt rendering.
    run_ctx: PlaybookContext,
    /// Opaque sandbox worktree reference the agent edits in.
    worktree_ref: String,
    /// The current failing CI logs (untrusted data; carried in context blocks).
    failing_logs: String,
    /// Optional DB playbook override YAML (else the embedded default is used).
    playbook_override_yaml: Option<String>,
    /// Test seam: inject a provider (e.g. `MockModelProvider`) instead of the
    /// real reqwest factory. `None` in production → [`build_model_provider`].
    provider_override: Option<Arc<dyn ModelProvider>>,
}

impl DbAgenticTier {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: DatabaseConnection,
        encryption: Arc<EncryptionService>,
        classifier: Arc<dyn FailureClassifier>,
        worktree: Arc<dyn AgentWorktree>,
        verifier: Arc<dyn CiVerifier>,
        air_gapped: bool,
        run_ctx: PlaybookContext,
        worktree_ref: impl Into<String>,
        failing_logs: impl Into<String>,
    ) -> Self {
        Self {
            db,
            encryption,
            classifier,
            worktree,
            verifier,
            air_gapped,
            run_ctx,
            worktree_ref: worktree_ref.into(),
            failing_logs: failing_logs.into(),
            playbook_override_yaml: None,
            provider_override: None,
        }
    }

    /// Inject a DB playbook override (else the embedded default ships).
    pub fn with_playbook_override(mut self, yaml: Option<String>) -> Self {
        self.playbook_override_yaml = yaml;
        self
    }

    /// Inject a provider (test seam). Production omits this and the kind-driven
    /// [`build_model_provider`] factory is used instead.
    pub fn with_provider_override(mut self, provider: Arc<dyn ModelProvider>) -> Self {
        self.provider_override = Some(provider);
        self
    }

    /// Select the model account to drive this run. Prefers an org-scoped default,
    /// then any enabled account, deterministically ordered by `created_at`.
    async fn select_account(&self) -> AmpelResult<model_provider_account::Model> {
        model_provider_account::Entity::find()
            .filter(model_provider_account::Column::Enabled.eq(true))
            .order_by_desc(model_provider_account::Column::IsDefault)
            .order_by_asc(model_provider_account::Column::CreatedAt)
            .one(&self.db)
            .await
            .map_err(|e| AmpelError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                AmpelError::ValidationError("no enabled model provider account configured".into())
            })
    }

    /// Persist the agent-session row. Never contains secrets.
    #[allow(clippy::too_many_arguments)]
    async fn persist_session(
        &self,
        run_id: Uuid,
        account_id: Option<Uuid>,
        outcome: &AgentOutcome,
        failure_class: Option<String>,
        classifier_source: Option<String>,
        classifier_confidence: Option<f64>,
        status: &str,
    ) -> AmpelResult<()> {
        let now = Utc::now();
        let session = remediation_agent_session::ActiveModel {
            id: Set(Uuid::new_v4()),
            remediation_run_id: Set(run_id),
            model_provider_account_id: Set(account_id),
            playbook_ref: Set(None),
            iterations: Set(outcome.iterations as i32),
            max_iterations: Set(None),
            tokens_used: Set(outcome.tokens_used as i64),
            cost_usd: Set(Some(outcome.cost.to_string())),
            status: Set(status.to_string()),
            transcript_ref: Set(outcome.transcript_ref.clone()),
            failure_class: Set(failure_class),
            classifier_source: Set(classifier_source),
            classifier_confidence: Set(classifier_confidence),
            started_at: Set(now),
            completed_at: Set(Some(now)),
            created_at: Set(now),
        };
        session
            .insert(&self.db)
            .await
            .map_err(|e| AmpelError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Resolve the effective playbook (embedded default, optionally overridden)
    /// and its budget.
    fn resolve_playbook(&self) -> AmpelResult<(Playbook, AgentBudget)> {
        let yaml = match &self.playbook_override_yaml {
            Some(y) => y.clone(),
            None => embedded_default_yaml()?,
        };
        let playbook = Playbook::from_yaml(&yaml)?;
        let budget = playbook.loop_cfg.to_budget()?;
        Ok((playbook, budget))
    }
}

#[async_trait]
impl AgenticTier for DbAgenticTier {
    async fn attempt(&self, run_id: Uuid) -> AmpelResult<AgentTierOutcome> {
        // 1. Select the account + provider.
        let account = self.select_account().await?;
        let kind: ProviderKind = account.provider_kind.parse()?;
        let provider: Arc<dyn ModelProvider> = match &self.provider_override {
            Some(p) => p.clone(),
            None => build_model_provider(kind)?,
        };

        // 2. ADR-014 egress gate BEFORE any inference. Persist + hand off on block.
        if let Err(e) = assert_egress_allowed(self.air_gapped, provider.capabilities().egress) {
            let blocked = AgentOutcome {
                passed: false,
                iterations: 0,
                tokens_used: 0,
                cost: rust_decimal::Decimal::ZERO,
                transcript_ref: None,
                terminal_reason: AgentTerminalReason::Error,
            };
            self.persist_session(
                run_id,
                Some(account.id),
                &blocked,
                None,
                None,
                None,
                "egress_blocked",
            )
            .await?;
            tracing::warn!(%run_id, error = %e, "agentic tier refused: egress blocked");
            return Ok(AgentTierOutcome::Exhausted);
        }

        // 3. Decrypt credentials AT THE CALL SITE (never logged).
        let api_key = match &account.credentials_encrypted {
            Some(bytes) => Some(self.encryption.decrypt(bytes)?),
            None => None,
        };
        let creds = ModelCredentials {
            api_key,
            endpoint_url: account.endpoint_url.clone(),
            model_id: account.model_id.clone(),
            model_path: account.model_path.clone(),
        };

        // 4. Resolve playbook + budget, then run the harness.
        let (playbook, budget) = self.resolve_playbook()?;
        let classification = self.classifier.classify(&self.failing_logs).await;

        let harness = RemediationAgentHarness::new(self.classifier.clone());
        let outcome = harness
            .run(
                self.failing_logs.clone(),
                self.run_ctx.clone(),
                &playbook,
                provider,
                &creds,
                self.worktree.clone(),
                self.verifier.clone(),
                &self.worktree_ref,
                budget,
            )
            .await;

        // 5. Persist the session + emit metrics.
        let status = terminal_label(outcome.terminal_reason);
        self.persist_session(
            run_id,
            Some(account.id),
            &outcome,
            Some(classification.class.to_string()),
            Some(classification.source.to_string()),
            Some(classification.confidence as f64),
            status,
        )
        .await?;
        crate::observability::record_agent_session(
            status,
            outcome.iterations,
            outcome.cost.to_string().parse::<f64>().unwrap_or(0.0),
        );

        Ok(AgentTierOutcome::from_agent_outcome(&outcome))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ampel_core::remediation::{
        ClassificationResult, ClassifierSource, FailureClass, InferenceResponse, Modality,
        ModelCaps, ModelKind, NormalizedProviderOutput, OutputContract,
    };
    use ampel_core::remediation::{CostModel, MockModelProvider};

    fn local_caps() -> ModelCaps {
        ModelCaps {
            kind: ModelKind::Inference,
            modality: Modality::LocalServer,
            tool_use: false,
            code_edit: true,
            max_context_tokens: 32_000,
            cost: CostModel::Free,
            egress: Egress::LocalOnly,
            output_contract: OutputContract::UnifiedDiff,
        }
    }

    fn external_caps() -> ModelCaps {
        ModelCaps {
            egress: Egress::External,
            ..local_caps()
        }
    }

    #[test]
    fn should_allow_agentic_for_fix_and_full_tiers_only() {
        assert!(tier_allows_agentic(RemediationTier::FixAndConsolidate));
        assert!(tier_allows_agentic(RemediationTier::FullRemediation));
        assert!(!tier_allows_agentic(RemediationTier::ConsolidateOnly));
    }

    #[test]
    fn should_block_external_egress_when_air_gapped() {
        assert!(assert_egress_allowed(true, Egress::External).is_err());
    }

    #[test]
    fn should_allow_external_egress_when_not_air_gapped() {
        assert!(assert_egress_allowed(false, Egress::External).is_ok());
    }

    #[test]
    fn should_allow_local_egress_even_when_air_gapped() {
        assert!(assert_egress_allowed(true, Egress::LocalOnly).is_ok());
    }

    #[test]
    fn should_map_ci_green_outcome_to_recovered() {
        let outcome = AgentOutcome {
            passed: true,
            iterations: 2,
            tokens_used: 10,
            cost: rust_decimal::Decimal::ZERO,
            transcript_ref: None,
            terminal_reason: AgentTerminalReason::CiGreen,
        };
        assert_eq!(
            AgentTierOutcome::from_agent_outcome(&outcome),
            AgentTierOutcome::Recovered
        );
    }

    #[test]
    fn should_map_budget_exhausted_outcome_to_exhausted() {
        let outcome = AgentOutcome {
            passed: false,
            iterations: 3,
            tokens_used: 10,
            cost: rust_decimal::Decimal::ZERO,
            transcript_ref: None,
            terminal_reason: AgentTerminalReason::BudgetExhausted,
        };
        assert_eq!(
            AgentTierOutcome::from_agent_outcome(&outcome),
            AgentTierOutcome::Exhausted
        );
    }

    // Confirm the test-seam types line up (a Mock provider's egress is honored by
    // the gate); the full DB-backed `attempt` is covered by an integration test.
    #[test]
    fn should_honor_mock_provider_egress_in_gate() {
        let external = MockModelProvider::new(external_caps());
        let local = MockModelProvider::new(local_caps());
        assert!(assert_egress_allowed(true, external.capabilities().egress).is_err());
        assert!(assert_egress_allowed(true, local.capabilities().egress).is_ok());
        // Silence unused-import lints for response types referenced by sibling tests.
        let _ = InferenceResponse {
            output: NormalizedProviderOutput::Classification(ClassificationResult {
                class: FailureClass::BuildError,
                source: ClassifierSource::Heuristic,
                confidence: 1.0,
            }),
            tokens_used: 0,
            cost: rust_decimal::Decimal::ZERO,
        };
    }
}
