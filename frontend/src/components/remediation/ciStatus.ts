import type { CiStatus } from '@/types/remediation';

export type TrafficTone = 'green' | 'yellow' | 'red';

/** Map a provider CI status string onto a traffic-light tone. */
export function ciStatusTone(status: CiStatus | null | undefined): TrafficTone {
  const s = (status ?? '').toLowerCase();
  if (['success', 'passed', 'passing', 'green', 'completed'].includes(s)) {
    return 'green';
  }
  if (['failure', 'failed', 'failing', 'error', 'red', 'cancelled', 'canceled'].includes(s)) {
    return 'red';
  }
  // pending, queued, running, in_progress, neutral, unknown, '' → yellow
  return 'yellow';
}

export const toneDotClass: Record<TrafficTone, string> = {
  green: 'bg-green-500',
  yellow: 'bg-yellow-500',
  red: 'bg-red-500',
};

export const toneTextClass: Record<TrafficTone, string> = {
  green: 'text-green-600 dark:text-green-400',
  yellow: 'text-yellow-600 dark:text-yellow-400',
  red: 'text-red-600 dark:text-red-400',
};
