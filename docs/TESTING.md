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
- **Coverage**: `cargo-tarpaulin`
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

**Target**: 80% code coverage

```bash
# Backend coverage
cargo tarpaulin --all-features --workspace --out Html

# Frontend coverage
cd frontend && pnpm test -- --run --coverage
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
