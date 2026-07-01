import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ChevronDown, ChevronRight, Download, Plus, ShieldAlert, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Progress } from '@/components/ui/progress';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  useCreateModelAccount,
  useDeleteModelAccount,
  useModelAccounts,
  useValidateModelAccount,
} from '@/hooks/useModelAccounts';
import {
  useModelCatalog,
  useOllamaTags,
  usePullOllamaModel,
  usePullStatus,
} from '@/hooks/useModelCatalog';
import type {
  CreateModelAccountRequest,
  ModelAccount,
  ModelValidationStatus,
  ProviderKind,
} from '@/types/modelAccount';
import type { CatalogModel, ModelCatalog, ModelCost } from '@/types/remediation';

const PROVIDER_KINDS: ProviderKind[] = ['claude', 'gemini', 'ollama', 'onnx'];

/** Local providers that take an endpoint URL rather than an API key. */
function isLocalProvider(kind: ProviderKind): boolean {
  return kind === 'ollama' || kind === 'onnx';
}

function validationVariant(status: ModelValidationStatus): 'success' | 'destructive' | 'secondary' {
  switch (status) {
    case 'valid':
      return 'success';
    case 'invalid':
      return 'destructive';
    default:
      return 'secondary';
  }
}

type TFn = (key: string, opts?: Record<string, unknown>) => string;

/** Human-readable price for a catalog model ("Free" or per-1K pricing). */
function formatCost(t: TFn, cost: ModelCost): string {
  if (cost.kind === 'free') return t('remediation:modelCatalog.cost.free');
  return t('remediation:modelCatalog.cost.perToken', {
    input: cost.inputPer1k ?? 0,
    output: cost.outputPer1k ?? 0,
  });
}

/** The models offered for a provider kind, or an empty list. */
function providerModels(catalog: ModelCatalog | undefined, kind: ProviderKind): CatalogModel[] {
  return catalog?.providers.find((p) => p.kind === kind)?.models ?? [];
}

/** Resolve a catalog model by its id across every provider. */
function resolveModel(catalog: ModelCatalog | undefined, modelId: string | null): CatalogModel | undefined {
  if (!modelId) return undefined;
  for (const provider of catalog?.providers ?? []) {
    const match = provider.models.find((m) => m.id === modelId);
    if (match) return match;
  }
  return undefined;
}

/** Compact capability pills shown inside a catalog model option. */
function ModelOptionMeta({ model }: { model: CatalogModel }) {
  const { t } = useTranslation(['remediation']);
  const pill = 'rounded bg-muted px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground';
  return (
    <span className="flex flex-wrap items-center gap-1.5">
      <span className="font-medium text-foreground">{model.name}</span>
      <span className={pill}>{t('remediation:modelCatalog.contextWindow', { tokens: model.contextWindow })}</span>
      <span className={pill}>{formatCost(t, model.cost)}</span>
      {model.toolUse && <span className={pill}>{t('remediation:modelCatalog.toolUse')}</span>}
      <span className={pill}>{t(`remediation:modelAccounts.egress.${model.egress}`)}</span>
    </span>
  );
}

interface CreateFormProps {
  onClose: () => void;
}

