/**
 * usePullRequestDiff Hook Tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { usePullRequestDiff, useDiffViewerPreferences } from '../usePullRequestDiff';
import apiClient from '../../api/client';
import React, { type ReactNode } from 'react';

// Mock API client
vi.mock('../../api/client', () => ({
  default: {
    get: vi.fn(),
  },
}));

describe('usePullRequestDiff', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, retryDelay: 0 },
      },
    });
    vi.clearAllMocks();
  });

  const wrapper = ({ children }: { children: ReactNode }) =>
    React.createElement(QueryClientProvider, { client: queryClient }, children);

  it('fetches pull request diff successfully', async () => {
    const mockDiff = `diff --git a/file.ts b/file.ts
--- a/file.ts
+++ b/file.ts
@@ -1,1 +1,1 @@
-old
+new`;

    vi.mocked(apiClient.get).mockResolvedValueOnce({
      data: { diff: mockDiff },
    });

    const { result } = renderHook(() => usePullRequestDiff(123), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data?.pullRequestId).toBe(123);
    expect(result.current.data?.files).toBeDefined();
    expect(result.current.data?.files.length).toBeGreaterThan(0);
  });

  it('does not fetch when pullRequestId is undefined', () => {
    const { result } = renderHook(() => usePullRequestDiff(undefined), { wrapper });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.data).toBeUndefined();
    expect(apiClient.get).not.toHaveBeenCalled();
  });

  it('handles fetch errors', async () => {
    // Mock all retry attempts to fail
    vi.mocked(apiClient.get).mockRejectedValue(new Error('API Error'));

    const { result } = renderHook(() => usePullRequestDiff(123), { wrapper });

    // Wait for isLoading to become false (indicating query completed)
    await waitFor(() => expect(result.current.isLoading).toBe(false), { timeout: 5000 });

    // Now check that error state is set
    expect(result.current.isError).toBe(true);
    expect(result.current.error?.message).toBe('API Error');
  });
});

describe('useDiffViewerPreferences', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('returns default preferences', () => {
    const { preferences } = useDiffViewerPreferences();

    expect(preferences.viewMode).toBe('unified');
    expect(preferences.syntaxHighlighting).toBe(true);
    expect(preferences.showLineNumbers).toBe(true);
    expect(preferences.wrapLines).toBe(false);
    expect(preferences.expandAllByDefault).toBe(false);
  });

  it('saves and retrieves preferences', () => {
    const { setPreferences } = useDiffViewerPreferences();

    setPreferences({ viewMode: 'split', syntaxHighlighting: false });

    const { preferences } = useDiffViewerPreferences();
    expect(preferences.viewMode).toBe('split');
    expect(preferences.syntaxHighlighting).toBe(false);
  });
});
