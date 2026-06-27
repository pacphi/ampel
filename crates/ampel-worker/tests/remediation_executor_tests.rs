//! CI-safe integration tests for the Phase-2 remediation write path.
//!
//! These exercise the real wiring — `SeaOrmRemediationRunRepository` on SQLite,
//! the worker's `ProviderAdapter` over `ampel_providers::MockProvider`, and the
//! `RemediationExecutor`/`RemediationOrchestrator` — with `FakeSandboxRunner`
//! standing in for the container. No containers, no network, no Postgres: they
//! run on CI as-is.

use std::sync::Arc;

use ampel_core::remediation::{AutonomyLevel, PrRef, RunState};
use ampel_core::services::{ConsolidateResult, FakeSandboxRunner};
use ampel_core::services::{
    CredentialHandle, MergeOutcome, RemediationOrchestrator, RemediationProvider,
    RemediationRunRepository, RepoContext, VerificationService,
};
use ampel_db::migrations::test_support::apply_remediation_schema;
use ampel_db::repositories::SeaOrmRemediationRunRepository;
use ampel_providers::mock::{MockProvider, RemediationCall};
use ampel_providers::traits::{ProviderCICheck, ProviderCredentials};
use ampel_providers::RemediationCapable;
use ampel_worker::services::{ProviderAdapter, RemediationExecutor, RunOutcome};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::SchemaManager;
use uuid::Uuid;

const OWNER: &str = "acme";
const REPO: &str = "widgets";
const CONSOLIDATED_PR: i64 = 9001;

/// Fresh in-memory SQLite with just the remediation schema applied.
async fn sqlite() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect sqlite");
    let manager = SchemaManager::new(&conn);
    apply_remediation_schema(&manager)
        .await
        .expect("apply remediation schema");
    conn
}

fn green_check(name: &str) -> ProviderCICheck {
    ProviderCICheck {
        name: name.to_string(),
        status: "completed".to_string(),
        conclusion: Some("success".to_string()),
        url: None,
        started_at: None,
        completed_at: None,
    }
}

fn red_check(name: &str) -> ProviderCICheck {
    ProviderCICheck {
        name: name.to_string(),
        status: "completed".to_string(),
        conclusion: Some("failure".to_string()),
        url: None,
        started_at: None,
        completed_at: None,
    }
}

fn pr(n: i32) -> PrRef {
    PrRef {
        number: n,
        title: format!("Bump dep {n}"),
        branch: format!("dependabot/dep-{n}"),
    }
}

fn repo_ctx() -> RepoContext {
    RepoContext {
        clone_url: "https://example.test/acme/widgets.git".into(),
        default_branch: "main".into(),
        credential: CredentialHandle::new("ghp_secret_pat"),
    }
}

fn adapter(mock: &MockProvider) -> Arc<dyn RemediationProvider> {
    let provider: Arc<dyn RemediationCapable> = Arc::new(mock.clone());
    Arc::new(ProviderAdapter::new(
        provider,
        ProviderCredentials::Pat {
            token: "ghp_secret_pat".into(),
            username: None,
        },
        OWNER,
        REPO,
        vec!["build".to_string()],
    ))
}

