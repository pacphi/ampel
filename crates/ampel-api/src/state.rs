use std::sync::Arc;

use metrics_exporter_prometheus::PrometheusHandle;
use sea_orm::DatabaseConnection;

use ampel_core::services::AuthService;
use ampel_db::encryption::EncryptionService;
use ampel_providers::ProviderFactory;

use crate::{cache::RedisConnectionManager, Config};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: Option<RedisConnectionManager>,
    pub auth_service: Arc<AuthService>,
    pub encryption_service: Arc<EncryptionService>,
    pub provider_factory: Arc<ProviderFactory>,
    pub config: Arc<Config>,
    pub metrics_handle: PrometheusHandle,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        redis: Option<RedisConnectionManager>,
        auth_service: AuthService,
        encryption_service: EncryptionService,
        provider_factory: ProviderFactory,
        config: Config,
        metrics_handle: PrometheusHandle,
    ) -> Self {
        Self {
            db,
            redis,
            auth_service: Arc::new(auth_service),
            encryption_service: Arc::new(encryption_service),
            provider_factory: Arc::new(provider_factory),
            config: Arc::new(config),
            metrics_handle,
        }
    }
}
