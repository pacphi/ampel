import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ModelAccountManager } from './ModelAccountManager';
import type { ModelAccount } from '@/types/modelAccount';
import type {
  ModelCatalog,
  OllamaPullResponse,
  OllamaPullStatusResponse,
  OllamaTagsResponse,
} from '@/types/remediation';

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

// Real catalog hooks run against this mocked client (network isolated).
const mockGetCatalog = vi.fn();
const mockGetOllamaTags = vi.fn();
const mockPullOllamaModel = vi.fn();
const mockGetPullStatus = vi.fn();

vi.mock('@/api/modelCatalog', () => ({
  modelCatalogApi: {
    getCatalog: (...args: unknown[]) => mockGetCatalog(...args),
    getOllamaTags: (...args: unknown[]) => mockGetOllamaTags(...args),
    pullOllamaModel: (...args: unknown[]) => mockPullOllamaModel(...args),
    getPullStatus: (...args: unknown[]) => mockGetPullStatus(...args),
  },
}));

const catalog: ModelCatalog = {
  providers: [
    {
      kind: 'claude',
      description: 'Anthropic hosted models',
      egress: 'external',
      models: [
        {
          id: 'claude-sonnet-4',
          name: 'Claude Sonnet 4',
          family: 'claude',
          quality: 'high',
          contextWindow: 200000,
          toolUse: true,
          codeEdit: true,
          egress: 'external',
          outputContract: 'text',
          cost: { kind: 'per_token', inputPer1k: 3, outputPer1k: 15 },
          params: '',
          quantization: '',
          community: false,
        },
      ],
    },
    {
      kind: 'ollama',
      description: 'Local Ollama models',
      egress: 'local_only',
      models: [
        {
          id: 'llama3-8b',
          name: 'Llama 3 8B',
          family: 'llama',
          quality: 'medium',
          ollamaTag: 'llama3:8b',
          contextWindow: 8192,
          toolUse: false,
          codeEdit: true,
          egress: 'local_only',
          outputContract: 'text',
          cost: { kind: 'free' },
          params: '8B',
          quantization: 'Q4_K_M',
          minRamGb: 16,
          diskGb: '5',
          hardwareTier: '16',
          community: true,
        },
        {
          id: 'qwen3-30b',
          name: 'Qwen3 30B',
          family: 'qwen',
          quality: 'high',
          ollamaTag: 'qwen3:30b',
          contextWindow: 32768,
          toolUse: true,
          codeEdit: true,
          egress: 'local_only',
          outputContract: 'text',
          cost: { kind: 'free' },
          params: '30B MoE (A3B)',
          quantization: 'Q4_K_M',
          minRamGb: 64,
          diskGb: '19',
          hardwareTier: '64',
          community: false,
        },
      ],
    },
  ],
};

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

const ollamaAccount: ModelAccount = {
  ...sampleAccount,
  id: 'acc-ollama',
  providerKind: 'ollama',
  displayName: 'Local Ollama',
  egressClass: 'local_only',
  endpointUrl: 'http://localhost:11434',
  modelId: 'llama3-8b',
  authType: 'none',
};

function renderManager() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <ModelAccountManager />
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  accountsData = [];
  mockGetCatalog.mockResolvedValue(catalog);
  mockGetOllamaTags.mockResolvedValue({ models: [] } as OllamaTagsResponse);
  mockPullOllamaModel.mockResolvedValue({
    jobId: 'job-1',
    status: 'queued',
  } as OllamaPullResponse);
  mockGetPullStatus.mockResolvedValue({
    jobId: 'job-1',
    status: 'ready',
    detail: 'Download complete',
  } as OllamaPullStatusResponse);
});

