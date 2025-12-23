import { Globe, Lock, Archive } from 'lucide-react';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';

interface RepositoryStatusIconsProps {
  isPrivate: boolean;
  isArchived: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const sizeClasses = {
  sm: 'h-3 w-3',
  md: 'h-3.5 w-3.5',
  lg: 'h-4 w-4',
};

export default function RepositoryStatusIcons({
  isPrivate,
  isArchived,
  size = 'md',
  className,
}: RepositoryStatusIconsProps) {
  const iconSize = sizeClasses[size];

  return (
    <TooltipProvider>
      <div className={cn('flex items-center gap-1', className)}>
        {/* Visibility Icon - Public or Private */}
        <Tooltip>
          <TooltipTrigger asChild>
            {isPrivate ? (
              <Lock className={cn(iconSize, 'text-amber-600')} />
            ) : (
              <Globe className={cn(iconSize, 'text-green-600')} />
            )}
          </TooltipTrigger>
          <TooltipContent>
            <p>{isPrivate ? 'Private repository' : 'Public repository'}</p>
          </TooltipContent>
        </Tooltip>

        {/* Archive Icon - Only shown when archived */}
        {isArchived && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Archive className={cn(iconSize, 'text-gray-500')} />
            </TooltipTrigger>
            <TooltipContent>
              <p>Archived repository</p>
            </TooltipContent>
          </Tooltip>
        )}
      </div>
    </TooltipProvider>
  );
}
