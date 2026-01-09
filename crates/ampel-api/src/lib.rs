rust_i18n::i18n!("locales", fallback = "en");

pub mod cache;
pub mod config;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod observability;
pub mod routes;
pub mod state;

pub use config::Config;
pub use observability::{health_handler, init_metrics, metrics_handler, readiness_handler};
pub use state::AppState;
