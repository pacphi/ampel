import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { FleetOverview } from './FleetOverview';
import type { ConsolidationPlan, FleetRow } from '@/types/remediation';

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

const mockUseFleet = vi.fn();
const mockPreviewMutate = vi.fn();
const mockUsePreview = vi.fn();

vi.mock('@/hooks/useFleetRemediation', () => ({
  useFleetRemediation: () => mockUseFleet(),
  usePreviewRepository: () => mockUsePreview(),
}));

const rows: FleetRow[] = [
  {
    repositoryId: 'repo-1',
    name: 'acme/app',
    openPrCount: 5,
    eligible: true,
    policyState: 'dry_run',
    airGapped: false,
  },
  {
    repositoryId: 'repo-2',
    name: 'acme/lib',
    openPrCount: 0,
    eligible: false,
    policyState: 'none',
    airGapped: true,
  },
];

beforeEach(() => {
  vi.clearAllMocks();
  mockUsePreview.mockReturnValue({ mutate: mockPreviewMutate, isPending: false, isError: false });
});

describe('FleetOverview', () => {
  it('should_renderRepositoryRows_when_dataLoaded', () => {
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);

    expect(screen.getByText('acme/app')).toBeInTheDocument();
    expect(screen.getByText('acme/lib')).toBeInTheDocument();
  });

  it('should_renderEligibilityBadges_when_dataLoaded', () => {
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);

    expect(screen.getByText('remediation:fleet.eligible')).toBeInTheDocument();
    expect(screen.getByText('remediation:fleet.notEligible')).toBeInTheDocument();
  });

  it('should_renderAirGappedIndicator_when_repoAirGapped', () => {
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);

    expect(screen.getByText('remediation:fleet.airGappedOn')).toBeInTheDocument();
  });

  it('should_renderEmptyState_when_noRepositories', () => {
    mockUseFleet.mockReturnValue({ data: [], isLoading: false, isError: false });

    render(<FleetOverview />);

    expect(screen.getByText('remediation:fleet.empty')).toBeInTheDocument();
  });

  it('should_invokePreview_when_previewClicked', async () => {
    const user = userEvent.setup();
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);

    const previewButtons = screen.getAllByRole('button', { name: /remediation:fleet.preview/ });
    await user.click(previewButtons[0]);

    await waitFor(() =>
      expect(mockPreviewMutate).toHaveBeenCalledWith('repo-1', expect.anything())
    );
  });

  it('should_showPlan_when_previewSucceeds', async () => {
    const user = userEvent.setup();
    const plan: ConsolidationPlan = {
      would_select: [{ number: 7, title: 'Bump deps', branch: 'deps' }],
      pr_count: 1,
      predicted_conflicts: [],
      estimated_duration_secs: 45,
      air_gapped: false,
      blocked_by_air_gap: false,
    };
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });
    mockPreviewMutate.mockImplementation(
      (_id: string, opts: { onSuccess: (p: ConsolidationPlan) => void }) => {
        opts.onSuccess(plan);
      }
    );

    render(<FleetOverview />);

    const previewButtons = screen.getAllByRole('button', { name: /remediation:fleet.preview/ });
    await user.click(previewButtons[0]);

    await waitFor(() => expect(screen.getByText('Bump deps')).toBeInTheDocument());
  });
});
