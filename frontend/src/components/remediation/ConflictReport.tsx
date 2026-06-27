import { useTranslation } from 'react-i18next';
import { AlertTriangle, SkipForward } from 'lucide-react';
import type { ConflictReport as ConflictReportData } from '@/types/remediation';

export interface ConflictReportProps {
  report: ConflictReportData | null;
}

export function ConflictReport({ report }: ConflictReportProps) {
  const { t } = useTranslation(['remediation']);

  const conflicts = report?.conflicts ?? [];
  const skipped = report?.skipped ?? [];

  if (conflicts.length === 0 && skipped.length === 0) {
    return <p className="text-sm text-muted-foreground">{t('remediation:conflicts.empty')}</p>;
  }

  return (
    <div className="space-y-4">
      {conflicts.length > 0 && (
        <div>
          <h4 className="mb-2 flex items-center gap-1.5 text-sm font-medium">
            <AlertTriangle className="h-4 w-4 text-red-500" aria-hidden="true" />
            {t('remediation:conflicts.conflicted')}
          </h4>
          <ul className="space-y-1.5">
            {conflicts.map((c) => (
              <li
                key={`conflict-${c.prNumber}`}
                className="rounded-md border border-red-500/30 bg-red-500/5 px-3 py-2 text-sm"
              >
                <span className="font-medium">#{c.prNumber}</span>
                <span className="ml-2 text-muted-foreground">{c.reason}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {skipped.length > 0 && (
        <div>
          <h4 className="mb-2 flex items-center gap-1.5 text-sm font-medium">
            <SkipForward className="h-4 w-4 text-yellow-500" aria-hidden="true" />
            {t('remediation:conflicts.skipped')}
          </h4>
          <ul className="space-y-1.5">
            {skipped.map((s) => (
              <li
                key={`skipped-${s.prNumber}`}
                className="rounded-md border border-yellow-500/30 bg-yellow-500/5 px-3 py-2 text-sm"
              >
                <span className="font-medium">#{s.prNumber}</span>
                <span className="ml-2 text-muted-foreground">{s.reason}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

export default ConflictReport;
