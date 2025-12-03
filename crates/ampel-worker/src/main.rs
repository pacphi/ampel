use std::str::FromStr;
use std::sync::Arc;

use apalis::prelude::*;
use apalis_cron::CronStream;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod jobs;

use jobs::{
    cleanup::CleanupJob, health_score::HealthScoreJob, metrics_collection::MetricsCollectionJob,
    poll_repository::PollRepositoryJob, refresh_token::RefreshTokenJob,
};

#[derive(Clone)]
pub struct WorkerState {
    pub db: sea_orm::DatabaseConnection,
    pub encryption_service: Arc<ampel_db::encryption::EncryptionService>,
    pub provider_factory: Arc<ampel_providers::ProviderFactory>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,ampel=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Ampel Worker...");

    // Load configuration
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let encryption_key = std::env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");

    // Initialize database
    let db = ampel_db::init_database(&database_url).await?;
    tracing::info!("Database connection established");

    // Initialize services
    let encryption_service = Arc::new(
        ampel_db::encryption::EncryptionService::from_base64_key(&encryption_key)
            .expect("Invalid encryption key"),
    );

    let provider_factory = Arc::new(ampel_providers::ProviderFactory::new(
        std::env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
        std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
        std::env::var("GITHUB_REDIRECT_URI").unwrap_or_default(),
        std::env::var("GITLAB_CLIENT_ID").unwrap_or_default(),
        std::env::var("GITLAB_CLIENT_SECRET").unwrap_or_default(),
        std::env::var("GITLAB_REDIRECT_URI").unwrap_or_default(),
        std::env::var("GITLAB_BASE_URL").ok(),
        std::env::var("BITBUCKET_CLIENT_ID").unwrap_or_default(),
        std::env::var("BITBUCKET_CLIENT_SECRET").unwrap_or_default(),
        std::env::var("BITBUCKET_REDIRECT_URI").unwrap_or_default(),
    ));

    let state = WorkerState {
        db,
        encryption_service,
        provider_factory,
    };

    // Create job monitors
    let poll_monitor = Monitor::new()
        .register({
            WorkerBuilder::new("poll-repository")
                .data(state.clone())
                .backend(CronStream::new(
                    // Run every minute
                    apalis_cron::Schedule::from_str("0 * * * * *").unwrap(),
                ))
                .build_fn(poll_repositories)
        })
        .register({
            WorkerBuilder::new("refresh-tokens")
                .data(state.clone())
                .backend(CronStream::new(
                    // Run every 30 minutes
                    apalis_cron::Schedule::from_str("0 */30 * * * *").unwrap(),
                ))
                .build_fn(refresh_tokens)
        })
        .register({
            WorkerBuilder::new("cleanup")
                .data(state.clone())
                .backend(CronStream::new(
                    // Run daily at midnight
                    apalis_cron::Schedule::from_str("0 0 0 * * *").unwrap(),
                ))
                .build_fn(run_cleanup)
        })
        .register({
            WorkerBuilder::new("metrics-collection")
                .data(state.clone())
                .backend(CronStream::new(
                    // Run every 5 minutes
                    apalis_cron::Schedule::from_str("0 */5 * * * *").unwrap(),
                ))
                .build_fn(collect_metrics)
        })
        .register({
            WorkerBuilder::new("health-score")
                .data(state.clone())
                .backend(CronStream::new(
                    // Run every hour
                    apalis_cron::Schedule::from_str("0 0 * * * *").unwrap(),
                ))
                .build_fn(calculate_health_scores)
        });

    tracing::info!("Starting job monitors...");

    poll_monitor.run().await?;

    Ok(())
}

async fn poll_repositories(_job: PollRepositoryJob, state: Data<WorkerState>) -> Result<(), Error> {
    tracing::info!("Running repository poll job");

    let job = jobs::poll_repository::PollRepositoryJob;
    if let Err(e) = job
        .execute(
            &state.db,
            &state.encryption_service,
            &state.provider_factory,
        )
        .await
    {
        tracing::error!("Poll job failed: {}", e);
    }

    Ok(())
}

async fn refresh_tokens(_job: RefreshTokenJob, state: Data<WorkerState>) -> Result<(), Error> {
    tracing::info!("Running token refresh job");

    let job = jobs::refresh_token::RefreshTokenJob;
    if let Err(e) = job
        .execute(
            &state.db,
            &state.encryption_service,
            &state.provider_factory,
        )
        .await
    {
        tracing::error!("Token refresh job failed: {}", e);
    }

    Ok(())
}

async fn run_cleanup(_job: CleanupJob, state: Data<WorkerState>) -> Result<(), Error> {
    tracing::info!("Running cleanup job");

    let job = jobs::cleanup::CleanupJob;
    if let Err(e) = job.execute(&state.db).await {
        tracing::error!("Cleanup job failed: {}", e);
    }

    Ok(())
}

async fn collect_metrics(
    _job: MetricsCollectionJob,
    state: Data<WorkerState>,
) -> Result<(), Error> {
    tracing::info!("Running metrics collection job");

    let job = jobs::metrics_collection::MetricsCollectionJob;
    if let Err(e) = job.execute(&state.db).await {
        tracing::error!("Metrics collection job failed: {}", e);
    }

    Ok(())
}

async fn calculate_health_scores(
    _job: HealthScoreJob,
    state: Data<WorkerState>,
) -> Result<(), Error> {
    tracing::info!("Running health score calculation job");

    let job = jobs::health_score::HealthScoreJob;
    if let Err(e) = job.execute(&state.db).await {
        tracing::error!("Health score job failed: {}", e);
    }

    Ok(())
}
