import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { RunTimeline } from './RunTimeline';
import type { RunDetail, RunState } from '@/types/remediation';

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

// Mock the SSE hook so we control the live state deterministically.
const mockUseEvents = vi.fn();
vi.mock('@/hooks/useRemediationRunEvents', () => ({
  useRemediationRunEvents: (opts: unknown) => mockUseEvents(opts),
}));

function makeRun(state: RunState): RunDetail {
  return {
    id: 'run-1',
    repositoryId: 'repo-1',
    policyId: 'pol-1',
    state,
    autonomyLevel: 'auto_with_approval',
    consolidatedPrNumber: 42,
    merged: false,
    branchName: 'remediation/run-1',
    ciStatus: 'pending',
    attempts: 1,
    startedAt: '2026-06-01T00:00:00Z',
    createdAt: '2026-06-01T00:00:00Z',
    updatedAt: '2026-06-01T00:00:00Z',
    dispositions: [
      { prNumber: 10, disposition: 'Consolidated' },
      { prNumber: 11, disposition: { SkippedConflict: { reason: 'merge conflict' } } },
    ],
    ciMatrix: null,
    conflictReport: null,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  mockUseEvents.mockReturnValue({ state: null, ciStatus: null, finished: false, connected: false });
});

describe('RunTimeline', () => {
  it('should_renderAllMainFlowStates_when_mounted', () => {
    render(<RunTimeline run={makeRun('consolidating')} />);

    expect(screen.getByText('remediation:runState.created')).toBeInTheDocument();
    expect(screen.getByText('remediation:runState.consolidating')).toBeInTheDocument();
    expect(screen.getByText('remediation:runState.completed')).toBeInTheDocument();
  });

  it('should_highlightCurrentState_when_runInProgress', () => {
    render(<RunTimeline run={makeRun('verifying')} />);

    const current = screen.getByText('remediation:runState.verifying');
    expect(current).toHaveAttribute('aria-current', 'step');
    expect(current).toHaveAttribute('data-status', 'current');
  });

  it('should_updateCurrentState_when_sseEventArrives', () => {
    // Persisted state is "selecting" but a live SSE event reports "merging".
    mockUseEvents.mockReturnValue({
      state: 'merging',
      ciStatus: 'success',
      finished: false,
      connected: true,
    });

    render(<RunTimeline run={makeRun('selecting')} />);

    const live = screen.getByText('remediation:runState.merging');
    expect(live).toHaveAttribute('data-status', 'current');
    // The persisted "selecting" step is now marked done.
    expect(screen.getByText('remediation:runState.selecting')).toHaveAttribute(
      'data-status',
      'done'
    );
  });

  it('should_renderDispositionBadges_when_dispositionsPresent', () => {
    render(<RunTimeline run={makeRun('completed')} />);

    expect(screen.getByText('remediation:disposition.consolidated')).toBeInTheDocument();
    expect(screen.getByText('remediation:disposition.skippedConflict')).toBeInTheDocument();
  });
});
