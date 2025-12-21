import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter, Outlet } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import App from './App';
import { authApi } from '@/api/auth';

// Mock the auth API
vi.mock('@/api/auth', () => ({
  authApi: {
    login: vi.fn(),
    register: vi.fn(),
    logout: vi.fn(),
    me: vi.fn(),
    updateProfile: vi.fn(),
  },
}));

// Mock ThemeProvider to avoid matchMedia issues
vi.mock('@/hooks/useTheme', () => ({
  ThemeProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  useTheme: () => ({ theme: 'light', setTheme: vi.fn(), resolvedTheme: 'light' }),
}));

const mockedAuthApi = vi.mocked(authApi);

// Mock the pages to keep tests simple
vi.mock('@/pages/Login', () => ({
  default: () => <div data-testid="login-page">Login Page</div>,
}));

vi.mock('@/pages/Register', () => ({
  default: () => <div data-testid="register-page">Register Page</div>,
}));

vi.mock('@/pages/Dashboard', () => ({
  default: () => <div data-testid="dashboard-page">Dashboard Page</div>,
}));

vi.mock('@/pages/Repositories', () => ({
  default: () => <div data-testid="repositories-page">Repositories Page</div>,
}));

vi.mock('@/pages/Analytics', () => ({
  default: () => <div data-testid="analytics-page">Analytics Page</div>,
}));

vi.mock('@/pages/Merge', () => ({
  default: () => <div data-testid="merge-page">Merge Page</div>,
}));

vi.mock('@/pages/Settings', () => ({
  default: () => <div data-testid="settings-page">Settings Page</div>,
}));

vi.mock('@/components/layout/Layout', () => ({
  default: function MockLayout() {
    return (
      <div data-testid="layout">
        <Outlet />
      </div>
    );
  },
}));

// Helper to render App with router
function renderApp(initialEntries: string[] = ['/']) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={initialEntries}>
        <App />
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('renders login page on /login route', async () => {
    renderApp(['/login']);

    await waitFor(() => {
      expect(screen.getByTestId('login-page')).toBeInTheDocument();
    });
  });

  it('renders register page on /register route', async () => {
    renderApp(['/register']);

    await waitFor(() => {
      expect(screen.getByTestId('register-page')).toBeInTheDocument();
    });
  });

  it('redirects to login when accessing protected route without auth', async () => {
    renderApp(['/dashboard']);

    await waitFor(() => {
      expect(screen.getByTestId('login-page')).toBeInTheDocument();
    });
  });

  it('renders dashboard when authenticated and navigating to /', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValueOnce({
      id: '1',
      email: 'test@example.com',
      createdAt: '2024-01-01T00:00:00Z',
    });

    renderApp(['/']);

    await waitFor(() => {
      expect(screen.getByTestId('dashboard-page')).toBeInTheDocument();
    });
  });

  it('renders dashboard when authenticated', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValueOnce({
      id: '1',
      email: 'test@example.com',
      createdAt: '2024-01-01T00:00:00Z',
    });

    renderApp(['/dashboard']);

    await waitFor(() => {
      expect(screen.getByTestId('dashboard-page')).toBeInTheDocument();
    });
  });

  it('renders repositories page when authenticated', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValueOnce({
      id: '1',
      email: 'test@example.com',
      createdAt: '2024-01-01T00:00:00Z',
    });

    renderApp(['/repositories']);

    await waitFor(() => {
      expect(screen.getByTestId('repositories-page')).toBeInTheDocument();
    });
  });

  it('renders merge page when authenticated', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValueOnce({
      id: '1',
      email: 'test@example.com',
      createdAt: '2024-01-01T00:00:00Z',
    });

    renderApp(['/merge']);

    await waitFor(() => {
      expect(screen.getByTestId('merge-page')).toBeInTheDocument();
    });
  });

  it('renders analytics page when authenticated', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValueOnce({
      id: '1',
      email: 'test@example.com',
      createdAt: '2024-01-01T00:00:00Z',
    });

    renderApp(['/analytics']);

    await waitFor(() => {
      expect(screen.getByTestId('analytics-page')).toBeInTheDocument();
    });
  });

  it('renders settings page when authenticated', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValueOnce({
      id: '1',
      email: 'test@example.com',
      createdAt: '2024-01-01T00:00:00Z',
    });

    renderApp(['/settings']);

    await waitFor(() => {
      expect(screen.getByTestId('settings-page')).toBeInTheDocument();
    });
  });

  it('wraps protected routes with Layout component', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValueOnce({
      id: '1',
      email: 'test@example.com',
      createdAt: '2024-01-01T00:00:00Z',
    });

    renderApp(['/dashboard']);

    await waitFor(() => {
      expect(screen.getByTestId('layout')).toBeInTheDocument();
    });
  });
});
