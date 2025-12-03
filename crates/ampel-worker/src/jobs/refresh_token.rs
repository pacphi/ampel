use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use ampel_db::encryption::EncryptionService;
use ampel_providers::ProviderFactory;

/// Token refresh job - no longer needed with PAT-based authentication
/// PATs don't expire like OAuth tokens, so this job is a no-op.
/// Kept for compatibility with existing job scheduling infrastructure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenJob;

impl From<DateTime<Utc>> for RefreshTokenJob {
    fn from(_: DateTime<Utc>) -> Self {
        Self
    }
}

impl RefreshTokenJob {
    pub async fn execute(
        &self,
        _db: &DatabaseConnection,
        _encryption_service: &EncryptionService,
        _provider_factory: &ProviderFactory,
    ) -> anyhow::Result<()> {
        // PATs don't expire, so no refresh needed
        // This job is kept for compatibility but does nothing
        tracing::debug!("RefreshTokenJob: PATs don't need refreshing, skipping");
        Ok(())
    }
}
