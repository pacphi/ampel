import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AgentSessionViewer } from './AgentSessionViewer';
import type { AgentSession } from '@/types/remediation';

// Passthrough i18n: t() returns the key so assertions are stable.
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: 'en' },
  }),
}));

const baseSession: AgentSession = {
  iterations: 3,
  maxIterations: 5,
  tokensUsed: 12345,
  costUsd: '0.42',
  status: 'succeeded',
  failureClass: 'build_error',
  classifierSource: 'rules',
  classifierConfidence: 0.9,
  transcriptRef: 'https://example.com/transcript/abc',
};

describe('AgentSessionViewer', () => {
  it('should_renderIterations_when_maxIterationsPresent', () => {
    render(<AgentSessionViewer session={baseSession} />);
    expect(screen.getByText('3 / 5')).toBeInTheDocument();
  });

  it('should_renderTokensAndCost_when_provided', () => {
    render(<AgentSessionViewer session={baseSession} />);
    expect(screen.getByText('12,345')).toBeInTheDocument();
    expect(screen.getByText('$0.42')).toBeInTheDocument();
  });

  it('should_renderStatusBadge_when_mounted', () => {
    render(<AgentSessionViewer session={baseSession} />);
    expect(screen.getByText('remediation:agentSession.status.succeeded')).toBeInTheDocument();
  });

  it('should_renderClassifierWithConfidence_when_provided', () => {
    render(<AgentSessionViewer session={baseSession} />);
    expect(screen.getByText('rules (90%)')).toBeInTheDocument();
  });

  it('should_renderTranscriptLink_when_transcriptRefPresent', () => {
    render(<AgentSessionViewer session={baseSession} />);
    const link = screen.getByRole('link', { name: 'remediation:agentSession.transcript' });
    expect(link).toHaveAttribute('href', 'https://example.com/transcript/abc');
  });

  it('should_omitTranscriptLink_when_transcriptRefMissing', () => {
    render(<AgentSessionViewer session={{ ...baseSession, transcriptRef: null }} />);
    expect(
      screen.queryByRole('link', { name: 'remediation:agentSession.transcript' })
    ).not.toBeInTheDocument();
  });
});
