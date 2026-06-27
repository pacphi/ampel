import { useTranslation } from 'react-i18next';
import { ExternalLink } from 'lucide-react';
import { cn } from '@/lib/utils';
import { ciStatusTone, toneDotClass, toneTextClass, type TrafficTone } from './ciStatus';
import type { CiMatrix } from '@/types/remediation';

export interface CiCheckMatrixProps {
  ciMatrix: CiMatrix | null;
}

interface CheckRow {
  id: string;
  label: string;
  required: boolean;
  tone: TrafficTone;
  detail?: string;
}

export function CiCheckMatrix({ ciMatrix }: CiCheckMatrixProps) {
  const { t } = useTranslation(['remediation']);

  if (!ciMatrix) {
    return <p className="text-sm text-muted-foreground">{t('remediation:ci.empty')}</p>;
  }

  const rows: CheckRow[] = [
    {
      id: 'ci-status',
      label: t('remediation:ci.statusCheck'),
      required: true,
      tone: ciStatusTone(ciMatrix.ciStatus),
      detail: ciMatrix.ciStatus,
    },
    ...ciMatrix.predictedConflicts.map((conflict, idx) => ({
      id: `conflict-${idx}`,
      label: t('remediation:ci.predictedConflict'),
      required: false,
      tone: 'yellow' as TrafficTone,
      detail: conflict,
    })),
  ];

  return (
    <div className="space-y-3">
      <table className="w-full text-sm" aria-label={t('remediation:ci.tableLabel')}>
        <thead>
          <tr className="border-b text-left text-muted-foreground">
            <th scope="col" className="py-2 pr-4 font-medium">
              {t('remediation:ci.columns.check')}
            </th>
            <th scope="col" className="py-2 pr-4 font-medium">
              {t('remediation:ci.columns.type')}
            </th>
            <th scope="col" className="py-2 font-medium">
              {t('remediation:ci.columns.status')}
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.id} className="border-b last:border-0" data-tone={row.tone}>
              <td className="py-2 pr-4">
                <span className="font-medium">{row.label}</span>
                {row.detail && (
                  <span className="block text-xs text-muted-foreground">{row.detail}</span>
                )}
              </td>
              <td className="py-2 pr-4 text-xs text-muted-foreground">
                {row.required ? t('remediation:ci.required') : t('remediation:ci.optional')}
              </td>
              <td className="py-2">
                <span className={cn('inline-flex items-center gap-1.5', toneTextClass[row.tone])}>
                  <span
                    className={cn('h-2.5 w-2.5 rounded-full', toneDotClass[row.tone])}
                    aria-hidden="true"
                  />
                  {t(`remediation:ci.tone.${row.tone}`)}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <dl className="grid grid-cols-2 gap-1 text-xs text-muted-foreground">
        <dt>{t('remediation:ci.headSha')}</dt>
        <dd className="text-right font-mono">{ciMatrix.headSha.slice(0, 12)}</dd>
        {ciMatrix.ciLogsUrl && (
          <>
            <dt>{t('remediation:ci.logs')}</dt>
            <dd className="text-right">
              <a
                href={ciMatrix.ciLogsUrl}
                target="_blank"
                rel="noreferrer noopener"
                className="inline-flex items-center gap-1 text-primary hover:underline"
              >
                {t('remediation:ci.viewLogs')}
                <ExternalLink className="h-3 w-3" aria-hidden="true" />
              </a>
            </dd>
          </>
        )}
      </dl>
    </div>
  );
}

export default CiCheckMatrix;
