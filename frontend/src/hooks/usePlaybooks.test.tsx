import { describe, expect, it, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactNode } from 'react';
import {
  usePlaybooks,
  useCreatePlaybook,
  useDeletePlaybook,
  usePreviewPlaybook,
} from './usePlaybooks';
import { playbooksApi } from '@/api/playbooks';
import type { Playbook, PlaybookPreviewResponse } from '@/types/playbook';

vi.mock('@/api/playbooks', () => ({
  playbooksApi: {
    listPlaybooks: vi.fn(),
    getPlaybook: vi.fn(),
    createPlaybook: vi.fn(),
    updatePlaybook: vi.fn(),
    deletePlaybook: vi.fn(),
    previewPlaybook: vi.fn(),
  },
}));

const mockedApi = vi.mocked(playbooksApi);

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

const mockPlaybook: Playbook = {
  id: 'pb-1',
  playbookId: 'custom',
  version: 1,
  source: 'db',
  name: 'Custom',
  description: null,
  content: 'role: fixer',
  enabled: true,
  scopeType: 'user',
  scopeId: 'user-1',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe('usePlaybooks hooks', () => {
  it('should_returnPlaybooks_when_listQuerySucceeds', async () => {
    mockedApi.listPlaybooks.mockResolvedValue([mockPlaybook]);

    const { result } = renderHook(() => usePlaybooks(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual([mockPlaybook]);
  });

  it('should_callCreatePlaybook_when_createMutationFires', async () => {
    mockedApi.createPlaybook.mockResolvedValue(mockPlaybook);

    const { result } = renderHook(() => useCreatePlaybook(), { wrapper: createWrapper() });
    result.current.mutate({ playbookId: 'custom', name: 'Custom', content: 'role: fixer' });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.createPlaybook).toHaveBeenCalledWith(
      expect.objectContaining({ playbookId: 'custom' })
    );
  });

  it('should_returnRenderedPrompt_when_previewMutationFires', async () => {
    const preview: PlaybookPreviewResponse = {
      failureClass: 'build_error',
      role: 'fixer',
      systemInstruction: 'You are a build fixer.',
      outputContract: 'unified_diff',
      allowedTools: ['read_file'],
    };
    mockedApi.previewPlaybook.mockResolvedValue(preview);

    const { result } = renderHook(() => usePreviewPlaybook(), { wrapper: createWrapper() });
    result.current.mutate({ id: 'pb-1', data: {} });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.previewPlaybook).toHaveBeenCalledWith('pb-1', {});
    expect(result.current.data).toEqual(preview);
  });

  it('should_callDeletePlaybook_when_deleteMutationFires', async () => {
    mockedApi.deletePlaybook.mockResolvedValue(undefined);

    const { result } = renderHook(() => useDeletePlaybook(), { wrapper: createWrapper() });
    result.current.mutate('pb-1');

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.deletePlaybook).toHaveBeenCalledWith('pb-1');
  });
});
