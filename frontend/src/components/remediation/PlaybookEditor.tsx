import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { FileCode, Plus, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  useCreatePlaybook,
  useDeletePlaybook,
  usePlaybooks,
  usePreviewPlaybook,
} from '@/hooks/usePlaybooks';
import type { CreatePlaybookRequest, Playbook } from '@/types/playbook';
import type { PlaybookPreviewResponse } from '@/types/playbook';
import type { ScopeType } from '@/types/remediation';

const SCOPE_TYPES: ScopeType[] = ['user', 'team', 'org', 'repository'];

interface PlaybookFormProps {
  onClose: () => void;
}

function PlaybookForm({ onClose }: PlaybookFormProps) {
  const { t } = useTranslation(['remediation', 'common']);
  const createMutation = useCreatePlaybook();

  const [playbookId, setPlaybookId] = useState('');
  const [name, setName] = useState('');
  const [scopeType, setScopeType] = useState<ScopeType>('user');
  const [scopeId, setScopeId] = useState('');
  const [content, setContent] = useState('');
  const [error, setError] = useState<string | null>(null);

  // Validate/Preview renders a SAVED playbook (the API operates on a stored row),
  // so it lives on each list row below. The create form surfaces server-side YAML
  // parse errors (422) inline on save.
  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!playbookId.trim()) {
      setError(t('remediation:playbooks.errors.playbookIdRequired'));
      return;
    }
    if (!name.trim()) {
      setError(t('remediation:playbooks.errors.nameRequired'));
      return;
    }
    if (!content.trim()) {
      setError(t('remediation:playbooks.errors.contentRequired'));
      return;
    }

    const payload: CreatePlaybookRequest = {
      playbookId: playbookId.trim(),
      name: name.trim(),
      content,
      scopeType,
    };
    if (scopeType !== 'user' && scopeId.trim()) payload.scopeId = scopeId.trim();

    createMutation.mutate(payload, {
      onSuccess: () => onClose(),
      onError: (err: unknown) => {
        const axiosError = err as { response?: { data?: { error?: string } } };
        setError(axiosError.response?.data?.error ?? t('remediation:playbooks.errors.saveFailed'));
      },
    });
  };

  return (
    <form onSubmit={handleSave} className="space-y-4 rounded-lg border p-4">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="playbook-id">{t('remediation:playbooks.playbookId')}</Label>
          <Input
            id="playbook-id"
            value={playbookId}
            onChange={(e) => setPlaybookId(e.target.value)}
            placeholder={t('remediation:playbooks.playbookIdPlaceholder')}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="playbook-name">{t('remediation:playbooks.name')}</Label>
          <Input
            id="playbook-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={t('remediation:playbooks.namePlaceholder')}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="playbook-scope-type">{t('remediation:playbooks.scopeType')}</Label>
          <Select value={scopeType} onValueChange={(v) => setScopeType(v as ScopeType)}>
            <SelectTrigger id="playbook-scope-type">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {SCOPE_TYPES.map((s) => (
                <SelectItem key={s} value={s}>
                  {t(`remediation:scopeType.${s}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        {scopeType !== 'user' && (
          <div className="space-y-2">
            <Label htmlFor="playbook-scope-id">{t('remediation:playbooks.scopeId')}</Label>
            <Input
              id="playbook-scope-id"
              value={scopeId}
              onChange={(e) => setScopeId(e.target.value)}
              placeholder={t('remediation:playbooks.scopeIdPlaceholder')}
            />
          </div>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="playbook-content">{t('remediation:playbooks.content')}</Label>
        <Textarea
          id="playbook-content"
          value={content}
          onChange={(e) => setContent(e.target.value)}
          rows={12}
          spellCheck={false}
          className="font-mono text-xs"
          placeholder={t('remediation:playbooks.contentPlaceholder')}
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
            ? t('remediation:playbooks.saving')
            : t('remediation:playbooks.save')}
        </Button>
      </div>
    </form>
  );
}

/** Per-row preview control: validates a SAVED playbook and shows the prompt. */
function PlaybookRow({ playbook }: { playbook: Playbook }) {
  const { t } = useTranslation(['remediation', 'common']);
  const previewMutation = usePreviewPlaybook();
  const deleteMutation = useDeletePlaybook();
  const [preview, setPreview] = useState<PlaybookPreviewResponse | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);

  const handlePreview = () => {
    setPreview(null);
    setPreviewError(null);
    previewMutation.mutate(
      { id: playbook.id, data: {} },
      {
        onSuccess: (data) => setPreview(data),
        onError: (err: unknown) => {
          const axiosError = err as { response?: { data?: { error?: string } } };
          setPreviewError(
            axiosError.response?.data?.error ?? t('remediation:playbooks.errors.previewFailed')
          );
        },
      }
    );
  };

  return (
    <li className="space-y-2 rounded-lg border p-3">
      <div className="flex flex-wrap items-center gap-3">
        <div className="min-w-0 flex-1">
          <p className="font-medium">{playbook.name}</p>
          <p className="text-xs text-muted-foreground">
            {playbook.playbookId} · {t(`remediation:scopeType.${playbook.scopeType}`)}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handlePreview}
          disabled={previewMutation.isPending}
        >
          {previewMutation.isPending
            ? t('remediation:playbooks.previewing')
            : t('remediation:playbooks.preview')}
        </Button>
        <Button
          variant="destructive"
          size="sm"
          onClick={() => deleteMutation.mutate(playbook.id)}
          disabled={deleteMutation.isPending}
          aria-label={t('common:actions.delete')}
        >
          <Trash2 className="h-4 w-4" aria-hidden="true" />
        </Button>
      </div>

      {previewError && (
        <p role="alert" className="text-sm text-destructive">
          {previewError}
        </p>
      )}

      {preview && (
        <div className="space-y-1 rounded-md border bg-muted/30 p-3">
          <h4 className="text-xs font-medium">{t('remediation:playbooks.renderedPrompt')}</h4>
          <pre className="max-h-64 overflow-auto whitespace-pre-wrap text-xs">
            {preview.systemInstruction}
          </pre>
        </div>
      )}
    </li>
  );
}

export function PlaybookEditor() {
  const { t } = useTranslation(['remediation', 'common']);
  const { data: playbooks, isLoading, isError } = usePlaybooks();
  const [creating, setCreating] = useState(false);

  return (
    <div className="space-y-4">
      {!creating && (
        <div className="flex justify-end">
          <Button onClick={() => setCreating(true)}>
            <Plus className="mr-1.5 h-4 w-4" aria-hidden="true" />
            {t('remediation:playbooks.create')}
          </Button>
        </div>
      )}

      {creating && <PlaybookForm onClose={() => setCreating(false)} />}

      {isLoading ? (
        <div className="flex h-32 items-center justify-center" role="status" aria-live="polite">
          <div className="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"></div>
        </div>
      ) : isError ? (
        <div className="py-8 text-center text-destructive">{t('remediation:playbooks.error')}</div>
      ) : !playbooks || playbooks.length === 0 ? (
        <div className="py-8 text-center text-muted-foreground">
          <FileCode className="mx-auto mb-2 h-8 w-8 opacity-50" aria-hidden="true" />
          {t('remediation:playbooks.empty')}
        </div>
      ) : (
        <ul className="space-y-2">
          {playbooks.map((playbook) => (
            <PlaybookRow key={playbook.id} playbook={playbook} />
          ))}
        </ul>
      )}
    </div>
  );
}

export default PlaybookEditor;
