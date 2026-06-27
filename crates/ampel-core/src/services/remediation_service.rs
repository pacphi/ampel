//! Phase 1 remediation orchestration: PR selection + dry-run preview.
//!
//! Security note: this service is **read-only** in Phase 1. `preview` performs
//! ZERO repository writes by construction — no `RemediationCapable` provider is
//! wired in, so no write primitive (push/merge/comment) is reachable from this
//! code path. Write-capable autonomy tiers are gated off until later phases (see
//! [`AutonomyLevel::allows_writes`]).

use crate::errors::{AmpelError, AmpelResult};
use crate::remediation::db;
use crate::remediation::{ConsolidationPlan, PrRef, PrSelectionStrategy, RemediationCriteria};
use crate::services::PolicyResolver;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashSet;
use uuid::Uuid;

/// Orchestrates Phase 1 remediation reads (selection + preview).
#[derive(Clone)]
pub struct RemediationService {
    db: DatabaseConnection,
    resolver: PolicyResolver,
}

impl RemediationService {
    pub fn new(db: DatabaseConnection) -> Self {
        let resolver = PolicyResolver::new(db.clone());
        Self { db, resolver }
    }

    /// Construct with an explicit resolver (e.g. a shared one).
    pub fn with_resolver(db: DatabaseConnection, resolver: PolicyResolver) -> Self {
        Self { db, resolver }
    }

