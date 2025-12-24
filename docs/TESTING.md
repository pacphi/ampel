# Testing Guide

This document provides an overview of Ampel's testing strategy and links to detailed guides.

## Quick Start

```bash
# Run all tests (backend + frontend)
make test

# Run only backend tests
make test-backend

# Run only frontend tests
make test-frontend
```

## Documentation Structure

| Document                           | Description                                                       |
| ---------------------------------- | ----------------------------------------------------------------- |
| [BACKEND.md](testing/BACKEND.md)   | Rust testing: unit tests, integration tests, TestDb, fixtures     |
| [FRONTEND.md](testing/FRONTEND.md) | React testing: Vitest, React Testing Library, component tests     |
| [CI.md](testing/CI.md)             | CI/CD pipeline: GitHub Actions, cargo-nextest profiles, artifacts |

## Testing Philosophy

1. **Test Organization**: Unit tests live alongside code, integration tests in `tests/` directories
2. **Isolation**: Each test runs in a completely isolated environment
3. **Fast Feedback**: SQLite for quick unit tests, PostgreSQL for comprehensive integration tests
4. **Real Data**: Use actual database queries, not mocks, for integration tests
5. **User-centric**: Test from the user's perspective, not implementation details

## Overview

### Backend (Rust)

- **Framework**: `cargo test` / `cargo-nextest`
- **Coverage**: `cargo-llvm-cov` (LLVM source-based coverage, 5-10x faster than tarpaulin)
- **Database**: PostgreSQL (integration) / SQLite (unit tests)

```bash
# Quick unit tests
cargo nextest run --profile fast

# Full integration tests
DATABASE_URL=postgres://... cargo nextest run
```

See [BACKEND.md](testing/BACKEND.md) for complete details.

### Frontend (TypeScript/React)

- **Framework**: Vitest
- **Component Testing**: React Testing Library
- **Coverage**: Built-in Vitest coverage

```bash
cd frontend
pnpm test -- --run           # Run tests
pnpm test                    # Watch mode
pnpm test -- --run --coverage  # With coverage
```

See [FRONTEND.md](testing/FRONTEND.md) for complete details.

### CI/CD

- **Platform**: GitHub Actions
- **Test Runner**: cargo-nextest
- **Coverage**: Codecov integration

Tests run automatically on all PRs and pushes to `main`/`develop`.

See [CI.md](testing/CI.md) for complete details.

## Test Organization

### Backend Structure

```text
crates/
├── ampel-api/
│   ├── src/**/*.rs          # Unit tests in #[cfg(test)] modules
│   └── tests/               # API integration tests
├── ampel-db/
│   └── tests/
│       ├── common/          # TestDb, fixtures
│       └── integration/     # Database integration tests
└── ampel-providers/
    └── tests/               # Provider integration tests
```

### Frontend Structure

```text
frontend/
├── src/
│   ├── components/**/__tests__/  # Component tests
│   ├── hooks/**/*.test.ts        # Hook tests
│   └── utils/**/*.test.ts        # Utility tests
└── tests/
    └── setup.ts                  # Global test setup
```

## Coverage

**Target**: 80% code coverage across the project

Coverage is automatically tracked via [Codecov](https://codecov.io) and reported on all pull requests. We use:

- **Backend**: cargo-llvm-cov for Rust code coverage (LLVM source-based instrumentation)
- **Frontend**: Vitest's built-in coverage (v8 provider)
- **CI Integration**: Automatic coverage reporting on PRs with thresholds

### Quick Commands

```bash
# Run all tests with coverage reports
make test-coverage

# Backend coverage only
make test-backend-coverage

# Frontend coverage only
make test-frontend-coverage
```

### Detailed Coverage Commands

#### Backend Coverage (Rust)

```bash
# Auto-installs cargo-llvm-cov if not present
cargo llvm-cov \
  --all-features \
  --workspace \
  --html \
  --output-dir coverage

# View HTML report
open coverage/html/index.html  # macOS
xdg-open coverage/html/index.html  # Linux
```

#### Frontend Coverage (TypeScript/React)

```bash
cd frontend

# Run tests with coverage
pnpm test -- --run --coverage

# View HTML report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

### Coverage Thresholds

Our Codecov configuration enforces these thresholds:

| Component       | Target | Threshold | Description                       |
| --------------- | ------ | --------- | --------------------------------- |
| **Project**     | 80%    | ±2%       | Overall project coverage          |
| **Patch (New)** | 70%    | ±1%       | New code in pull requests         |
| ampel-api       | 75%    | ±2%       | API server routes and handlers    |
| ampel-core      | 85%    | ±2%       | Business logic (highest standard) |
| ampel-db        | 80%    | ±2%       | Database queries and migrations   |
| ampel-providers | 75%    | ±2%       | Git provider integrations         |
| ampel-worker    | 70%    | ±2%       | Background job processing         |
| frontend        | 75%    | ±2%       | React components and UI logic     |

### Coverage in CI/CD

Coverage is automatically:

1. **Collected** on every PR via GitHub Actions
2. **Reported** to Codecov with backend/frontend flags
3. **Commented** on PRs showing coverage changes
4. **Blocked** if coverage drops below thresholds

#### Understanding Coverage Status

Pull request coverage comments show:

- 🟢 **Green (≥80%)**: Excellent coverage, target met
- 🟡 **Yellow (60-79%)**: Acceptable coverage, could improve
- 🔴 **Red (<60%)**: Needs improvement, add more tests

### Local Coverage Workflow

```bash
# 1. Make code changes
vim crates/ampel-core/src/lib.rs

# 2. Write tests
vim crates/ampel-core/tests/integration_tests.rs

# 3. Run tests with coverage
make test-backend-coverage

# 4. Check HTML report
open coverage/html/index.html

# 5. Improve coverage until green (≥80%)
# Add more tests...

# 6. Commit and push - CI will verify coverage
git add . && git commit -m "Add feature with tests"
git push
```

## Best Practices

### Backend

- Create new `TestDb` for each test
- Use real database queries (not mocks) for integration tests
- Use fixture builders for consistent test data
- Test both success and error cases

### Frontend

- Test behavior, not implementation details
- Use accessible queries (`getByRole`, `getByLabelText`)
- Mock external dependencies (API calls)
- Test user interactions with `userEvent`

## References

- [The Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/react)
- [cargo-nextest](https://nexte.st/)

---

**Last Updated:** 2025-12-21
