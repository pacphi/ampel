/**
 * Custom Test Utilities
 *
 * Provides a custom render function that wraps components with all necessary providers.
 * This ensures tests have access to routing, i18n, query client, and other context.
 *
 * @example
 * ```typescript
 * import { render, screen, userEvent } from '@/tests/setup/test-utils';
 *
 * test('renders dashboard', async () => {
 *   const { user } = render(<Dashboard />);
 *
 *   // Use userEvent for interactions
 *   await user.click(screen.getByRole('button', { name: /refresh/i }));
 *
 *   // Assert
 *   expect(screen.getByText('Dashboard')).toBeInTheDocument();
 * });
 * ```
 */

import React, { ReactElement, ReactNode, Suspense } from 'react';
import { render as rtlRender, RenderOptions, RenderResult } from '@testing-library/react';
import { screen, within, waitFor, waitForElementToBeRemoved } from '@testing-library/react';
import userEvent, { UserEvent } from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { I18nextProvider } from 'react-i18next';
import i18n from './i18n-test-config';

// Re-export everything from testing-library
export * from '@testing-library/react';
export { userEvent };

// ============================================================================
// Provider Setup
// ============================================================================

/**
 * Create a fresh QueryClient for testing.
 * Disables retries and caching for predictable test behavior.
 */
export function createTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
        staleTime: 0,
        refetchOnWindowFocus: false,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

/**
 * Options for custom render function
 */
export interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  /** Initial route for MemoryRouter */
  route?: string;
  /** Initial entries for MemoryRouter history */
  routerHistory?: string[];
  /** Custom QueryClient instance */
  queryClient?: QueryClient;
  /** Initial language for i18n */
  language?: string;
  /** Whether to wrap with Suspense (default: true) */
  withSuspense?: boolean;
  /** Custom routes configuration */
  routes?: ReactNode;
}

/**
 * Extended render result with user event instance
 */
export interface CustomRenderResult extends RenderResult {
  /** User event instance for interactions */
  user: UserEvent;
  /** Query client instance for cache manipulation */
  queryClient: QueryClient;
  /** Change the current route */
  navigateTo: (path: string) => void;
}

/**
 * All Providers wrapper component
 */
interface AllProvidersProps {
  children: ReactNode;
  queryClient: QueryClient;
  routerHistory: string[];
  routes?: ReactNode;
  withSuspense: boolean;
}

function AllProviders({
  children,
  queryClient,
  routerHistory,
  routes,
  withSuspense,
}: AllProvidersProps): ReactElement {
  const content = routes ? (
    <Routes>
      {routes}
      <Route path="*" element={children} />
    </Routes>
  ) : (
    children
  );

  const wrappedContent = withSuspense ? (
    <Suspense fallback={<div data-testid="loading-fallback">Loading...</div>}>{content}</Suspense>
  ) : (
    content
  );

  return (
    <I18nextProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={routerHistory}>{wrappedContent}</MemoryRouter>
      </QueryClientProvider>
    </I18nextProvider>
  );
}

/**
 * Custom render function that wraps component with all providers.
 *
 * @param ui - React element to render
 * @param options - Custom render options
 * @returns Extended render result with user event and utilities
 *
 * @example
 * ```typescript
 * // Basic usage
 * const { user } = render(<MyComponent />);
 *
 * // With route
 * render(<SettingsPage />, { route: '/settings' });
 *
 * // With custom routes
 * render(<App />, {
 *   routes: (
 *     <>
 *       <Route path="/dashboard" element={<Dashboard />} />
 *       <Route path="/settings" element={<Settings />} />
 *     </>
 *   ),
 *   route: '/dashboard',
 * });
 * ```
 */
export function render(ui: ReactElement, options: CustomRenderOptions = {}): CustomRenderResult {
  const {
    route = '/',
    routerHistory = [route],
    queryClient = createTestQueryClient(),
    language = 'en',
    withSuspense = true,
    routes,
    ...renderOptions
  } = options;

  // Set i18n language
  if (i18n.language !== language) {
    i18n.changeLanguage(language);
  }

  // Track current route for navigation
  let currentHistory = [...routerHistory];

  const wrapper = ({ children }: { children: ReactNode }) => (
    <AllProviders
      queryClient={queryClient}
      routerHistory={currentHistory}
      routes={routes}
      withSuspense={withSuspense}
    >
      {children}
    </AllProviders>
  );

  const renderResult = rtlRender(ui, { wrapper, ...renderOptions });
  const user = userEvent.setup();

  return {
    ...renderResult,
    user,
    queryClient,
    navigateTo: (path: string) => {
      currentHistory = [path];
      renderResult.rerender(ui);
    },
  };
}

// ============================================================================
// Test Helpers
// ============================================================================

/**
 * Wait for loading state to finish
 */
export async function waitForLoadingToFinish(): Promise<void> {
  await waitForElementToBeRemoved(
    () => [
      ...screen.queryAllByTestId('loading-fallback'),
      ...screen.queryAllByTestId('loading'),
      ...screen.queryAllByText(/loading/i),
    ],
    { timeout: 5000 }
  ).catch(() => {
    // Loading might not be present, that's okay
  });
}

/**
 * Wait for element and then interact with it
 */
export async function waitForAndClick(
  user: UserEvent,
  getElement: () => HTMLElement
): Promise<void> {
  await waitFor(() => expect(getElement()).toBeInTheDocument());
  await user.click(getElement());
}

/**
 * Fill a form field
 */
export async function fillField(
  user: UserEvent,
  label: string | RegExp,
  value: string
): Promise<void> {
  const field = screen.getByLabelText(label);
  await user.clear(field);
  await user.type(field, value);
}

/**
 * Submit a form
 */
export async function submitForm(user: UserEvent, formName?: string): Promise<void> {
  const form = formName ? screen.getByRole('form', { name: formName }) : screen.getByRole('form');

  const submitButton = within(form).getByRole('button', { name: /submit|save/i });
  await user.click(submitButton);
}

/**
 * Assert that an element is not in the document
 */
export function expectNotInDocument(element: HTMLElement | null): void {
  expect(element).not.toBeInTheDocument();
}

/**
 * Wait for API call to complete (based on loading states)
 */
export async function waitForApiCall(): Promise<void> {
  // Wait a tick for React Query to process
  await waitFor(() => {
    // Query client will update loading states
  });
}

// ============================================================================
// Mock Helpers
// ============================================================================

/**
 * Create a mock function that tracks calls with type safety
 */
export function createMockFn<T extends (...args: unknown[]) => unknown>(): jest.Mock<
  ReturnType<T>,
  Parameters<T>
> {
  return vi.fn() as unknown as jest.Mock<ReturnType<T>, Parameters<T>>;
}

/**
 * Setup localStorage mock with initial values
 */
export function setupLocalStorage(initial: Record<string, string> = {}): void {
  Object.entries(initial).forEach(([key, value]) => {
    localStorage.setItem(key, value);
  });
}

/**
 * Clear all localStorage items
 */
export function clearLocalStorage(): void {
  localStorage.clear();
}

/**
 * Setup an authenticated user context
 */
export function setupAuthenticatedUser(
  accessToken = 'test-access-token',
  refreshToken = 'test-refresh-token'
): void {
  setupLocalStorage({
    accessToken,
    refreshToken,
  });
}

/**
 * Clear authenticated user context
 */
export function clearAuthenticatedUser(): void {
  localStorage.removeItem('accessToken');
  localStorage.removeItem('refreshToken');
}
