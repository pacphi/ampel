import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KillSwitch } from './KillSwitch';
import { selectTopScopePolicy } from './killSwitchScope';
import type { RemediationPolicy } from '@/types/remediation';

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

const mockUsePolicies = vi.fn();
const mockToggle = vi.fn();
vi.mock('@/hooks/useRemediationPolicies', () => ({
  useRemediationPolicies: () => mockUsePolicies(),
  useToggleRemediationPolicy: () => ({ mutate: mockToggle, isPending: false }),
}));

function policy(overrides: Partial<RemediationPolicy>): RemediationPolicy {
  return {
    id: 'pol-1',
    scopeType: 'org',
    scopeId: 'org-1',
    enabled: true,
    minOpenPrs: 1,
    prSelection: 'all_open',
    autonomyLevel: 'auto_with_approval',
    remediationTier: 'consolidate_only',
    maxPrsPerRun: 5,
    allowedTargets: [],
    skipDraft: true,
    requireGreenBeforeMerge: true,
    airGapped: false,
    autoMergeEnabled: false,
    autoMergeRule: null,
    requireHumanApproval: true,
    agentBudget: null,
    notificationConfig: null,
    playbookRef: null,
    createdAt: '2026-06-01T00:00:00Z',
    updatedAt: '2026-06-01T00:00:00Z',
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe('selectTopScopePolicy', () => {
  it('should_pickOrgScope_when_multipleScopesPresent', () => {
    const policies = [
      policy({ id: 'repo', scopeType: 'repository' }),
      policy({ id: 'org', scopeType: 'org' }),
      policy({ id: 'team', scopeType: 'team' }),
    ];

    expect(selectTopScopePolicy(policies)?.id).toBe('org');
  });

  it('should_returnUndefined_when_noPolicies', () => {
    expect(selectTopScopePolicy([])).toBeUndefined();
  });
});

describe('KillSwitch', () => {
  it('should_reflectActiveState_when_topPolicyEnabled', () => {
    mockUsePolicies.mockReturnValue({ data: [policy({ enabled: true })], isLoading: false });

    render(<KillSwitch />);

    const sw = screen.getByRole('switch', { name: 'remediation:killSwitch.label' });
    expect(sw).toHaveAttribute('aria-checked', 'false'); // not paused
    expect(screen.getByText('remediation:killSwitch.active')).toBeInTheDocument();
  });

  it('should_reflectPausedState_when_topPolicyDisabled', () => {
    mockUsePolicies.mockReturnValue({ data: [policy({ enabled: false })], isLoading: false });

    render(<KillSwitch />);

    const sw = screen.getByRole('switch', { name: 'remediation:killSwitch.label' });
    expect(sw).toHaveAttribute('aria-checked', 'true'); // paused
    expect(screen.getByText('remediation:killSwitch.paused')).toBeInTheDocument();
  });

  it('should_toggleTopPolicy_when_switchClicked', async () => {
    const user = userEvent.setup();
    mockUsePolicies.mockReturnValue({
      data: [policy({ id: 'pol-top', enabled: true })],
      isLoading: false,
    });

    render(<KillSwitch />);

    await user.click(screen.getByRole('switch', { name: 'remediation:killSwitch.label' }));

    expect(mockToggle).toHaveBeenCalledWith('pol-top');
  });
});
