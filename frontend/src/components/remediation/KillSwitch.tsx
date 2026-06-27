import { useTranslation } from 'react-i18next';
import { ShieldAlert } from 'lucide-react';
import { Switch } from '@/components/ui/switch';
import { cn } from '@/lib/utils';
import { useRemediationPolicies, useToggleRemediationPolicy } from '@/hooks/useRemediationPolicies';
import { selectTopScopePolicy } from './killSwitchScope';

/**
 * Persistent "Pause all remediation" kill-switch. Toggles the top-scope
 * policy's `enabled` flag via the existing policy toggle endpoint. When the
 * policy is disabled, remediation is paused (switch ON = paused).
 */
export function KillSwitch() {
  const { t } = useTranslation(['remediation']);
  const { data: policies, isLoading } = useRemediationPolicies();
  const toggleMutation = useToggleRemediationPolicy();

  const topPolicy = selectTopScopePolicy(policies);
  const paused = topPolicy ? !topPolicy.enabled : false;
  const disabled = isLoading || !topPolicy || toggleMutation.isPending;

  return (
    <div
      className={cn(
        'flex items-center gap-2 rounded-md border px-3 py-2',
        paused && 'border-red-500/40 bg-red-500/5'
      )}
    >
      <ShieldAlert
        className={cn('h-4 w-4', paused ? 'text-red-500' : 'text-muted-foreground')}
        aria-hidden="true"
      />
      <label htmlFor="remediation-kill-switch" className="text-sm font-medium">
        {t('remediation:killSwitch.label')}
      </label>
      <Switch
        id="remediation-kill-switch"
        aria-label={t('remediation:killSwitch.label')}
        checked={paused}
        disabled={disabled}
        onCheckedChange={() => {
          if (topPolicy) toggleMutation.mutate(topPolicy.id);
        }}
      />
      <span className="text-xs text-muted-foreground" aria-live="polite">
        {topPolicy
          ? paused
            ? t('remediation:killSwitch.paused')
            : t('remediation:killSwitch.active')
          : t('remediation:killSwitch.noPolicy')}
      </span>
    </div>
  );
}

export default KillSwitch;
