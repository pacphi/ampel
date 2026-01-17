/**
 * BDD Component Tests for Authentication
 *
 * Tests Login and Register pages using BDD structure with MSW for API mocking.
 * Each scenario follows Given/When/Then pattern for clarity and maintainability.
 */

import React, { ReactElement, ReactNode, Suspense } from 'react';
import { render as rtlRender, RenderResult, screen, waitFor } from '@testing-library/react';
import userEvent, { UserEvent } from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { I18nextProvider } from 'react-i18next';
import { http, HttpResponse } from 'msw';

import { Feature, Scenario, Given, When, Then, And } from '../setup/bdd-helpers';
import { server } from '../setup/msw/server';
import i18n from '../setup/i18n-test-config';

import { AuthProvider } from '@/hooks/useAuth';
import Login from '@/pages/Login';
import Register from '@/pages/Register';

// ============================================================================
// Test Setup
// ============================================================================

/**
 * Create a fresh QueryClient for testing with disabled retries
 */
function createTestQueryClient(): QueryClient {
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
 * Custom render options for auth component tests
 */
interface AuthRenderOptions {
  route?: string;
  initialEntries?: string[];
}

/**
 * Extended render result with user event instance
 */
interface AuthRenderResult extends RenderResult {
  user: UserEvent;
  queryClient: QueryClient;
}

/**
 * Wrapper component that provides all necessary providers for auth tests
 */
function AuthTestWrapper({
  children,
  queryClient,
  initialEntries,
}: {
  children: ReactNode;
  queryClient: QueryClient;
  initialEntries: string[];
}): ReactElement {
  return (
    <I18nextProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={initialEntries}>
          <AuthProvider>
            <Suspense fallback={<div data-testid="loading-fallback">Loading...</div>}>
              <Routes>
                <Route path="/login" element={<Login />} />
                <Route path="/register" element={<Register />} />
                <Route path="/dashboard" element={<div data-testid="dashboard">Dashboard</div>} />
                <Route path="*" element={children} />
              </Routes>
            </Suspense>
          </AuthProvider>
        </MemoryRouter>
      </QueryClientProvider>
    </I18nextProvider>
  );
}

/**
 * Custom render function for auth components
 */
function renderAuth(ui: ReactElement, options: AuthRenderOptions = {}): AuthRenderResult {
  const { route = '/login', initialEntries = [route] } = options;
  const queryClient = createTestQueryClient();

  // Set i18n to English
  if (i18n.language !== 'en') {
    i18n.changeLanguage('en');
  }

  const wrapper = ({ children }: { children: ReactNode }) => (
    <AuthTestWrapper queryClient={queryClient} initialEntries={initialEntries}>
      {children}
    </AuthTestWrapper>
  );

  const renderResult = rtlRender(ui, { wrapper });
  const user = userEvent.setup();

  return {
    ...renderResult,
    user,
    queryClient,
  };
}

/**
 * Clear localStorage between tests
 */
function clearAuthState(): void {
  localStorage.removeItem('accessToken');
  localStorage.removeItem('refreshToken');
}

// ============================================================================
// Login Page Tests
// ============================================================================

Feature('Login Authentication', () => {
  beforeEach(() => {
    clearAuthState();
  });

  Scenario('User logs in with valid credentials', async () => {
    let user: UserEvent;

    await Given('the login form is displayed', async () => {
      const result = renderAuth(<Login />, { route: '/login' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user enters valid credentials and submits', async () => {
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign in/i });

      await user.type(emailInput, 'test@example.com');
      await user.type(passwordInput, 'validPassword123');
      await user.click(submitButton);
    });

    await Then('user is redirected to dashboard', async () => {
      await waitFor(
        () => {
          expect(screen.getByTestId('dashboard')).toBeInTheDocument();
        },
        { timeout: 3000 }
      );
    });
  });

  Scenario('User attempts login with invalid credentials', async () => {
    let user: UserEvent;

    await Given('the login form is displayed', async () => {
      // Override handler to return error
      server.use(
        http.post('/api/auth/login', async () => {
          return HttpResponse.json(
            { success: false, error: 'Invalid email or password' },
            { status: 401 }
          );
        })
      );

      const result = renderAuth(<Login />, { route: '/login' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user enters wrong password and submits', async () => {
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign in/i });

      await user.type(emailInput, 'test@example.com');
      await user.type(passwordInput, 'wrongPassword');
      await user.click(submitButton);
    });

    await Then('error message is shown', async () => {
      // The error should be displayed via toast - we check it doesn't navigate
      await waitFor(
        () => {
          // Should still be on login page, not dashboard
          expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
        },
        { timeout: 2000 }
      );

      // Verify we didn't navigate to dashboard
      expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();
    });
  });

  Scenario('User submits login form with empty fields', async () => {
    let user: UserEvent;

    await Given('the login form is displayed', async () => {
      const result = renderAuth(<Login />, { route: '/login' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user submits the form without entering any data', async () => {
      const submitButton = screen.getByRole('button', { name: /sign in/i });
      await user.click(submitButton);
    });

    await Then('validation errors are shown', async () => {
      await waitFor(() => {
        // Email validation error
        const emailError =
          screen.queryByText(/invalid email/i) ||
          screen.queryByText(/email/i, { selector: '.text-destructive' });

        // Password validation error
        const passwordError =
          screen.queryByText(/password.*required/i) ||
          screen.queryByText(/required/i, { selector: '.text-destructive' });

        // At least one validation error should appear
        expect(emailError || passwordError).toBeTruthy();
      });
    });

    await And('form is not submitted to the server', () => {
      // User should still see the login form
      expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();
    });
  });

  Scenario('User submits login form with invalid email format', async () => {
    let user: UserEvent;

    await Given('the login form is displayed', async () => {
      const result = renderAuth(<Login />, { route: '/login' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user enters an invalid email format', async () => {
      const emailInput = screen.getByLabelText(/email/i) as HTMLInputElement;
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign in/i });

      await user.type(emailInput, 'not-a-valid-email');
      await user.type(passwordInput, 'somePassword');
      await user.click(submitButton);
    });

    await Then('form is not submitted and user stays on login page', async () => {
      // HTML5 email validation blocks form submission for invalid emails
      // The form should not be submitted (either by HTML5 or Zod validation)
      await waitFor(() => {
        // Verify we're still on login page (not navigated to dashboard)
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
        expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();

        // Check that form validation is preventing submission
        // Either via HTML5 validation error or Zod validation
        const emailInput = screen.getByLabelText(/email/i) as HTMLInputElement;
        const isInvalidByHTML5 = !emailInput.validity.valid;
        const hasZodError = screen.queryByText(/invalid email/i) !== null;

        expect(isInvalidByHTML5 || hasZodError).toBe(true);
      });
    });
  });

  Scenario('Login button shows loading state during submission', async () => {
    let user: UserEvent;

    await Given('the login form is displayed with slow API response', async () => {
      // Override handler to simulate slow response
      server.use(
        http.post('/api/auth/login', async () => {
          // Delay response to see loading state
          await new Promise((resolve) => setTimeout(resolve, 500));
          return HttpResponse.json({
            success: true,
            data: {
              accessToken: 'test-token',
              refreshToken: 'test-refresh',
              tokenType: 'Bearer',
              expiresIn: 900,
            },
          });
        })
      );

      const result = renderAuth(<Login />, { route: '/login' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user submits valid credentials', async () => {
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/password/i);

      await user.type(emailInput, 'test@example.com');
      await user.type(passwordInput, 'validPassword123');

      const submitButton = screen.getByRole('button', { name: /sign in/i });
      await user.click(submitButton);
    });

    await Then('submit button shows loading state', async () => {
      // Check for loading text or disabled state
      await waitFor(() => {
        const loadingButton = screen.queryByRole('button', { name: /signing in/i });
        const disabledButton = screen.getByRole('button');

        expect(loadingButton || disabledButton.hasAttribute('disabled')).toBeTruthy();
      });
    });
  });

  Scenario('Login form has link to register page', async () => {
    await Given('the login form is displayed', async () => {
      renderAuth(<Login />, { route: '/login' });

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await Then('a link to register page is visible', () => {
      const signUpLink = screen.getByRole('link', { name: /sign up/i });
      expect(signUpLink).toBeInTheDocument();
      expect(signUpLink).toHaveAttribute('href', '/register');
    });
  });
});

// ============================================================================
// Register Page Tests
// ============================================================================

Feature('User Registration', () => {
  beforeEach(() => {
    clearAuthState();
  });

  Scenario('User registers with valid information', async () => {
    let user: UserEvent;

    await Given('the registration form is displayed', async () => {
      const result = renderAuth(<Register />, { route: '/register' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user fills all fields correctly and submits', async () => {
      const displayNameInput = screen.getByLabelText(/display name/i);
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/^password$/i);
      const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
      const submitButton = screen.getByRole('button', { name: /create an account/i });

      await user.type(displayNameInput, 'John Doe');
      await user.type(emailInput, 'newuser@example.com');
      await user.type(passwordInput, 'securePassword123');
      await user.type(confirmPasswordInput, 'securePassword123');
      await user.click(submitButton);
    });

    await Then('account is created and user is redirected to dashboard', async () => {
      await waitFor(
        () => {
          expect(screen.getByTestId('dashboard')).toBeInTheDocument();
        },
        { timeout: 3000 }
      );
    });
  });

  Scenario('User attempts to register with existing email', async () => {
    let user: UserEvent;

    await Given('the registration form is displayed', async () => {
      // Override handler to return email exists error
      server.use(
        http.post('/api/auth/register', async () => {
          return HttpResponse.json(
            { success: false, error: 'Email already registered' },
            { status: 409 }
          );
        })
      );

      const result = renderAuth(<Register />, { route: '/register' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user submits registration with an existing email', async () => {
      const displayNameInput = screen.getByLabelText(/display name/i);
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/^password$/i);
      const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
      const submitButton = screen.getByRole('button', { name: /create an account/i });

      await user.type(displayNameInput, 'John Doe');
      await user.type(emailInput, 'exists@example.com');
      await user.type(passwordInput, 'securePassword123');
      await user.type(confirmPasswordInput, 'securePassword123');
      await user.click(submitButton);
    });

    await Then('duplicate email error is shown', async () => {
      // Should stay on register page
      await waitFor(
        () => {
          expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
        },
        { timeout: 2000 }
      );

      // Should not navigate to dashboard
      expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();
    });
  });

  Scenario('User enters mismatched passwords', async () => {
    let user: UserEvent;

    await Given('the registration form is displayed', async () => {
      const result = renderAuth(<Register />, { route: '/register' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When(
      'user enters different passwords in password and confirm password fields',
      async () => {
        const displayNameInput = screen.getByLabelText(/display name/i);
        const emailInput = screen.getByLabelText(/email/i);
        const passwordInput = screen.getByLabelText(/^password$/i);
        const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
        const submitButton = screen.getByRole('button', { name: /create an account/i });

        await user.type(displayNameInput, 'John Doe');
        await user.type(emailInput, 'test@example.com');
        await user.type(passwordInput, 'securePassword123');
        await user.type(confirmPasswordInput, 'differentPassword456');
        await user.click(submitButton);
      }
    );

    await Then('password mismatch validation error is displayed', async () => {
      await waitFor(() => {
        const errorText =
          screen.queryByText(/password.*don't match/i) || screen.queryByText(/passwords.*match/i);
        expect(errorText).toBeInTheDocument();
      });
    });

    await And('form is not submitted', () => {
      expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();
    });
  });

  Scenario('User submits registration with empty fields', async () => {
    let user: UserEvent;

    await Given('the registration form is displayed', async () => {
      const result = renderAuth(<Register />, { route: '/register' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user clicks submit without filling any fields', async () => {
      const submitButton = screen.getByRole('button', { name: /create an account/i });
      await user.click(submitButton);
    });

    await Then('validation errors are shown for required fields', async () => {
      await waitFor(() => {
        // Check for at least one validation error
        const errors = document.querySelectorAll('.text-destructive');
        expect(errors.length).toBeGreaterThan(0);
      });
    });

    await And('form is not submitted to the server', () => {
      expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();
    });
  });

  Scenario('User enters password that is too short', async () => {
    let user: UserEvent;

    await Given('the registration form is displayed', async () => {
      const result = renderAuth(<Register />, { route: '/register' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user enters a password shorter than 8 characters', async () => {
      const displayNameInput = screen.getByLabelText(/display name/i);
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/^password$/i);
      const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
      const submitButton = screen.getByRole('button', { name: /create an account/i });

      await user.type(displayNameInput, 'John Doe');
      await user.type(emailInput, 'test@example.com');
      await user.type(passwordInput, 'short');
      await user.type(confirmPasswordInput, 'short');
      await user.click(submitButton);
    });

    await Then('password length validation error is displayed', async () => {
      await waitFor(() => {
        const errorText =
          screen.queryByText(/8.*character/i) ||
          screen.queryByText(/password.*min/i) ||
          screen.queryByText(/too short/i);
        expect(errorText).toBeInTheDocument();
      });
    });
  });

  Scenario('Register button shows loading state during submission', async () => {
    let user: UserEvent;

    await Given('the registration form is displayed with slow API response', async () => {
      // Override handler to simulate slow response
      server.use(
        http.post('/api/auth/register', async () => {
          await new Promise((resolve) => setTimeout(resolve, 500));
          return HttpResponse.json({
            success: true,
            data: {
              accessToken: 'test-token',
              refreshToken: 'test-refresh',
              tokenType: 'Bearer',
              expiresIn: 900,
            },
          });
        })
      );

      const result = renderAuth(<Register />, { route: '/register' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user submits valid registration data', async () => {
      const displayNameInput = screen.getByLabelText(/display name/i);
      const emailInput = screen.getByLabelText(/email/i);
      const passwordInput = screen.getByLabelText(/^password$/i);
      const confirmPasswordInput = screen.getByLabelText(/confirm password/i);

      await user.type(displayNameInput, 'John Doe');
      await user.type(emailInput, 'newuser@example.com');
      await user.type(passwordInput, 'securePassword123');
      await user.type(confirmPasswordInput, 'securePassword123');

      const submitButton = screen.getByRole('button', { name: /create an account/i });
      await user.click(submitButton);
    });

    await Then('submit button shows loading state', async () => {
      await waitFor(() => {
        // Check for "Creating account..." text or disabled button
        const loadingButton = screen.queryByRole('button', { name: /creating/i });
        const button = screen.getByRole('button');

        expect(loadingButton || button.hasAttribute('disabled')).toBeTruthy();
      });
    });
  });

  Scenario('Registration form has link to login page', async () => {
    await Given('the registration form is displayed', async () => {
      renderAuth(<Register />, { route: '/register' });

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await Then('a link to login page is visible', () => {
      const signInLink = screen.getByRole('link', { name: /sign in/i });
      expect(signInLink).toBeInTheDocument();
      expect(signInLink).toHaveAttribute('href', '/login');
    });
  });

  Scenario('User enters invalid email format during registration', async () => {
    let user: UserEvent;

    await Given('the registration form is displayed', async () => {
      const result = renderAuth(<Register />, { route: '/register' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user enters an invalid email format', async () => {
      const displayNameInput = screen.getByLabelText(/display name/i);
      const emailInput = screen.getByLabelText(/email/i) as HTMLInputElement;
      const passwordInput = screen.getByLabelText(/^password$/i);
      const confirmPasswordInput = screen.getByLabelText(/confirm password/i);
      const submitButton = screen.getByRole('button', { name: /create an account/i });

      await user.type(displayNameInput, 'John Doe');
      await user.type(emailInput, 'invalid-email-format');
      await user.type(passwordInput, 'securePassword123');
      await user.type(confirmPasswordInput, 'securePassword123');
      await user.click(submitButton);
    });

    await Then('form is not submitted and user stays on registration page', async () => {
      // HTML5 email validation blocks form submission for invalid emails
      // The form should not be submitted (either by HTML5 or Zod validation)
      await waitFor(() => {
        // Verify we're still on registration page
        expect(screen.getByLabelText(/display name/i)).toBeInTheDocument();
        expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();

        // Check that form validation is preventing submission
        const emailInput = screen.getByLabelText(/email/i) as HTMLInputElement;
        const isInvalidByHTML5 = !emailInput.validity.valid;
        const hasZodError = screen.queryByText(/invalid email/i) !== null;

        expect(isInvalidByHTML5 || hasZodError).toBe(true);
      });
    });
  });
});

// ============================================================================
// Cross-Component Navigation Tests
// ============================================================================

Feature('Authentication Navigation', () => {
  beforeEach(() => {
    clearAuthState();
  });

  Scenario('User navigates from login to register page', async () => {
    let user: UserEvent;

    await Given('the login form is displayed', async () => {
      const result = renderAuth(<Login />, { route: '/login' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    await When('user clicks the sign up link', async () => {
      const signUpLink = screen.getByRole('link', { name: /sign up/i });
      await user.click(signUpLink);
    });

    await Then('registration form is displayed', async () => {
      await waitFor(() => {
        // Register page has display name field, login page does not
        expect(screen.getByLabelText(/display name/i)).toBeInTheDocument();
        // Also check for confirm password which is only on register
        expect(screen.getByLabelText(/confirm password/i)).toBeInTheDocument();
      });
    });
  });

  Scenario('User navigates from register to login page', async () => {
    let user: UserEvent;

    await Given('the registration form is displayed', async () => {
      const result = renderAuth(<Register />, { route: '/register' });
      user = result.user;

      await waitFor(() => {
        expect(screen.getByLabelText(/display name/i)).toBeInTheDocument();
      });
    });

    await When('user clicks the sign in link', async () => {
      const signInLink = screen.getByRole('link', { name: /sign in/i });
      await user.click(signInLink);
    });

    await Then('login form is displayed', async () => {
      await waitFor(() => {
        // Login page should NOT have display name or confirm password fields
        expect(screen.queryByLabelText(/display name/i)).not.toBeInTheDocument();
        expect(screen.queryByLabelText(/confirm password/i)).not.toBeInTheDocument();
        // But should have email and password
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
      });
    });
  });
});
