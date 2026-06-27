import { useTranslation } from 'react-i18next';
import { Bot, ExternalLink } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import type { AgentSession, AgentSessionStatus } from '@/types/remediation';

function statusVariant(
  status: AgentSessionStatus
): 'default' | 'secondary' | 'success' | 'warning' | 'destructive' {
  switch (status) {
    case 'succeeded':
      return 'success';
    case 'failed':
    case 'budget_exceeded':
      return 'destructive';
    case 'handoff_human':
      return 'warning';
    default:
      return 'default';
  }
}

interface AgentSessionViewerProps {
  session: AgentSession;
}

/**
 * Agentic remediation session telemetry (Phase 4). Rendered in run detail when a
 * run engaged the Tier-2 agent: iteration count, token/cost spend, terminal
 * status, the failure classification + classifier provenance, and a transcript
 * link when one is available.
 */
export function AgentSessionViewer({ session }: AgentSessionViewerProps) {
  const { t } = useTranslation(['remediation', 'common']);

  const iterations =
    session.maxIterations != null
      ? `${session.iterations} / ${session.maxIterations}`
      : `${session.iterations}`;

  const cost = session.costUsd != null ? `$${session.costUsd}` : '—';
  const confidence =
    session.classifierConfidence != null
      ? `${Math.round(session.classifierConfidence * 100)}%`
      : null;

  return (
    <div className="space-y-3" aria-label={t('remediation:agentSession.title')}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm font-medium">
          <Bot className="h-4 w-4" aria-hidden="true" />
          {t('remediation:agentSession.title')}
        </div>
        <Badge variant={statusVariant(session.status)}>
          {t(`remediation:agentSession.status.${session.status}`)}
        </Badge>
      </div>

      <dl className="grid grid-cols-2 gap-2 text-sm">
        <dt className="text-muted-foreground">{t('remediation:agentSession.iterations')}</dt>
        <dd className="text-right font-medium">{iterations}</dd>

        <dt className="text-muted-foreground">{t('remediation:agentSession.tokensUsed')}</dt>
        <dd className="text-right font-medium">{session.tokensUsed.toLocaleString()}</dd>

        <dt className="text-muted-foreground">{t('remediation:agentSession.cost')}</dt>
        <dd className="text-right font-medium">{cost}</dd>

        {session.failureClass && (
          <>
            <dt className="text-muted-foreground">{t('remediation:agentSession.failureClass')}</dt>
            <dd className="text-right font-medium">{session.failureClass}</dd>
          </>
        )}

        {session.classifierSource && (
          <>
            <dt className="text-muted-foreground">{t('remediation:agentSession.classifier')}</dt>
            <dd className="text-right font-medium">
              {session.classifierSource}
              {confidence ? ` (${confidence})` : ''}
            </dd>
          </>
        )}
      </dl>

      {session.transcriptRef && (
        <a
          href={session.transcriptRef}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1.5 text-sm text-primary hover:underline"
        >
          <ExternalLink className="h-3.5 w-3.5" aria-hidden="true" />
          {t('remediation:agentSession.transcript')}
        </a>
      )}
    </div>
  );
}

export default AgentSessionViewer;
