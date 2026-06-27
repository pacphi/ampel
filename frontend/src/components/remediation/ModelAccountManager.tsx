import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, ShieldAlert, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
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
import type {
  CreateModelAccountRequest,
  ModelAccount,
  ModelValidationStatus,
  ProviderKind,
} from '@/types/modelAccount';

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

interface CreateFormProps {
  onClose: () => void;
}

function CreateForm({ onClose }: CreateFormProps) {
  const { t } = useTranslation(['remediation', 'common']);
  const createMutation = useCreateModelAccount();

  const [providerKind, setProviderKind] = useState<ProviderKind>('claude');
  const [displayName, setDisplayName] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [endpointUrl, setEndpointUrl] = useState('');
  const [modelId, setModelId] = useState('');
  const [spendCapUsd, setSpendCapUsd] = useState('');
  const [error, setError] = useState<string | null>(null);

  const local = isLocalProvider(providerKind);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!displayName.trim()) {
      setError(t('remediation:modelAccounts.errors.displayNameRequired'));
      return;
    }

    const payload: CreateModelAccountRequest = {
      providerKind,
      displayName: displayName.trim(),
    };
    if (!local && apiKey) payload.apiKey = apiKey;
    if (local && endpointUrl.trim()) payload.endpointUrl = endpointUrl.trim();
    if (modelId.trim()) payload.modelId = modelId.trim();
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
          <Select value={providerKind} onValueChange={(v) => setProviderKind(v as ProviderKind)}>
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

      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="model-model-id">{t('remediation:modelAccounts.modelId')}</Label>
          <Input
            id="model-model-id"
            value={modelId}
            onChange={(e) => setModelId(e.target.value)}
            placeholder={t('remediation:modelAccounts.modelIdPlaceholder')}
          />
        </div>
        <div className="space-y-2">
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

function AccountRow({ account }: { account: ModelAccount }) {
  const { t } = useTranslation(['remediation', 'common']);
  const validateMutation = useValidateModelAccount();
  const deleteMutation = useDeleteModelAccount();

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
          {account.isDefault && (
            <Badge variant="default">{t('remediation:modelAccounts.default')}</Badge>
          )}
        </div>
        <p className="text-xs text-muted-foreground">
          {account.modelId ?? '—'} · {t('remediation:modelAccounts.spend')}: {spend}
        </p>
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
