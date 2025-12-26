import { lazy, Suspense } from 'react';
import { Skeleton } from '@/components/ui/skeleton';

// Lazy load the heavy diff component
const GitDiffViewComponent = lazy(() => import('./GitDiffView'));

interface GitDiffViewLazyProps {
  repoId: string;
  prId: string;
  isExpanded?: boolean;
}

// Loading skeleton for better perceived performance
function GitDiffSkeleton() {
  return (
    <div className="space-y-4 p-4">
      <div className="flex items-center justify-between">
        <Skeleton className="h-6 w-32" />
        <Skeleton className="h-6 w-48" />
      </div>
      <div className="space-y-2">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="space-y-2">
            <Skeleton className="h-8 w-full" />
            <Skeleton className="h-32 w-full" />
          </div>
        ))}
      </div>
    </div>
  );
}

export default function GitDiffViewLazy({
  repoId,
  prId,
  isExpanded = false,
}: GitDiffViewLazyProps) {
  // Only render when expanded to avoid unnecessary loading
  if (!isExpanded) {
    return null;
  }

  return (
    <Suspense fallback={<GitDiffSkeleton />}>
      <GitDiffViewComponent repoId={repoId} prId={prId} />
    </Suspense>
  );
}
