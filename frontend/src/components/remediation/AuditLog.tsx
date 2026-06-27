import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Download } from 'lucide-react';
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
import { useRemediationRuns } from '@/hooks/useRemediationRuns';
import { auditAction, buildAuditCsv, isAuditEntry } from './auditCsv';
import type { ListRunsFilters } from '@/types/remediation';

const ALL_REPOS = '__all__';

export interface AuditLogProps {
  /** Optional repository options for the filter dropdown (id → label). */
  repositories?: { id: string; name: string }[];
}

export function AuditLog({ repositories = [] }: AuditLogProps) {
  const { t } = useTranslation(['remediation', 'common']);
  const [repoId, setRepoId] = useState<string>(ALL_REPOS);
  const [since, setSince] = useState<string>('');
  const [until, setUntil] = useState<string>('');

  const filters: ListRunsFilters = useMemo(() => {
    const f: ListRunsFilters = {};
    if (repoId !== ALL_REPOS) f.repositoryId = repoId;
    if (since) f.since = new Date(since).toISOString();
    if (until) f.until = new Date(until).toISOString();
    return f;
  }, [repoId, since, until]);

  const { data: runs, isLoading } = useRemediationRuns(filters);

  const entries = useMemo(() => (runs ?? []).filter(isAuditEntry), [runs]);

  const handleExport = () => {
    const csv = buildAuditCsv(entries);
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `remediation-audit-${new Date().toISOString().slice(0, 10)}.csv`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-end gap-3">
        <div className="space-y-1">
          <Label htmlFor="audit-repo">{t('remediation:audit.filterRepo')}</Label>
          <Select value={repoId} onValueChange={setRepoId}>
            <SelectTrigger id="audit-repo" className="w-48">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_REPOS}>{t('remediation:audit.allRepos')}</SelectItem>
              {repositories.map((repo) => (
                <SelectItem key={repo.id} value={repo.id}>
                  {repo.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1">
          <Label htmlFor="audit-since">{t('remediation:audit.since')}</Label>
          <Input
            id="audit-since"
            type="date"
            value={since}
            onChange={(e) => setSince(e.target.value)}
            className="w-40"
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="audit-until">{t('remediation:audit.until')}</Label>
          <Input
            id="audit-until"
            type="date"
            value={until}
            onChange={(e) => setUntil(e.target.value)}
            className="w-40"
          />
        </div>
        <Button
          variant="outline"
          onClick={handleExport}
          disabled={entries.length === 0}
          className="ml-auto"
        >
          <Download className="h-4 w-4 mr-1.5" aria-hidden="true" />
          {t('remediation:audit.exportCsv')}
        </Button>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center h-32" role="status" aria-live="polite">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      ) : entries.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground">{t('remediation:audit.empty')}</div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm" aria-label={t('remediation:audit.tableLabel')}>
            <thead>
              <tr className="border-b text-left text-muted-foreground">
                <th scope="col" className="py-2 pr-4 font-medium">
                  {t('remediation:audit.columns.action')}
                </th>
                <th scope="col" className="py-2 pr-4 font-medium">
                  {t('remediation:audit.columns.repository')}
                </th>
                <th scope="col" className="py-2 pr-4 font-medium">
                  {t('remediation:audit.columns.pr')}
                </th>
                <th scope="col" className="py-2 pr-4 font-medium">
                  {t('remediation:audit.columns.autonomy')}
                </th>
                <th scope="col" className="py-2 font-medium">
                  {t('remediation:audit.columns.completedAt')}
                </th>
              </tr>
            </thead>
            <tbody>
              {entries.map((run) => (
                <tr key={run.id} className="border-b last:border-0">
                  <td className="py-2 pr-4">
                    <Badge variant={run.merged ? 'success' : 'secondary'}>
                      {t(`remediation:audit.action.${auditAction(run)}`)}
                    </Badge>
                  </td>
                  <td className="py-2 pr-4 font-mono text-xs">{run.repositoryId}</td>
                  <td className="py-2 pr-4">
                    {run.consolidatedPrNumber != null ? `#${run.consolidatedPrNumber}` : '—'}
                  </td>
                  <td className="py-2 pr-4 text-xs text-muted-foreground">
                    {t(`remediation:autonomyLevel.${run.autonomyLevel}`)}
                  </td>
                  <td className="py-2 text-xs text-muted-foreground">
                    {run.completedAt ? new Date(run.completedAt).toLocaleString() : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

export default AuditLog;
