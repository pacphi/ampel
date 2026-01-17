import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { settingsApi, type UpdateUserSettingsRequest } from '@/api/settings';
import { useToast } from '@/components/ui/use-toast';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Slider } from '@/components/ui/slider';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

export function BehaviorSettings() {
  const { t } = useTranslation(['behavior']);
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const { data: settings, isLoading } = useQuery({
    queryKey: ['user-settings', 'behavior'],
    queryFn: () => settingsApi.getBehavior(),
  });

  const updateMutation = useMutation({
    mutationFn: (data: UpdateUserSettingsRequest) => settingsApi.updateBehavior(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-settings', 'behavior'] });
      toast({
        title: t('behavior:toast.updated'),
        description: t('behavior:toast.updatedDescription'),
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Failed to update settings',
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const handleUpdate = (updates: UpdateUserSettingsRequest) => {
    updateMutation.mutate(updates);
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-8">
          <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>{t('behavior:title')}</CardTitle>
          <CardDescription>{t('behavior:strategy.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Default Merge Strategy */}
          <div className="space-y-2">
            <Label>{t('behavior:strategy.title')}</Label>
            <p className="text-sm text-muted-foreground mb-2">
              {t('behavior:strategy.squashDescription')}
            </p>
            <Select
              value={settings?.defaultMergeStrategy || 'squash'}
              onValueChange={(value) => handleUpdate({ defaultMergeStrategy: value })}
            >
              <SelectTrigger className="w-[200px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="squash">{t('behavior:strategy.squash')}</SelectItem>
                <SelectItem value="merge">{t('behavior:strategy.merge')}</SelectItem>
                <SelectItem value="rebase">{t('behavior:strategy.rebase')}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Merge Delay */}
          <div className="space-y-2">
            <Label>{t('behavior:delay.title')}</Label>
            <p className="text-sm text-muted-foreground mb-2">{t('behavior:delay.description')}</p>
            <div className="flex items-center gap-4">
              <Slider
                value={[settings?.mergeDelaySeconds || 5]}
                onValueChange={([value]) => handleUpdate({ mergeDelaySeconds: value })}
                min={0}
                max={60}
                step={5}
                className="w-[200px]"
              />
              <span className="text-sm text-muted-foreground min-w-[60px]">
                {settings?.mergeDelaySeconds || 5}s
              </span>
            </div>
          </div>

          {/* Delete Branches */}
          <div className="flex items-center justify-between">
            <div>
              <Label>{t('behavior:options.deleteBranch')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('behavior:options.deleteBranchDescription')}
              </p>
            </div>
            <Switch
              checked={settings?.deleteBranchesDefault || false}
              onCheckedChange={(checked) => handleUpdate({ deleteBranchesDefault: checked })}
            />
          </div>

          {/* Require Approval */}
          <div className="flex items-center justify-between">
            <div>
              <Label>{t('behavior:options.requireApproval')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('behavior:options.requireApprovalDescription')}
              </p>
            </div>
            <Switch
              checked={settings?.requireApproval || false}
              onCheckedChange={(checked) => handleUpdate({ requireApproval: checked })}
            />
          </div>

          {/* Skip Review Requirement */}
          <div className="flex items-center justify-between">
            <div>
              <Label>{t('behavior:options.allowNoReviews')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('behavior:options.allowNoReviewsDescription')}
              </p>
            </div>
            <Switch
              checked={settings?.skipReviewRequirement || false}
              onCheckedChange={(checked) => handleUpdate({ skipReviewRequirement: checked })}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
