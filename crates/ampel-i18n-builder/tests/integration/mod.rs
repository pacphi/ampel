// Integration tests for ampel-i18n-builder
//
// Test Coverage:
// - Format parsing (YAML/JSON) with nested structures
// - Pluralization rules (2, 3, 6 forms for different languages)
// - Variable placeholder validation and preservation
// - Translation coverage and validation
// - DeepL API client with retry and rate limiting
// - Caching with TTL and LRU eviction
// - CLI command execution
// - Code generation (TypeScript types, Rust constants)

mod api_client_tests;
mod cache_tests;
mod cli_tests;
mod code_generation_tests;
mod config_tests;
mod fallback_tests;
mod fallback_router_tests;
mod format_parser_tests;
mod pluralization_tests;
mod provider_tests;
mod rate_limiting_tests;
mod recursive_translation_tests;
mod translation_api_tests;
mod validation_tests;