    /// Select the open PRs for `repo_id` that satisfy `criteria`.
    ///
    /// Filters applied: open state, optional draft skip, optional target-branch
    /// allow-list, then the [`PrSelectionStrategy`], finally capped at
    /// `max_prs_per_run`. Read-only.
    pub async fn select_prs(
        &self,
        repo_id: Uuid,
        criteria: &RemediationCriteria,
    ) -> AmpelResult<Vec<PrRef>> {
        let mut query = db::pull_requests::Entity::find()
            .filter(db::pull_requests::Column::RepositoryId.eq(repo_id))
            .filter(db::pull_requests::Column::State.eq("open"));

        if criteria.skip_draft {
            query = query.filter(db::pull_requests::Column::IsDraft.eq(false));
        }
        if !criteria.allowed_targets.is_empty() {
            query = query.filter(
                db::pull_requests::Column::TargetBranch.is_in(criteria.allowed_targets.clone()),
            );
        }

        // Deterministic oldest-first ordering; strategies refine from here.
        let mut rows = query
            .order_by_asc(db::pull_requests::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(db_err)?;

        rows = apply_strategy(rows, &criteria.pr_selection);

        // Hard cap regardless of strategy.
        let cap = criteria.max_prs_per_run.max(0) as usize;
        rows.truncate(cap);

        Ok(rows.into_iter().map(to_pr_ref).collect())
    }

    /// Produce a dry-run [`ConsolidationPlan`] for `repo_id`. Read-only.
    ///
    /// Returns [`AmpelError::NotFound`] when no policy resolves for the repo.
    pub async fn preview(&self, repo_id: Uuid) -> AmpelResult<ConsolidationPlan> {
        let criteria =
            self.resolver.resolve(repo_id).await?.ok_or_else(|| {
                AmpelError::NotFound("no remediation policy for repository".into())
            })?;

        let selected = self.select_prs(repo_id, &criteria).await?;
        Ok(ConsolidationPlan::from_selection(
            selected,
            criteria.air_gapped,
        ))
    }
}

fn db_err(e: DbErr) -> AmpelError {
    AmpelError::DatabaseError(e.to_string())
}

fn to_pr_ref(m: db::pull_requests::Model) -> PrRef {
    PrRef {
        number: m.number,
        title: m.title,
        branch: m.source_branch,
    }
}

/// Apply the selection strategy to the already-filtered, oldest-first rows.
fn apply_strategy(
    rows: Vec<db::pull_requests::Model>,
    strategy: &PrSelectionStrategy,
) -> Vec<db::pull_requests::Model> {
    match strategy {
        PrSelectionStrategy::AllOpen => rows,
        PrSelectionStrategy::OldestFirst { max } => {
            let mut rows = rows;
            rows.truncate(*max as usize);
            rows
        }
        // Phase 1: PR labels are not persisted on `pull_requests`, so there is
        // no label data to match against — resolves to an empty selection.
        PrSelectionStrategy::ByLabel { .. } => Vec::new(),
        PrSelectionStrategy::ExplicitIds { ids } => {
            let wanted: HashSet<i64> = ids.iter().copied().collect();
            rows.into_iter()
                .filter(|r| wanted.contains(&(r.number as i64)))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remediation::testkit;
    use crate::remediation::{AutonomyLevel, RemediationTier};

    fn criteria(strategy: PrSelectionStrategy, skip_draft: bool, max: i32) -> RemediationCriteria {
        RemediationCriteria {
            min_open_prs: 1,
            pr_selection: strategy,
            max_prs_per_run: max,
            allowed_targets: vec![],
            skip_draft,
            require_green_before_merge: true,
            air_gapped: false,
            autonomy_level: AutonomyLevel::DryRunOnly,
            remediation_tier: RemediationTier::ConsolidateOnly,
        }
    }

    async fn seeded_repo(db: &DatabaseConnection) -> Uuid {
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(db, user).await;
        // oldest -> newest by offset (larger offset = older)
        testkit::seed_pr(db, repo, 1, "main", "open", false, 300).await; // oldest
        testkit::seed_pr(db, repo, 2, "main", "open", false, 200).await;
        testkit::seed_pr(db, repo, 3, "develop", "open", true, 100).await; // draft
        testkit::seed_pr(db, repo, 4, "main", "closed", false, 50).await; // closed
        repo
    }

    #[tokio::test]
    async fn should_select_all_open_non_closed_prs() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(repo, &criteria(PrSelectionStrategy::AllOpen, false, 100))
            .await
            .unwrap();

        // Assert: 3 open PRs (the closed one is excluded), drafts included.
        assert_eq!(prs.len(), 3);
    }

    #[tokio::test]
    async fn should_skip_draft_prs_when_skip_draft_set() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(repo, &criteria(PrSelectionStrategy::AllOpen, true, 100))
            .await
            .unwrap();

        // Assert: draft PR #3 excluded -> 2 remain.
        assert_eq!(prs.len(), 2);
        assert!(prs.iter().all(|p| p.number != 3));
    }

    #[tokio::test]
    async fn should_select_oldest_first_with_max() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(
                repo,
                &criteria(PrSelectionStrategy::OldestFirst { max: 1 }, false, 100),
            )
            .await
            .unwrap();

        // Assert: only the single oldest (PR #1) is selected.
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 1);
    }

    #[tokio::test]
    async fn should_cap_at_max_prs_per_run() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act: AllOpen would yield 3, but max cap is 2.
        let prs = svc
            .select_prs(repo, &criteria(PrSelectionStrategy::AllOpen, false, 2))
            .await
            .unwrap();

        // Assert
        assert_eq!(prs.len(), 2);
    }

    #[tokio::test]
    async fn should_select_only_explicit_ids() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(
                repo,
                &criteria(
                    PrSelectionStrategy::ExplicitIds { ids: vec![2] },
                    false,
                    100,
                ),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 2);
    }

    #[tokio::test]
    async fn should_select_nothing_for_by_label_in_phase_1() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let svc = RemediationService::new(db);

        // Act
        let prs = svc
            .select_prs(
                repo,
                &criteria(
                    PrSelectionStrategy::ByLabel {
                        labels: vec!["deps".into()],
                    },
                    false,
                    100,
                ),
            )
            .await
            .unwrap();

        // Assert: labels not persisted yet -> empty.
        assert!(prs.is_empty());
    }

    #[tokio::test]
    async fn should_filter_by_allowed_targets() {
        // Arrange
        let db = testkit::memory_db().await;
        let repo = seeded_repo(&db).await;
        let mut c = criteria(PrSelectionStrategy::AllOpen, false, 100);
        c.allowed_targets = vec!["develop".into()];
        let svc = RemediationService::new(db);

        // Act
        let prs = svc.select_prs(repo, &c).await.unwrap();

        // Assert: only the develop-targeted PR (#3) matches.
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 3);
    }

    #[tokio::test]
    async fn should_preview_with_correct_pr_count() {
        // Arrange
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        testkit::seed_pr(&db, repo, 1, "main", "open", false, 200).await;
        testkit::seed_pr(&db, repo, 2, "main", "open", false, 100).await;
        testkit::seed_policy(
            &db,
            "repository",
            repo,
            "dry_run_only",
            "consolidate_only",
            "\"all_open\"",
            "[\"main\"]",
            false,
            10,
            false,
        )
        .await;
        let svc = RemediationService::new(db);

        // Act
        let plan = svc.preview(repo).await.unwrap();

        // Assert
        assert_eq!(plan.pr_count, 2);
        assert_eq!(plan.would_select.len(), 2);
        assert!(!plan.blocked_by_air_gap);
    }

    #[tokio::test]
    async fn should_flag_blocked_by_air_gap_in_preview_when_air_gapped() {
        // Arrange: org ceiling makes the resolved criteria air-gapped.
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let _org = testkit::seed_org(&db, user, true).await;
        testkit::seed_pr(&db, repo, 1, "main", "open", false, 100).await;
        testkit::seed_policy(
            &db,
            "repository",
            repo,
            "dry_run_only",
            "consolidate_only",
            "\"all_open\"",
            "[\"main\"]",
            false,
            10,
            false,
        )
        .await;
        let svc = RemediationService::new(db);

        // Act
        let plan = svc.preview(repo).await.unwrap();

        // Assert
        assert!(plan.air_gapped);
        assert!(plan.blocked_by_air_gap);
    }

    #[tokio::test]
    async fn should_error_preview_when_no_policy() {
        // Arrange
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let svc = RemediationService::new(db);

        // Act
        let result = svc.preview(repo).await;

        // Assert
        assert!(matches!(result, Err(AmpelError::NotFound(_))));
    }
}
