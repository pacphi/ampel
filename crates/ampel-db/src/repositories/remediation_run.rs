//! SeaORM implementation of `ampel_core::services::RemediationRunRepository`.
//!
//! This is the DI seam that breaks the `ampel-db -> ampel-core` cycle: the trait
//! lives in `ampel-core`; the concrete write-side lives here over the canonical
//! `remediation_run` / `remediation_run_pr` entities.
//!
//! The state machine's only mutator, [`transition_state`], is a true
//! compare-and-swap: `UPDATE ... SET state = :to WHERE id = :id AND state = :from`.
//! A `rows_affected != 1` result means the run was not in `from` (a concurrent
//! modification) and surfaces as `Ok(false)` — the caller must re-read.
//!
//! [`transition_state`]: RemediationRunRepository::transition_state

use std::str::FromStr;

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::remediation::{AutonomyLevel, ConsolidationPlan, MergeDisposition, RunState};
use ampel_core::services::{RemediationRun, RemediationRunRepository, RunUpdate};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::entities::{remediation_run, remediation_run_pr};

/// PostgreSQL/SQLite-backed [`RemediationRunRepository`].
#[derive(Clone)]
pub struct SeaOrmRemediationRunRepository {
    db: DatabaseConnection,
}

impl SeaOrmRemediationRunRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn db_err(e: DbErr) -> AmpelError {
    AmpelError::DatabaseError(e.to_string())
}

/// Project an entity row into the orchestrator's read model, parsing the
/// snake_case string columns back into typed enums.
fn to_read_model(m: remediation_run::Model) -> AmpelResult<RemediationRun> {
    Ok(RemediationRun {
        id: m.id,
        repository_id: m.repository_id,
        state: RunState::from_str(&m.state)?,
        autonomy_level: AutonomyLevel::from_str(&m.autonomy_level)?,
        consolidated_pr_number: m.consolidated_pr_number,
        head_sha: m.head_sha,
        error_message: m.error_message,
    })
}

#[async_trait]
impl RemediationRunRepository for SeaOrmRemediationRunRepository {
    async fn create_run(
        &self,
        repository_id: Uuid,
        autonomy_level: AutonomyLevel,
    ) -> AmpelResult<RemediationRun> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        // The deterministic branch name (ADR-005). `policy_id` is set to the nil
        // UUID sentinel here: the `RemediationRunRepository` contract does not
        // thread a policy through `create_run`, and the column carries no FK.
        let model = remediation_run::ActiveModel {
            id: Set(id),
            repository_id: Set(repository_id),
            policy_id: Set(Uuid::nil()),
            triggered_by: Set("system".to_string()),
            triggered_by_user_id: Set(None),
            state: Set(RunState::Created.to_string()),
            autonomy_level: Set(autonomy_level.to_string()),
            head_sha: Set(None),
            pr_selection_snapshot: Set("[]".to_string()),
            consolidation_plan: Set(None),
            consolidated_pr_number: Set(None),
            merged: Set(false),
            branch_name: Set(format!("ampel/remediation/{id}")),
            ci_status: Set("pending".to_string()),
            ci_logs_url: Set(None),
            merge_strategy: Set(None),
            attempts: Set(0),
            error_message: Set(None),
            error_class: Set(None),
            started_at: Set(now),
            completed_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        remediation_run::Entity::insert(model)
            .exec(&self.db)
            .await
            .map_err(db_err)?;

        Ok(RemediationRun {
            id,
            repository_id,
            state: RunState::Created,
            autonomy_level,
            consolidated_pr_number: None,
            head_sha: None,
            error_message: None,
        })
    }

    async fn get_run(&self, id: Uuid) -> AmpelResult<Option<RemediationRun>> {
        let found = remediation_run::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(db_err)?;
        match found {
            Some(m) => Ok(Some(to_read_model(m)?)),
            None => Ok(None),
        }
    }

