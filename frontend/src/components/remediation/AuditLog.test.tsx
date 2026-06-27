import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AuditLog } from './AuditLog';
import { buildAuditCsv, isAuditEntry } from './auditCsv';
import type { RemediationRun } from '@/types/remediation';

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

const mockUseRuns = vi.fn();
vi.mock('@/hooks/useRemediationRuns', () => ({
  useRemediationRuns: (filters: unknown) => mockUseRuns(filters),
}));

function makeRun(overrides: Partial<RemediationRun>): RemediationRun {
  return {
    id: 'run-1',
    repositoryId: 'repo-1',
    policyId: 'pol-1',
    state: 'completed',
    autonomyLevel: 'fully_autonomous',
    consolidatedPrNumber: 42,
    merged: true,
    branchName: 'remediation/run-1',
    ciStatus: 'success',
    attempts: 1,
    startedAt: '2026-06-01T00:00:00Z',
    completedAt: '2026-06-01T01:00:00Z',
    createdAt: '2026-06-01T00:00:00Z',
    updatedAt: '2026-06-01T01:00:00Z',
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe('AuditLog', () => {
  it('should_renderAuditEntries_when_autonomousActionsExist', () => {
    mockUseRuns.mockReturnValue({
      data: [makeRun({ id: 'run-1', merged: true })],
      isLoading: false,
    });

    render(<AuditLog />);

    expect(screen.getByText('remediation:audit.action.merged')).toBeInTheDocument();
  });

  it('should_renderEmptyState_when_noAuditEntries', () => {
    // A run that neither merged nor consolidated is not an audit entry.
    mockUseRuns.mockReturnValue({
      data: [makeRun({ merged: false, consolidatedPrNumber: null, state: 'failed' })],
      isLoading: false,
    });

    render(<AuditLog />);

    expect(screen.getByText('remediation:audit.empty')).toBeInTheDocument();
  });
});

describe('isAuditEntry', () => {
  it('should_returnTrue_when_runMerged', () => {
    expect(isAuditEntry(makeRun({ merged: true }))).toBe(true);
  });

  it('should_returnFalse_when_runNeitherMergedNorConsolidated', () => {
    expect(isAuditEntry(makeRun({ merged: false, consolidatedPrNumber: null }))).toBe(false);
  });
});

describe('buildAuditCsv', () => {
  it('should_produceHeaderPlusOneRowPerEntry', () => {
    const csv = buildAuditCsv([
      makeRun({ id: 'run-1' }),
      makeRun({ id: 'run-2', merged: false, consolidatedPrNumber: 7 }),
    ]);

    const lines = csv.split('\r\n');
    expect(lines).toHaveLength(3); // header + 2 rows
    expect(lines[0]).toContain('runId');
    expect(lines[1]).toContain('run-1');
    expect(lines[2]).toContain('run-2');
  });

  it('should_escapeQuotesInValues_when_present', () => {
    const csv = buildAuditCsv([makeRun({ repositoryId: 'a "quoted" repo' })]);

    expect(csv).toContain('"a ""quoted"" repo"');
  });
});
