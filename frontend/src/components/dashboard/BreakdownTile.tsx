import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Globe, Lock, Archive } from 'lucide-react';
import type { VisibilityBreakdown } from '@/types';
import type { ComponentType } from 'react';

interface BreakdownTileProps {
  title: string;
  breakdown: VisibilityBreakdown;
  icon: ComponentType<{ className?: string }>;
  isLoading?: boolean;
}

export default function BreakdownTile({
  title,
  breakdown,
  icon: Icon,
  isLoading,
}: BreakdownTileProps) {
  return (
    <Card role="region" aria-label={`${title} breakdown by visibility`}>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-4">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary" />
          </div>
        ) : (
          <div className="space-y-2" role="list" aria-label="Visibility breakdown">
            <div
              className="flex items-center justify-between text-sm"
              role="listitem"
              aria-label={`Public repositories: ${breakdown.public}`}
            >
              <div className="flex items-center gap-2">
                <Globe className="h-3.5 w-3.5 text-green-600" aria-hidden="true" />
                <span className="text-muted-foreground">Public</span>
              </div>
              <span className="font-semibold">{breakdown.public}</span>
            </div>

            <div
              className="flex items-center justify-between text-sm"
              role="listitem"
              aria-label={`Private repositories: ${breakdown.private}`}
            >
              <div className="flex items-center gap-2">
                <Lock className="h-3.5 w-3.5 text-amber-600" aria-hidden="true" />
                <span className="text-muted-foreground">Private</span>
              </div>
              <span className="font-semibold">{breakdown.private}</span>
            </div>

            <div
              className="flex items-center justify-between text-sm"
              role="listitem"
              aria-label={`Archived repositories: ${breakdown.archived}`}
            >
              <div className="flex items-center gap-2">
                <Archive className="h-3.5 w-3.5 text-gray-500" aria-hidden="true" />
                <span className="text-muted-foreground">Archived</span>
              </div>
              <span className="font-semibold">{breakdown.archived}</span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
