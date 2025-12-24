/// Integration tests module
///
/// This module organizes all integration tests for the database layer.
/// Each test file focuses on a specific domain or query module.
// Re-export common test utilities
#[path = "../common/mod.rs"]
mod common;

// Test modules
mod dashboard_queries;
mod pr_queries;
mod provider_account_queries;
mod repo_queries;
mod user_queries;
