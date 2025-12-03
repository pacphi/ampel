use chrono::{DateTime, Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use ampel_core::models::GitProvider;
use ampel_db::encryption::EncryptionService;
use ampel_db::entities::git_provider;
use ampel_providers::ProviderFactory;

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
        db: &DatabaseConnection,
        encryption_service: &EncryptionService,
        provider_factory: &ProviderFactory,
    ) -> anyhow::Result<()> {
        let now = Utc::now();
        // Find tokens expiring in the next hour
        let expiry_threshold = now + Duration::hours(1);

        let connections = git_provider::Entity::find()
            .filter(git_provider::Column::TokenExpiresAt.lt(expiry_threshold))
            .filter(git_provider::Column::RefreshTokenEncrypted.is_not_null())
            .all(db)
            .await?;

        tracing::info!("Found {} tokens to refresh", connections.len());

        for connection in connections {
            if let Err(e) = self
                .refresh_single_token(db, encryption_service, provider_factory, connection)
                .await
            {
                tracing::error!("Failed to refresh token: {}", e);
            }
        }

        Ok(())
    }

    async fn refresh_single_token(
        &self,
        db: &DatabaseConnection,
        encryption_service: &EncryptionService,
        provider_factory: &ProviderFactory,
        connection: git_provider::Model,
    ) -> anyhow::Result<()> {
        let provider_type: GitProvider = connection
            .provider
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;

        // Decrypt refresh token
        let refresh_token_encrypted = connection
            .refresh_token_encrypted
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No refresh token"))?;

        let refresh_token = encryption_service.decrypt(refresh_token_encrypted)?;

        let provider = provider_factory.create(provider_type);

        // Request new tokens
        let new_tokens = provider.refresh_token(&refresh_token).await?;

        // Encrypt new tokens
        let access_token_encrypted = encryption_service.encrypt(&new_tokens.access_token)?;
        let refresh_token_encrypted = new_tokens
            .refresh_token
            .as_ref()
            .map(|rt| encryption_service.encrypt(rt))
            .transpose()?;

        // Update connection
        let mut active: git_provider::ActiveModel = connection.into();
        active.access_token_encrypted = Set(access_token_encrypted);
        if let Some(rt) = refresh_token_encrypted {
            active.refresh_token_encrypted = Set(Some(rt));
        }
        active.token_expires_at = Set(new_tokens.expires_at);
        active.updated_at = Set(Utc::now());
        active.update(db).await?;

        tracing::info!(
            "Successfully refreshed token for {} provider",
            provider_type
        );

        Ok(())
    }
}
