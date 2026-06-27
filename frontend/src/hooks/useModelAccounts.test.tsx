import { describe, expect, it, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactNode } from 'react';
import {
  useModelAccounts,
  useCreateModelAccount,
  useValidateModelAccount,
  useDeleteModelAccount,
} from './useModelAccounts';
import { modelAccountsApi } from '@/api/modelAccounts';
import type { ModelAccount, ModelValidationResult } from '@/types/modelAccount';

vi.mock('@/api/modelAccounts', () => ({
  modelAccountsApi: {
    listAccounts: vi.fn(),
    getAccount: vi.fn(),
    createAccount: vi.fn(),
    updateAccount: vi.fn(),
    deleteAccount: vi.fn(),
    validateAccount: vi.fn(),
  },
}));

const mockedApi = vi.mocked(modelAccountsApi);

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

const mockAccount: ModelAccount = {
  id: 'acc-1',
  organizationId: null,
  userId: 'user-1',
  providerKind: 'claude',
  displayName: 'My Claude key',
  endpointUrl: null,
  egressClass: 'external',
  modelId: 'claude-sonnet-4',
  modelPath: null,
  authType: 'api_key',
  validationStatus: 'unvalidated',
  spendCapUsd: null,
  spendUsedUsd: '0',
  lastValidatedAt: null,
  enabled: true,
  isDefault: false,
  hasCredentials: true,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe('useModelAccounts hooks', () => {
  it('should_returnAccounts_when_listQuerySucceeds', async () => {
    mockedApi.listAccounts.mockResolvedValue([mockAccount]);

    const { result } = renderHook(() => useModelAccounts(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual([mockAccount]);
  });

  it('should_callCreateAccount_when_createMutationFires', async () => {
    mockedApi.createAccount.mockResolvedValue(mockAccount);

    const { result } = renderHook(() => useCreateModelAccount(), { wrapper: createWrapper() });
    result.current.mutate({ providerKind: 'claude', displayName: 'My Claude key', apiKey: 'x' });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.createAccount).toHaveBeenCalledWith(
      expect.objectContaining({ providerKind: 'claude', apiKey: 'x' })
    );
  });

  it('should_callValidateAccount_when_validateMutationFires', async () => {
    const validation: ModelValidationResult = {
      isValid: true,
      validationStatus: 'valid',
      lastValidatedAt: '2026-01-01T00:00:00Z',
    };
    mockedApi.validateAccount.mockResolvedValue(validation);

    const { result } = renderHook(() => useValidateModelAccount(), { wrapper: createWrapper() });
    result.current.mutate('acc-1');

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.validateAccount).toHaveBeenCalledWith('acc-1');
    expect(result.current.data).toEqual(validation);
  });

  it('should_callDeleteAccount_when_deleteMutationFires', async () => {
    mockedApi.deleteAccount.mockResolvedValue(undefined);

    const { result } = renderHook(() => useDeleteModelAccount(), { wrapper: createWrapper() });
    result.current.mutate('acc-1');

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApi.deleteAccount).toHaveBeenCalledWith('acc-1');
  });
});
