import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Eye, ShieldOff } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useFleetRemediation, usePreviewRepository } from '@/hooks/useFleetRemediation';
import { markFleetPreviewed } from '@/lib/fleetPreviewGate';
import type { ConsolidationPlan, FleetRow, PolicyState } from '@/types/remediation';

function policyStateVariant(
  state: PolicyState
): 'default' | 'secondary' | 'outline' | 'success' | 'warning' {
  switch (state) {
    case 'auto_merge':
      return 'success';
    case 'auto_with_approval':
    case 'suggest':
      return 'default';
    case 'dry_run':
      return 'warning';
    case 'disabled':
    case 'none':
    default:
      return 'secondary';
  }
}

export function FleetOverview() {
  const { t } = useTranslation(['remediation', 'common']);
  const { data: rows, isLoading, isError } = useFleetRemediation();
  const previewMutation = usePreviewRepository();
  const [activeRepo, setActiveRepo] = useState<FleetRow | null>(null);
  const [plan, setPlan] = useState<ConsolidationPlan | null>(null);
  // Default to the actionable rows: repos with open PRs that are remediation-eligible.
  // Either filter can be toggled off independently to widen the view.
  const [onlyWithPrs, setOnlyWithPrs] = useState(true);
  const [onlyEligible, setOnlyEligible] = useState(true);

  const handlePreview = (row: FleetRow) => {
    setActiveRepo(row);
    setPlan(null);
    previewMutation.mutate(row.repositoryId, {
      onSuccess: (data) => {
        setPlan(data);
        // Unlocks the auto-merge-first-time gate in the PolicyEditor.
        markFleetPreviewed();
      },
    });
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-48" role="status" aria-live="polite">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (isError) {
    return <div className="text-center py-8 text-destructive">{t('remediation:fleet.error')}</div>;
  }

  const fleet = rows ?? [];

  if (fleet.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">{t('remediation:fleet.empty')}</div>
    );
  }

  // Active filters AND together; toggling either off relaxes it. Purely client-side over
  // the already-fetched rows — no backend/API involvement.
  const filteredFleet = fleet.filter(
    (row) =>
      (!onlyWithPrs || row.openPrCount > 0) && (!onlyEligible || row.eligible)
  );

  const clearFilters = () => {
    setOnlyWithPrs(false);
    setOnlyEligible(false);
  };

  const filters = (
    <div className="flex flex-wrap items-center gap-x-6 gap-y-2 pb-4">
      <div className="flex items-center gap-2">
        <Switch id="fleet-filter-open-prs" checked={onlyWithPrs} onCheckedChange={setOnlyWithPrs} />
        <Label htmlFor="fleet-filter-open-prs" className="cursor-pointer text-sm font-normal">
          {t('remediation:fleet.filters.onlyWithPrs')}
        </Label>
      </div>
      <div className="flex items-center gap-2">
        <Switch id="fleet-filter-eligible" checked={onlyEligible} onCheckedChange={setOnlyEligible} />
        <Label htmlFor="fleet-filter-eligible" className="cursor-pointer text-sm font-normal">
          {t('remediation:fleet.filters.onlyEligible')}
        </Label>
      </div>
    </div>
  );

  if (filteredFleet.length === 0) {
    return (
      <>
        {filters}
        <div className="text-center py-8 text-muted-foreground">
          <p>{t('remediation:fleet.filters.noMatch')}</p>
          <Button variant="outline" size="sm" className="mt-3" onClick={clearFilters}>
            {t('remediation:fleet.filters.clear')}
          </Button>
        </div>
      </>
    );
  }

  return (
    <>
      {filters}
      <div className="overflow-x-auto">
        <table className="w-full text-sm" aria-label={t('remediation:fleet.tableLabel')}>
          <thead>
            <tr className="border-b text-left text-muted-foreground">
              <th scope="col" className="py-2 pr-4 font-medium">
                {t('remediation:fleet.columns.repository')}
              </th>
              <th scope="col" className="py-2 pr-4 font-medium">
                {t('remediation:fleet.columns.openPrs')}
              </th>
              <th scope="col" className="py-2 pr-4 font-medium">
                {t('remediation:fleet.columns.eligibility')}
              </th>
              <th scope="col" className="py-2 pr-4 font-medium">
                {t('remediation:fleet.columns.policyState')}
              </th>
              <th scope="col" className="py-2 pr-4 font-medium">
                {t('remediation:fleet.columns.airGapped')}
              </th>
              <th scope="col" className="py-2 font-medium text-right">
                {t('remediation:fleet.columns.actions')}
              </th>
            </tr>
          </thead>
          <tbody>
            {filteredFleet.map((row) => (
              <tr key={row.repositoryId} className="border-b last:border-0">
                <td className="py-2 pr-4 font-medium">{row.name}</td>
                <td className="py-2 pr-4">{row.openPrCount}</td>
                <td className="py-2 pr-4">
                  <Badge variant={row.eligible ? 'success' : 'secondary'}>
                    {row.eligible
                      ? t('remediation:fleet.eligible')
                      : t('remediation:fleet.notEligible')}
                  </Badge>
                </td>
                <td className="py-2 pr-4">
                  <Badge variant={policyStateVariant(row.policyState)}>
                    {t(`remediation:policyState.${row.policyState}`)}
                  </Badge>
                </td>
                <td className="py-2 pr-4">
                  {row.airGapped ? (
                    <span
                      className="inline-flex items-center gap-1 text-xs text-amber-600 dark:text-amber-400"
                      title={t('remediation:fleet.airGappedOn')}
                    >
                      <ShieldOff className="h-3.5 w-3.5" aria-hidden="true" />
                      {t('remediation:fleet.airGappedOn')}
                    </span>
                  ) : (
                    <span className="text-xs text-muted-foreground">
                      {t('remediation:fleet.airGappedOff')}
                    </span>
                  )}
                </td>
                <td className="py-2 text-right">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handlePreview(row)}
                    disabled={
                      previewMutation.isPending && activeRepo?.repositoryId === row.repositoryId
                    }
                  >
                    <Eye className="h-4 w-4 mr-1.5" aria-hidden="true" />
                    {t('remediation:fleet.preview')}
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <Dialog
        open={!!activeRepo}
        onOpenChange={(open) => {
          if (!open) {
            setActiveRepo(null);
            setPlan(null);
          }
        }}
      >
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle className="truncate pr-8">
              {t('remediation:preview.title', { repo: activeRepo?.name })}
            </DialogTitle>
            <DialogDescription>{t('remediation:preview.description')}</DialogDescription>
          </DialogHeader>

          {previewMutation.isPending && (
            <div className="py-6 text-center text-muted-foreground">
              {t('remediation:preview.loading')}
            </div>
          )}

          {previewMutation.isError && (
            <div className="py-6 text-center text-destructive">
              {t('remediation:preview.error')}
            </div>
          )}

          {plan && (
            <div className="max-h-[70vh] space-y-4 overflow-y-auto pr-1">
              {plan.blocked_by_air_gap && (
                <div className="rounded-md bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-400">
                  {t('remediation:preview.blockedByAirGap')}
                </div>
              )}
              <dl className="grid grid-cols-2 gap-2 text-sm">
                <dt className="text-muted-foreground">{t('remediation:preview.prCount')}</dt>
                <dd className="text-right font-medium">{plan.pr_count}</dd>
                <dt className="text-muted-foreground">
                  {t('remediation:preview.estimatedDuration')}
                </dt>
                <dd className="text-right font-medium">
                  {t('remediation:preview.seconds', { count: plan.estimated_duration_secs })}
                </dd>
                <dt className="text-muted-foreground">{t('remediation:preview.conflicts')}</dt>
                <dd className="text-right font-medium">{plan.predicted_conflicts.length}</dd>
              </dl>

              {plan.would_select.length > 0 ? (
                <div>
                  <h4 className="mb-2 text-sm font-medium">
                    {t('remediation:preview.wouldSelect')}
                  </h4>
                  <ul className="space-y-1">
                    {plan.would_select.map((pr) => (
                      <li
                        key={pr.number}
                        className="flex items-center gap-2 overflow-hidden rounded-md border px-3 py-1.5 text-sm"
                      >
                        <span className="shrink-0 text-muted-foreground">#{pr.number}</span>
                        <span className="min-w-0 flex-1 truncate">{pr.title}</span>
                        <span className="min-w-0 max-w-[45%] shrink truncate text-xs text-muted-foreground">
                          {pr.branch}
                        </span>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">
                  {t('remediation:preview.noneSelected')}
                </p>
              )}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}

export default FleetOverview;
