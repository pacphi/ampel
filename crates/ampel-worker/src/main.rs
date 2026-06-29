use std::str::FromStr;
use std::sync::Arc;

use apalis::prelude::*;
use apalis_cron::CronStream;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

rust_i18n::i18n!("locales", fallback = "en");

mod jobs;
mod observability;
mod providers;
mod services;

use ampel_core::services::SandboxRunner;
use jobs::{
    cleanup::CleanupJob, health_score::HealthScoreJob, metrics_collection::MetricsCollectionJob,
    poll_repository::PollRepositoryJob, remediation_sweep::RemediationSweepJob,
};
use services::PodmanSandboxRunner;

#[derive(Clone)]
pub struct WorkerState {
    pub db: sea_orm::DatabaseConnection,
    pub encryption_service: Arc<ampel_db::encryption::EncryptionService>,
    pub provider_factory: Arc<ampel_providers::ProviderFactory>,
    /// Sandbox runner used by the remediation jobs (Podman/Docker in prod).
    pub sandbox_runner: Arc<dyn SandboxRunner>,
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

    // Install the Prometheus scrape endpoint for remediation metrics. The
    // exporter serves `/metrics` on METRICS_PORT (default 9100) for Prometheus
    // to scrape; describe the metric names/units up front.
    let metrics_port = std::env::var("METRICS_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(9100);
    let metrics_addr = std::net::SocketAddr::from(([0, 0, 0, 0], metrics_port));
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(metrics_addr)
        .install()?;
    observability::describe_metrics();
    tracing::info!("Prometheus metrics listening on http://{metrics_addr}/metrics");

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

    let provider_factory = Arc::new(ampel_providers::ProviderFactory::new());

    // Sandbox runner for remediation. Detection is deferred to first use in
    // prod; if no runtime is configured/available we log and fall back to a
    // runner whose execution path errors cleanly (no panics at startup).
    let sandbox_runner: Arc<dyn SandboxRunner> = match PodmanSandboxRunner::from_env() {
        Ok(runner) => Arc::new(runner),
        Err(e) => {
            tracing::warn!("Sandbox runtime not configured ({e}); remediation runs will error until configured");
            Arc::new(PodmanSandboxRunner::new(services::SandboxConfig {
                runtime: services::sandbox_runner::SandboxRuntime::Podman,
                image: "ghcr.io/ampel/remediation-sandbox:latest".to_string(),
                clone_depth: 50,
                subprocess_timeout: std::time::Duration::from_secs(300),
            }))
        }
    };

    let state = WorkerState {
        db,
        encryption_service,
        provider_factory,
        sandbox_runner,
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
        })
        .register({
            WorkerBuilder::new("remediation-sweep")
                .data(state.clone())
                .backend(CronStream::new(
                    // Run every 15 minutes
                    apalis_cron::Schedule::from_str("0 */15 * * * *").unwrap(),
                ))
                .build_fn(run_remediation_sweep)
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

async fn run_remediation_sweep(
    _job: RemediationSweepJob,
    state: Data<WorkerState>,
) -> Result<(), Error> {
    tracing::info!("Running remediation sweep job");

    let job = jobs::remediation_sweep::RemediationSweepJob;
    if let Err(e) = job
        .execute(
            &state.db,
            &state.encryption_service,
            state.sandbox_runner.clone(),
        )
        .await
    {
        tracing::error!("Remediation sweep job failed: {}", e);
    }

    Ok(())
}
