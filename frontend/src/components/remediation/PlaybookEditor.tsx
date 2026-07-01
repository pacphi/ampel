import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Copy, FileCode, FileDown, Plus, Trash2 } from 'lucide-react';
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
  useLoadEmbeddedPlaybook,
  usePlaybooks,
  usePreviewPlaybook,
} from '@/hooks/usePlaybooks';
import type { CreatePlaybookRequest, Playbook } from '@/types/playbook';
import type { PlaybookPreviewResponse } from '@/types/playbook';
import type { ScopeType } from '@/types/remediation';

const SCOPE_TYPES: ScopeType[] = ['user', 'team', 'org', 'repository'];

/** Initial values a form is opened with (blank create, loaded default, or duplicate). */
interface PlaybookFormSeed {
  playbookId: string;
  name: string;
  content: string;
  scopeType: ScopeType;
}

const BLANK_SEED: PlaybookFormSeed = {
  playbookId: '',
  name: '',
  content: '',
  scopeType: 'user',
};

/** Coerce an arbitrary stored `scopeType` string to a known form value. */
function toScopeType(value: string): ScopeType {
  return (SCOPE_TYPES as string[]).includes(value) ? (value as ScopeType) : 'user';
}

interface PlaybookFormProps {
  initial: PlaybookFormSeed;
  onClose: () => void;
}

function PlaybookForm({ initial, onClose }: PlaybookFormProps) {
  const { t } = useTranslation(['remediation', 'common']);
  const createMutation = useCreatePlaybook();

  const [playbookId, setPlaybookId] = useState(initial.playbookId);
  const [name, setName] = useState(initial.name);
  const [scopeType, setScopeType] = useState<ScopeType>(initial.scopeType);
  const [scopeId, setScopeId] = useState('');
  const [content, setContent] = useState(initial.content);
  const [error, setError] = useState<string | null>(null);

  // Validate/Preview renders a SAVED playbook (the API operates on a stored row),
  // so it lives on each list row below. The create form surfaces server-side YAML
  // parse errors (422, now with a field path) inline on save.
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

const PILL = 'rounded bg-muted px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground';

/** Compact "what this playbook grants" pills, mirroring the model-catalog pills. */
function PreviewMeta({ preview }: { preview: PlaybookPreviewResponse }) {
  const { t } = useTranslation(['remediation']);
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      <span className={PILL}>
        {t('remediation:playbooks.meta.role')}: {preview.role}
      </span>
      <span className={PILL}>
        {t('remediation:playbooks.meta.outputContract')}: {preview.outputContract}
      </span>
      <span className="text-[10px] font-medium text-muted-foreground">
        {t('remediation:playbooks.meta.allowedTools')}:
      </span>
      {preview.allowedTools.length === 0 ? (
        <span className={PILL}>{t('remediation:playbooks.meta.noTools')}</span>
      ) : (
        preview.allowedTools.map((tool) => (
          <span key={tool} className={PILL}>
            {tool}
          </span>
        ))
      )}
    </div>
  );
}

interface PlaybookRowProps {
  playbook: Playbook;
  onDuplicate: (playbook: Playbook) => void;
}

/** Per-row preview control: validates a SAVED playbook and shows the prompt. */
function PlaybookRow({ playbook, onDuplicate }: PlaybookRowProps) {
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
        <Button variant="outline" size="sm" onClick={() => onDuplicate(playbook)}>
          <Copy className="mr-1.5 h-4 w-4" aria-hidden="true" />
          {t('remediation:playbooks.duplicate')}
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
        <div className="space-y-2 rounded-md border bg-muted/30 p-3">
          <PreviewMeta preview={preview} />
          <div className="space-y-1">
            <h4 className="text-xs font-medium">{t('remediation:playbooks.renderedPrompt')}</h4>
            <pre className="max-h-64 overflow-auto whitespace-pre-wrap text-xs">
              {preview.systemInstruction}
            </pre>
          </div>
        </div>
      )}
    </li>
  );
}

export function PlaybookEditor() {
  const { t } = useTranslation(['remediation', 'common']);
  const { data: playbooks, isLoading, isError } = usePlaybooks();
  const loadDefaultMutation = useLoadEmbeddedPlaybook();
  // `null` = form closed. `nonce` forces a fresh form mount so a new seed
  // re-initializes the fields even if a form is already open.
  const [formSeed, setFormSeed] = useState<PlaybookFormSeed | null>(null);
  const [formNonce, setFormNonce] = useState(0);
  const [loadError, setLoadError] = useState<string | null>(null);

  const openForm = (seed: PlaybookFormSeed) => {
    setLoadError(null);
    setFormSeed(seed);
    setFormNonce((n) => n + 1);
  };

  const handleLoadDefault = () => {
    setLoadError(null);
    loadDefaultMutation.mutate(undefined, {
      onSuccess: (embedded) =>
        // Sanitized copy: fresh id/name so it never collides with the built-in,
        // the default YAML as the editable starting point.
        openForm({ playbookId: '', name: '', content: embedded.content, scopeType: 'user' }),
      onError: () => setLoadError(t('remediation:playbooks.errors.loadDefaultFailed')),
    });
  };

  const handleDuplicate = (playbook: Playbook) => {
    openForm({
      playbookId: `${playbook.playbookId}-copy`,
      name: playbook.name,
      content: playbook.content,
      scopeType: toScopeType(playbook.scopeType),
    });
  };

  return (
    <div className="space-y-4">
      {!formSeed && (
        <div className="flex flex-wrap justify-end gap-2">
          <Button
            variant="outline"
            onClick={handleLoadDefault}
            disabled={loadDefaultMutation.isPending}
          >
            <FileDown className="mr-1.5 h-4 w-4" aria-hidden="true" />
            {loadDefaultMutation.isPending
              ? t('remediation:playbooks.loadingDefault')
              : t('remediation:playbooks.loadDefault')}
          </Button>
          <Button onClick={() => openForm(BLANK_SEED)}>
            <Plus className="mr-1.5 h-4 w-4" aria-hidden="true" />
            {t('remediation:playbooks.create')}
          </Button>
        </div>
      )}

      {loadError && (
        <p role="alert" className="text-sm text-destructive">
          {loadError}
        </p>
      )}

      {formSeed && (
        <PlaybookForm key={formNonce} initial={formSeed} onClose={() => setFormSeed(null)} />
      )}

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
            <PlaybookRow key={playbook.id} playbook={playbook} onDuplicate={handleDuplicate} />
          ))}
        </ul>
      )}
    </div>
  );
}

export default PlaybookEditor;
