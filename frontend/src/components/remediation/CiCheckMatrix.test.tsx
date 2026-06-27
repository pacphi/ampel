import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CiCheckMatrix } from './CiCheckMatrix';
import type { CiMatrix } from '@/types/remediation';

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

describe('CiCheckMatrix', () => {
  it('should_renderEmptyState_when_matrixNull', () => {
    render(<CiCheckMatrix ciMatrix={null} />);

    expect(screen.getByText('remediation:ci.empty')).toBeInTheDocument();
  });

  it('should_renderGreenTone_when_ciStatusSuccess', () => {
    const matrix: CiMatrix = {
      ciStatus: 'success',
      headSha: 'abcdef1234567890',
      ciLogsUrl: null,
      predictedConflicts: [],
    };

    render(<CiCheckMatrix ciMatrix={matrix} />);

    const row = screen.getByText('remediation:ci.statusCheck').closest('tr');
    expect(row).toHaveAttribute('data-tone', 'green');
    expect(screen.getByText('remediation:ci.tone.green')).toBeInTheDocument();
  });

  it('should_renderRedTone_when_ciStatusFailure', () => {
    const matrix: CiMatrix = {
      ciStatus: 'failure',
      headSha: 'deadbeef',
      predictedConflicts: [],
    };

    render(<CiCheckMatrix ciMatrix={matrix} />);

    const row = screen.getByText('remediation:ci.statusCheck').closest('tr');
    expect(row).toHaveAttribute('data-tone', 'red');
  });

  it('should_renderOptionalYellowRows_when_predictedConflictsPresent', () => {
    const matrix: CiMatrix = {
      ciStatus: 'success',
      headSha: 'abc123',
      predictedConflicts: ['src/main.rs', 'Cargo.lock'],
    };

    render(<CiCheckMatrix ciMatrix={matrix} />);

    const conflictRows = screen.getAllByText('remediation:ci.predictedConflict');
    expect(conflictRows).toHaveLength(2);
    expect(conflictRows[0].closest('tr')).toHaveAttribute('data-tone', 'yellow');
  });

  it('should_renderLogsLink_when_logsUrlProvided', () => {
    const matrix: CiMatrix = {
      ciStatus: 'pending',
      headSha: 'abc123',
      ciLogsUrl: 'https://ci.example.com/run/1',
      predictedConflicts: [],
    };

    render(<CiCheckMatrix ciMatrix={matrix} />);

    const link = screen.getByRole('link', { name: /remediation:ci.viewLogs/ });
    expect(link).toHaveAttribute('href', 'https://ci.example.com/run/1');
  });
});
