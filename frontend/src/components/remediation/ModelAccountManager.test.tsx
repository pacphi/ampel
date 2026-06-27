import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ModelAccountManager } from './ModelAccountManager';
import type { ModelAccount } from '@/types/modelAccount';

// Passthrough i18n: t() returns the key so assertions are stable.
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

const mockCreate = vi.fn();
const mockValidate = vi.fn();
const mockDelete = vi.fn();
let accountsData: ModelAccount[] = [];

vi.mock('@/hooks/useModelAccounts', () => ({
  useModelAccounts: () => ({ data: accountsData, isLoading: false, isError: false }),
  useCreateModelAccount: () => ({ mutate: mockCreate, isPending: false }),
  useValidateModelAccount: () => ({ mutate: mockValidate, isPending: false }),
  useDeleteModelAccount: () => ({ mutate: mockDelete, isPending: false }),
}));

const sampleAccount: ModelAccount = {
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
  spendCapUsd: '50.00',
  spendUsedUsd: '1.25',
  lastValidatedAt: null,
  enabled: true,
  isDefault: false,
  hasCredentials: true,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
};

beforeEach(() => {
  vi.clearAllMocks();
  accountsData = [];
});

describe('ModelAccountManager', () => {
  it('should_renderAccountList_when_accountsExist', () => {
    accountsData = [sampleAccount];
    render(<ModelAccountManager />);

    expect(screen.getByText('My Claude key')).toBeInTheDocument();
    expect(
      screen.getByText('remediation:modelAccounts.validationStatus.unvalidated')
    ).toBeInTheDocument();
  });

  it('should_renderEmptyState_when_noAccounts', () => {
    accountsData = [];
    render(<ModelAccountManager />);
    expect(screen.getByText('remediation:modelAccounts.empty')).toBeInTheDocument();
  });

  it('should_renderApiKeyAsPasswordField_when_creatingHostedProvider', async () => {
    const user = userEvent.setup();
    render(<ModelAccountManager />);

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));

    const apiKeyInput = screen.getByLabelText('remediation:modelAccounts.apiKey');
    expect(apiKeyInput).toHaveAttribute('type', 'password');
  });

  it('should_notEchoApiKeyValue_when_typed', async () => {
    const user = userEvent.setup();
    render(<ModelAccountManager />);

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));
    const apiKeyInput = screen.getByLabelText(
      'remediation:modelAccounts.apiKey'
    ) as HTMLInputElement;
    await user.type(apiKeyInput, 'sk-secret-123');

    // The value lives in the field but the field never reveals it (type=password)
    // and no element renders the secret text content.
    expect(apiKeyInput.type).toBe('password');
    expect(screen.queryByText('sk-secret-123')).not.toBeInTheDocument();
  });

  it('should_submitCreate_when_formFilled', async () => {
    const user = userEvent.setup();
    render(<ModelAccountManager />);

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));
    await user.type(screen.getByLabelText('remediation:modelAccounts.displayName'), 'Prod Claude');
    await user.type(screen.getByLabelText('remediation:modelAccounts.apiKey'), 'sk-secret-123');
    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.save' }));

    await waitFor(() => expect(mockCreate).toHaveBeenCalled());
    expect(mockCreate).toHaveBeenCalledWith(
      expect.objectContaining({
        providerKind: 'claude',
        displayName: 'Prod Claude',
        apiKey: 'sk-secret-123',
      }),
      expect.anything()
    );
  });

  it('should_showAirGappedError_when_createReturns422', async () => {
    mockCreate.mockImplementation((_payload, options) => {
      options.onError({ response: { status: 422 } });
    });
    const user = userEvent.setup();
    render(<ModelAccountManager />);

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));
    await user.type(
      screen.getByLabelText('remediation:modelAccounts.displayName'),
      'External Claude'
    );
    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.save' }));

    expect(
      await screen.findByText('remediation:modelAccounts.errors.airGapped')
    ).toBeInTheDocument();
  });

  it('should_callValidate_when_validateClicked', async () => {
    accountsData = [sampleAccount];
    const user = userEvent.setup();
    render(<ModelAccountManager />);

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.validate' }));

    expect(mockValidate).toHaveBeenCalledWith('acc-1');
  });
});
