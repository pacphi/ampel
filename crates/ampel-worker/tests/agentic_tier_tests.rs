//! CI-safe integration test for the Phase-4 Tier-2 `DbAgenticTier`.
//!
//! Exercises the full DB-facing path — account selection, the (no-key) local
//! provider, playbook + budget resolution, the harness loop driven by a
//! `MockModelProvider` (no network, no ONNX), and persistence of the
//! `remediation_agent_session` row — against in-memory SQLite. The model HTTP is
//! never touched (a scripted mock + fake worktree/verifier stand in).

use std::sync::Arc;

use ampel_core::errors::AmpelResult;
use ampel_core::remediation::{
    CostModel, Egress, HeuristicClassifier, InferenceResponse, MockModelProvider, Modality,
    ModelCaps, ModelKind, NormalizedProviderOutput, OutputContract,
};
use ampel_db::encryption::EncryptionService;
use ampel_db::entities::{model_provider_account, remediation_agent_session};
use ampel_worker::services::agent_harness::{AgentWorktree, CiVerifier, VerificationStatus};
use ampel_worker::services::agentic_tier::{AgentTierOutcome, AgenticTier, DbAgenticTier};
use ampel_worker::services::playbook_resolver::PlaybookContext;
use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, Schema, Set,
};
use uuid::Uuid;

/// Fresh SQLite with the two Phase-4 tables built directly from their entities
/// (the full Migrator is not SQLite-compatible).
async fn sqlite() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    // The agent_session entity carries an FK to `remediation_run`, which this
    // isolated test does not create; disable FK enforcement so the two Phase-4
    // tables can stand alone.
    db.execute_unprepared("PRAGMA foreign_keys = OFF;")
        .await
        .unwrap();
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    for stmt in [
        schema.create_table_from_entity(model_provider_account::Entity),
        schema.create_table_from_entity(remediation_agent_session::Entity),
    ] {
        db.execute(backend.build(&stmt)).await.unwrap();
    }
    db
}

async fn seed_ollama_account(db: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    model_provider_account::ActiveModel {
        id: Set(id),
        organization_id: Set(None),
        user_id: Set(Some(Uuid::new_v4())),
        provider_kind: Set("ollama".to_string()),
        display_name: Set("Local".to_string()),
        credentials_encrypted: Set(None),
        endpoint_url: Set(Some("http://localhost:11434".to_string())),
        egress_class: Set("local_only".to_string()),
        model_id: Set(Some("qwen2.5-coder".to_string())),
        enabled: Set(true),
        auth_type: Set("none".to_string()),
        spend_cap_usd: Set(None),
        spend_used_usd: Set("0".to_string()),
        validation_status: Set("valid".to_string()),
        last_validated_at: Set(None),
        model_path: Set(None),
        is_default: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await
    .unwrap();
    id
}

/// Fake worktree: records nothing, succeeds on every apply/commit.
struct OkWorktree;

#[async_trait]
impl AgentWorktree for OkWorktree {
    async fn apply_output(&self, _r: &str, _o: &NormalizedProviderOutput) -> AmpelResult<()> {
        Ok(())
    }
    async fn commit_and_push(&self, _r: &str, _m: &str) -> AmpelResult<()> {
        Ok(())
    }
}

/// Verifier that reports green on the first re-verify (the fix worked).
struct GreenVerifier;

#[async_trait]
impl CiVerifier for GreenVerifier {
    async fn verify(&self, _worktree_ref: &str) -> AmpelResult<VerificationStatus> {
        Ok(VerificationStatus {
            green: true,
            logs: String::new(),
        })
    }
}

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

fn diff_response() -> InferenceResponse {
    InferenceResponse {
        output: NormalizedProviderOutput::UnifiedDiff("--- a\n+++ b\n".to_string()),
        tokens_used: 42,
        cost: Decimal::ZERO,
    }
}

fn run_ctx() -> PlaybookContext {
    PlaybookContext {
        repo_full_name: "octo/ampel".into(),
        base_branch: "main".into(),
        failure_class: "build_error".into(),
    }
}

#[tokio::test]
async fn should_recover_and_persist_session_via_db_agentic_tier() {
    // Arrange: a local (no-egress) account, a mock model that emits one diff, a
    // worktree that applies cleanly, and CI that turns green.
    let db = sqlite().await;
    let account_id = seed_ollama_account(&db).await;
    let run_id = Uuid::new_v4();

    let provider = Arc::new(MockModelProvider::new(local_caps()).with_response(diff_response()));
    let tier = DbAgenticTier::new(
        db.clone(),
        Arc::new(EncryptionService::new(&[7u8; 32])),
        Arc::new(HeuristicClassifier),
        Arc::new(OkWorktree),
        Arc::new(GreenVerifier),
        /* air_gapped */ false,
        run_ctx(),
        "worktree-1",
        "error[E0432]: build failed",
    )
    .with_provider_override(provider);

    // Act
    let outcome = tier.attempt(run_id).await.unwrap();

    // Assert: the agent recovered, and exactly one session row was persisted with
    // the classifier snapshot + a green terminal status. No secrets anywhere.
    assert_eq!(outcome, AgentTierOutcome::Recovered);
    let sessions = remediation_agent_session::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    let s = &sessions[0];
    assert_eq!(s.remediation_run_id, run_id);
    assert_eq!(s.model_provider_account_id, Some(account_id));
    assert_eq!(s.status, "ci_green");
    assert!(s.iterations >= 1);
    assert_eq!(s.failure_class.as_deref(), Some("build_error"));
    assert_eq!(s.classifier_source.as_deref(), Some("heuristic"));
}

#[tokio::test]
async fn should_handoff_and_persist_egress_blocked_when_air_gapped_external() {
    // Arrange: an air-gapped policy with an External-egress provider must be
    // refused BEFORE any inference (ADR-014), persisting an `egress_blocked` row.
    let db = sqlite().await;
    seed_ollama_account(&db).await;
    let run_id = Uuid::new_v4();

    // External-egress mock; the gate should reject it under air_gapped = true.
    let external_caps = ModelCaps {
        egress: Egress::External,
        ..local_caps()
    };
    let provider = Arc::new(MockModelProvider::new(external_caps));
    let tier = DbAgenticTier::new(
        db.clone(),
        Arc::new(EncryptionService::new(&[7u8; 32])),
        Arc::new(HeuristicClassifier),
        Arc::new(OkWorktree),
        Arc::new(GreenVerifier),
        /* air_gapped */ true,
        run_ctx(),
        "worktree-1",
        "boom",
    )
    .with_provider_override(provider);

    // Act
    let outcome = tier.attempt(run_id).await.unwrap();

    // Assert: handed off, the model was never called, and the block was recorded.
    assert_eq!(outcome, AgentTierOutcome::Exhausted);
    let sessions = remediation_agent_session::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].status, "egress_blocked");
    assert_eq!(sessions[0].iterations, 0);
}
