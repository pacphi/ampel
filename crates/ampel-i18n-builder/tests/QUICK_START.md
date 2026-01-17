# Quick Start: Running Tests

## Run All Tests

```bash
# Run all unit and integration tests
cargo test --package ampel-i18n-builder --all-features

# Or from the root directory
make test-backend
```

## Run Specific Test Suites

```bash
# Fallback routing tests
cargo test --package ampel-i18n-builder fallback_tests

# Provider-specific tests
cargo test --package ampel-i18n-builder provider_tests

# Configuration tests
cargo test --package ampel-i18n-builder config_tests
```

## Real API Integration Tests

**Warning**: These tests make real API calls and consume API credits.

**Setup**:

```bash
export DEEPL_API_KEY="your_deepl_api_key"
export GOOGLE_API_KEY="your_google_api_key"
export OPENAI_API_KEY="your_openai_api_key"
```

**Run**:

```bash
cargo test --package ampel-i18n-builder --features integration-tests -- --ignored
```

## Watch Mode (Development)

```bash
cargo watch -x "test --package ampel-i18n-builder"
```

## Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --package ampel-i18n-builder --all-features --out Html --output-dir coverage

# Open in browser
open coverage/index.html
```

## Debugging

```bash
# Show test output
cargo test --package ampel-i18n-builder -- --nocapture

# Run specific test
cargo test --package ampel-i18n-builder test_fallback_chain_systran_fails_deepl_succeeds -- --exact

# Enable debug logging
RUST_LOG=debug cargo test --package ampel-i18n-builder
```

For comprehensive documentation, see [TEST_DOCUMENTATION.md](./TEST_DOCUMENTATION.md).
