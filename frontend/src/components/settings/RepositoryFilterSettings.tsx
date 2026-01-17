import { useTranslation } from 'react-i18next';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Globe, Lock, Archive } from 'lucide-react';
import { useRepositoryFilters } from '@/hooks/useRepositoryFilters';

export function RepositoryFilterSettings() {
  const { t } = useTranslation(['settings']);
  const { filters, setFilters } = useRepositoryFilters();

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>{t('settings:repositories.visibility.title')}</CardTitle>
          <CardDescription>{t('settings:repositories.visibility.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Show Public Repositories */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Globe className="h-4 w-4 text-muted-foreground" />
              <div>
                <Label>{t('settings:repositories.visibility.public')}</Label>
                <p className="text-sm text-muted-foreground">
                  {t('settings:repositories.visibility.publicDescription')}
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
                <Label>{t('settings:repositories.visibility.private')}</Label>
                <p className="text-sm text-muted-foreground">
                  {t('settings:repositories.visibility.privateDescription')}
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
                <Label>{t('settings:repositories.visibility.archived')}</Label>
                <p className="text-sm text-muted-foreground">
                  {t('settings:repositories.visibility.archivedDescription')}
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
              {t('settings:repositories.visibility.bitbucketNote')}
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
