# Frontend Testing Guide

This document covers all aspects of testing the React/TypeScript frontend, including component tests, integration tests, and testing utilities.

## Table of Contents

- [Overview](#overview)
- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [Test Utilities](#test-utilities)
- [Best Practices](#best-practices)
- [Debugging](#debugging)
- [Coverage](#coverage)

## Overview

The frontend uses Vitest with React Testing Library:

- **Test Framework**: [Vitest](https://vitest.dev/)
- **Component Testing**: [React Testing Library](https://testing-library.com/react)
- **DOM Environment**: jsdom
- **Coverage**: Built-in Vitest coverage

### Testing Philosophy

1. **User-centric**: Test from the user's perspective, not implementation details
2. **Component isolation**: Mock external dependencies
3. **Accessibility**: Include accessibility checks in component tests
4. **Real behavior**: Test actual user interactions

## Test Organization

### Directory Structure

```text
frontend/
├── src/
│   ├── components/
│   │   ├── ui/
│   │   │   └── **/__tests__/    # UI component tests
│   │   ├── layout/
│   │   │   └── **/__tests__/    # Layout component tests
│   │   └── dashboard/
│   │       └── **/__tests__/    # Dashboard component tests
│   ├── hooks/
│   │   └── **/*.test.ts         # Hook tests
│   ├── utils/
│   │   └── **/*.test.ts         # Utility function tests
│   └── pages/
│       └── **/*.test.tsx        # Page component tests
├── tests/
│   ├── setup.ts                 # Global test setup
│   └── fixtures/                # Shared test data
└── vitest.config.ts             # Vitest configuration
```

### File Naming Conventions

- Component tests: `ComponentName.test.tsx`
- Utility tests: `utilityName.test.ts`
- Hook tests: `useHookName.test.ts`
- Test directories: `__tests__/`

## Running Tests

### Quick Reference

```bash
# Run all frontend tests
make test-frontend              # pnpm test -- --run

# Or directly
cd frontend
pnpm test -- --run
```

### All Tests

```bash
cd frontend
pnpm test -- --run
```

### Watch Mode

```bash
cd frontend
pnpm test
```

### Specific Test File

```bash
cd frontend
pnpm test src/components/Button.test.tsx
```

### Pattern Matching

```bash
cd frontend
pnpm test -- -t "test name pattern"
```

### UI Mode (Interactive)

```bash
cd frontend
pnpm test -- --ui
```

## Writing Tests

### Component Tests

Test React components in isolation with mocked dependencies:

```tsx
// src/components/__tests__/Button.test.tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { Button } from '../Button';

describe('Button', () => {
  it('renders with text', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('calls onClick when clicked', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();

    render(<Button onClick={handleClick}>Click me</Button>);
    await user.click(screen.getByText('Click me'));

    expect(handleClick).toHaveBeenCalledOnce();
  });

  it('is disabled when disabled prop is true', () => {
    render(<Button disabled>Click me</Button>);
    expect(screen.getByText('Click me')).toBeDisabled();
  });
});
```

### Hook Tests

Test custom hooks using `renderHook`:

```tsx
// src/hooks/__tests__/useCounter.test.ts
import { renderHook, act } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { useCounter } from '../useCounter';

describe('useCounter', () => {
  it('initializes with default value', () => {
    const { result } = renderHook(() => useCounter());
    expect(result.current.count).toBe(0);
  });

  it('increments counter', () => {
    const { result } = renderHook(() => useCounter());

    act(() => {
      result.current.increment();
    });

    expect(result.current.count).toBe(1);
  });

  it('accepts initial value', () => {
    const { result } = renderHook(() => useCounter(10));
    expect(result.current.count).toBe(10);
  });
});
```

### Utility Function Tests

Test pure functions directly:

```ts
// src/utils/__tests__/formatDate.test.ts
import { describe, it, expect } from 'vitest';
import { formatDate, formatRelativeTime } from '../formatDate';

describe('formatDate', () => {
  it('formats date correctly', () => {
    const date = new Date('2025-01-15T10:30:00Z');
    expect(formatDate(date)).toBe('Jan 15, 2025');
  });

  it('handles invalid date', () => {
    expect(formatDate(null)).toBe('Invalid date');
  });
});

describe('formatRelativeTime', () => {
  it('shows "just now" for recent dates', () => {
    const now = new Date();
    expect(formatRelativeTime(now)).toBe('just now');
  });
});
```

### Testing with Providers

Wrap components that need context providers:

```tsx
// tests/utils/renderWithProviders.tsx
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter } from 'react-router-dom';

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  route?: string;
}

export function renderWithProviders(
  ui: React.ReactElement,
  { route = '/', ...options }: CustomRenderOptions = {}
) {
  window.history.pushState({}, 'Test page', route);

  const queryClient = createTestQueryClient();

  function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>{children}</BrowserRouter>
      </QueryClientProvider>
    );
  }

  return {
    ...render(ui, { wrapper: Wrapper, ...options }),
    queryClient,
  };
}
```

Usage:

```tsx
import { renderWithProviders } from '../tests/utils/renderWithProviders';

it('renders dashboard', () => {
  renderWithProviders(<Dashboard />, { route: '/dashboard' });
  expect(screen.getByText('Dashboard')).toBeInTheDocument();
});
```

### Mocking API Calls

Use MSW (Mock Service Worker) for API mocking:

```tsx
// tests/mocks/handlers.ts
import { http, HttpResponse } from 'msw';

export const handlers = [
  http.get('/api/users', () => {
    return HttpResponse.json([
      { id: 1, name: 'John Doe' },
      { id: 2, name: 'Jane Doe' },
    ]);
  }),

  http.post('/api/login', async ({ request }) => {
    const body = await request.json();
    if (body.email === 'test@example.com') {
      return HttpResponse.json({ token: 'fake-token' });
    }
    return HttpResponse.json({ error: 'Invalid credentials' }, { status: 401 });
  }),
];
```

```tsx
// tests/setup.ts
import { setupServer } from 'msw/node';
import { handlers } from './mocks/handlers';

export const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

## Test Utilities

### Global Setup (`tests/setup.ts`)

The setup file provides:

- DOM mocking (matchMedia, IntersectionObserver, ResizeObserver)
- Automatic cleanup after each test
- CI environment detection
- Test utilities (wait, flushPromises)

### Common Utilities

```ts
// Wait for async operations
await waitFor(() => {
  expect(screen.getByText('Loaded')).toBeInTheDocument();
});

// Wait for element to disappear
await waitForElementToBeRemoved(() => screen.queryByText('Loading...'));

// Flush promises
await new Promise((resolve) => setTimeout(resolve, 0));
```

### Custom Matchers

Vitest includes jest-dom matchers:

```ts
expect(element).toBeInTheDocument();
expect(element).toBeVisible();
expect(element).toBeDisabled();
expect(element).toHaveTextContent('text');
expect(element).toHaveAttribute('href', '/path');
expect(element).toHaveClass('active');
```

## Best Practices

### DO

- Test behavior, not implementation
- Use accessible queries (`getByRole`, `getByLabelText`)
- Mock external dependencies (API calls, browser APIs)
- Test user interactions with `userEvent`
- Write descriptive test names
- Test error states and edge cases
- Keep tests focused and small
- Use `screen` for queries (better error messages)

### DON'T

- Test implementation details (internal state, props)
- Use `getByTestId` as primary query (prefer accessible queries)
- Mock too much (test real interactions when possible)
- Write tests that depend on each other
- Test third-party library behavior
- Use `fireEvent` when `userEvent` works (userEvent is more realistic)

### Query Priority

Use queries in this order (most to least preferred):

1. `getByRole` - accessible to everyone
2. `getByLabelText` - form fields
3. `getByPlaceholderText` - when label is not available
4. `getByText` - non-interactive content
5. `getByDisplayValue` - current value of form elements
6. `getByAltText` - images
7. `getByTitle` - title attribute
8. `getByTestId` - last resort

### Example: Testing a Form

```tsx
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { LoginForm } from '../LoginForm';

describe('LoginForm', () => {
  it('submits form with valid data', async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();

    render(<LoginForm onSubmit={onSubmit} />);

    // Fill form using accessible queries
    await user.type(screen.getByLabelText(/email/i), 'test@example.com');
    await user.type(screen.getByLabelText(/password/i), 'password123');

    // Submit
    await user.click(screen.getByRole('button', { name: /sign in/i }));

    // Verify
    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({
        email: 'test@example.com',
        password: 'password123',
      });
    });
  });

  it('shows validation errors for empty fields', async () => {
    const user = userEvent.setup();

    render(<LoginForm onSubmit={vi.fn()} />);

    await user.click(screen.getByRole('button', { name: /sign in/i }));

    expect(screen.getByText(/email is required/i)).toBeInTheDocument();
    expect(screen.getByText(/password is required/i)).toBeInTheDocument();
  });

  it('disables submit button while loading', () => {
    render(<LoginForm onSubmit={vi.fn()} isLoading />);

    expect(screen.getByRole('button', { name: /sign in/i })).toBeDisabled();
  });
});
```

## Debugging

### Verbose Output

```bash
pnpm test -- --watch --reporter=verbose
```

### Debug DOM

```tsx
import { screen } from '@testing-library/react';

// Print current DOM
screen.debug();

// Print specific element
screen.debug(screen.getByRole('button'));
```

### Interactive UI Mode

```bash
pnpm test -- --ui
```

### Log Playground URL

```tsx
import { screen, logRoles } from '@testing-library/react';

// Log all accessible roles
logRoles(screen.getByTestId('container'));
```

## Coverage

### Generate Coverage

```bash
cd frontend
pnpm test -- --run --coverage
```

### Open Report

```bash
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

### Coverage Targets

- **Lines**: 80%+
- **Functions**: 75%+
- **Branches**: 75%+
- **Statements**: 80%+

### Coverage Configuration

In `vitest.config.ts`:

```ts
export default defineConfig({
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      exclude: ['node_modules/', 'tests/', '**/*.d.ts', '**/*.config.*'],
    },
  },
});
```

## Configuration

### Vitest Configuration (`vitest.config.ts`)

```ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    isolate: true,
    threads: true,
    maxThreads: 4,
    clearMocks: true,
    mockReset: true,
    restoreMocks: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
    },
  },
});
```

### Test Isolation

Each test runs in isolated environment:

1. Each test runs in isolated environment
2. Mocks are reset between tests
3. DOM cleanup after each test
4. Up to 4 parallel threads

## Performance Targets

- **Component tests**: < 200ms per test
- **Full suite**: < 2 minutes (CI)

## References

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Testing Library Queries](https://testing-library.com/docs/queries/about)
- [MSW (Mock Service Worker)](https://mswjs.io/)
- [User Event](https://testing-library.com/docs/user-event/intro)
- [Kent C. Dodds - Common Testing Mistakes](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)
