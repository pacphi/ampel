# Contributing to Ampel

Thank you for your interest in contributing to Ampel! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful and inclusive. We welcome contributions from everyone regardless of experience level.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/ampel.git`
3. Set up your development environment (see [DEVELOPMENT.md](DEVELOPMENT.md))
4. Create a feature branch: `git checkout -b feature/your-feature-name`

## Development Workflow

### Branch Naming

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions or fixes

### Making Changes

1. **Write code** following the style guidelines below
2. **Add tests** for new functionality
3. **Update documentation** if needed
4. **Run checks locally** before committing:

```bash
# Using Make (recommended)
make format    # Format all code
make lint      # Run all linters
make test      # Run all tests

# Or manually:
# Backend
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

# Frontend
cd frontend
pnpm run lint
pnpm run type-check
pnpm test
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```text
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:

```text
feat(api): add endpoint for repository health metrics
fix(frontend): resolve dashboard loading state issue
docs: update deployment instructions for Fly.io
```

## Pull Request Process

### Before Submitting

1. Ensure all CI checks pass
2. Update relevant documentation
3. Add tests for new functionality
4. Rebase on latest `main` if needed

### PR Template

When opening a PR, include:

- **Description**: What does this PR do?
- **Related Issue**: Link to related issue(s)
- **Type of Change**: Feature, bug fix, docs, etc.
- **Testing**: How was this tested?
- **Checklist**:
  - [ ] Tests added/updated
  - [ ] Documentation updated
  - [ ] CI passes

### Review Process

1. At least one maintainer review required
2. All CI checks must pass
3. Discussions should be resolved before merging
4. Squash and merge preferred for clean history

## Style Guidelines

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` with default settings
- Address all `clippy` warnings
- Document public APIs with doc comments
- Use meaningful variable and function names

```rust
/// Calculates the health score for a repository based on PR metrics.
///
/// # Arguments
/// * `metrics` - The repository metrics to evaluate
///
/// # Returns
/// A score between 0 and 100
pub fn calculate_health_score(metrics: &RepositoryMetrics) -> u8 {
    // Implementation
}
```

### TypeScript/React

- Use functional components with hooks
- Follow ESLint rules configured in the project
- Use TypeScript strict mode
- Prefer named exports for components
- Keep components focused and composable

```typescript
interface Props {
  status: AmpelStatus;
  size?: 'sm' | 'md' | 'lg';
}

export function StatusBadge({ status, size = 'md' }: Props) {
  // Implementation
}
```

### CSS/Styling

- Use Tailwind CSS utility classes
- Follow the existing color scheme (ampel-green, ampel-yellow, ampel-red)
- Ensure responsive design
- Maintain dark mode support

## Testing

### Backend Tests

```bash
# Run all tests
cargo test --all-features

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Frontend Tests

```bash
cd frontend

# Run all tests
pnpm test

# Run in watch mode
pnpm test -- --watch

# Run with coverage
pnpm test -- --coverage
```

## Reporting Issues

### Bug Reports

Include:

- Clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, browser, versions)
- Logs or screenshots if applicable

### Feature Requests

Include:

- Clear description of the feature
- Use case / motivation
- Proposed implementation (if any)
- Alternatives considered

## Getting Help

- Open a GitHub Discussion for questions
- Check existing issues and discussions
- Reach out to maintainers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
