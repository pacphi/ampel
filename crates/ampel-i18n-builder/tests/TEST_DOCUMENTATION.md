# Testing Documentation - ampel-i18n-builder

## Overview

This document describes the comprehensive test suite for the ampel-i18n-builder crate, focusing on the 4-tier translation provider architecture with intelligent fallback routing.

## Test Coverage

### Current Test Suite Statistics

- **Total Test Files**: 13 integration test modules
- **Coverage Goal**: 80%+ code coverage
- **Test Types**: Unit tests, integration tests, feature-gated real API tests

### Test Organization

```
tests/integration/
├── api_client_tests.rs        # API client implementation tests
├── cache_tests.rs              # LRU cache and TTL tests
├── cli_tests.rs                # CLI command execution
├── code_generation_tests.rs    # TypeScript/Rust codegen
├── config_tests.rs             # Configuration parsing/validation (NEW)
├── fallback_tests.rs           # 4-tier fallback routing (NEW)
├── format_parser_tests.rs      # YAML/JSON parsing
├── pluralization_tests.rs      # Plural form handling
├── provider_tests.rs           # Provider-specific tests (NEW)
├── rate_limiting_tests.rs      # Rate limiting behavior
├── recursive_translation_tests.rs # Nested structure translation
├── translation_api_tests.rs    # Translation API mocks
└── validation_tests.rs         # Placeholder validation
```

## Running Tests

### All Tests (Unit + Integration)

```bash
cargo test --package ampel-i18n-builder --all-features
```

### Unit Tests Only

```bash
cargo test --package ampel-i18n-builder --lib
```

### Integration Tests Only

```bash
cargo test --package ampel-i18n-builder --test integration
```

### Specific Test File

```bash
cargo test --package ampel-i18n-builder --test fallback_tests
cargo test --package ampel-i18n-builder --test provider_tests
cargo test --package ampel-i18n-builder --test config_tests
```

### Feature-Gated Tests (Real API Calls)

Tests that make real API calls are feature-gated and ignored by default.

**Setup**:

```bash
# Set API keys
export DEEPL_API_KEY="your_deepl_key"
export GOOGLE_API_KEY="your_google_key"
export OPENAI_API_KEY="your_openai_key"
```

**Run**:

```bash
# Run all integration tests including real API calls
cargo test --package ampel-i18n-builder --features integration-tests -- --ignored

# Run specific real API test
cargo test --package ampel-i18n-builder --features integration-tests test_real_deepl_translation -- --ignored
```

### Watch Mode (Development)

```bash
cargo watch -x "test --package ampel-i18n-builder"
```

## Test Categories

### 1. Fallback Routing Tests (`fallback_tests.rs`)

Tests the 4-tier provider fallback system.

**Test Scenarios**:

- ✅ Router initialization with no providers (error case)
- ✅ Router initialization with single provider
- ✅ Provider selection for DeepL-preferred languages (fi, sv, de, fr, pl, cs)
- ✅ Provider selection for Google-preferred languages (ar, th, vi, hi)
- ✅ Multiple providers fallback priority
- ✅ Batch size limits (DeepL: 50, Google: 100)
- ✅ Empty text batch handling
- ✅ Concurrent translation requests (thread-safety)
- ✅ Provider tier ordering validation

**Feature-Gated Tests** (`#[cfg(feature = "integration-tests")]`):

- Real DeepL API translation (requires `DEEPL_API_KEY`)
- Fallback from invalid DeepL key to Google (requires `GOOGLE_API_KEY`)

**Example**:

```bash
# Run fallback tests
cargo test --package ampel-i18n-builder fallback_tests

# Run with real APIs
export DEEPL_API_KEY="your_key"
export GOOGLE_API_KEY="your_key"
cargo test --features integration-tests test_real_fallback_deepl_to_google -- --ignored
```

### 2. Provider-Specific Tests (`provider_tests.rs`)

Tests individual provider configurations and behavior.

**Test Coverage**:

**ProviderConfig Tests**:

