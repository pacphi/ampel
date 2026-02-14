# Coverage Guide

This guide provides detailed information about test coverage tracking, reporting, and best practices in Ampel.

## Table of Contents

- [Overview](#overview)
- [Coverage Tools](#coverage-tools)
- [Running Coverage Locally](#running-coverage-locally)
- [CI/CD Integration](#cicd-integration)
- [Understanding Coverage Reports](#understanding-coverage-reports)
- [Improving Coverage](#improving-coverage)
- [Troubleshooting](#troubleshooting)

## Overview

Ampel uses comprehensive test coverage tracking to ensure code quality and catch regressions early. We aim for **80% overall coverage** with higher standards for critical business logic.

### Coverage Architecture

```
┌─────────────────────────────────────────┐
│           Pull Request                  │
└─────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│        GitHub Actions CI                │
│  ┌────────────┐      ┌──────────────┐   │
│  │  Backend   │      │   Frontend   │   │
│  │ (llvm-cov) │      │   (vitest)   │   │
│  └────────────┘      └──────────────┘   │
└─────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│            Codecov                      │
│   - Aggregate reports                   │
│   - Check thresholds                    │
│   - Generate insights                   │
└─────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│      PR Comment with Coverage           │
│      🟢 Backend: 82%                    │
│      🟢 Frontend: 78%                   │
│      🟢 Overall: 80%                    │
└─────────────────────────────────────────┘
```

## Coverage Tools

### Backend: cargo-llvm-cov

[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) is a fast, LLVM-based code coverage tool for Rust. It uses LLVM source-based code coverage, which is 5-10x faster than ptrace-based tools like tarpaulin.

**Why cargo-llvm-cov?**

- **5-10x faster** than tarpaulin (compile-time instrumentation vs runtime ptrace)
- **More accurate** coverage data using LLVM's instrumentation
- **Better integration** with CI/CD pipelines
- **Lower memory usage** during coverage collection

**Installation**:

```bash
# Automatic via Makefile
make test-backend-coverage

# Manual installation
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview
```

### Frontend: Vitest Coverage

Vitest uses [v8](https://v8.dev/blog/javascript-code-coverage) for native JavaScript coverage.

**Configuration**: See `frontend/vitest.config.ts` for coverage settings.

## Running Coverage Locally

### Quick Start

```bash
# Run all coverage (backend + frontend)
make test-coverage

# Backend only
make test-backend-coverage

# Frontend only
make test-frontend-coverage
```

### Backend Coverage (Detailed)

```bash
# Run with default settings (HTML + LCOV reports)
cargo llvm-cov --all-features --workspace --html --output-dir coverage
cargo llvm-cov --all-features --workspace --lcov --output-path coverage/lcov.info

# Run with specific crate
cargo llvm-cov --package ampel-core --html

# Run with text output (quick summary)
cargo llvm-cov --all-features --workspace

# Generate Codecov-compatible JSON output
cargo llvm-cov --all-features --workspace --codecov --output-path coverage/codecov.json

# View report
open coverage/html/index.html  # macOS
xdg-open coverage/html/index.html  # Linux
```

### Frontend Coverage (Detailed)

```bash
cd frontend

# Run with default thresholds
pnpm test -- --run --coverage

# Run with specific file pattern
pnpm test src/components/dashboard -- --run --coverage

# Generate specific report formats
pnpm test -- --run --coverage --coverage.reporter=html,lcov,text

# View report
open coverage/index.html
```

## CI/CD Integration

### Workflow Overview

Coverage is collected in three GitHub Actions jobs:

1. **Backend Unit Tests (SQLite)** - Fast unit test execution
2. **Backend Integration Tests (PostgreSQL)** - Full feature testing
3. **Backend Coverage** - Combines coverage from both test types
4. **Frontend Test** - Runs Vitest with coverage

### Coverage Workflow (`.github/workflows/ci.yml`)

```yaml
backend-coverage:
  name: Backend Test Coverage
  needs: [backend-unit-test, backend-integration-test]
  runs-on: ubuntu-latest
  env:
    RUSTC_WRAPPER: '' # Disable sccache for coverage

  steps:
    - uses: actions/checkout@v6

    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@master
      with:
        components: llvm-tools-preview

    - name: Install cargo-llvm-cov
      uses: taiki-e/install-action@cargo-llvm-cov

    - name: Generate coverage
      run: |
        cargo llvm-cov \
          --all-features \
          --workspace \
          --codecov \
          --output-path coverage/codecov.json

    - name: Upload to Codecov
      uses: codecov/codecov-action@v4
      with:
        files: ./coverage/codecov.json
        flags: backend
```

### PR Coverage Comments

The `coverage-pr-comment.yml` workflow automatically posts coverage updates:

```markdown
## 📊 Coverage Report

| Component   | Coverage   | Status |
| ----------- | ---------- | ------ |
| Backend     | 82.45%     | 🟢     |
| Frontend    | 78.30%     | 🟡     |
| **Overall** | **80.38%** | **🟢** |

### Coverage Thresholds

- 🟢 Green: ≥ 80% (target)
- 🟡 Yellow: 60-79% (acceptable)
- 🔴 Red: < 60% (needs improvement)
```

## Understanding Coverage Reports

### Codecov Dashboard

Visit [codecov.io/gh/pacphi/ampel](https://codecov.io/gh/pacphi/ampel) to see:

- **Sunburst Chart**: Visual representation of coverage by component
- **File Browser**: Line-by-line coverage for each file
- **Trends**: Coverage changes over time
- **Flags**: Separate backend/frontend tracking

### Coverage Metrics

| Metric    | Description                                  | Goal |
| --------- | -------------------------------------------- | ---- |
| Line      | Percentage of executed lines                 | 80%  |
| Function  | Percentage of called functions               | 75%  |
| Branch    | Percentage of executed conditional branches  | 75%  |
| Statement | Percentage of executed statements (frontend) | 80%  |

### Reading cargo-llvm-cov Reports

**HTML Report Colors**:

- 🟢 **Green**: Line was executed
- 🔴 **Red**: Line was not executed
- ⚪ **White**: Not executable (comments, whitespace)

**Key Sections**:

1. **Summary**: Overall coverage percentage
2. **File List**: Coverage by file with percentages
3. **Source View**: Line-by-line coverage with execution counts

### Reading Vitest Reports

Navigate to `frontend/coverage/index.html`:

- **Summary**: Overall metrics (lines, functions, branches, statements)
- **File Tree**: Navigate by directory
- **Source View**: See covered (green) and uncovered (red) lines

## Improving Coverage

### 1. Identify Coverage Gaps

```bash
# Backend: Generate report and view uncovered lines
make test-backend-coverage
open coverage/html/index.html

# Frontend: View coverage in terminal
cd frontend && pnpm test -- --run --coverage
```

### 2. Focus on Critical Paths

**Priority Areas** (in order):

1. **Authentication & Authorization** (ampel-api)
2. **Business Logic** (ampel-core)
3. **Database Operations** (ampel-db)
4. **Provider Integrations** (ampel-providers)
5. **Background Jobs** (ampel-worker)
6. **UI Components** (frontend)

### 3. Write Effective Tests

**Backend Example**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_creation() {
        // Arrange
        let db = TestDb::new().await;
        let user = NewUser {
            email: "test@example.com".to_string(),
            password: "secure-password".to_string(),
        };

        // Act
        let result = create_user(&db.pool, user).await;

        // Assert
        assert!(result.is_ok());
        let created_user = result.unwrap();
        assert_eq!(created_user.email, "test@example.com");
    }
}
```

**Frontend Example**:

```typescript
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { LoginForm } from './LoginForm';

describe('LoginForm', () => {
  it('submits form with valid credentials', async () => {
    const onSubmit = vi.fn();
    render(<LoginForm onSubmit={onSubmit} />);

    await userEvent.type(screen.getByLabelText('Email'), 'user@example.com');
    await userEvent.type(screen.getByLabelText('Password'), 'password123');
    await userEvent.click(screen.getByRole('button', { name: 'Login' }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({
        email: 'user@example.com',
        password: 'password123',
      });
    });
  });
});
```

### 4. Test Edge Cases

Don't just test the happy path:

```rust
#[tokio::test]
async fn test_user_creation_duplicate_email() {
    let db = TestDb::new().await;

    // Create first user
    let user = NewUser { email: "test@example.com", ... };
    create_user(&db.pool, user.clone()).await.unwrap();

    // Try to create duplicate - should fail
    let result = create_user(&db.pool, user).await;
    assert!(result.is_err());
}
```

### 5. Coverage Workflow

```bash
# 1. Run current coverage
make test-backend-coverage

# 2. Identify gaps (red lines in HTML report)
open coverage/html/index.html

# 3. Write tests for uncovered code
vim crates/ampel-core/tests/integration_tests.rs

# 4. Re-run coverage
make test-backend-coverage

# 5. Verify improvement
# (Check coverage percentage increased)

# 6. Commit and push
git add .
git commit -m "test: improve coverage for user module"
git push
```

## Troubleshooting

### Backend: cargo-llvm-cov Issues

#### Issue: "llvm-tools-preview not installed"

```bash
# Solution: Install the LLVM tools component
rustup component add llvm-tools-preview
```

#### Issue: "No coverage data generated"

```bash
# Solution: Ensure tests are actually running
cargo test --all-features  # Verify tests pass first

# Then run coverage with verbose output
cargo llvm-cov --all-features --workspace
```

#### Issue: Coverage percentage seems wrong

```bash
# Solution: Clean build and re-run
cargo llvm-cov clean --workspace
make test-backend-coverage
```

#### Issue: Tests fail during coverage but pass normally

```bash
# Check for global state conflicts (common with metrics/loggers)
# Ensure init functions are idempotent for parallel test execution
cargo test --all-features -- --test-threads=1
```

### Frontend: Vitest Coverage Issues

#### Issue: Coverage thresholds failing locally

```typescript
// frontend/vitest.config.ts
coverage: {
  lines: 80,
  functions: 75,
  branches: 75,
  statements: 80,
}
```

Check `frontend/vitest.config.ts` and ensure your code meets thresholds.

#### Issue: "No coverage directory generated"

```bash
# Ensure coverage provider is installed
cd frontend
pnpm install @vitest/coverage-v8 --save-dev

# Run with coverage explicitly
pnpm test -- --run --coverage
```

### CI/CD Issues

#### Issue: Codecov upload fails

Check `.github/workflows/ci.yml`:

```yaml
- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: ./coverage/cobertura.xml
    flags: backend
    fail_ci_if_error: false # Don't fail CI if Codecov is down
```

#### Issue: Coverage comment not appearing on PR

1. Check workflow run logs in GitHub Actions
2. Verify `GITHUB_TOKEN` has `pull-requests: write` permission
3. Ensure PR is from same repository (not fork)

## Best Practices

### DO ✅

- **Run coverage locally** before pushing
- **Write tests for new code** before implementation (TDD)
- **Test error paths** not just happy paths
- **Focus on business logic** for highest coverage
- **Use real data** in integration tests
- **Review coverage reports** regularly

### DON'T ❌

- **Don't skip tests** to "save time"
- **Don't mock excessively** - test real behavior
- **Don't chase 100% coverage** - focus on critical paths
- **Don't test third-party code** (shadcn/ui, etc.)
- **Don't ignore failing coverage** in CI

## Additional Resources

- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Vitest Coverage Guide](https://vitest.dev/guide/coverage.html)
- [Codecov Documentation](https://docs.codecov.com/)
- [Backend Testing Guide](./BACKEND.md)
- [Frontend Testing Guide](./FRONTEND.md)
- [CI/CD Guide](./CI.md)

---

**Last Updated:** 2025-12-24
