//! Per-provider parity suite for the Fleet PR Remediation path (Phase 5a DoD).
//!
//! The SAME consolidation/verify/merge/finalize scenario is driven three times —
//! once per provider mock (GitHub all caps, GitLab all caps, Bitbucket partial
//! caps) — through the real worker wiring: `SeaOrmRemediationRunRepository` on
//! SQLite, the worker `ProviderAdapter` over `ampel_providers::MockProvider`, and
//! the `RemediationExecutor`/`RemediationOrchestrator`, with `FakeSandboxRunner`
//! standing in for the container. No network, no containers, no Postgres.
//!
//! The DoD is two-fold:
//!   1. *Parity*: all three providers reach the SAME successful end state
//!      (`completed`, every source PR closed with a "Superseded by" comment).
//!   2. *Fallback proof*: GitHub/GitLab take the PRIMARY `get_status_for_ref`
//!      path (recorded as `RemediationCall::GetStatusForRef`), while Bitbucket —
//!      whose caps mark arbitrary-ref status unsupported — takes the
//!      `get_ci_checks` FALLBACK and records ZERO `GetStatusForRef` calls, yet
//!      still reaches the same end state.
//!
//! ## Bitbucket caps modeled here
//!
//! The live `BitbucketProvider::capabilities()` reports `update_branch_from_base`
//! and `add_labels` as unsupported. This suite models those *and* additionally
//! marks `get_status_for_ref` unsupported, because the only `RemediationProvider`
//! seam operation with an asserted primary-vs-fallback split is CI status. This
//! lets the parity suite prove the capability-driven fallback machinery end to
//! end. (`update_branch_from_base`/`add_labels` are not part of this seam: the
//! sandbox clone-push already yields the merged branch, and labels are never
//! issued — both degrade to no-ops rather than fallbacks.)

use std::sync::Arc;

use ampel_core::models::GitProvider as Provider;
use ampel_core::remediation::{AutonomyLevel, PrRef, RunState};
use ampel_core::services::FakeSandboxRunner;
use ampel_core::services::{
    CredentialHandle, RemediationProvider, RemediationRunRepository, RepoContext,
    VerificationService,
};
use ampel_db::migrations::test_support::apply_remediation_schema;
use ampel_db::repositories::SeaOrmRemediationRunRepository;
use ampel_providers::mock::{MockProvider, RemediationCall};
use ampel_providers::remediation::RemediationCaps;
use ampel_providers::traits::{ProviderCICheck, ProviderCredentials, ProviderPullRequest};
use ampel_providers::RemediationCapable;
use ampel_worker::services::{ProviderAdapter, RemediationExecutor, RunOutcome};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::SchemaManager;
use uuid::Uuid;

const OWNER: &str = "acme";
const REPO: &str = "widgets";
const CONSOLIDATED_PR: i64 = 9001;

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