function CreateForm({ onClose }: CreateFormProps) {
  const { t } = useTranslation(['remediation', 'common']);
  const createMutation = useCreateModelAccount();
  const { data: catalog, isLoading: catalogLoading, isError: catalogError } = useModelCatalog();

  const [providerKind, setProviderKind] = useState<ProviderKind>('claude');
  const [displayName, setDisplayName] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [endpointUrl, setEndpointUrl] = useState('');
  const [catalogModelId, setCatalogModelId] = useState('');
  const [customModelId, setCustomModelId] = useState('');
  const [spendCapUsd, setSpendCapUsd] = useState('');
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const local = isLocalProvider(providerKind);
  const models = providerModels(catalog, providerKind);

  const handleProviderChange = (v: string) => {
    setProviderKind(v as ProviderKind);
    // Catalog options differ per provider; clear the stale selection.
    setCatalogModelId('');
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!displayName.trim()) {
      setError(t('remediation:modelAccounts.errors.displayNameRequired'));
      return;
    }

    // Custom (Advanced) model id wins over the catalog selection when set.
    const modelId = customModelId.trim() || catalogModelId;

    const payload: CreateModelAccountRequest = {
      providerKind,
      displayName: displayName.trim(),
    };
    if (!local && apiKey) payload.apiKey = apiKey;
    if (local && endpointUrl.trim()) payload.endpointUrl = endpointUrl.trim();
    if (modelId) payload.modelId = modelId;
    if (spendCapUsd.trim()) payload.spendCapUsd = spendCapUsd.trim();

    createMutation.mutate(payload, {
      onSuccess: () => onClose(),
      onError: (err: unknown) => {
        const axiosError = err as { response?: { status?: number; data?: { error?: string } } };
        if (axiosError.response?.status === 422) {
          // ADR-014: external-egress provider rejected in an air-gapped org.
          setError(t('remediation:modelAccounts.errors.airGapped'));
        } else {
          setError(
            axiosError.response?.data?.error ?? t('remediation:modelAccounts.errors.createFailed')
          );
        }
      },
    });
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="space-y-4 rounded-lg border p-4"
      aria-label={t('remediation:modelAccounts.createTitle')}
    >
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="model-provider-kind">{t('remediation:modelAccounts.providerKind')}</Label>
          <Select value={providerKind} onValueChange={handleProviderChange}>
            <SelectTrigger id="model-provider-kind">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PROVIDER_KINDS.map((k) => (
                <SelectItem key={k} value={k}>
                  {t(`remediation:modelAccounts.providerKindLabel.${k}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-2">
          <Label htmlFor="model-display-name">{t('remediation:modelAccounts.displayName')}</Label>
          <Input
            id="model-display-name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder={t('remediation:modelAccounts.displayNamePlaceholder')}
          />
        </div>
      </div>

      {/* API key — hosted providers only. Write-only password field. */}
      {!local && (
        <div className="space-y-2">
          <Label htmlFor="model-api-key">{t('remediation:modelAccounts.apiKey')}</Label>
          <Input
            id="model-api-key"
            type="password"
            autoComplete="off"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={t('remediation:modelAccounts.apiKeyPlaceholder')}
          />
          <p className="text-xs text-muted-foreground">
            {t('remediation:modelAccounts.apiKeyHint')}
          </p>
        </div>
      )}

      {/* Endpoint URL — local providers (e.g. Ollama). */}
      {local && (
        <div className="space-y-2">
          <Label htmlFor="model-endpoint-url">{t('remediation:modelAccounts.endpointUrl')}</Label>
          <Input
            id="model-endpoint-url"
            type="url"
            value={endpointUrl}
            onChange={(e) => setEndpointUrl(e.target.value)}
            placeholder={t('remediation:modelAccounts.endpointUrlPlaceholder')}
          />
        </div>
      )}

      {/* Catalog-driven model picker. */}
      <div className="space-y-2">
        <Label htmlFor="model-catalog-select">{t('remediation:modelCatalog.model')}</Label>
        {catalogLoading ? (
          <p className="text-xs text-muted-foreground">{t('remediation:modelCatalog.loading')}</p>
        ) : catalogError ? (
          <p className="text-xs text-destructive">{t('remediation:modelCatalog.loadError')}</p>
        ) : models.length === 0 ? (
          <p className="text-xs text-muted-foreground">{t('remediation:modelCatalog.noModels')}</p>
        ) : (
          <Select value={catalogModelId} onValueChange={setCatalogModelId}>
            <SelectTrigger id="model-catalog-select">
              <SelectValue placeholder={t('remediation:modelCatalog.modelPlaceholder')} />
            </SelectTrigger>
            <SelectContent>
              {models.map((m) => (
                <SelectItem key={m.id} value={m.id} textValue={m.name}>
                  <ModelOptionMeta model={m} />
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </div>

      {/* Advanced: custom / unlisted model id (overrides the catalog selection). */}
      <div className="space-y-2">
        <button
          type="button"
          onClick={() => setAdvancedOpen((o) => !o)}
          aria-expanded={advancedOpen}
          className="flex items-center gap-1 text-sm font-medium text-muted-foreground hover:text-foreground"
        >
          {advancedOpen ? (
            <ChevronDown className="h-4 w-4" aria-hidden="true" />
          ) : (
            <ChevronRight className="h-4 w-4" aria-hidden="true" />
          )}
          {t('remediation:modelCatalog.advanced')}
        </button>
        {advancedOpen && (
          <div className="space-y-2">
            <Label htmlFor="model-model-id">{t('remediation:modelCatalog.customModelId')}</Label>
            <Input
              id="model-model-id"
              value={customModelId}
              onChange={(e) => setCustomModelId(e.target.value)}
              placeholder={t('remediation:modelCatalog.customModelIdPlaceholder')}
            />
            <p className="text-xs text-muted-foreground">
              {t('remediation:modelCatalog.customModelIdHint')}
            </p>
          </div>
        )}
      </div>

      <div className="space-y-2 sm:max-w-[50%]">
        <Label htmlFor="model-spend-cap">{t('remediation:modelAccounts.spendCap')}</Label>
        <Input
          id="model-spend-cap"
          type="number"
          min={0}
          step="0.01"
          value={spendCapUsd}
          onChange={(e) => setSpendCapUsd(e.target.value)}
          placeholder="0.00"
        />
      </div>

      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}

      <div className="flex justify-end gap-2">
        <Button
          type="button"
          variant="outline"
          onClick={onClose}
          disabled={createMutation.isPending}
        >
          {t('common:actions.cancel')}
        </Button>
        <Button type="submit" disabled={createMutation.isPending}>
          {createMutation.isPending
            ? t('remediation:modelAccounts.saving')
            : t('remediation:modelAccounts.save')}
        </Button>
      </div>
    </form>
  );
}

/** Percentage for the small pull-progress bar, by lifecycle status. */
function pullPercent(status: string): number {
  switch (status) {
    case 'queued':
      return 10;
    case 'downloading':
      return 60;
    case 'ready':
      return 100;
    case 'error':
      return 100;
    default:
      return 0;
  }
}

/**
 * Ollama discovery + pull for a saved account. Shows a "Pull model" button when
 * the resolved catalog tag is not yet present on the server, then polls to done.
 */
function OllamaPullControl({ account, ollamaTag }: { account: ModelAccount; ollamaTag: string }) {
  const { t } = useTranslation(['remediation']);
  const { data: tags } = useOllamaTags(account.id);
  const pullMutation = usePullOllamaModel();
  const [jobId, setJobId] = useState<string | null>(null);
  const { data: pullStatus } = usePullStatus(jobId ?? undefined);

  const discovered = tags?.models.map((m) => m.name) ?? [];
  const alreadyPresent = discovered.includes(ollamaTag);

  if (alreadyPresent) {
    return (
      <Badge variant="success">{t('remediation:modelCatalog.pull.status.ready')}</Badge>
    );
  }

  const started = jobId !== null;

  return (
    <div className="flex w-full flex-col gap-1">
      {!started && (
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={pullMutation.isPending}
          onClick={() =>
            pullMutation.mutate(
              { accountId: account.id, model: ollamaTag },
              { onSuccess: (res) => setJobId(res.jobId) }
            )
          }
        >
          <Download className="mr-1.5 h-4 w-4" aria-hidden="true" />
          {pullMutation.isPending
            ? t('remediation:modelCatalog.pull.pulling')
            : t('remediation:modelCatalog.pull.pullModel')}
        </Button>
      )}
      {started && pullStatus && (
        <div role="status" aria-live="polite" className="flex flex-col gap-1">
          <div className="flex items-center gap-2">
            <Badge
              variant={pullStatus.status === 'error' ? 'destructive' : 'secondary'}
            >
              {t(`remediation:modelCatalog.pull.status.${pullStatus.status}`)}
            </Badge>
            {pullStatus.detail && (
              <span className="text-xs text-muted-foreground">{pullStatus.detail}</span>
            )}
          </div>
          <Progress value={pullPercent(pullStatus.status)} className="h-2" />
        </div>
      )}
    </div>
  );
}

function AccountRow({ account }: { account: ModelAccount }) {
  const { t } = useTranslation(['remediation', 'common']);
  const validateMutation = useValidateModelAccount();
  const deleteMutation = useDeleteModelAccount();
  const { data: catalog } = useModelCatalog();

  const resolved = resolveModel(catalog, account.modelId);

  const spend = account.spendCapUsd
    ? `$${account.spendUsedUsd} / $${account.spendCapUsd}`
    : `$${account.spendUsedUsd}`;

  return (
    <li className="flex flex-wrap items-center gap-3 rounded-lg border p-3">
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <span className="font-medium">{account.displayName}</span>
          <Badge variant="outline">
            {t(`remediation:modelAccounts.providerKindLabel.${account.providerKind}`)}
          </Badge>
          <Badge variant={validationVariant(account.validationStatus)}>
            {t(`remediation:modelAccounts.validationStatus.${account.validationStatus}`)}
          </Badge>
          <Badge variant="secondary">
            {t(`remediation:modelAccounts.egress.${account.egressClass}`)}
          </Badge>
          {resolved && (
            <>
              <Badge variant="outline">
                {t('remediation:modelCatalog.contextWindow', { tokens: resolved.contextWindow })}
              </Badge>
              <Badge variant="outline">{formatCost(t, resolved.cost)}</Badge>
            </>
          )}
          {account.isDefault && (
            <Badge variant="default">{t('remediation:modelAccounts.default')}</Badge>
          )}
        </div>
        <p className="text-xs text-muted-foreground">
          {account.modelId ?? '—'} · {t('remediation:modelAccounts.spend')}: {spend}
        </p>
        {account.providerKind === 'ollama' && resolved?.ollamaTag && (
          <div className="mt-2">
            <OllamaPullControl account={account} ollamaTag={resolved.ollamaTag} />
          </div>
        )}
      </div>
      <Button
        variant="outline"
        size="sm"
        onClick={() => validateMutation.mutate(account.id)}
        disabled={validateMutation.isPending}
      >
        {validateMutation.isPending
          ? t('remediation:modelAccounts.validating')
          : t('remediation:modelAccounts.validate')}
      </Button>
      <Button
        variant="destructive"
        size="sm"
        onClick={() => deleteMutation.mutate(account.id)}
        disabled={deleteMutation.isPending}
        aria-label={t('common:actions.delete')}
      >
        <Trash2 className="h-4 w-4" aria-hidden="true" />
      </Button>
    </li>
  );
}

export function ModelAccountManager() {
  const { t } = useTranslation(['remediation', 'common']);
  const { data: accounts, isLoading, isError } = useModelAccounts();
  const [creating, setCreating] = useState(false);

  return (
    <div className="space-y-4">
      <div className="flex items-start gap-2 rounded-md bg-muted/50 p-3 text-xs text-muted-foreground">
        <ShieldAlert className="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
        <span>{t('remediation:modelAccounts.airGapNotice')}</span>
      </div>

      {!creating && (
        <div className="flex justify-end">
          <Button onClick={() => setCreating(true)}>
            <Plus className="mr-1.5 h-4 w-4" aria-hidden="true" />
            {t('remediation:modelAccounts.create')}
          </Button>
        </div>
      )}

      {creating && <CreateForm onClose={() => setCreating(false)} />}

      {isLoading ? (
        <div className="flex h-32 items-center justify-center" role="status" aria-live="polite">
          <div className="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"></div>
        </div>
      ) : isError ? (
        <div className="py-8 text-center text-destructive">
          {t('remediation:modelAccounts.error')}
        </div>
      ) : !accounts || accounts.length === 0 ? (
        <div className="py-8 text-center text-muted-foreground">
          {t('remediation:modelAccounts.empty')}
        </div>
      ) : (
        <ul className="space-y-2">
          {accounts.map((account) => (
            <AccountRow key={account.id} account={account} />
          ))}
        </ul>
      )}
    </div>
  );
}

export default ModelAccountManager;
