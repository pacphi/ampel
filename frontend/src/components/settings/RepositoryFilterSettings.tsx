import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Globe, Lock, Archive } from 'lucide-react';
import { useRepositoryFilters } from '@/hooks/useRepositoryFilters';

export function RepositoryFilterSettings() {
  const { filters, setFilters } = useRepositoryFilters();

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Repository Visibility Filters</CardTitle>
          <CardDescription>
            Control which repositories are displayed in the dashboard based on their visibility and
            status
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Show Public Repositories */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Globe className="h-4 w-4 text-muted-foreground" />
              <div>
                <Label>Show public repositories</Label>
                <p className="text-sm text-muted-foreground">
                  Display repositories that are publicly accessible
                </p>
              </div>
            </div>
            <Switch
              checked={filters.includePublic}
              onCheckedChange={(checked) =>
                setFilters({
                  ...filters,
                  includePublic: checked,
                })
              }
            />
          </div>

          {/* Show Private Repositories */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Lock className="h-4 w-4 text-muted-foreground" />
              <div>
                <Label>Show private repositories</Label>
                <p className="text-sm text-muted-foreground">
                  Display repositories with restricted access
                </p>
              </div>
            </div>
            <Switch
              checked={filters.includePrivate}
              onCheckedChange={(checked) =>
                setFilters({
                  ...filters,
                  includePrivate: checked,
                })
              }
            />
          </div>

          {/* Show Archived Repositories */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Archive className="h-4 w-4 text-muted-foreground" />
              <div>
                <Label>Show archived repositories</Label>
                <p className="text-sm text-muted-foreground">
                  Display repositories that have been archived
                </p>
              </div>
            </div>
            <Switch
              checked={filters.includeArchived}
              onCheckedChange={(checked) =>
                setFilters({
                  ...filters,
                  includeArchived: checked,
                })
              }
            />
          </div>

          {/* Note about Bitbucket */}
          <div className="mt-6 rounded-md bg-muted p-4">
            <p className="text-sm text-muted-foreground">
              <strong>Note:</strong> Bitbucket does not support the archive feature. All Bitbucket
              repositories are treated as non-archived.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