- ✅ Default configuration values
- ✅ Custom configuration
- ✅ Exponential backoff calculation
- ✅ Max retries configuration
- ✅ Timeout values (short/long)

**Batch Size Tests**:

- ✅ DeepL batch limit (50 texts)
- ✅ Google batch limit (100 texts)
- ✅ Batch splitting calculation for large datasets

**Rate Limiting Tests**:

- ✅ Rate limit configuration (DeepL: 10 req/sec, Google: 100 req/sec)
- ✅ Rate limit calculation (requests per minute, delay per request)

**Feature-Gated Tests**:

- Real DeepL translation with retry logic
- Real Google translation
- Provider retry on rate limit (needs mockito for safe testing)

**Placeholder Preservation**:

- ✅ Detection of placeholders like `{{count}}`, `{{name}}`
- ✅ Multiple placeholders in single text

**Provider Tier Values**:

- ✅ Tier 1: Systran
- ✅ Tier 2: DeepL
- ✅ Tier 3: Google
- ✅ Tier 4: OpenAI

### 3. Configuration Tests (`config_tests.rs`)

Tests configuration loading and validation.

**Test Coverage**:

- ✅ Default configuration values
- ✅ Translation config defaults
- ✅ Environment variable overrides (DEEPL_API_KEY, GOOGLE_API_KEY, OPENAI_API_KEY)
- ✅ YAML file loading (`.ampel-i18n.yaml`)
- ✅ Missing file fallback to defaults
- ✅ Timeout validation (1s to 300s)
- ✅ Batch size validation (10 to 200 texts)
- ✅ All providers configuration
- ✅ Config serialization/deserialization
- ✅ Partial YAML config (some fields specified, rest default)
- ✅ Config cloning
- ✅ Debug output
- ✅ Invalid YAML handling (error case)
- ✅ Environment variable override priority

**Example Config File** (`.ampel-i18n.yaml`):

```yaml
translation_dir: 'frontend/public/locales'
translation:
  timeout_secs: 60
  batch_size: 100
```

### 4. Existing Test Suites

All existing tests remain functional and are part of the comprehensive suite:

- **API Client Tests**: DeepL/Google API client implementation
- **Cache Tests**: LRU cache with TTL and eviction
- **CLI Tests**: Command-line interface execution
- **Code Generation**: TypeScript types and Rust constants
- **Format Parser**: YAML/JSON parsing with nested structures
- **Pluralization**: 2/3/6 plural forms for different languages
- **Rate Limiting**: Token bucket rate limiting
- **Recursive Translation**: Nested structure traversal
- **Translation API**: Mocked API responses
- **Validation**: Placeholder validation and preservation

## Coverage Reporting

### Generate Coverage Report

**Using tarpaulin**:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --package ampel-i18n-builder --all-features --out Html --output-dir coverage

# Open report
open coverage/index.html
```

**Using llvm-cov**:

```bash
# Install llvm-cov
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --package ampel-i18n-builder --html

# Open report
open target/llvm-cov/html/index.html
```

### Coverage Goals

| Component           | Target | Critical Paths                     |
| ------------------- | ------ | ---------------------------------- |
| **Overall**         | 80%+   | -                                  |
| **Fallback Router** | 100%   | Provider selection, fallback logic |
| **Configuration**   | 100%   | Parsing, validation, env overrides |
| **Providers**       | 90%+   | Retry logic, batch splitting       |
| **Error Handling**  | 100%   | All error paths                    |

## Test Fixtures and Utilities

### Mock HTTP Responses

Tests use `mockito` for stubbing API responses:

```rust
use mockito::{Matcher, Server};

