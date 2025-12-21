/// Integration tests module
///
/// This module organizes all integration tests for the database layer.
/// Each test file focuses on a specific domain or query module.
// Re-export common test utilities
#[path = "../common/mod.rs"]
mod common;

// Test modules
mod provider_account_queries;

// Additional test modules can be added here as needed:
// mod user_queries;
// mod repository_queries;
// mod pull_request_queries;