describe('ModelAccountManager', () => {
  it('should_renderAccountList_when_accountsExist', () => {
    accountsData = [sampleAccount];
    renderManager();

    expect(screen.getByText('My Claude key')).toBeInTheDocument();
    expect(
      screen.getByText('remediation:modelAccounts.validationStatus.unvalidated')
    ).toBeInTheDocument();
  });

  it('should_renderEmptyState_when_noAccounts', () => {
    accountsData = [];
    renderManager();
    expect(screen.getByText('remediation:modelAccounts.empty')).toBeInTheDocument();
  });

  it('should_renderApiKeyAsPasswordField_when_creatingHostedProvider', async () => {
    const user = userEvent.setup();
    renderManager();

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));

    const apiKeyInput = screen.getByLabelText('remediation:modelAccounts.apiKey');
    expect(apiKeyInput).toHaveAttribute('type', 'password');
  });

  it('should_notEchoApiKeyValue_when_typed', async () => {
    const user = userEvent.setup();
    renderManager();

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));
    const apiKeyInput = screen.getByLabelText(
      'remediation:modelAccounts.apiKey'
    ) as HTMLInputElement;
    await user.type(apiKeyInput, 'sk-secret-123');

    expect(apiKeyInput.type).toBe('password');
    expect(screen.queryByText('sk-secret-123')).not.toBeInTheDocument();
  });

  it('should_submitCreate_when_formFilled', async () => {
    const user = userEvent.setup();
    renderManager();

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
    renderManager();

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
    renderManager();

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.validate' }));

    expect(mockValidate).toHaveBeenCalledWith('acc-1');
  });

  it('should_setModelIdFromCatalog_when_optionSelected', async () => {
    const user = userEvent.setup();
    renderManager();

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));
    await user.type(screen.getByLabelText('remediation:modelAccounts.displayName'), 'Prod Claude');

    // Catalog dropdown renders options from the mocked GET /api/model-catalog.
    const modelTrigger = await screen.findByRole('combobox', {
      name: 'remediation:modelCatalog.model',
    });
    await user.click(modelTrigger);
    await user.click(await screen.findByRole('option', { name: /Claude Sonnet 4/ }));

    await user.type(screen.getByLabelText('remediation:modelAccounts.apiKey'), 'sk-abc');
    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.save' }));

    await waitFor(() => expect(mockCreate).toHaveBeenCalled());
    expect(mockCreate).toHaveBeenCalledWith(
      expect.objectContaining({ modelId: 'claude-sonnet-4' }),
      expect.anything()
    );
  });

  it('should_overrideCatalogSelection_when_customModelIdProvided', async () => {
    const user = userEvent.setup();
    renderManager();

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));
    await user.type(screen.getByLabelText('remediation:modelAccounts.displayName'), 'Prod Claude');

    const modelTrigger = await screen.findByRole('combobox', {
      name: 'remediation:modelCatalog.model',
    });
    await user.click(modelTrigger);
    await user.click(await screen.findByRole('option', { name: /Claude Sonnet 4/ }));

    // Advanced escape hatch: the custom model id wins over the catalog selection.
    await user.click(screen.getByRole('button', { name: 'remediation:modelCatalog.advanced' }));
    await user.type(
      screen.getByLabelText('remediation:modelCatalog.customModelId'),
      'my-custom-model'
    );

    await user.type(screen.getByLabelText('remediation:modelAccounts.apiKey'), 'sk-abc');
    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.save' }));

    await waitFor(() => expect(mockCreate).toHaveBeenCalled());
    expect(mockCreate).toHaveBeenCalledWith(
      expect.objectContaining({ modelId: 'my-custom-model' }),
      expect.anything()
    );
  });

  it('should_renderHardwarePillsAndCommunityMarker_when_ollamaOptionShown', async () => {
    const user = userEvent.setup();
    renderManager();

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));

    // Switch to the local Ollama provider to surface hardware-bearing models.
    const providerTrigger = screen.getByRole('combobox', {
      name: 'remediation:modelAccounts.providerKind',
    });
    await user.click(providerTrigger);
    await user.click(
      await screen.findByRole('option', {
        name: 'remediation:modelAccounts.providerKindLabel.ollama',
      })
    );

    const modelTrigger = await screen.findByRole('combobox', {
      name: 'remediation:modelCatalog.model',
    });
    await user.click(modelTrigger);

    // The community tier-16 option renders params/quant/tier pills + community marker.
    const communityOption = await screen.findByRole('option', { name: /Llama 3 8B/ });
    expect(
      within(communityOption).getByText('remediation:modelCatalog.hardware.params')
    ).toBeInTheDocument();
    expect(
      within(communityOption).getByText('remediation:modelCatalog.hardware.quantization')
    ).toBeInTheDocument();
    expect(
      within(communityOption).getByText('remediation:modelCatalog.hardware.ramTier')
    ).toBeInTheDocument();
    expect(
      within(communityOption).getByText('remediation:modelCatalog.hardware.diskSize')
    ).toBeInTheDocument();
    expect(
      within(communityOption).getByText('remediation:modelCatalog.hardware.community')
    ).toBeInTheDocument();
  });

  it('should_narrowLocalOptionsToTier16_when_ramTierFilterSelected', async () => {
    const user = userEvent.setup();
    renderManager();

    await user.click(screen.getByRole('button', { name: 'remediation:modelAccounts.create' }));

    // Local provider so the RAM-tier filter applies.
    await user.click(
      screen.getByRole('combobox', { name: 'remediation:modelAccounts.providerKind' })
    );
    await user.click(
      await screen.findByRole('option', {
        name: 'remediation:modelAccounts.providerKindLabel.ollama',
      })
    );

    // Choose the 16 GB tier.
    await user.click(
      screen.getByRole('combobox', { name: 'remediation:modelCatalog.hardware.filterLabel' })
    );
    await user.click(
      await screen.findByRole('option', { name: 'remediation:modelCatalog.hardware.tier.16' })
    );

    // Only the tier-16 model remains; the tier-64 model is filtered out.
    await user.click(screen.getByRole('combobox', { name: 'remediation:modelCatalog.model' }));
    expect(await screen.findByRole('option', { name: /Llama 3 8B/ })).toBeInTheDocument();
    expect(screen.queryByRole('option', { name: /Qwen3 30B/ })).not.toBeInTheDocument();

    // Hosted models are unaffected by the filter.
    await user.keyboard('{Escape}');
    await user.click(
      screen.getByRole('combobox', { name: 'remediation:modelAccounts.providerKind' })
    );
    await user.click(
      await screen.findByRole('option', {
        name: 'remediation:modelAccounts.providerKindLabel.claude',
      })
    );
    await user.click(screen.getByRole('combobox', { name: 'remediation:modelCatalog.model' }));
    expect(await screen.findByRole('option', { name: /Claude Sonnet 4/ })).toBeInTheDocument();
  });

  it('should_pullOllamaModel_when_tagNotDiscovered', async () => {
    accountsData = [ollamaAccount];
    const user = userEvent.setup();
    renderManager();

    // Tag not present on the server → the pull button is offered.
    const pullButton = await screen.findByRole('button', {
      name: 'remediation:modelCatalog.pull.pullModel',
    });
    await user.click(pullButton);

    // Pull is POSTed with the resolved Ollama tag...
    await waitFor(() =>
      expect(mockPullOllamaModel).toHaveBeenCalledWith('acc-ollama', 'llama3:8b')
    );

    // ...and status is polled until it reaches `ready`.
    await waitFor(() => expect(mockGetPullStatus).toHaveBeenCalledWith('job-1'));
    expect(
      await screen.findByText('remediation:modelCatalog.pull.status.ready')
    ).toBeInTheDocument();
  });
});
