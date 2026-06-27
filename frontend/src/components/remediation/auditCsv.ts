import type { RemediationRun } from '@/types/remediation';

/** A run counts as an audit entry once it autonomously merged or closed PRs. */
export function isAuditEntry(run: RemediationRun): boolean {
  return run.merged || run.consolidatedPrNumber != null;
}

export function auditAction(run: RemediationRun): 'merged' | 'consolidated' {
  return run.merged ? 'merged' : 'consolidated';
}

/** CSV value escaping (RFC 4180): quote and double inner quotes. */
function csvCell(value: string | number | boolean | null | undefined): string {
  const s = value == null ? '' : String(value);
  return `"${s.replace(/"/g, '""')}"`;
}

/**
 * Build a CSV document (header + one row per audit entry) from run summaries.
 * Pure + exported so it can be unit-tested without DOM download plumbing.
 */
export function buildAuditCsv(runs: RemediationRun[]): string {
  const header = [
    'runId',
    'repositoryId',
    'action',
    'state',
    'consolidatedPrNumber',
    'merged',
    'autonomyLevel',
    'completedAt',
  ];
  const lines = [header.map(csvCell).join(',')];
  for (const run of runs) {
    lines.push(
      [
        run.id,
        run.repositoryId,
        auditAction(run),
        run.state,
        run.consolidatedPrNumber ?? '',
        run.merged,
        run.autonomyLevel,
        run.completedAt ?? '',
      ]
        .map(csvCell)
        .join(',')
    );
  }
  return lines.join('\r\n');
}
