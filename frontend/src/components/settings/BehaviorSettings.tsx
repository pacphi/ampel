import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
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
        title: 'Settings updated',
        description: 'Your behavior settings have been saved.',
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
          <CardTitle>Merge Behavior</CardTitle>
          <CardDescription>Configure default settings for merging pull requests</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Default Merge Strategy */}
          <div className="space-y-2">
            <Label>Default Merge Strategy</Label>
            <p className="text-sm text-muted-foreground mb-2">
              The default strategy used when merging PRs
            </p>
            <Select
              value={settings?.defaultMergeStrategy || 'squash'}
              onValueChange={(value) => handleUpdate({ defaultMergeStrategy: value })}
            >
              <SelectTrigger className="w-[200px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="squash">Squash and merge</SelectItem>
                <SelectItem value="merge">Create a merge commit</SelectItem>
                <SelectItem value="rebase">Rebase and merge</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Merge Delay */}
          <div className="space-y-2">
            <Label>Merge Delay (seconds)</Label>
            <p className="text-sm text-muted-foreground mb-2">
              Delay between consecutive merges in the same repository to avoid conflicts
            </p>
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
              <Label>Delete branches after merge</Label>
              <p className="text-sm text-muted-foreground">
                Automatically delete source branches after successful merge
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
              <Label>Require approval before merge</Label>
              <p className="text-sm text-muted-foreground">
                Show confirmation dialog before executing bulk merges
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
              <Label>Allow merge without reviews</Label>
              <p className="text-sm text-muted-foreground">
                Skip the review requirement and allow merging PRs without approvals
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
