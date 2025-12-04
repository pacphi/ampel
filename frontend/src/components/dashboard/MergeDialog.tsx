import { useState } from 'react';
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
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [strategy, setStrategy] = useState<MergeStrategy>('squash');
  const [deleteBranch, setDeleteBranch] = useState(true);

  const mergeMutation = useMutation({
    mutationFn: (request: MergeRequest) => pullRequestsApi.merge(pr.repositoryId, pr.id, request),
    onSuccess: (result) => {
      if (result.merged) {
        toast({
          title: 'PR Merged',
          description: `Successfully merged #${pr.number}: ${pr.title}`,
        });
        queryClient.invalidateQueries({ queryKey: ['dashboard'] });
        queryClient.invalidateQueries({ queryKey: ['pull-requests'] });
        onOpenChange(false);
      } else {
        toast({
          variant: 'destructive',
          title: 'Merge failed',
          description: result.message,
        });
      }
    },
    onError: (error: unknown) => {
      const axiosError = error as { response?: { data?: { error?: string } } };
      toast({
        variant: 'destructive',
        title: 'Merge failed',
        description: axiosError.response?.data?.error || 'An error occurred',
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
    { value: 'squash', label: 'Squash and merge', description: 'Combine all commits into one' },
    { value: 'merge', label: 'Create a merge commit', description: 'All commits preserved' },
    { value: 'rebase', label: 'Rebase and merge', description: 'Linear commit history' },
  ];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <GitMerge className="h-5 w-5" />
            Merge Pull Request
          </DialogTitle>
          <DialogDescription>
            #{pr.number}: {pr.title}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>Merge strategy</Label>
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
              Delete branch after merge
            </Label>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleMerge}
            disabled={mergeMutation.isPending}
            className="bg-ampel-green hover:bg-ampel-green/90"
          >
            {mergeMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                Merging...
              </>
            ) : (
              <>
                <GitMerge className="h-4 w-4 mr-2" />
                Merge PR
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
