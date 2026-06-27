import { useId, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  useCreateRemediationPolicy,
  useUpdateRemediationPolicy,
} from '@/hooks/useRemediationPolicies';
import { hasFleetPreviewed } from '@/lib/fleetPreviewGate';
import type {
  AutonomyLevel,
  CreatePolicyRequest,
  RemediationPolicy,
  RemediationTier,
  ScopeType,
  UpdatePolicyRequest,
} from '@/types/remediation';

/** UI-level autonomy "stops" — the 4-stop selector. */
export type AutonomyStop = 'off' | 'dry_run' | 'consolidate' | 'auto_merge';

const STOPS: AutonomyStop[] = ['off', 'dry_run', 'consolidate', 'auto_merge'];

function deriveStop(policy?: RemediationPolicy): AutonomyStop {
  if (!policy || !policy.enabled) return policy ? 'off' : 'dry_run';
  if (policy.autoMergeEnabled || policy.autonomyLevel === 'fully_autonomous') return 'auto_merge';
  if (policy.autonomyLevel === 'auto_with_approval') return 'consolidate';
  return 'dry_run';
}

interface PolicyFields {
  enabled: boolean;
  autonomyLevel: AutonomyLevel;
  remediationTier: RemediationTier;
  autoMergeEnabled: boolean;
  requireHumanApproval: boolean;
}

function fieldsForStop(stop: AutonomyStop): PolicyFields {
  switch (stop) {
    case 'off':
      return {
        enabled: false,
        autonomyLevel: 'dry_run_only',
        remediationTier: 'consolidate_only',
        autoMergeEnabled: false,
        requireHumanApproval: false,
      };
    case 'dry_run':
      return {
        enabled: true,
        autonomyLevel: 'dry_run_only',
        remediationTier: 'consolidate_only',
        autoMergeEnabled: false,
        requireHumanApproval: false,
      };
    case 'consolidate':
      return {
        enabled: true,
        autonomyLevel: 'auto_with_approval',
        remediationTier: 'consolidate_only',
        autoMergeEnabled: false,
        requireHumanApproval: true,
      };
    case 'auto_merge':
      return {
        enabled: true,
        autonomyLevel: 'fully_autonomous',
        remediationTier: 'consolidate_only',
        autoMergeEnabled: true,
        requireHumanApproval: false,
      };
  }
}

interface PolicyEditorProps {
  policy?: RemediationPolicy;
  /** Optional preset scope id for new policies (e.g. selected repository). */
  defaultScopeId?: string;
  /**
   * Whether a fleet dry-run preview has been run. Gates the FIRST move to
   * Auto-merge (Phase 4 safeguard). Defaults to the persisted gate state; tests
   * pass it explicitly.
   */
  fleetPreviewed?: boolean;
  onSaved?: (policy: RemediationPolicy) => void;
  onCancel?: () => void;
}

