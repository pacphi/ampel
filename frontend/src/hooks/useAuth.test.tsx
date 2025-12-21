import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { AuthProvider, useAuth } from './useAuth';
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

const mockedAuthApi = vi.mocked(authApi);

// Test component that uses useAuth hook
function TestComponent() {
  const { user, isLoading, isAuthenticated, login, register, logout, refreshUser } = useAuth();

  const handleLogout = async () => {
    try {
      await logout();
    } catch {
      // Expected in some tests
    }
  };

  return (
    <div>
      <div data-testid="loading">{isLoading ? 'loading' : 'not-loading'}</div>
      <div data-testid="authenticated">
        {isAuthenticated ? 'authenticated' : 'not-authenticated'}
      </div>
      <div data-testid="user">{user ? user.email : 'no-user'}</div>
      <button onClick={() => login('test@example.com', 'password')}>Login</button>
      <button onClick={() => register('test@example.com', 'password', 'Test User')}>
        Register
      </button>
      <button onClick={() => handleLogout()}>Logout</button>
      <button onClick={() => refreshUser()}>Refresh</button>
    </div>
  );
}

// Test component that doesn't use provider
function ComponentWithoutProvider() {
  try {
    useAuth();
    return <div>no-error</div>;
  } catch (e) {
    return <div data-testid="error">{(e as Error).message}</div>;
  }
}

describe('useAuth', () => {
  const mockUser = {
    id: '1',
    email: 'test@example.com',
    displayName: 'Test User',
    createdAt: '2024-01-01T00:00:00Z',
  };

  const mockTokens = {
    accessToken: 'access-token',
    refreshToken: 'refresh-token',
    tokenType: 'Bearer',
    expiresIn: 3600,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('throws error when useAuth is used outside of AuthProvider', () => {
    render(<ComponentWithoutProvider />);
    expect(screen.getByTestId('error')).toHaveTextContent(
      'useAuth must be used within an AuthProvider'
    );
  });

  it('shows loading state initially when token exists', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockImplementation(() => new Promise(() => {})); // Never resolves

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    expect(screen.getByTestId('loading')).toHaveTextContent('loading');
  });

  it('shows not loading when no token exists', async () => {
    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('loading')).toHaveTextContent('not-loading');
    });
  });

  it('loads user data when token exists', async () => {
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValueOnce(mockUser);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('authenticated')).toHaveTextContent('authenticated');
      expect(screen.getByTestId('user')).toHaveTextContent('test@example.com');
    });
  });

  it('clears tokens when me() fails', async () => {
    localStorage.setItem('accessToken', 'test-token');
    localStorage.setItem('refreshToken', 'refresh-token');
    mockedAuthApi.me.mockRejectedValueOnce(new Error('Unauthorized'));

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('authenticated')).toHaveTextContent('not-authenticated');
      expect(localStorage.getItem('accessToken')).toBeNull();
      expect(localStorage.getItem('refreshToken')).toBeNull();
    });
  });

  it('logs in user successfully', async () => {
    const user = userEvent.setup();
    mockedAuthApi.login.mockResolvedValueOnce(mockTokens);
    mockedAuthApi.me.mockResolvedValue(mockUser);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('loading')).toHaveTextContent('not-loading');
    });

    await user.click(screen.getByText('Login'));

    await waitFor(() => {
      expect(mockedAuthApi.login).toHaveBeenCalledWith('test@example.com', 'password');
      expect(localStorage.getItem('accessToken')).toBe('access-token');
      expect(localStorage.getItem('refreshToken')).toBe('refresh-token');
      expect(screen.getByTestId('authenticated')).toHaveTextContent('authenticated');
    });
  });

  it('registers user successfully', async () => {
    const user = userEvent.setup();
    mockedAuthApi.register.mockResolvedValueOnce(mockTokens);
    mockedAuthApi.me.mockResolvedValue(mockUser);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('loading')).toHaveTextContent('not-loading');
    });

    await user.click(screen.getByText('Register'));

    await waitFor(() => {
      expect(mockedAuthApi.register).toHaveBeenCalledWith(
        'test@example.com',
        'password',
        'Test User'
      );
      expect(localStorage.getItem('accessToken')).toBe('access-token');
      expect(localStorage.getItem('refreshToken')).toBe('refresh-token');
    });
  });

  it('logs out user successfully', async () => {
    const user = userEvent.setup();
    localStorage.setItem('accessToken', 'test-token');
    localStorage.setItem('refreshToken', 'refresh-token');
    mockedAuthApi.me.mockResolvedValueOnce(mockUser);
    mockedAuthApi.logout.mockResolvedValueOnce(undefined);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('authenticated')).toHaveTextContent('authenticated');
    });

    await user.click(screen.getByText('Logout'));

    await waitFor(() => {
      expect(mockedAuthApi.logout).toHaveBeenCalled();
      expect(localStorage.getItem('accessToken')).toBeNull();
      expect(localStorage.getItem('refreshToken')).toBeNull();
      expect(screen.getByTestId('authenticated')).toHaveTextContent('not-authenticated');
    });
  });

  it('clears tokens even if logout API call fails', async () => {
    const user = userEvent.setup();
    localStorage.setItem('accessToken', 'test-token');
    localStorage.setItem('refreshToken', 'refresh-token');
    mockedAuthApi.me.mockResolvedValueOnce(mockUser);
    mockedAuthApi.logout.mockRejectedValueOnce(new Error('Network error'));

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('authenticated')).toHaveTextContent('authenticated');
    });

    await user.click(screen.getByText('Logout'));

    await waitFor(() => {
      expect(localStorage.getItem('accessToken')).toBeNull();
      expect(localStorage.getItem('refreshToken')).toBeNull();
      expect(screen.getByTestId('authenticated')).toHaveTextContent('not-authenticated');
    });
  });

  it('refreshes user data', async () => {
    const user = userEvent.setup();
    localStorage.setItem('accessToken', 'test-token');
    mockedAuthApi.me.mockResolvedValue(mockUser);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('authenticated')).toHaveTextContent('authenticated');
    });

    // Clear mock and set up for refresh
    mockedAuthApi.me.mockClear();
    const updatedUser = { ...mockUser, displayName: 'Updated Name' };
    mockedAuthApi.me.mockResolvedValueOnce(updatedUser);

    await user.click(screen.getByText('Refresh'));

    await waitFor(() => {
      expect(mockedAuthApi.me).toHaveBeenCalled();
    });
  });
});
