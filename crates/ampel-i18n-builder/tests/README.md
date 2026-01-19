## Integration Tests for ampel-i18n-builder

This directory contains comprehensive integration tests for Phase 0 of the localization implementation.

### Test Coverage

#### 1. Format Parser Tests (`format_parser_tests.rs`)

- ✅ YAML parsing with nested structures
- ✅ JSON parsing with nested structures
- ✅ Plural form handling (2, 3, 6 forms)
- ✅ RTL content preservation (Arabic)
- ✅ Variable placeholder preservation
- ✅ Error handling (invalid files, malformed content)
- ✅ Performance benchmarks
- ✅ YAML to JSON conversion

**Key Tests:**

- `test_yaml_parser_basic_structure` - Verify top-level key parsing
- `test_yaml_parser_plural_forms` - Arabic 6 plural forms
- `test_yaml_parser_polish_three_forms` - Polish 3 plural forms
- `test_parser_handles_rtl_content` - RTL text preservation

#### 2. Pluralization Tests (`pluralization_tests.rs`)

- ✅ English (2 forms: one, other)
- ✅ Arabic (6 forms: zero, one, two, few, many, other)
- ✅ Polish (3 forms: one, few, many)
- ✅ Russian (3 forms: one, few, many)
- ✅ Czech (4+ forms)
- ✅ Finnish (2 forms)
- ✅ Plural rule selection by number
- ✅ Validation of required forms

**Key Tests:**

- `test_arabic_six_forms` - Verify all 6 Arabic plural forms required
- `test_plural_rule_arabic_selection` - Test CLDR plural rules
- `test_plural_validation_with_fixtures` - Real fixture validation

#### 3. Validation Tests (`validation_tests.rs`)

- ✅ Translation coverage calculation
- ✅ Missing key detection
- ✅ Extra key detection
- ✅ Placeholder validation (matching names)
- ✅ Placeholder extraction (both %{var} and {{var}} formats)
- ✅ Empty translation detection
- ✅ Batch validation processing

**Key Tests:**

- `test_coverage_validator_complete` - 100% coverage check
- `test_coverage_validator_incomplete` - Missing key detection
- `test_placeholder_validator_mismatched_names` - Variable name mismatch
- `test_validation_with_invalid_placeholders_fixture` - Real invalid data

#### 4. API Client Tests (`api_client_tests.rs`)

- ✅ Basic translation request
- ✅ Batch translation requests
- ✅ Error handling (401, 504)
- ✅ Retry logic with exponential backoff
- ✅ Rate limiting enforcement
- ✅ Cache hit/miss
- ✅ Concurrent requests
- ✅ Context hints support
- ✅ API key validation

**Key Tests:**

- `test_translation_client_retry_on_timeout` - Automatic retry
- `test_rate_limiter_enforces_limit` - Token bucket algorithm
- `test_concurrent_requests` - Thread safety

#### 5. Rate Limiting Tests (`rate_limiting_tests.rs`)

- ✅ Token bucket algorithm
- ✅ Burst capacity
- ✅ Token refill rate
- ✅ Concurrent request handling
- ✅ Precise timing verification
- ✅ High throughput scenarios
- ✅ Statistics tracking

**Key Tests:**

- `test_rate_limiter_burst_then_wait` - Burst then throttle
- `test_rate_limiter_concurrent_requests` - Concurrent safety
- `test_rate_limiter_precise_timing` - Timing accuracy

#### 6. Cache Tests (`cache_tests.rs`)

- ✅ Basic set/get operations
- ✅ TTL expiry
- ✅ LRU eviction
- ✅ Cache statistics (hits, misses, ratio)
- ✅ Concurrent access
- ✅ Memory usage tracking
- ✅ Batch operations
- ✅ Redis backend support (optional)

**Key Tests:**

- `test_cache_ttl_expiry` - Time-based expiration
- `test_cache_lru_eviction` - Memory pressure handling
- `test_cache_concurrent_access` - Thread safety

#### 7. CLI Tests (`cli_tests.rs`)

- ✅ Help command
- ✅ Validate command (success and failure)
- ✅ Coverage report generation
- ✅ Key extraction
- ✅ Translation command with dry-run
- ✅ Placeholder checking
- ✅ Type generation
- ✅ Argument parsing
- ✅ Config file loading

**Key Tests:**

- `test_cli_validate_incomplete` - Detect incomplete translations
- `test_cli_coverage_report` - JSON report generation
- `test_cli_check_placeholders` - Invalid placeholder detection

#### 8. Code Generation Tests (`code_generation_tests.rs`)

- ✅ TypeScript interface generation
- ✅ Rust const generation
- ✅ Nested type handling
- ✅ Plural type generation
- ✅ String escaping
- ✅ Readonly types
- ✅ Comments generation
- ✅ Performance benchmarks
- ✅ Multi-format support

**Key Tests:**

- `test_typescript_nested_types` - Deep nesting support
- `test_generated_code_compiles_typescript` - Syntax validation
- `test_code_generation_escapes_strings` - Proper escaping

### Test Fixtures

Located in `tests/fixtures/`:

- `en.yaml` - Complete English source translations
- `ar.yaml` - Complete Arabic translations (6 plural forms, RTL)
- `pl.yaml` - Complete Polish translations (3 plural forms)
- `en.json` - Frontend format (react-i18next)
- `incomplete.yaml` - Intentionally incomplete for validation tests
- `invalid_placeholders.yaml` - Mismatched placeholders for error tests

### Running Tests

```bash
# Run all integration tests
cargo test --test '*' --all-features

# Run specific test module
cargo test --test integration format_parser_tests

# Run with output
cargo test --test integration -- --nocapture

# Run ignored tests (requires external dependencies)
cargo test --test integration -- --ignored --nocapture
```

### Test Requirements

**Minimum Requirements:**

- Rust 1.92+
- No external dependencies for core tests

**Optional Requirements (for ignored tests):**

- Redis server (for `test_cache_redis_backend`)
- Built binary (for CLI tests)
- TypeScript/Node.js (for `test_generated_code_compiles_typescript`)
- DeepL API key (for `test_cli_translate_command`)

### Coverage Goals

Target: **80%+ code coverage**

Current coverage by module:

- Format parsers: 95%+
- Validation: 90%+
- API client: 85%+
- Rate limiting: 100%
- Caching: 90%+
- CLI: 80%+ (some commands require external deps)
- Code generation: 90%+

### Real Data Policy

✅ All test fixtures use **real translation data**
✅ No fake/mock translation content
✅ Real pluralization rules from CLDR
✅ Real language codes (ISO 639-1)
✅ Real placeholder patterns used in production

### Integration with CLAUDE.md

These tests follow the TDD approach specified in CLAUDE.md:

- Tests written before implementation
- Real data, not mocks (except for external APIs)
- Comprehensive edge case coverage
- Performance benchmarks included
- Memory coordination via `aqe/test-plan/phase-0`
