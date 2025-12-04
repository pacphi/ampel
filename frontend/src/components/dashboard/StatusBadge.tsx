import { cn } from '@/lib/utils';
import type { AmpelStatus } from '@/types';

interface StatusBadgeProps {
  status: AmpelStatus;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
}

const statusConfig = {
  green: {
    label: 'Ready',
    description: 'Ready to merge - CI passed, approved, no conflicts',
    color: 'bg-ampel-green',
    textColor: 'text-ampel-green',
  },
  yellow: {
    label: 'Pending',
    description: 'In progress - CI running, awaiting review, or draft',
    color: 'bg-ampel-yellow',
    textColor: 'text-ampel-yellow',
  },
  red: {
    label: 'Blocked',
    description: 'Blocked - CI failed, conflicts, or changes requested',
    color: 'bg-ampel-red',
    textColor: 'text-ampel-red',
  },
  none: {
    label: 'No PRs',
    description: 'No open pull requests',
    color: 'bg-muted-foreground',
    textColor: 'text-muted-foreground',
  },
};

const sizeConfig = {
  sm: 'h-2 w-2',
  md: 'h-3 w-3',
  lg: 'h-4 w-4',
};

export default function StatusBadge({ status, size = 'md', showLabel = false }: StatusBadgeProps) {
  // Don't render anything for "none" status (no PRs)
  if (status === 'none') {
    return null;
  }

  const config = statusConfig[status];

  return (
    <div className="flex items-center gap-2" title={config.description}>
      <span className={cn('rounded-full cursor-help', config.color, sizeConfig[size])} />
      {showLabel && <span className={cn('text-sm', config.textColor)}>{config.label}</span>}
    </div>
  );
}
