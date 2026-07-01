import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
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

// Fixture spans all four filter quadrants so each toggle can be proven independently:
//   app  — has PRs + eligible      → visible by default (actionable)
//   lib  — no PRs + eligible       → hidden only by "only with PRs"
//   tool — has PRs + not eligible  → hidden only by "only eligible"
//   old  — no PRs + not eligible   → hidden by both (also air-gapped)
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
    eligible: true,
    policyState: 'suggest',
    airGapped: false,
  },
  {
    repositoryId: 'repo-3',
    name: 'acme/tool',
    openPrCount: 3,
    eligible: false,
    policyState: 'none',
    airGapped: false,
  },
  {
    repositoryId: 'repo-4',
    name: 'acme/old',
    openPrCount: 0,
    eligible: false,
    policyState: 'none',
    airGapped: true,
  },
];

const onlyWithPrsToggle = () => screen.getByLabelText('remediation:fleet.filters.onlyWithPrs');
const onlyEligibleToggle = () => screen.getByLabelText('remediation:fleet.filters.onlyEligible');

beforeEach(() => {
  vi.clearAllMocks();
  mockUsePreview.mockReturnValue({ mutate: mockPreviewMutate, isPending: false, isError: false });
});

describe('FleetOverview', () => {
  it('should_showOnlyActionableRows_when_defaultFiltersActive', () => {
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);

    // Default: both filters ON — only the has-PRs + eligible repo shows.
    expect(screen.getByText('acme/app')).toBeInTheDocument();
    // The 0-PR repo and the ineligible repo are hidden by default.
    expect(screen.queryByText('acme/lib')).not.toBeInTheDocument();
    expect(screen.queryByText('acme/tool')).not.toBeInTheDocument();
    expect(screen.queryByText('acme/old')).not.toBeInTheDocument();
  });

  it('should_showEveryActionableRow_when_multipleReposPassDefaults', () => {
    // Two rows both satisfy the default filters (PRs > 0 AND eligible): both must show,
    // guarding against a predicate that is accidentally too strict on a passing row.
    const twoActionable: FleetRow[] = [
      { ...rows[0], repositoryId: 'a', name: 'acme/one', openPrCount: 2, eligible: true },
      { ...rows[0], repositoryId: 'b', name: 'acme/two', openPrCount: 9, eligible: true },
    ];
    mockUseFleet.mockReturnValue({ data: twoActionable, isLoading: false, isError: false });

    render(<FleetOverview />);

    expect(screen.getByText('acme/one')).toBeInTheDocument();
    expect(screen.getByText('acme/two')).toBeInTheDocument();
  });

  it('should_reapplyFilter_when_toggledOffThenBackOn', async () => {
    const user = userEvent.setup();
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);
    // Off → the 0-PR eligible repo appears.
    await user.click(onlyWithPrsToggle());
    expect(screen.getByText('acme/lib')).toBeInTheDocument();
    // On again → it is hidden once more (state round-trips correctly).
    await user.click(onlyWithPrsToggle());
    expect(screen.queryByText('acme/lib')).not.toBeInTheDocument();
    expect(screen.getByText('acme/app')).toBeInTheDocument();
  });

  it('should_revealZeroPrRepo_when_onlyWithPrsToggledOff', async () => {
    const user = userEvent.setup();
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);
    await user.click(onlyWithPrsToggle());

    // onlyWithPrs OFF, onlyEligible still ON → eligible repos regardless of PR count.
    expect(screen.getByText('acme/app')).toBeInTheDocument();
    expect(screen.getByText('acme/lib')).toBeInTheDocument();
    // Still hidden: the ineligible ones (onlyEligible remains active).
    expect(screen.queryByText('acme/tool')).not.toBeInTheDocument();
    expect(screen.queryByText('acme/old')).not.toBeInTheDocument();
  });

  it('should_revealIneligibleRepo_when_onlyEligibleToggledOff', async () => {
    const user = userEvent.setup();
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);
    await user.click(onlyEligibleToggle());

    // onlyEligible OFF, onlyWithPrs still ON → repos with PRs regardless of eligibility.
    expect(screen.getByText('acme/app')).toBeInTheDocument();
    expect(screen.getByText('acme/tool')).toBeInTheDocument();
    // Still hidden: the 0-PR ones (onlyWithPrs remains active).
    expect(screen.queryByText('acme/lib')).not.toBeInTheDocument();
    expect(screen.queryByText('acme/old')).not.toBeInTheDocument();
  });

  it('should_showAllRows_when_bothFiltersToggledOff', async () => {
    const user = userEvent.setup();
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);
    await user.click(onlyWithPrsToggle());
    await user.click(onlyEligibleToggle());

    expect(screen.getByText('acme/app')).toBeInTheDocument();
    expect(screen.getByText('acme/lib')).toBeInTheDocument();
    expect(screen.getByText('acme/tool')).toBeInTheDocument();
    expect(screen.getByText('acme/old')).toBeInTheDocument();
    // Both eligibility badges and the air-gapped indicator are now visible.
    expect(screen.getAllByText('remediation:fleet.notEligible').length).toBeGreaterThan(0);
    expect(screen.getAllByText('remediation:fleet.eligible').length).toBeGreaterThan(0);
    expect(screen.getByText('remediation:fleet.airGappedOn')).toBeInTheDocument();
  });

  it('should_renderFilteredEmptyState_when_activeFiltersHideEveryRow', () => {
    // A fleet with no actionable repos: default filters exclude everything.
    mockUseFleet.mockReturnValue({
      data: [rows[3]], // acme/old: 0 PRs + ineligible
      isLoading: false,
      isError: false,
    });

    render(<FleetOverview />);

    expect(screen.getByText('remediation:fleet.filters.noMatch')).toBeInTheDocument();
    // Distinct from the zero-repositories empty state.
    expect(screen.queryByText('remediation:fleet.empty')).not.toBeInTheDocument();
    expect(screen.queryByText('acme/old')).not.toBeInTheDocument();
  });

  it('should_restoreRows_when_filtersClearedFromEmptyState', async () => {
    const user = userEvent.setup();
    mockUseFleet.mockReturnValue({
      data: [rows[3]], // acme/old: hidden by default filters
      isLoading: false,
      isError: false,
    });

    render(<FleetOverview />);
    expect(screen.getByText('remediation:fleet.filters.noMatch')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /remediation:fleet.filters.clear/ }));

    // Clearing the filters widens the view so the previously-hidden repo appears.
    expect(screen.getByText('acme/old')).toBeInTheDocument();
    expect(screen.queryByText('remediation:fleet.filters.noMatch')).not.toBeInTheDocument();
  });

  it('should_escapeFilteredEmptyState_when_singleToggleFlippedOff', async () => {
    const user = userEvent.setup();
    // acme/tool: has PRs but ineligible → hidden only by onlyEligible.
    mockUseFleet.mockReturnValue({ data: [rows[2]], isLoading: false, isError: false });

    render(<FleetOverview />);
    expect(screen.getByText('remediation:fleet.filters.noMatch')).toBeInTheDocument();

    // The toggles remain available in the empty state; flipping the one filter that
    // hides the row reveals it — no need for the nuclear "clear".
    await user.click(onlyEligibleToggle());

    expect(screen.getByText('acme/tool')).toBeInTheDocument();
    expect(screen.queryByText('remediation:fleet.filters.noMatch')).not.toBeInTheDocument();
  });

  it('should_renderEmptyState_when_noRepositories', () => {
    mockUseFleet.mockReturnValue({ data: [], isLoading: false, isError: false });

    render(<FleetOverview />);

    expect(screen.getByText('remediation:fleet.empty')).toBeInTheDocument();
    // Not the filtered-empty state — there are genuinely zero repositories.
    expect(screen.queryByText('remediation:fleet.filters.noMatch')).not.toBeInTheDocument();
  });

  it('should_invokePreview_when_previewClicked', async () => {
    const user = userEvent.setup();
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);

    // acme/app is visible under the default filters.
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

  it('should_previewRevealedRow_when_previewClickedAfterToggle', async () => {
    const user = userEvent.setup();
    mockUseFleet.mockReturnValue({ data: rows, isLoading: false, isError: false });

    render(<FleetOverview />);
    // acme/tool is hidden by default; widen the view so it renders, then preview it.
    await user.click(onlyEligibleToggle());
    const toolRow = screen.getByText('acme/tool').closest('tr') as HTMLElement;
    await user.click(within(toolRow).getByRole('button', { name: /remediation:fleet.preview/ }));

    await waitFor(() =>
      expect(mockPreviewMutate).toHaveBeenCalledWith('repo-3', expect.anything())
    );
  });
});
