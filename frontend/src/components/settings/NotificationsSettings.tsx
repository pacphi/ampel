import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { settingsApi, type UpdateNotificationPreferencesRequest } from '@/api/settings';
import { useToast } from '@/components/ui/use-toast';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Plus, X, Send, Mail, MessageSquare } from 'lucide-react';

export function NotificationsSettings() {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [newToEmail, setNewToEmail] = useState('');

  const { data: prefs, isLoading } = useQuery({
    queryKey: ['notification-preferences'],
    queryFn: () => settingsApi.getNotifications(),
  });

  const updateMutation = useMutation({
    mutationFn: (data: UpdateNotificationPreferencesRequest) =>
      settingsApi.updateNotifications(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notification-preferences'] });
      toast({
        title: 'Settings updated',
        description: 'Your notification settings have been saved.',
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

  const testSlackMutation = useMutation({
    mutationFn: () => settingsApi.testSlack(),
    onSuccess: () => {
      toast({
        title: 'Test successful',
        description: 'Slack test message sent successfully!',
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Test failed',
        description: axiosError.response?.data?.error || 'Failed to send test message',
      });
    },
  });

  const testEmailMutation = useMutation({
    mutationFn: () => settingsApi.testEmail(),
    onSuccess: () => {
      toast({
        title: 'Test successful',
        description: 'Test email sent successfully!',
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Test failed',
        description: axiosError.response?.data?.error || 'Failed to send test email',
      });
    },
  });

  const handleUpdate = (updates: UpdateNotificationPreferencesRequest) => {
    updateMutation.mutate(updates);
  };

  const addToEmail = () => {
    if (!newToEmail.trim() || !prefs) return;
    const updated = [...(prefs.smtpToEmails || []), newToEmail.trim()];
    updateMutation.mutate({ smtpToEmails: updated });
    setNewToEmail('');
  };

  const removeToEmail = (email: string) => {
    if (!prefs) return;
    const updated = (prefs.smtpToEmails || []).filter((e) => e !== email);
    updateMutation.mutate({ smtpToEmails: updated });
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
      {/* Slack Settings */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <MessageSquare className="h-5 w-5" />
            <CardTitle>Slack Notifications</CardTitle>
          </div>
          <CardDescription>Receive notifications via Slack webhook</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label>Enable Slack notifications</Label>
            <Switch
              checked={prefs?.slackEnabled || false}
              onCheckedChange={(checked) => handleUpdate({ slackEnabled: checked })}
            />
          </div>

          <div className="space-y-2">
            <Label>Webhook URL</Label>
            <Input
              type="url"
              placeholder="https://hooks.slack.com/services/..."
              value={prefs?.slackWebhookUrl || ''}
              onChange={(e) => handleUpdate({ slackWebhookUrl: e.target.value })}
            />
          </div>

          <div className="space-y-2">
            <Label>Channel (optional)</Label>
            <Input
              placeholder="#deployments"
              value={prefs?.slackChannel || ''}
              onChange={(e) => handleUpdate({ slackChannel: e.target.value })}
            />
            <p className="text-sm text-muted-foreground">
              Override the default channel configured in the webhook
            </p>
          </div>

          <Button
            variant="outline"
            size="sm"
            onClick={() => testSlackMutation.mutate()}
            disabled={testSlackMutation.isPending || !prefs?.slackWebhookUrl}
          >
            <Send className="h-4 w-4 mr-2" />
            {testSlackMutation.isPending ? 'Sending...' : 'Send Test Message'}
          </Button>
        </CardContent>
      </Card>

      {/* Email Settings */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Mail className="h-5 w-5" />
            <CardTitle>Email Notifications</CardTitle>
          </div>
          <CardDescription>Receive notifications via email (SMTP)</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label>Enable email notifications</Label>
            <Switch
              checked={prefs?.emailEnabled || false}
              onCheckedChange={(checked) => handleUpdate({ emailEnabled: checked })}
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>SMTP Host</Label>
              <Input
                placeholder="smtp.gmail.com"
                value={prefs?.smtpHost || ''}
                onChange={(e) => handleUpdate({ smtpHost: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>SMTP Port</Label>
              <Input
                type="number"
                placeholder="587"
                value={prefs?.smtpPort || ''}
                onChange={(e) =>
                  handleUpdate({ smtpPort: parseInt(e.target.value, 10) || undefined })
                }
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>Username</Label>
              <Input
                placeholder="user@example.com"
                value={prefs?.smtpUsername || ''}
                onChange={(e) => handleUpdate({ smtpUsername: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>Password</Label>
              <Input
                type="password"
                placeholder="Enter password"
                onChange={(e) => handleUpdate({ smtpPassword: e.target.value })}
              />
              <p className="text-xs text-muted-foreground">Password is encrypted before storage</p>
            </div>
          </div>

          <div className="space-y-2">
            <Label>From Email</Label>
            <Input
              type="email"
              placeholder="ampel@example.com"
              value={prefs?.smtpFromEmail || ''}
              onChange={(e) => handleUpdate({ smtpFromEmail: e.target.value })}
            />
          </div>

          <div className="space-y-2">
            <Label>To Emails</Label>
            <p className="text-sm text-muted-foreground mb-2">Recipients for notification emails</p>
            <div className="flex flex-wrap gap-2 mb-3">
              {prefs?.smtpToEmails?.map((email) => (
                <span
                  key={email}
                  className="inline-flex items-center gap-1 px-3 py-1 rounded-full bg-primary/10 text-sm"
                >
                  {email}
                  <button
                    onClick={() => removeToEmail(email)}
                    className="hover:text-destructive ml-1"
                    disabled={updateMutation.isPending}
                  >
                    <X className="h-3 w-3" />
                  </button>
                </span>
              ))}
            </div>
            <div className="flex gap-2">
              <Input
                type="email"
                placeholder="email@example.com"
                value={newToEmail}
                onChange={(e) => setNewToEmail(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && addToEmail()}
                className="max-w-xs"
              />
              <Button
                variant="outline"
                size="sm"
                onClick={addToEmail}
                disabled={!newToEmail.trim() || updateMutation.isPending}
              >
                <Plus className="h-4 w-4 mr-1" />
                Add
              </Button>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <Label>Use TLS</Label>
            <Switch
              checked={prefs?.smtpUseTls ?? true}
              onCheckedChange={(checked) => handleUpdate({ smtpUseTls: checked })}
            />
          </div>

          <Button
            variant="outline"
            size="sm"
            onClick={() => testEmailMutation.mutate()}
            disabled={testEmailMutation.isPending || !prefs?.smtpHost}
          >
            <Send className="h-4 w-4 mr-2" />
            {testEmailMutation.isPending ? 'Sending...' : 'Send Test Email'}
          </Button>
        </CardContent>
      </Card>

      {/* Merge Notification Triggers */}
      <Card>
        <CardHeader>
          <CardTitle>Merge Notifications</CardTitle>
          <CardDescription>
            Choose when to receive notifications for merge operations
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>Notify on successful merges</Label>
              <p className="text-sm text-muted-foreground">
                Send notification when PRs are successfully merged
              </p>
            </div>
            <Switch
              checked={prefs?.notifyOnMergeSuccess ?? true}
              onCheckedChange={(checked) => handleUpdate({ notifyOnMergeSuccess: checked })}
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <Label>Notify on failed merges</Label>
              <p className="text-sm text-muted-foreground">
                Send notification when merge operations fail
              </p>
            </div>
            <Switch
              checked={prefs?.notifyOnMergeFailure ?? true}
              onCheckedChange={(checked) => handleUpdate({ notifyOnMergeFailure: checked })}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