#[tokio::test]
async fn should_reach_completed_and_close_sources_on_happy_path() {
    // Arrange: 3 PRs, green CI for the consolidated PR, fully-autonomous policy.
    let conn = sqlite().await;
    let run_repo: Arc<dyn RemediationRunRepository> =
        Arc::new(SeaOrmRemediationRunRepository::new(conn.clone()));
    let sandbox = Arc::new(FakeSandboxRunner::with_outcome(
        Some(CONSOLIDATED_PR),
        "headsha",
    ));
    let mock = MockProvider::new().with_ci_checks(
        OWNER,
        REPO,
        CONSOLIDATED_PR as i32,
        vec![green_check("build")],
    );
    let executor = RemediationExecutor::new(
        run_repo.clone(),
        sandbox,
        VerificationService::new(),
        adapter(&mock),
    );
    let run = run_repo
        .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
        .await
        .unwrap();

    // Act
    let outcome = executor
        .execute(run.id, vec![pr(1), pr(2), pr(3)], repo_ctx())
        .await
        .unwrap();

    // Assert: completed, with each source PR closed + a "Superseded by" comment.
    assert_eq!(outcome, RunOutcome::Completed);
    assert_eq!(
        run_repo.get_run(run.id).await.unwrap().unwrap().state,
        RunState::Completed
    );

    let calls = mock.remediation_calls();
    let closed: Vec<i32> = calls
        .iter()
        .filter_map(|c| match c {
            RemediationCall::ClosePullRequest { pr_number, .. } => Some(*pr_number),
            _ => None,
        })
        .collect();
    assert_eq!(closed, vec![1, 2, 3]);
    let comments: Vec<&str> = calls
        .iter()
        .filter_map(|c| match c {
            RemediationCall::CreateComment { body, .. } => Some(body.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(comments.len(), 3);
    assert!(comments
        .iter()
        .all(|b| *b == format!("Superseded by #{CONSOLIDATED_PR}")));
}

#[tokio::test]
async fn should_no_op_with_zero_provider_writes_under_dry_run() {
    // Arrange: dry-run autonomy must not touch the sandbox or the provider.
    let conn = sqlite().await;
    let run_repo: Arc<dyn RemediationRunRepository> =
        Arc::new(SeaOrmRemediationRunRepository::new(conn.clone()));
    let sandbox = Arc::new(FakeSandboxRunner::new());
    let mock = MockProvider::new();
    let executor = RemediationExecutor::new(
        run_repo.clone(),
        sandbox.clone(),
        VerificationService::new(),
        adapter(&mock),
    );
    let run = run_repo
        .create_run(Uuid::new_v4(), AutonomyLevel::DryRunOnly)
        .await
        .unwrap();

    // Act
    let outcome = executor
        .execute(run.id, vec![pr(1), pr(2), pr(3)], repo_ctx())
        .await
        .unwrap();

    // Assert: parked in no_op; zero provider writes; sandbox untouched.
    assert_eq!(outcome, RunOutcome::NoOp);
    assert_eq!(
        run_repo.get_run(run.id).await.unwrap().unwrap().state,
        RunState::NoOp
    );
    assert!(mock.remediation_calls().is_empty());
    assert!(!sandbox.was_invoked());
}

#[tokio::test]
async fn should_handoff_without_merge_when_sha_changes_at_gate() {
    // Arrange: drive the orchestrator step-by-step so the consolidated branch's
    // anchor SHA can move between verify and the merge-gate re-verify (TOCTOU).
    let conn = sqlite().await;
    let run_repo: Arc<dyn RemediationRunRepository> =
        Arc::new(SeaOrmRemediationRunRepository::new(conn.clone()));
    let sandbox = Arc::new(FakeSandboxRunner::with_outcome(
        Some(CONSOLIDATED_PR),
        "headsha",
    ));
    let mock = MockProvider::new()
        .with_ci_checks(
            OWNER,
            REPO,
            CONSOLIDATED_PR as i32,
            vec![green_check("build")],
        )
        .with_default_branch_sha("sha-old");
    let orchestrator = RemediationOrchestrator::new(
        run_repo.clone(),
        sandbox,
        VerificationService::new(),
        adapter(&mock),
    );
    let run = run_repo
        .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
        .await
        .unwrap();

    // Act: consolidate + verify (green, SHA "sha-old"), then the base moves.
    let consolidated = orchestrator
        .consolidate(run.id, vec![pr(1), pr(2), pr(3)], repo_ctx())
        .await
        .unwrap();
    assert!(matches!(consolidated, ConsolidateResult::Consolidated(_)));
    let verification = orchestrator.verify(run.id).await.unwrap();
    assert!(verification.is_safe_to_merge());

    // The anchor SHA changes out from under the run before the merge gate.
    let _ = mock.clone().with_default_branch_sha("sha-new");
    let merge = orchestrator.do_merge(run.id).await.unwrap();

    // Assert: handed off, never merged, never closed a source PR.
    assert!(matches!(merge, MergeOutcome::HandedOff { .. }));
    assert_eq!(
        run_repo.get_run(run.id).await.unwrap().unwrap().state,
        RunState::HandoffHuman
    );
    assert!(!mock
        .remediation_calls()
        .iter()
        .any(|c| matches!(c, RemediationCall::ClosePullRequest { .. })));
}

#[tokio::test]
async fn should_handoff_without_merge_when_ci_turns_red_at_gate() {
    // Arrange: green at verify, but CI flips red before the merge-gate re-verify.
    let conn = sqlite().await;
    let run_repo: Arc<dyn RemediationRunRepository> =
        Arc::new(SeaOrmRemediationRunRepository::new(conn.clone()));
    let sandbox = Arc::new(FakeSandboxRunner::with_outcome(
        Some(CONSOLIDATED_PR),
        "headsha",
    ));
    let mock = MockProvider::new().with_ci_checks(
        OWNER,
        REPO,
        CONSOLIDATED_PR as i32,
        vec![green_check("build")],
    );
    let orchestrator = RemediationOrchestrator::new(
        run_repo.clone(),
        sandbox,
        VerificationService::new(),
        adapter(&mock),
    );
    let run = run_repo
        .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
        .await
        .unwrap();

    // Act: consolidate + verify (green), then the required check turns red.
    orchestrator
        .consolidate(run.id, vec![pr(1), pr(2), pr(3)], repo_ctx())
        .await
        .unwrap();
    assert!(orchestrator
        .verify(run.id)
        .await
        .unwrap()
        .is_safe_to_merge());
    let _ = mock.clone().with_ci_checks(
        OWNER,
        REPO,
        CONSOLIDATED_PR as i32,
        vec![red_check("build")],
    );
    let merge = orchestrator.do_merge(run.id).await.unwrap();

    // Assert: handed off (not safe), never merged, never closed a source PR.
    assert!(matches!(merge, MergeOutcome::HandedOff { .. }));
    assert_eq!(
        run_repo.get_run(run.id).await.unwrap().unwrap().state,
        RunState::HandoffHuman
    );
    assert!(!mock
        .remediation_calls()
        .iter()
        .any(|c| matches!(c, RemediationCall::ClosePullRequest { .. })));
}