export function PolicyEditor({
  policy,
  defaultScopeId,
  fleetPreviewed,
  onSaved,
  onCancel,
}: PolicyEditorProps) {
  const { t } = useTranslation(['remediation', 'common']);
  const isEdit = !!policy;

  const createMutation = useCreateRemediationPolicy();
  const updateMutation = useUpdateRemediationPolicy();

  const [scopeType, setScopeType] = useState<ScopeType>(policy?.scopeType ?? 'repository');
  const [scopeId, setScopeId] = useState<string>(policy?.scopeId ?? defaultScopeId ?? '');
  const [enabled, setEnabled] = useState<boolean>(policy?.enabled ?? true);
  const [stop, setStop] = useState<AutonomyStop>(deriveStop(policy));
  const [minOpenPrs, setMinOpenPrs] = useState<number>(policy?.minOpenPrs ?? 2);
  const [maxPrsPerRun, setMaxPrsPerRun] = useState<number>(policy?.maxPrsPerRun ?? 10);

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const groupLabelId = useId();

  const isSaving = createMutation.isPending || updateMutation.isPending;

  // Auto-merge-first-time gate (Phase 4): the first time a policy is moved to
  // Auto-merge, the operator must have run a fleet preview. A policy already in
  // Auto-merge is exempt (it has been through the gate before).
  const previewSatisfied = fleetPreviewed ?? hasFleetPreviewed();
  const alreadyAutoMerge = policy?.autoMergeEnabled === true;
  const autoMergeBlocked = !alreadyAutoMerge && !previewSatisfied;

  const selectStop = (next: AutonomyStop) => {
    setError(null);
    if (next === 'auto_merge') {
      // Reaching Auto-merge requires an explicit confirmation.
      setConfirmOpen(true);
      return;
    }
    setStop(next);
    if (next === 'off') {
      setEnabled(false);
    } else {
      setEnabled(true);
    }
  };

  const confirmAutoMerge = () => {
    // Gate: refuse to apply Auto-merge the first time without a fleet preview.
    if (autoMergeBlocked) {
      return;
    }
    setStop('auto_merge');
    setEnabled(true);
    setConfirmOpen(false);
  };

  const handleSave = () => {
    setError(null);
    if (!scopeId.trim()) {
      setError(t('remediation:editor.errors.scopeIdRequired'));
      return;
    }

    const fields = fieldsForStop(stop);
    // The standalone enable toggle can force-disable an otherwise-active stop.
    const effectiveEnabled = stop === 'off' ? false : enabled;

    if (isEdit && policy) {
      const payload: UpdatePolicyRequest = {
        enabled: effectiveEnabled,
        autonomyLevel: fields.autonomyLevel,
        remediationTier: fields.remediationTier,
        autoMergeEnabled: fields.autoMergeEnabled,
        requireHumanApproval: fields.requireHumanApproval,
        minOpenPrs,
        maxPrsPerRun,
      };
      updateMutation.mutate(
        { id: policy.id, data: payload },
        {
          onSuccess: (saved) => onSaved?.(saved),
          onError: () => setError(t('remediation:editor.errors.saveFailed')),
        }
      );
      return;
    }

    const payload: CreatePolicyRequest = {
      scopeType,
      scopeId: scopeId.trim(),
      enabled: effectiveEnabled,
      autonomyLevel: fields.autonomyLevel,
      remediationTier: fields.remediationTier,
      autoMergeEnabled: fields.autoMergeEnabled,
      requireHumanApproval: fields.requireHumanApproval,
      minOpenPrs,
      maxPrsPerRun,
    };
    createMutation.mutate(payload, {
      onSuccess: (saved) => onSaved?.(saved),
      onError: () => setError(t('remediation:editor.errors.saveFailed')),
    });
  };

  return (
    <div className="space-y-6">
      {/* Scope */}
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="policy-scope-type">{t('remediation:editor.scopeType')}</Label>
          <Select
            value={scopeType}
            onValueChange={(v) => setScopeType(v as ScopeType)}
            disabled={isEdit}
          >
            <SelectTrigger id="policy-scope-type">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="repository">{t('remediation:scopeType.repository')}</SelectItem>
              <SelectItem value="team">{t('remediation:scopeType.team')}</SelectItem>
              <SelectItem value="org">{t('remediation:scopeType.org')}</SelectItem>
              <SelectItem value="user">{t('remediation:scopeType.user')}</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-2">
          <Label htmlFor="policy-scope-id">{t('remediation:editor.scopeId')}</Label>
          <Input
            id="policy-scope-id"
            value={scopeId}
            onChange={(e) => setScopeId(e.target.value)}
            placeholder={t('remediation:editor.scopeIdPlaceholder')}
            disabled={isEdit}
          />
        </div>
      </div>

      {/* Enable toggle */}
      <div className="flex items-center gap-3">
        <Switch
          id="policy-enabled"
          checked={enabled && stop !== 'off'}
          onCheckedChange={(checked) => {
            setEnabled(checked);
            if (!checked) {
              setStop('off');
            } else if (stop === 'off') {
              setStop('dry_run');
            }
          }}
        />
        <Label htmlFor="policy-enabled">{t('remediation:editor.enabled')}</Label>
      </div>

      {/* 4-stop autonomy selector */}
      <fieldset className="space-y-2">
        <legend id={groupLabelId} className="text-sm font-medium">
          {t('remediation:editor.autonomy')}
        </legend>
        <div
          role="radiogroup"
          aria-labelledby={groupLabelId}
          className="grid grid-cols-2 gap-2 sm:grid-cols-4"
        >
          {STOPS.map((s) => {
            const checked = stop === s;
            return (
              <label
                key={s}
                className={`flex cursor-pointer items-center justify-center rounded-md border px-3 py-2 text-sm font-medium transition-colors ${
                  checked ? 'border-primary bg-primary text-primary-foreground' : 'hover:bg-accent'
                }`}
              >
                <input
                  type="radio"
                  name="autonomy-stop"
                  value={s}
                  checked={checked}
                  onChange={() => selectStop(s)}
                  className="sr-only"
                />
                {t(`remediation:autonomyStop.${s}`)}
              </label>
            );
          })}
        </div>
        <p className="text-xs text-muted-foreground">{t(`remediation:autonomyHint.${stop}`)}</p>
      </fieldset>

      {/* Thresholds */}
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="policy-min-open-prs">{t('remediation:editor.minOpenPrs')}</Label>
          <Input
            id="policy-min-open-prs"
            type="number"
            min={1}
            value={minOpenPrs}
            onChange={(e) => setMinOpenPrs(Number(e.target.value))}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="policy-max-prs-per-run">{t('remediation:editor.maxPrsPerRun')}</Label>
          <Input
            id="policy-max-prs-per-run"
            type="number"
            min={1}
            value={maxPrsPerRun}
            onChange={(e) => setMaxPrsPerRun(Number(e.target.value))}
          />
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <div className="flex justify-end gap-2">
        {onCancel && (
          <Button variant="outline" onClick={onCancel} disabled={isSaving}>
            {t('common:actions.cancel')}
          </Button>
        )}
        <Button onClick={handleSave} disabled={isSaving}>
          {isSaving ? t('remediation:editor.saving') : t('remediation:editor.save')}
        </Button>
      </div>

      {/* Auto-merge confirmation */}
      <Dialog open={confirmOpen} onOpenChange={setConfirmOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('remediation:editor.confirmAutoMerge.title')}</DialogTitle>
            <DialogDescription>
              {t('remediation:editor.confirmAutoMerge.description')}
            </DialogDescription>
          </DialogHeader>
          {autoMergeBlocked && (
            <div
              role="alert"
              className="rounded-md bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-400"
            >
              {t('remediation:editor.confirmAutoMerge.previewRequired')}
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmOpen(false)}>
              {t('common:actions.cancel')}
            </Button>
            <Button variant="destructive" onClick={confirmAutoMerge} disabled={autoMergeBlocked}>
              {t('remediation:editor.confirmAutoMerge.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default PolicyEditor;
