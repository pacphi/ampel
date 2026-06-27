import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PolicyEditor } from './PolicyEditor';

// Passthrough i18n: t() returns the key so assertions are stable.
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

const mockCreate = vi.fn();
const mockUpdate = vi.fn();

vi.mock('@/hooks/useRemediationPolicies', () => ({
  useCreateRemediationPolicy: () => ({ mutate: mockCreate, isPending: false }),
  useUpdateRemediationPolicy: () => ({ mutate: mockUpdate, isPending: false }),
}));

beforeEach(() => {
  vi.clearAllMocks();
});

describe('PolicyEditor', () => {
  it('should_renderAutonomyStops_when_mounted', () => {
    render(<PolicyEditor />);

    expect(screen.getByRole('radio', { name: 'remediation:autonomyStop.off' })).toBeInTheDocument();
    expect(
      screen.getByRole('radio', { name: 'remediation:autonomyStop.dry_run' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('radio', { name: 'remediation:autonomyStop.consolidate' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('radio', { name: 'remediation:autonomyStop.auto_merge' })
    ).toBeInTheDocument();
  });

  it('should_renderScopeSelector_when_creatingNewPolicy', () => {
    render(<PolicyEditor />);
    expect(screen.getByLabelText('remediation:editor.scopeId')).toBeInTheDocument();
  });

  it('should_requireConfirmation_when_autoMergeSelected', async () => {
    const user = userEvent.setup();
    const { container } = render(<PolicyEditor />);

    await user.click(screen.getByRole('radio', { name: 'remediation:autonomyStop.auto_merge' }));

    // Confirm dialog appears; the stop is NOT applied yet. (While the dialog is
    // open Radix marks the background aria-hidden, so query the input directly.)
    expect(screen.getByText('remediation:editor.confirmAutoMerge.title')).toBeInTheDocument();
    const autoMergeRadio = container.querySelector<HTMLInputElement>('input[value="auto_merge"]');
    expect(autoMergeRadio).not.toBeChecked();
  });

  it('should_applyAutoMerge_when_confirmationAccepted', async () => {
    const user = userEvent.setup();
    render(<PolicyEditor />);

    await user.type(screen.getByLabelText('remediation:editor.scopeId'), 'repo-1');
    await user.click(screen.getByRole('radio', { name: 'remediation:autonomyStop.auto_merge' }));
    await user.click(
      screen.getByRole('button', { name: 'remediation:editor.confirmAutoMerge.confirm' })
    );

    expect(
      screen.getByRole('radio', { name: 'remediation:autonomyStop.auto_merge' })
    ).toBeChecked();

    await user.click(screen.getByRole('button', { name: 'remediation:editor.save' }));

    await waitFor(() => expect(mockCreate).toHaveBeenCalled());
    expect(mockCreate).toHaveBeenCalledWith(
      expect.objectContaining({
        scopeId: 'repo-1',
        autonomyLevel: 'fully_autonomous',
        autoMergeEnabled: true,
        requireHumanApproval: false,
      }),
      expect.anything()
    );
  });

  it('should_blockSave_when_scopeIdMissing', async () => {
    const user = userEvent.setup();
    render(<PolicyEditor />);

    await user.click(screen.getByRole('button', { name: 'remediation:editor.save' }));

    expect(screen.getByText('remediation:editor.errors.scopeIdRequired')).toBeInTheDocument();
    expect(mockCreate).not.toHaveBeenCalled();
  });
});
