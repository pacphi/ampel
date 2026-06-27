import { useTranslation } from 'react-i18next';
import { Check, CircleDot, Circle } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import { useRemediationRunEvents } from '@/hooks/useRemediationRunEvents';
import type { Disposition, RunDetail, RunState } from '@/types/remediation';

/** Happy-path order of the run state machine. */
const MAIN_FLOW: RunState[] = [
  'created',
  'selecting',
  'consolidating',
  'verifying',
  'awaiting_approval',
  'merging',
  'finalizing',
  'completed',
];

/** Off-path / terminal states shown only when the run actually reaches them. */
const BRANCH_STATES: RunState[] = ['agent_fixing', 'handoff_human', 'failed', 'cancelled', 'no_op'];

const TERMINAL: ReadonlySet<RunState> = new Set([
  'completed',
  'handoff_human',
  'failed',
  'cancelled',
  'no_op',
]);

type StepStatus = 'done' | 'current' | 'pending';

function dispositionVariant(
  d: Disposition
): 'success' | 'secondary' | 'warning' | 'destructive' | 'outline' {
  if (d === 'Consolidated') return 'success';
  if ('ClosedWithRef' in d) return 'secondary';
  if ('SkippedConflict' in d) return 'warning';
  return 'outline'; // LeftOpen
}

function dispositionKey(d: Disposition): string {
  if (d === 'Consolidated') return 'consolidated';
  if ('ClosedWithRef' in d) return 'closedWithRef';
  if ('SkippedConflict' in d) return 'skippedConflict';
  return 'leftOpen';
}

export interface RunTimelineProps {
  run: RunDetail;
}

export function RunTimeline({ run }: RunTimelineProps) {
  const { t } = useTranslation(['remediation']);

  // Live updates: fall back to the persisted run state until an event arrives.
  const live = useRemediationRunEvents({
    runId: run.id,
    enabled: !TERMINAL.has(run.state),
  });
  const currentState: RunState = live.state ?? run.state;

  // Build the displayed sequence: main flow + the current branch state if any.
  const steps: RunState[] = [...MAIN_FLOW];
  if (BRANCH_STATES.includes(currentState) && !steps.includes(currentState)) {
    const insertAt = steps.indexOf('completed');
    steps.splice(insertAt, 0, currentState);
  }

  const currentIndex = steps.indexOf(currentState);

  return (
    <div className="space-y-6">
      <ol className="relative" aria-label={t('remediation:runs.timelineLabel')}>
        {steps.map((step, idx) => {
          const status: StepStatus =
            idx < currentIndex ? 'done' : idx === currentIndex ? 'current' : 'pending';
          const isLast = idx === steps.length - 1;
          return (
            <li key={step} className="flex gap-3 pb-4 last:pb-0">
              <div className="flex flex-col items-center">
                <span
                  className={cn(
                    'flex h-6 w-6 shrink-0 items-center justify-center rounded-full border',
                    status === 'done' && 'border-green-500 bg-green-500 text-white',
                    status === 'current' && 'border-primary bg-primary text-primary-foreground',
                    status === 'pending' && 'border-muted-foreground/30 text-muted-foreground'
                  )}
                  aria-hidden="true"
                >
                  {status === 'done' ? (
                    <Check className="h-3.5 w-3.5" />
                  ) : status === 'current' ? (
                    <CircleDot className="h-3.5 w-3.5" />
                  ) : (
                    <Circle className="h-3 w-3" />
                  )}
                </span>
                {!isLast && <span className="mt-1 w-px flex-1 bg-border" aria-hidden="true" />}
              </div>
              <div className="pt-0.5">
                <span
                  className={cn(
                    'text-sm',
                    status === 'current' ? 'font-semibold text-foreground' : 'text-muted-foreground'
                  )}
                  aria-current={status === 'current' ? 'step' : undefined}
                  data-state={step}
                  data-status={status}
                >
                  {t(`remediation:runState.${step}`)}
                </span>
              </div>
            </li>
          );
        })}
      </ol>

      <div>
        <h4 className="mb-2 text-sm font-medium">{t('remediation:runs.dispositions')}</h4>
        {run.dispositions.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t('remediation:runs.noDispositions')}</p>
        ) : (
          <ul className="space-y-1.5">
            {run.dispositions.map((d) => (
              <li
                key={d.prNumber}
                className="flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm"
              >
                <span className="text-muted-foreground">#{d.prNumber}</span>
                <span className="flex-1" />
                <Badge variant={dispositionVariant(d.disposition)}>
                  {t(`remediation:disposition.${dispositionKey(d.disposition)}`)}
                </Badge>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

export default RunTimeline;
