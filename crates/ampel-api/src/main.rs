use std::net::SocketAddr;

use axum::http::{header, Method};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ampel_api::{observability, routes, AppState, Config};
use ampel_core::services::AuthService;
use ampel_db::{encryption::EncryptionService, init_database, run_migrations};
use ampel_providers::ProviderFactory;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,ampel=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Ampel API server...");

    // Initialize metrics
    let metrics_handle = observability::init_metrics();
    tracing::info!("Metrics exporter initialized");

    // Load configuration
    let config = Config::from_env();
    tracing::info!("Configuration loaded");

    // Initialize database
    let db = init_database(&config.database_url).await?;
    tracing::info!("Database connection established");

    // Run migrations
    run_migrations(&db).await?;
    tracing::info!("Database migrations applied");

    // Initialize Redis connection (optional)
    let redis = if let Some(redis_url) = &config.redis_url {
        match redis::Client::open(redis_url.as_str()) {
            Ok(client) => match client.get_connection_manager().await {
                Ok(conn_mgr) => {
                    tracing::info!("Redis connection established");
                    Some(conn_mgr)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to connect to Redis, continuing without cache");
                    None
                }
            },
            Err(e) => {
                tracing::warn!(error = %e, "Failed to create Redis client, continuing without cache");
                None
            }
        }
    } else {
        tracing::info!("Redis URL not configured, caching disabled");
        None
    };

    // Initialize services
    let auth_service = AuthService::new(
        config.jwt_secret.clone(),
        config.jwt_access_expiry_minutes,
        config.jwt_refresh_expiry_days,
    );

    let encryption_service =
        EncryptionService::from_base64_key(&config.encryption_key).expect("Invalid encryption key");

    let provider_factory = ProviderFactory::new();

    // Create app state
    let state = AppState::new(
        db,
        redis,
        auth_service,
        encryption_service,
        provider_factory,
        config.clone(),
        metrics_handle,
    );

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .cors_origins
                .iter()
                .map(|o| o.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true);

    // Build router with Swagger UI
    let app = routes::create_router(state.clone())
        .merge(ampel_api::swagger_ui())
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
