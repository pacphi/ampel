import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDate(date: string | Date): string {
  return new Date(date).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export function formatRelativeTime(date: string | Date): string {
  const now = new Date();
  const then = new Date(date);
  const diffMs = now.getTime() - then.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return 'just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;

  return formatDate(date);
}

export function getProviderIcon(provider: string): string {
  switch (provider.toLowerCase()) {
    case 'github':
      return 'Github';
    case 'gitlab':
      return 'Gitlab';
    case 'bitbucket':
      return 'Boxes';
    default:
      return 'GitBranch';
  }
}

export function getStatusColor(status: string): string {
  switch (status.toLowerCase()) {
    case 'green':
      return 'text-ampel-green';
    case 'yellow':
      return 'text-ampel-yellow';
    case 'red':
      return 'text-ampel-red';
    default:
      return 'text-muted-foreground';
  }
}

export function getStatusBgColor(status: string): string {
  switch (status.toLowerCase()) {
    case 'green':
      return 'bg-ampel-green';
    case 'yellow':
      return 'bg-ampel-yellow';
    case 'red':
      return 'bg-ampel-red';
    default:
      return 'bg-muted';
  }
}
