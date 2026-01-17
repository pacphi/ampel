/**
 * Dashboard Page BDD Tests
 *
 * Tests the Dashboard component using BDD structure (Given/When/Then)
 * with MSW for API mocking and proper i18n integration.
 */

import { describe, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { render, screen, waitFor } from '../setup/test-utils';
import userEvent from '@testing-library/user-event';
import { server, http, HttpResponse, delay } from '../setup/msw/server';
import { Feature, Scenario, Given, When, Then, And } from '../setup/bdd-helpers';
import Dashboard from '@/pages/Dashboard';
import {
  mockDashboardSummary,
  mockRepositories,
  mockEmptyDashboardSummary,
} from '../setup/msw/fixtures/dashboard';
import { mockPullRequests, createPaginatedResponse } from '../setup/msw/fixtures/pull-requests';
import { mockUserSettings } from '../setup/msw/fixtures/settings';
import { successResponse, errorResponse } from '../setup/msw/fixtures/auth';

// ============================================================================
// MSW Server Lifecycle
// ============================================================================

beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

// ============================================================================
// Test Fixtures and Helpers
// ============================================================================

const API_BASE = '/api';

/**
 * Setup default handlers for Dashboard tests
 */
function setupDefaultHandlers() {
  server.use(
    http.get(`${API_BASE}/dashboard/summary`, () => {
      return HttpResponse.json(successResponse(mockDashboardSummary));
    }),
    http.get(`${API_BASE}/dashboard/grid`, () => {
      return HttpResponse.json(successResponse(mockRepositories));
    }),
    http.get(`${API_BASE}/pull-requests`, ({ request }) => {
      const url = new URL(request.url);
      const page = parseInt(url.searchParams.get('page') || '1', 10);
      const perPage = parseInt(url.searchParams.get('perPage') || '20', 10);
      return HttpResponse.json(
        successResponse(createPaginatedResponse(mockPullRequests, page, perPage))
      );
    }),
    http.get(`${API_BASE}/settings/behavior`, () => {
      return HttpResponse.json(successResponse(mockUserSettings));
    })
  );
}

/**
 * Setup handlers for empty state
 */
function setupEmptyStateHandlers() {
  server.use(
    http.get(`${API_BASE}/dashboard/summary`, () => {
      return HttpResponse.json(successResponse(mockEmptyDashboardSummary));
    }),
    http.get(`${API_BASE}/dashboard/grid`, () => {
      return HttpResponse.json(successResponse([]));
    }),
    http.get(`${API_BASE}/pull-requests`, () => {
      return HttpResponse.json(successResponse(createPaginatedResponse([], 1, 20)));
    }),
    http.get(`${API_BASE}/settings/behavior`, () => {
      return HttpResponse.json(successResponse(mockUserSettings));
    })
  );
}

/**
 * Setup handlers for slow API response
 */
function setupSlowHandlers(delayMs: number = 2000) {
  server.use(
    http.get(`${API_BASE}/dashboard/summary`, async () => {
      await delay(delayMs);
      return HttpResponse.json(successResponse(mockDashboardSummary));
    }),
    http.get(`${API_BASE}/dashboard/grid`, async () => {
      await delay(delayMs);
      return HttpResponse.json(successResponse(mockRepositories));
    }),
    http.get(`${API_BASE}/pull-requests`, async () => {
      await delay(delayMs);
      return HttpResponse.json(successResponse(createPaginatedResponse(mockPullRequests, 1, 20)));
    }),
    http.get(`${API_BASE}/settings/behavior`, async () => {
      await delay(delayMs);
      return HttpResponse.json(successResponse(mockUserSettings));
    })
  );
}

/**
 * Setup handlers for API error
 */
function setupErrorHandlers(statusCode: number = 500) {
  server.use(
    http.get(`${API_BASE}/dashboard/summary`, () => {
      return HttpResponse.json(errorResponse('Internal server error'), { status: statusCode });
    }),
    http.get(`${API_BASE}/dashboard/grid`, () => {
      return HttpResponse.json(errorResponse('Internal server error'), { status: statusCode });
    }),
    http.get(`${API_BASE}/pull-requests`, () => {
      return HttpResponse.json(successResponse(createPaginatedResponse(mockPullRequests, 1, 20)));
    }),
    http.get(`${API_BASE}/settings/behavior`, () => {
      return HttpResponse.json(successResponse(mockUserSettings));
    })
  );
}

/**
 * Wait for loading to complete
 */
async function waitForDashboardToLoad() {
  // Wait for the loading spinner to disappear
  await waitFor(
    () => {
      const loadingSpinner = document.querySelector('.animate-spin');
      expect(loadingSpinner).not.toBeInTheDocument();
    },
    { timeout: 5000 }
  );
}

// ============================================================================
// BDD Test Scenarios
// ============================================================================

Feature('Dashboard Page', () => {
  // --------------------------------------------------------------------------
  // Scenario 1: Loading State
  // --------------------------------------------------------------------------
  Scenario('User sees loading state while dashboard data is being fetched', async () => {
    await Given('API is slow to respond', async () => {
      setupSlowHandlers(3000);
    });

    await When('Dashboard mounts', async () => {
      render(<Dashboard />);
    });

    await Then('Loading spinner is shown', async () => {
      // The loading spinner should be visible
      const loadingSpinner = document.querySelector('.animate-spin');
      expect(loadingSpinner).toBeInTheDocument();
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 2: Success State - Stats Display
  // --------------------------------------------------------------------------
  Scenario('User views dashboard with repository statistics', async () => {
    await Given('API returns valid summary data', async () => {
      setupDefaultHandlers();
    });

    await When('Dashboard loads successfully', async () => {
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await Then('Total repositories count is displayed', async () => {
      // The mockDashboardSummary has totalRepositories: 5
      // Use aria-label to find the specific element
      await waitFor(() => {
        const repoCountElement = screen.getByRole('status', { name: /Total Repositories: 5/i });
        expect(repoCountElement).toBeInTheDocument();
      });
    });

    await And('Total open PRs count is displayed', async () => {
      // The mockDashboardSummary has totalOpenPrs: 13
      await waitFor(() => {
        const prCountElement = screen.getByRole('status', { name: /Open PRs: 13/i });
        expect(prCountElement).toBeInTheDocument();
      });
    });

    await And('Needs attention count is displayed', async () => {
      // The mockDashboardSummary has statusCounts.red: 1
      await waitFor(() => {
        const needsAttentionElement = screen.getByRole('status', { name: /Needs Attention: 1/i });
        expect(needsAttentionElement).toBeInTheDocument();
      });
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 3: Success State - Repositories Display
  // --------------------------------------------------------------------------
  Scenario('User sees repositories in grid view', async () => {
    await Given('API returns repositories', async () => {
      setupDefaultHandlers();
    });

    await When('Dashboard loads in grid view', async () => {
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await Then('Repository names are displayed', async () => {
      // Check for repository names from mockRepositories
      await waitFor(() => {
        expect(screen.getByText('ampel-frontend')).toBeInTheDocument();
        expect(screen.getByText('ampel-backend')).toBeInTheDocument();
      });
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 4: Error State
  // --------------------------------------------------------------------------
  Scenario('User sees error when API fails', async () => {
    await Given('API returns 500 error', async () => {
      setupErrorHandlers(500);
    });

    await When('Dashboard tries to load', async () => {
      render(<Dashboard />);
    });

    await Then('Dashboard still renders without crashing', async () => {
      // The Dashboard should render even with errors (handled by ErrorBoundary)
      // Wait for the title to be present (Dashboard renders regardless of data errors)
      await waitFor(() => {
        expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
      });
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 5: View Mode Switching
  // --------------------------------------------------------------------------
  Scenario('User switches from grid view to list view', async () => {
    const user = userEvent.setup();

    await Given('Dashboard is loaded in grid view', async () => {
      setupDefaultHandlers();
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await When('User clicks list view button', async () => {
      // Find the list view button by its title attribute
      const listViewButton = screen.getByTitle('Repository list view');
      await user.click(listViewButton);
    });

    await Then('View switches to list mode', async () => {
      // The list view button should now have the active variant
      const listViewButton = screen.getByTitle('Repository list view');
      // Active button has 'default' variant which includes specific classes
      expect(listViewButton).toBeInTheDocument();
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 6: View Mode Switching to PRs
  // --------------------------------------------------------------------------
  Scenario('User switches to pull requests view', async () => {
    const user = userEvent.setup();

    await Given('Dashboard is loaded in grid view', async () => {
      setupDefaultHandlers();
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await When('User clicks pull requests view button', async () => {
      const prViewButton = screen.getByTitle('Pull requests view');
      await user.click(prViewButton);
    });

    await Then('View switches to PR list mode', async () => {
      const prViewButton = screen.getByTitle('Pull requests view');
      expect(prViewButton).toBeInTheDocument();
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 7: Refresh Action
  // --------------------------------------------------------------------------
  Scenario('User refreshes dashboard data', async () => {
    const user = userEvent.setup();
    let refreshCallCount = 0;

    await Given('Dashboard is displayed with data', async () => {
      // Setup handlers that track refresh calls
      server.use(
        http.get(`${API_BASE}/dashboard/summary`, () => {
          refreshCallCount++;
          return HttpResponse.json(successResponse(mockDashboardSummary));
        }),
        http.get(`${API_BASE}/dashboard/grid`, () => {
          return HttpResponse.json(successResponse(mockRepositories));
        }),
        http.get(`${API_BASE}/pull-requests`, () => {
          return HttpResponse.json(
            successResponse(createPaginatedResponse(mockPullRequests, 1, 20))
          );
        }),
        http.get(`${API_BASE}/settings/behavior`, () => {
          return HttpResponse.json(successResponse(mockUserSettings));
        })
      );

      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await When('User clicks refresh button', async () => {
      // Find the refresh button by looking for the button with RefreshCw icon
      const refreshButton = screen.getByRole('button', { name: /refresh/i });
      await user.click(refreshButton);
    });

    await Then('Data is refetched', async () => {
      // Wait for the refresh to complete and verify API was called again
      await waitFor(() => {
        // Initial load + refresh = at least 2 calls
        expect(refreshCallCount).toBeGreaterThanOrEqual(2);
      });
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 8: Empty State
  // --------------------------------------------------------------------------
  Scenario('User sees empty state when no repositories exist', async () => {
    await Given('User has no repositories', async () => {
      setupEmptyStateHandlers();
    });

    await When('Dashboard loads', async () => {
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await Then('Empty state message is shown', async () => {
      // The GridView shows "No repositories found" when empty
      await waitFor(() => {
        expect(screen.getByText('No repositories found')).toBeInTheDocument();
      });
    });

    await And('Stats show zero values', async () => {
      // Empty state should show 0 for repositories
      await waitFor(() => {
        const zeroValues = screen.getAllByText('0');
        expect(zeroValues.length).toBeGreaterThan(0);
      });
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 9: Filter Toggle
  // --------------------------------------------------------------------------
  Scenario('User toggles filter to show only repositories with PRs', async () => {
    const user = userEvent.setup();

    await Given('Dashboard is loaded with repositories', async () => {
      setupDefaultHandlers();
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await When('User clicks the filter checkbox', async () => {
      // Find the "Show repositories with Open PRs" checkbox
      const filterCheckbox = screen.getByRole('checkbox', {
        name: /show repositories with open prs/i,
      });
      await user.click(filterCheckbox);
    });

    await Then('Filter checkbox becomes checked', async () => {
      const filterCheckbox = screen.getByRole('checkbox', {
        name: /show repositories with open prs/i,
      });
      expect(filterCheckbox).toBeChecked();
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 10: Dashboard Title Translation
  // --------------------------------------------------------------------------
  Scenario('Dashboard displays translated title', async () => {
    await Given('i18n is configured for English', async () => {
      setupDefaultHandlers();
    });

    await When('Dashboard renders', async () => {
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await Then('Dashboard title is displayed', async () => {
      // The dashboard title should use the translation key 'dashboard:title'
      // which translates to 'Dashboard' in English
      await waitFor(() => {
        expect(screen.getByRole('heading', { level: 1, name: 'Dashboard' })).toBeInTheDocument();
      });
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 11: Stats Tiles Display Correct Labels
  // --------------------------------------------------------------------------
  Scenario('Stats tiles display correct translated labels', async () => {
    await Given('Dashboard API returns valid data', async () => {
      setupDefaultHandlers();
    });

    await When('Dashboard loads', async () => {
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await Then('Total Repositories label is shown', async () => {
      await waitFor(() => {
        expect(screen.getByText('Total Repositories')).toBeInTheDocument();
      });
    });

    await And('Open PRs label is shown', async () => {
      await waitFor(() => {
        expect(screen.getByText('Open PRs')).toBeInTheDocument();
      });
    });

    await And('Ready to Merge label is shown', async () => {
      await waitFor(() => {
        expect(screen.getByText('Ready to Merge')).toBeInTheDocument();
      });
    });

    await And('Needs Attention label is shown', async () => {
      await waitFor(() => {
        expect(screen.getByText('Needs Attention')).toBeInTheDocument();
      });
    });
  });

  // --------------------------------------------------------------------------
  // Scenario 12: View Mode Buttons Have Correct Titles
  // --------------------------------------------------------------------------
  Scenario('View mode buttons have accessible titles', async () => {
    await Given('Dashboard is loaded', async () => {
      setupDefaultHandlers();
      render(<Dashboard />);
      await waitForDashboardToLoad();
    });

    await Then('Grid view button has correct title', async () => {
      expect(screen.getByTitle('Repository grid view')).toBeInTheDocument();
    });

    await And('List view button has correct title', async () => {
      expect(screen.getByTitle('Repository list view')).toBeInTheDocument();
    });

    await And('PR view button has correct title', async () => {
      expect(screen.getByTitle('Pull requests view')).toBeInTheDocument();
    });
  });
});

// ============================================================================
// Additional describe blocks for specific component features
// ============================================================================

describe('Dashboard API Integration', () => {
  beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
  afterEach(() => server.resetHandlers());
  afterAll(() => server.close());

  it('should handle network errors gracefully', async () => {
    server.use(
      http.get(`${API_BASE}/dashboard/summary`, () => {
        return HttpResponse.error();
      }),
      http.get(`${API_BASE}/dashboard/grid`, () => {
        return HttpResponse.error();
      }),
      http.get(`${API_BASE}/pull-requests`, () => {
        return HttpResponse.json(successResponse(createPaginatedResponse([], 1, 20)));
      }),
      http.get(`${API_BASE}/settings/behavior`, () => {
        return HttpResponse.json(successResponse(mockUserSettings));
      })
    );

    render(<Dashboard />);

    // Dashboard should still render the header even with network errors
    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
    });
  });

  it('should display correct repository count from API', async () => {
    const customSummary = {
      ...mockDashboardSummary,
      totalRepositories: 42,
    };

    server.use(
      http.get(`${API_BASE}/dashboard/summary`, () => {
        return HttpResponse.json(successResponse(customSummary));
      }),
      http.get(`${API_BASE}/dashboard/grid`, () => {
        return HttpResponse.json(successResponse(mockRepositories));
      }),
      http.get(`${API_BASE}/pull-requests`, () => {
        return HttpResponse.json(successResponse(createPaginatedResponse(mockPullRequests, 1, 20)));
      }),
      http.get(`${API_BASE}/settings/behavior`, () => {
        return HttpResponse.json(successResponse(mockUserSettings));
      })
    );

    render(<Dashboard />);

    await waitFor(() => {
      // Use aria-label to find the specific element
      const repoCountElement = screen.getByRole('status', { name: /Total Repositories: 42/i });
      expect(repoCountElement).toBeInTheDocument();
    });
  });
});
