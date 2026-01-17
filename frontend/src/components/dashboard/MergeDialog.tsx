import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { pullRequestsApi, type MergeRequest } from '@/api/pullRequests';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { useToast } from '@/components/ui/use-toast';
import type { PullRequestWithDetails } from '@/types';
import { GitMerge, Loader2 } from 'lucide-react';

interface MergeDialogProps {
  pr: PullRequestWithDetails;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

type MergeStrategy = 'merge' | 'squash' | 'rebase';

export default function MergeDialog({ pr, open, onOpenChange }: MergeDialogProps) {
  const { t } = useTranslation(['merge', 'common']);
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [strategy, setStrategy] = useState<MergeStrategy>('squash');
  const [deleteBranch, setDeleteBranch] = useState(true);

  const mergeMutation = useMutation({
    mutationFn: (request: MergeRequest) => pullRequestsApi.merge(pr.repositoryId, pr.id, request),
    onSuccess: (result) => {
      if (result.merged) {
        toast({
          title: t('merge:toast.success'),
          description: t('merge:toast.successDescription', { number: pr.number, title: pr.title }),
        });
        queryClient.invalidateQueries({ queryKey: ['dashboard'] });
        queryClient.invalidateQueries({ queryKey: ['pull-requests'] });
        onOpenChange(false);
      } else {
        toast({
          variant: 'destructive',
          title: t('merge:toast.failed'),
          description: t('merge:toast.failedDescription', {
            number: pr.number,
            error: result.message,
          }),
        });
      }
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: t('merge:toast.failed'),
        description: t('merge:toast.failedDescription', {
          number: pr.number,
          error: axiosError.response?.data?.error || 'An error occurred',
        }),
      });
    },
  });

  const handleMerge = () => {
    mergeMutation.mutate({
      strategy,
      deleteBranch,
    });
  };

  const strategies: { value: MergeStrategy; label: string; description: string }[] = [
    {
      value: 'squash',
      label: t('merge:dialog.strategy.squash'),
      description: t('merge:dialog.strategy.squashDescription'),
    },
    {
      value: 'merge',
      label: t('merge:dialog.strategy.merge'),
      description: t('merge:dialog.strategy.mergeDescription'),
    },
    {
      value: 'rebase',
      label: t('merge:dialog.strategy.rebase'),
      description: t('merge:dialog.strategy.rebaseDescription'),
    },
  ];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <GitMerge className="h-5 w-5" />
            {t('merge:dialog.title')}
          </DialogTitle>
          <DialogDescription>
            #{pr.number}: {pr.title}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>{t('merge:dialog.strategy.title')}</Label>
            <div className="space-y-2">
              {strategies.map((s) => (
                <label
                  key={s.value}
                  className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                    strategy === s.value
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:border-primary/50'
                  }`}
                >
                  <input
                    type="radio"
                    name="strategy"
                    value={s.value}
                    checked={strategy === s.value}
                    onChange={() => setStrategy(s.value)}
                    className="mt-1"
                  />
                  <div>
                    <div className="font-medium">{s.label}</div>
                    <div className="text-sm text-muted-foreground">{s.description}</div>
                  </div>
                </label>
              ))}
            </div>
          </div>

          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="deleteBranch"
              checked={deleteBranch}
              onChange={(e) => setDeleteBranch(e.target.checked)}
              className="h-4 w-4"
            />
            <Label htmlFor="deleteBranch" className="cursor-pointer">
              {t('merge:dialog.deleteBranch')}
            </Label>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('merge:dialog.cancel')}
          </Button>
          <Button
            onClick={handleMerge}
            disabled={mergeMutation.isPending}
            className="bg-ampel-green hover:bg-ampel-green/90"
          >
            {mergeMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                {t('merge:dialog.merging')}
              </>
            ) : (
              <>
                <GitMerge className="h-4 w-4 mr-2" />
                {t('merge:dialog.merge')}
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
