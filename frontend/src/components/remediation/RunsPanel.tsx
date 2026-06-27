import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ArrowLeft, Check, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { RunTimeline } from './RunTimeline';
import { CiCheckMatrix } from './CiCheckMatrix';
import { ConflictReport } from './ConflictReport';
import { AgentSessionViewer } from './AgentSessionViewer';
import {
  useApproveRun,
  useCancelRun,
  useRemediationRun,
  useRemediationRuns,
} from '@/hooks/useRemediationRuns';
import type { RemediationRun, RunState } from '@/types/remediation';

function stateVariant(
  state: RunState
): 'default' | 'secondary' | 'success' | 'warning' | 'destructive' {
  switch (state) {
    case 'completed':
      return 'success';
    case 'awaiting_approval':
      return 'warning';
    case 'failed':
      return 'destructive';
    case 'cancelled':
    case 'handoff_human':
    case 'no_op':
      return 'secondary';
    default:
      return 'default';
  }
}

function RunDetailView({ runId, onBack }: { runId: string; onBack: () => void }) {
  const { t } = useTranslation(['remediation', 'common']);
  const { data: run, isLoading, isError } = useRemediationRun(runId);
  const approveMutation = useApproveRun();
  const cancelMutation = useCancelRun();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-48" role="status" aria-live="polite">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (isError || !run) {
    return <div className="text-center py-8 text-destructive">{t('remediation:runs.error')}</div>;
  }

  const awaitingApproval = run.state === 'awaiting_approval';

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Button variant="ghost" size="sm" onClick={onBack}>
          <ArrowLeft className="h-4 w-4 mr-1.5" aria-hidden="true" />
          {t('remediation:runs.back')}
        </Button>
        {awaitingApproval && (
          <div className="flex gap-2">
            <Button
              size="sm"
              onClick={() => approveMutation.mutate(run.id)}
              disabled={approveMutation.isPending}
            >
              <Check className="h-4 w-4 mr-1.5" aria-hidden="true" />
              {t('remediation:runs.approve')}
            </Button>
            <Button
              size="sm"
              variant="destructive"
              onClick={() => cancelMutation.mutate(run.id)}
              disabled={cancelMutation.isPending}
            >
              <X className="h-4 w-4 mr-1.5" aria-hidden="true" />
              {t('remediation:runs.reject')}
            </Button>
          </div>
        )}
      </div>

      {run.errorMessage && (
        <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
          {run.errorMessage}
        </div>
      )}

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t('remediation:runs.timeline')}</CardTitle>
          </CardHeader>
          <CardContent>
            <RunTimeline run={run} />
          </CardContent>
        </Card>

        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">{t('remediation:ci.title')}</CardTitle>
            </CardHeader>
            <CardContent>
              <CiCheckMatrix ciMatrix={run.ciMatrix} />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">{t('remediation:conflicts.title')}</CardTitle>
            </CardHeader>
            <CardContent>
              <ConflictReport report={run.conflictReport} />
            </CardContent>
          </Card>

          {run.agentSession && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">{t('remediation:agentSession.title')}</CardTitle>
              </CardHeader>
              <CardContent>
                <AgentSessionViewer session={run.agentSession} />
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}

function RunList({ onSelect }: { onSelect: (run: RemediationRun) => void }) {
  const { t } = useTranslation(['remediation', 'common']);
  const { data: runs, isLoading, isError } = useRemediationRuns();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-48" role="status" aria-live="polite">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (isError) {
    return <div className="text-center py-8 text-destructive">{t('remediation:runs.error')}</div>;
  }

  const list = runs ?? [];

  if (list.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">{t('remediation:runs.empty')}</div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm" aria-label={t('remediation:runs.tableLabel')}>
        <thead>
          <tr className="border-b text-left text-muted-foreground">
            <th scope="col" className="py-2 pr-4 font-medium">
              {t('remediation:runs.columns.repository')}
            </th>
            <th scope="col" className="py-2 pr-4 font-medium">
              {t('remediation:runs.columns.state')}
            </th>
            <th scope="col" className="py-2 pr-4 font-medium">
              {t('remediation:runs.columns.pr')}
            </th>
            <th scope="col" className="py-2 pr-4 font-medium">
              {t('remediation:runs.columns.started')}
            </th>
            <th scope="col" className="py-2 font-medium text-right">
              {t('remediation:runs.columns.actions')}
            </th>
          </tr>
        </thead>
        <tbody>
          {list.map((run) => (
            <tr key={run.id} className="border-b last:border-0">
              <td className="py-2 pr-4 font-mono text-xs">{run.repositoryId}</td>
              <td className="py-2 pr-4">
                <Badge variant={stateVariant(run.state)}>
                  {t(`remediation:runState.${run.state}`)}
                </Badge>
              </td>
              <td className="py-2 pr-4">
                {run.consolidatedPrNumber != null ? `#${run.consolidatedPrNumber}` : '—'}
              </td>
              <td className="py-2 pr-4 text-xs text-muted-foreground">
                {run.startedAt ? new Date(run.startedAt).toLocaleString() : '—'}
              </td>
              <td className="py-2 text-right">
                <Button variant="outline" size="sm" onClick={() => onSelect(run)}>
                  {t('remediation:runs.view')}
                </Button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function RunsPanel() {
  const [selectedId, setSelectedId] = useState<string | null>(null);

  if (selectedId) {
    return <RunDetailView runId={selectedId} onBack={() => setSelectedId(null)} />;
  }
  return <RunList onSelect={(run) => setSelectedId(run.id)} />;
}

export default RunsPanel;
