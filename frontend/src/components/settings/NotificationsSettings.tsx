import { useState } from 'react';
import { useTranslation } from 'react-i18next';
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
  const { t } = useTranslation(['notifications']);
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
        title: t('notifications:toast.updated'),
        description: t('notifications:toast.updatedDescription'),
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('notifications:toast.updateFailed'),
        description: axiosError.response?.data?.error || 'An error occurred',
      });
    },
  });

  const testSlackMutation = useMutation({
    mutationFn: () => settingsApi.testSlack(),
    onSuccess: () => {
      toast({
        title: t('notifications:slack.testSuccess'),
        description: t('notifications:slack.testSuccess'),
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('notifications:slack.testFailed'),
        description: axiosError.response?.data?.error || t('notifications:slack.testFailed'),
      });
    },
  });

  const testEmailMutation = useMutation({
    mutationFn: () => settingsApi.testEmail(),
    onSuccess: () => {
      toast({
        title: t('notifications:email.testSuccess'),
        description: t('notifications:email.testSuccess'),
      });
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('notifications:email.testFailed'),
        description: axiosError.response?.data?.error || t('notifications:email.testFailed'),
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
            <CardTitle>{t('notifications:slack.title')}</CardTitle>
          </div>
          <CardDescription>{t('notifications:slack.enableDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label>{t('notifications:slack.enable')}</Label>
            <Switch
              checked={prefs?.slackEnabled || false}
              onCheckedChange={(checked) => handleUpdate({ slackEnabled: checked })}
            />
          </div>

          <div className="space-y-2">
            <Label>{t('notifications:slack.webhookUrl')}</Label>
            <Input
              type="url"
              placeholder={t('notifications:slack.webhookUrlPlaceholder')}
              value={prefs?.slackWebhookUrl || ''}
              onChange={(e) => handleUpdate({ slackWebhookUrl: e.target.value })}
            />
            <p className="text-sm text-muted-foreground">
              {t('notifications:slack.webhookUrlDescription')}
            </p>
          </div>

          <div className="space-y-2">
            <Label>{t('notifications:slack.channel')}</Label>
            <Input
              placeholder={t('notifications:slack.channelPlaceholder')}
              value={prefs?.slackChannel || ''}
              onChange={(e) => handleUpdate({ slackChannel: e.target.value })}
            />
            <p className="text-sm text-muted-foreground">
              {t('notifications:slack.channelDescription')}
            </p>
          </div>

          <Button
            variant="outline"
            size="sm"
            onClick={() => testSlackMutation.mutate()}
            disabled={testSlackMutation.isPending || !prefs?.slackWebhookUrl}
          >
            <Send className="h-4 w-4 mr-2" />
            {testSlackMutation.isPending
              ? t('notifications:slack.testSending')
              : t('notifications:slack.testMessage')}
          </Button>
        </CardContent>
      </Card>

      {/* Email Settings */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Mail className="h-5 w-5" />
            <CardTitle>{t('notifications:email.title')}</CardTitle>
          </div>
          <CardDescription>{t('notifications:email.enableDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label>{t('notifications:email.enable')}</Label>
            <Switch
              checked={prefs?.emailEnabled || false}
              onCheckedChange={(checked) => handleUpdate({ emailEnabled: checked })}
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>{t('notifications:email.smtpHost')}</Label>
              <Input
                placeholder={t('notifications:email.smtpHostPlaceholder')}
                value={prefs?.smtpHost || ''}
                onChange={(e) => handleUpdate({ smtpHost: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('notifications:email.smtpPort')}</Label>
              <Input
                type="number"
                placeholder={t('notifications:email.smtpPortPlaceholder')}
                value={prefs?.smtpPort || ''}
                onChange={(e) =>
                  handleUpdate({ smtpPort: parseInt(e.target.value, 10) || undefined })
                }
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>{t('notifications:email.smtpUser')}</Label>
              <Input
                placeholder={t('notifications:email.smtpUserPlaceholder')}
                value={prefs?.smtpUsername || ''}
                onChange={(e) => handleUpdate({ smtpUsername: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('notifications:email.smtpPassword')}</Label>
              <Input
                type="password"
                placeholder={t('notifications:email.smtpPasswordPlaceholder')}
                onChange={(e) => handleUpdate({ smtpPassword: e.target.value })}
              />
            </div>
          </div>

          <div className="space-y-2">
            <Label>{t('notifications:email.fromAddress')}</Label>
            <Input
              type="email"
              placeholder={t('notifications:email.fromAddressPlaceholder')}
              value={prefs?.smtpFromEmail || ''}
              onChange={(e) => handleUpdate({ smtpFromEmail: e.target.value })}
            />
          </div>

          <div className="space-y-2">
            <Label>{t('notifications:email.toAddresses')}</Label>
            <p className="text-sm text-muted-foreground mb-2">
              {t('notifications:email.toAddressesDescription')}
            </p>
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
                placeholder={t('notifications:email.toAddressesPlaceholder')}
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
                {t('notifications:email.add')}
              </Button>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <Label>{t('notifications:email.useTls')}</Label>
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
            {testEmailMutation.isPending
              ? t('notifications:email.testSending')
              : t('notifications:email.testMessage')}
          </Button>
        </CardContent>
      </Card>

      {/* Merge Notification Triggers */}
      <Card>
        <CardHeader>
          <CardTitle>{t('notifications:merge.title')}</CardTitle>
          <CardDescription>{t('notifications:merge.description')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>{t('notifications:merge.notifyOnSuccess')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('notifications:merge.notifyOnSuccessDescription')}
              </p>
            </div>
            <Switch
              checked={prefs?.notifyOnMergeSuccess ?? true}
              onCheckedChange={(checked) => handleUpdate({ notifyOnMergeSuccess: checked })}
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <Label>{t('notifications:merge.notifyOnFailure')}</Label>
              <p className="text-sm text-muted-foreground">
                {t('notifications:merge.notifyOnFailureDescription')}
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