fn consolidated_pr_record(number: i32) -> ProviderPullRequest {
    let now = chrono::Utc::now();
    ProviderPullRequest {
        provider_id: number.to_string(),
        number,
        title: "Consolidated".to_string(),
        description: None,
        url: format!("https://example.test/pr/{number}"),
        state: "open".to_string(),
        source_branch: "ampel/remediation".to_string(),
        target_branch: "main".to_string(),
        author: "ampel".to_string(),
        author_avatar_url: None,
        is_draft: false,
        is_mergeable: Some(true),
        has_conflicts: false,
        additions: 0,
        deletions: 0,
        changed_files: 0,
        commits_count: 0,
        comments_count: 0,
        created_at: now,
        updated_at: now,
        merged_at: None,
        closed_at: None,
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

/// Caps modeling a partial-support Bitbucket deployment: no arbitrary-ref status,
/// no branch-update primitive, no PR labels. Mirrors the live provider's two
/// unsupported flags and adds `get_status_for_ref = false` (see module docs).
fn bitbucket_partial_caps() -> RemediationCaps {
    RemediationCaps {
        update_branch_from_base: false,
        add_labels: false,
        get_status_for_ref: false,
        ..RemediationCaps::all()
    }
}

/// A green, mergeable consolidated PR keyed under `OWNER/REPO/CONSOLIDATED_PR`.
/// The same CI-checks key is read by BOTH the primary (`get_status_for_ref` with
/// the PR number as ref) and the fallback (`get_ci_checks` for the PR number),
/// so the scenario is byte-for-byte identical across providers.
fn green_mergeable_mock(kind: Provider, caps: Option<RemediationCaps>) -> MockProvider {
    let mut mock = MockProvider::new_with_provider(kind)
        .with_ci_checks(
            OWNER,
            REPO,
            CONSOLIDATED_PR as i32,
            vec![green_check("build")],
        )
        .with_pull_requests(
            OWNER,
            REPO,
            vec![consolidated_pr_record(CONSOLIDATED_PR as i32)],
        );
    if let Some(caps) = caps {
        mock = mock.with_remediation_caps(caps);
    }
    mock
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

fn used_get_status_for_ref(calls: &[RemediationCall]) -> bool {
    calls
        .iter()
        .any(|c| matches!(c, RemediationCall::GetStatusForRef { .. }))
}

fn closed_pr_numbers(calls: &[RemediationCall]) -> Vec<i32> {
    calls
        .iter()
        .filter_map(|c| match c {
            RemediationCall::ClosePullRequest { pr_number, .. } => Some(*pr_number),
            _ => None,
        })
        .collect()
}

/// Run the identical happy-path scenario against one provider mock and return
/// `(outcome, final_state, remediation_calls)`.
async fn run_scenario(mock: &MockProvider) -> (RunOutcome, RunState, Vec<RemediationCall>) {
    let conn = sqlite().await;
    let run_repo: Arc<dyn RemediationRunRepository> =
        Arc::new(SeaOrmRemediationRunRepository::new(conn.clone()));
    let sandbox = Arc::new(FakeSandboxRunner::with_outcome(
        Some(CONSOLIDATED_PR),
        "headsha",
    ));
    let executor = RemediationExecutor::new(
        run_repo.clone(),
        sandbox,
        VerificationService::new(),
        adapter(mock),
    );
    let run = run_repo
        .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
        .await
        .unwrap();

    let outcome = executor
        .execute(run.id, vec![pr(1), pr(2), pr(3)], repo_ctx())
        .await
        .unwrap();
    let state = run_repo.get_run(run.id).await.unwrap().unwrap().state;
    (outcome, state, mock.remediation_calls())
}

#[tokio::test]
async fn should_reach_completed_via_primary_status_path_for_github() {
    // Arrange: GitHub advertises all caps (default).
    let mock = green_mergeable_mock(Provider::GitHub, None);

    // Act
    let (outcome, state, calls) = run_scenario(&mock).await;

    // Assert: completed via the PRIMARY get_status_for_ref path; sources closed.
    assert_eq!(outcome, RunOutcome::Completed);
    assert_eq!(state, RunState::Completed);
    assert!(used_get_status_for_ref(&calls));
    assert_eq!(closed_pr_numbers(&calls), vec![1, 2, 3]);
}

#[tokio::test]
async fn should_reach_completed_via_primary_status_path_for_gitlab() {
    // Arrange: GitLab advertises all caps (default).
    let mock = green_mergeable_mock(Provider::GitLab, None);

    // Act
    let (outcome, state, calls) = run_scenario(&mock).await;

    // Assert: completed via the PRIMARY get_status_for_ref path; sources closed.
    assert_eq!(outcome, RunOutcome::Completed);
    assert_eq!(state, RunState::Completed);
    assert!(used_get_status_for_ref(&calls));
    assert_eq!(closed_pr_numbers(&calls), vec![1, 2, 3]);
}

#[tokio::test]
async fn should_reach_completed_via_fallback_status_path_for_bitbucket() {
    // Arrange: Bitbucket advertises PARTIAL caps (no arbitrary-ref status).
    let mock = green_mergeable_mock(Provider::Bitbucket, Some(bitbucket_partial_caps()));

    // Act
    let (outcome, state, calls) = run_scenario(&mock).await;

    // Assert: SAME end state as GitHub/GitLab, but the FALLBACK was used — zero
    // GetStatusForRef calls (the adapter routed to get_ci_checks instead), yet
    // every source PR is still closed.
    assert_eq!(outcome, RunOutcome::Completed);
    assert_eq!(state, RunState::Completed);
    assert!(
        !used_get_status_for_ref(&calls),
        "Bitbucket must use the get_ci_checks fallback, not the arbitrary-ref primary path"
    );
    assert_eq!(closed_pr_numbers(&calls), vec![1, 2, 3]);
}

#[tokio::test]
async fn should_close_sources_with_superseded_comment_across_all_providers() {
    // Arrange + Act + Assert: identical finalize behavior (audit-trail comment +
    // close) for every provider, proving cross-provider parity of the close path.
    for (kind, caps) in [
        (Provider::GitHub, None),
        (Provider::GitLab, None),
        (Provider::Bitbucket, Some(bitbucket_partial_caps())),
    ] {
        let mock = green_mergeable_mock(kind, caps);
        let (outcome, _state, calls) = run_scenario(&mock).await;
        assert_eq!(outcome, RunOutcome::Completed, "{kind:?} should complete");

        let comments: Vec<&str> = calls
            .iter()
            .filter_map(|c| match c {
                RemediationCall::CreateComment { body, .. } => Some(body.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(comments.len(), 3, "{kind:?} should comment on 3 sources");
        assert!(
            comments
                .iter()
                .all(|b| *b == format!("Superseded by #{CONSOLIDATED_PR}")),
            "{kind:?} comments must be the superseded audit trail"
        );
    }
}
