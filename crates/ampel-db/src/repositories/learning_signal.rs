//! SeaORM implementation of the `ampel-core` strategy-learning seams (Phase 5b):
//! [`LearningSignalRecorder`] (write) and [`LearningStatsReader`] (read).
//!
//! These are the DI seam that breaks the `ampel-db -> ampel-core` cycle: the
//! traits live in `ampel-core`; the concrete persistence lives here over the
//! `learning_signal` entity. The recorder is append-only; the reader aggregates
//! per-provider pass-rate for one failure class.

use std::collections::HashMap;
use std::str::FromStr;

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::remediation::{FailureClass, ProviderKind};
use ampel_core::services::{
    LearningSignal, LearningSignalRecorder, LearningStatsReader, ProviderStats,
};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::entities::learning_signal;

/// PostgreSQL/SQLite-backed recorder + reader for `learning_signal`.
#[derive(Clone)]
pub struct SeaOrmLearningSignalRepository {
    db: DatabaseConnection,
}

impl SeaOrmLearningSignalRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn db_err(e: DbErr) -> AmpelError {
    AmpelError::DatabaseError(e.to_string())
}

#[async_trait]
impl LearningSignalRecorder for SeaOrmLearningSignalRepository {
    async fn record(&self, signal: LearningSignal) -> AmpelResult<()> {
        let model = learning_signal::ActiveModel {
            id: Set(Uuid::new_v4()),
            provider: Set(signal.provider.to_string()),
            failure_class: Set(signal.failure_class.to_string()),
            playbook_id: Set(signal.playbook_id),
            playbook_version: Set(signal.playbook_version),
            outcome: Set(signal.outcome.to_string()),
            duration_secs: Set(signal.duration_secs),
            cost_usd: Set(signal.cost_usd.map(|c| c.to_string())),
            created_at: Set(Utc::now()),
        };
        learning_signal::Entity::insert(model)
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}

#[async_trait]
impl LearningStatsReader for SeaOrmLearningSignalRepository {
    async fn provider_stats(&self, failure_class: FailureClass) -> AmpelResult<Vec<ProviderStats>> {
        let rows = learning_signal::Entity::find()
            .filter(learning_signal::Column::FailureClass.eq(failure_class.to_string()))
            .all(&self.db)
            .await
            .map_err(db_err)?;

        // Aggregate (total, passed) per parseable provider kind. Rows with an
        // unrecognized provider string are skipped rather than failing the read.
        let mut agg: HashMap<ProviderKind, (u64, u64)> = HashMap::new();
        for row in rows {
            let Ok(provider) = ProviderKind::from_str(&row.provider) else {
                continue;
            };
            let entry = agg.entry(provider).or_insert((0, 0));
            entry.0 += 1;
            if row.outcome == "passed" {
                entry.1 += 1;
            }
        }

        Ok(agg
            .into_iter()
            .map(|(provider, (total, passed))| ProviderStats {
                provider,
                total,
                passed,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ampel_core::services::LearningOutcome;
    use sea_orm::{ConnectionTrait, DbBackend};
    use sea_orm::{Database, Schema};

    async fn sqlite_with_learning_signal() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let backend = db.get_database_backend();
        assert_eq!(backend, DbBackend::Sqlite);
        let schema = Schema::new(backend);
        let stmt = schema.create_table_from_entity(learning_signal::Entity);
        db.execute(backend.build(&stmt)).await.unwrap();
        db
    }

    fn signal(provider: ProviderKind, outcome: LearningOutcome) -> LearningSignal {
        LearningSignal {
            provider,
            failure_class: FailureClass::BuildError,
            playbook_id: "global".into(),
            playbook_version: 1,
            outcome,
            duration_secs: 5,
            cost_usd: None,
        }
    }

    #[tokio::test]
    async fn should_persist_a_signal_row() {
        // Arrange
        let repo = SeaOrmLearningSignalRepository::new(sqlite_with_learning_signal().await);

        // Act
        repo.record(signal(ProviderKind::Claude, LearningOutcome::Passed))
            .await
            .unwrap();

        // Assert
        let rows = learning_signal::Entity::find().all(&repo.db).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].provider, "claude");
        assert_eq!(rows[0].outcome, "passed");
        assert_eq!(rows[0].failure_class, "build_error");
    }

    #[tokio::test]
    async fn should_aggregate_pass_rate_per_provider() {
        // Arrange: Claude 2/3 passed; Ollama 0/1 for build_error.
        let repo = SeaOrmLearningSignalRepository::new(sqlite_with_learning_signal().await);
        for outcome in [
            LearningOutcome::Passed,
            LearningOutcome::Passed,
            LearningOutcome::Exhausted,
        ] {
            repo.record(signal(ProviderKind::Claude, outcome))
                .await
                .unwrap();
        }
        repo.record(signal(ProviderKind::Ollama, LearningOutcome::Exhausted))
            .await
            .unwrap();

        // Act
        let mut stats = repo.provider_stats(FailureClass::BuildError).await.unwrap();
        stats.sort_by_key(|s| s.provider.to_string());

        // Assert
        let claude = stats
            .iter()
            .find(|s| s.provider == ProviderKind::Claude)
            .unwrap();
        assert_eq!((claude.total, claude.passed), (3, 2));
        let ollama = stats
            .iter()
            .find(|s| s.provider == ProviderKind::Ollama)
            .unwrap();
        assert_eq!((ollama.total, ollama.passed), (1, 0));
    }

    #[tokio::test]
    async fn should_exclude_other_failure_classes_from_stats() {
        // Arrange: one build_error passed signal; the query is for a different class.
        let repo = SeaOrmLearningSignalRepository::new(sqlite_with_learning_signal().await);
        repo.record(signal(ProviderKind::Claude, LearningOutcome::Passed))
            .await
            .unwrap();

        // Act
        let stats = repo
            .provider_stats(FailureClass::LockfileConflict)
            .await
            .unwrap();

        // Assert
        assert!(stats.is_empty());
    }
}
