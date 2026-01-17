pub mod i18n_backend;

use i18n_backend::NestedYamlBackend;

// Initialize rust-i18n with our custom nested YAML backend
// The backend reads from locales/{lang}/*.yml at runtime
rust_i18n::i18n!(
    "locales",
    fallback = "en",
    backend = NestedYamlBackend::new(concat!(env!("CARGO_MANIFEST_DIR"), "/locales"))
);

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