    async fn transition_state(
        &self,
        id: Uuid,
        from: RunState,
        to: RunState,
        updates: RunUpdate,
    ) -> AmpelResult<bool> {
        // Reject illegal edges before touching the DB (mirrors the in-memory fake).
        if !from.can_transition_to(to) {
            return Err(AmpelError::ValidationError(format!(
                "illegal run transition: {from} -> {to}"
            )));
        }

        // CAS: the WHERE clause pins the observed `from` state. Exactly one row
        // updates iff the run is still in `from`; otherwise rows_affected == 0.
        let mut update = remediation_run::Entity::update_many()
            .col_expr(remediation_run::Column::State, Expr::value(to.to_string()))
            .col_expr(remediation_run::Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(remediation_run::Column::Id.eq(id))
            .filter(remediation_run::Column::State.eq(from.to_string()));

        if let Some(pr) = updates.consolidated_pr_number {
            update = update.col_expr(
                remediation_run::Column::ConsolidatedPrNumber,
                Expr::value(pr),
            );
        }
        if let Some(sha) = updates.head_sha {
            update = update.col_expr(remediation_run::Column::HeadSha, Expr::value(sha));
        }
        if let Some(msg) = updates.error_message {
            update = update.col_expr(remediation_run::Column::ErrorMessage, Expr::value(msg));
        }

        let res = update.exec(&self.db).await.map_err(db_err)?;
        Ok(res.rows_affected == 1)
    }

    async fn record_disposition(
        &self,
        run_id: Uuid,
        pr_number: i64,
        disposition: MergeDisposition,
    ) -> AmpelResult<()> {
        let disposition_json = serde_json::to_string(&disposition)
            .map_err(|e| AmpelError::InternalError(e.to_string()))?;
        let model = remediation_run_pr::ActiveModel {
            id: Set(Uuid::new_v4()),
            remediation_run_id: Set(run_id),
            pr_number: Set(pr_number),
            disposition: Set(disposition_json),
            created_at: Set(Utc::now()),
        };
        remediation_run_pr::Entity::insert(model)
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn set_consolidation_plan(
        &self,
        run_id: Uuid,
        plan: ConsolidationPlan,
    ) -> AmpelResult<()> {
        let plan_json =
            serde_json::to_string(&plan).map_err(|e| AmpelError::InternalError(e.to_string()))?;
        remediation_run::Entity::update_many()
            .col_expr(
                remediation_run::Column::ConsolidationPlan,
                Expr::value(plan_json),
            )
            .col_expr(remediation_run::Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(remediation_run::Column::Id.eq(run_id))
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn set_consolidated_pr(&self, run_id: Uuid, pr_number: i64) -> AmpelResult<()> {
        remediation_run::Entity::update_many()
            .col_expr(
                remediation_run::Column::ConsolidatedPrNumber,
                Expr::value(pr_number),
            )
            .col_expr(remediation_run::Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(remediation_run::Column::Id.eq(run_id))
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn closed_source_prs(&self, run_id: Uuid) -> AmpelResult<Vec<i64>> {
        let rows = remediation_run_pr::Entity::find()
            .filter(remediation_run_pr::Column::RemediationRunId.eq(run_id))
            .all(&self.db)
            .await
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            // A `ClosedWithRef` disposition marks a source PR already superseded.
            if let Ok(MergeDisposition::ClosedWithRef { .. }) =
                serde_json::from_str::<MergeDisposition>(&row.disposition)
            {
                out.push(row.pr_number);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::SchemaManager;

    /// Build a fresh in-memory SQLite DB with just the remediation tables +
    /// Phase-2 columns, applying the self-contained migrations directly (the full
    /// `Migrator` skips SQLite — see `migrations/mod.rs`).
    async fn sqlite_with_remediation_tables() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect sqlite");
        let manager = SchemaManager::new(&conn);
        crate::migrations::test_support::apply_remediation_schema(&manager)
            .await
            .expect("apply remediation schema");
        conn
    }

    #[tokio::test]
    async fn should_create_run_in_created_state_and_read_it_back() {
        // Arrange
        let repo = SeaOrmRemediationRunRepository::new(sqlite_with_remediation_tables().await);
        let repository_id = Uuid::new_v4();

        // Act
        let created = repo
            .create_run(repository_id, AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        let fetched = repo.get_run(created.id).await.unwrap().unwrap();

        // Assert
        assert_eq!(fetched.state, RunState::Created);
        assert_eq!(fetched.autonomy_level, AutonomyLevel::FullyAutonomous);
        assert_eq!(fetched.repository_id, repository_id);
    }

    #[tokio::test]
    async fn should_cas_transition_when_from_matches() {
        // Arrange
        let repo = SeaOrmRemediationRunRepository::new(sqlite_with_remediation_tables().await);
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act
        let swapped = repo
            .transition_state(
                run.id,
                RunState::Created,
                RunState::Selecting,
                RunUpdate::none(),
            )
            .await
            .unwrap();

        // Assert
        assert!(swapped);
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::Selecting
        );
    }

    #[tokio::test]
    async fn should_fail_cas_when_from_state_is_wrong() {
        // Arrange: advance past `created` so a stale `from = created` writer loses.
        let repo = SeaOrmRemediationRunRepository::new(sqlite_with_remediation_tables().await);
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        repo.transition_state(
            run.id,
            RunState::Created,
            RunState::Selecting,
            RunUpdate::none(),
        )
        .await
        .unwrap();

        // Act: a second writer still believes the state is `created`.
        let swapped = repo
            .transition_state(
                run.id,
                RunState::Created,
                RunState::Selecting,
                RunUpdate::none(),
            )
            .await
            .unwrap();

        // Assert: CAS loses the race, state unchanged.
        assert!(!swapped);
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::Selecting
        );
    }

    #[tokio::test]
    async fn should_reject_illegal_transition_and_leave_state_unchanged() {
        // Arrange: a fresh run in `created`.
        let repo = SeaOrmRemediationRunRepository::new(sqlite_with_remediation_tables().await);
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act: created -> merging is not a legal edge (skips selecting/…/verifying).
        let result = repo
            .transition_state(
                run.id,
                RunState::Created,
                RunState::Merging,
                RunUpdate::none(),
            )
            .await;

        // Assert: rejected before touching the DB; persisted state still `created`.
        assert!(matches!(result, Err(AmpelError::ValidationError(_))));
        assert_eq!(
            repo.get_run(run.id).await.unwrap().unwrap().state,
            RunState::Created
        );
    }

    #[tokio::test]
    async fn should_persist_head_sha_with_transition() {
        // Arrange
        let repo = SeaOrmRemediationRunRepository::new(sqlite_with_remediation_tables().await);
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();
        repo.transition_state(
            run.id,
            RunState::Created,
            RunState::Selecting,
            RunUpdate::none(),
        )
        .await
        .unwrap();
        repo.transition_state(
            run.id,
            RunState::Selecting,
            RunState::Consolidating,
            RunUpdate::none(),
        )
        .await
        .unwrap();

        // Act
        repo.transition_state(
            run.id,
            RunState::Consolidating,
            RunState::Verifying,
            RunUpdate::with_head_sha("deadbeefcafe"),
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(
            repo.get_run(run.id)
                .await
                .unwrap()
                .unwrap()
                .head_sha
                .as_deref(),
            Some("deadbeefcafe")
        );
    }

    #[tokio::test]
    async fn should_record_disposition_row() {
        // Arrange
        let conn = sqlite_with_remediation_tables().await;
        let repo = SeaOrmRemediationRunRepository::new(conn.clone());
        let run = repo
            .create_run(Uuid::new_v4(), AutonomyLevel::FullyAutonomous)
            .await
            .unwrap();

        // Act
        repo.record_disposition(run.id, 7, MergeDisposition::Consolidated)
            .await
            .unwrap();

        // Assert: exactly one disposition row persisted for the run with the
        // JSON-serialized value object.
        let rows = remediation_run_pr::Entity::find()
            .filter(remediation_run_pr::Column::RemediationRunId.eq(run.id))
            .all(&conn)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pr_number, 7);
        assert!(rows[0].disposition.contains("consolidated"));
    }
}