let mut server = Server::new_async().await;
let mock = server
    .mock("POST", "/v2/translate")
    .with_status(200)
    .with_body(r#"{"translations": [{"text": "Hei"}]}"#)
    .create_async()
    .await;
```

### Test Configuration Helper

```rust
fn create_test_config() -> Config {
    Config {
        translation_dir: PathBuf::from("test"),
        translation: TranslationConfig {
            deepl_api_key: None,
            google_api_key: None,
            openai_api_key: None,
            timeout_secs: 5,
            batch_size: 50,
        },
    }
}
```

### Temporary Directory Helper

```rust
use tempfile::TempDir;

let temp_dir = TempDir::new().unwrap();
let config_path = temp_dir.path().join(".ampel-i18n.yaml");
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
- name: Run tests
  run: cargo test --package ampel-i18n-builder --all-features

- name: Run integration tests with real APIs
  if: github.event_name == 'schedule' # Only on scheduled runs
  env:
    DEEPL_API_KEY: ${{ secrets.DEEPL_API_KEY }}
    GOOGLE_API_KEY: ${{ secrets.GOOGLE_API_KEY }}
  run: cargo test --package ampel-i18n-builder --features integration-tests -- --ignored

- name: Generate coverage
  run: cargo tarpaulin --package ampel-i18n-builder --out Lcov

- name: Upload coverage
  uses: codecov/codecov-action@v3
```

## Test Maintenance

### Adding New Tests

1. **Identify test category**: Unit vs integration, provider-specific, etc.
2. **Choose test file**: Use existing file or create new module
3. **Write test**:

   ```rust
   #[tokio::test]
   async fn test_new_feature() {
       // Arrange
       let config = create_test_config();

       // Act
       let result = feature_under_test(&config).await;

       // Assert
       assert!(result.is_ok());
   }
   ```

4. **Add to mod.rs** if new module
5. **Run tests**: `cargo test --package ampel-i18n-builder`
6. **Check coverage**: Ensure new code is covered

### Best Practices

1. **Test naming**: Use descriptive names (`test_fallback_chain_systran_fails_deepl_succeeds`)
2. **Arrange-Act-Assert**: Clear test structure
3. **One assertion per test**: Focus on single behavior
4. **Feature gates**: Use `#[cfg(feature = "integration-tests")]` for real API calls
5. **Cleanup**: Remove environment variables after tests
6. **Error messages**: Use descriptive assertion messages
7. **Documentation**: Add module-level documentation explaining test purpose

### Common Pitfalls

1. **Environment pollution**: Always clean up env vars
2. **Race conditions**: Use proper async/await patterns
3. **Flaky tests**: Avoid time-dependent assertions
4. **Missing cleanup**: Use `TempDir` for file system tests
5. **Shared state**: Tests should be independent

## Debugging Tests

### Run Single Test

```bash
cargo test --package ampel-i18n-builder test_fallback_chain_systran_fails_deepl_succeeds -- --exact
```

### Show Output

```bash
cargo test --package ampel-i18n-builder -- --nocapture
```

### Enable Logging

```bash
RUST_LOG=debug cargo test --package ampel-i18n-builder
```

### Run in Single Thread

```bash
cargo test --package ampel-i18n-builder -- --test-threads=1
```

## Future Enhancements

### Planned Test Additions

1. **Systran Provider Tests**: Once Systran is wired up to router
2. **Advanced Fallback Scenarios**:
   - Provider timeout triggers fallback
   - Large batch splitting (500+ texts)
   - CLI parameter overrides
   - Cache persistence across multiple calls
3. **Performance Benchmarks**: Using `criterion`
4. **Property-Based Testing**: Using `proptest`
5. **Mutation Testing**: Using `cargo-mutants`

### Coverage Improvements

- Increase edge case coverage
- Add stress tests for concurrent requests
- Test recovery from network failures
- Validate all error paths

## References

- [4-Tier Provider Architecture](../../../docs/localization/4-TIER-PROVIDER-ARCHITECTURE.md)
- [Testing Strategy](../../../docs/localization/4-TIER-PROVIDER-ARCHITECTURE.md#testing-strategy)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Tokio Testing](https://docs.rs/tokio/latest/tokio/attr.test.html)
- [Mockito Documentation](https://docs.rs/mockito/latest/mockito/)

---

**Last Updated**: 2025-12-28
**Test Suite Version**: 1.0.0
**Coverage Status**: In Progress (Target: 80%+)
