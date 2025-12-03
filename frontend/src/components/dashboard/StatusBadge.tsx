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
    color: 'bg-ampel-green',
    textColor: 'text-ampel-green',
  },
  yellow: {
    label: 'Pending',
    color: 'bg-ampel-yellow',
    textColor: 'text-ampel-yellow',
  },
  red: {
    label: 'Blocked',
    color: 'bg-ampel-red',
    textColor: 'text-ampel-red',
  },
  none: {
    label: 'No PRs',
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
  const config = statusConfig[status];

  return (
    <div className="flex items-center gap-2">
      <span className={cn('rounded-full', config.color, sizeConfig[size])} />
      {showLabel && <span className={cn('text-sm', config.textColor)}>{config.label}</span>}
    </div>
  );
}
