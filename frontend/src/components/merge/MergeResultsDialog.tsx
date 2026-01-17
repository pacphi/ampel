import { useTranslation } from 'react-i18next';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Check, X, AlertCircle } from 'lucide-react';
import type { BulkMergeResponse } from '@/api/merge';

interface MergeResultsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  results: BulkMergeResponse | null;
}

export function MergeResultsDialog({ open, onOpenChange, results }: MergeResultsDialogProps) {
  const { t } = useTranslation(['merge']);

  if (!results) return null;

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'success':
        return <Check className="h-4 w-4 text-green-500" />;
      case 'failed':
        return <X className="h-4 w-4 text-red-500" />;
      case 'skipped':
        return <AlertCircle className="h-4 w-4 text-yellow-500" />;
      default:
        return null;
    }
  };

  const getStatusBadgeClass = (status: string) => {
    switch (status) {
      case 'success':
        return 'bg-green-500/10 text-green-700';
      case 'failed':
        return 'bg-red-500/10 text-red-700';
      case 'skipped':
        return 'bg-yellow-500/10 text-yellow-700';
      default:
        return 'bg-muted text-muted-foreground';
    }
  };

  const getStatusLabel = (status: string) => {
    switch (status) {
      case 'success':
        return t('merge:results.status.success');
      case 'failed':
        return t('merge:results.status.failed');
      case 'skipped':
        return t('merge:results.status.skipped');
      default:
        return status;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t('merge:results.title')}</DialogTitle>
          <DialogDescription>
            {results.status === 'completed' && results.failed === 0
              ? t('merge:results.allSuccess')
              : t('merge:results.summary', { success: results.success, failed: results.failed, skipped: results.skipped })}
          </DialogDescription>
        </DialogHeader>

        {/* Summary */}
        <div className="grid grid-cols-4 gap-4 py-4">
          <div className="text-center p-3 rounded-lg bg-muted">
            <div className="text-2xl font-bold">{results.total}</div>
            <div className="text-sm text-muted-foreground">{t('merge:results.stats.total')}</div>
          </div>
          <div className="text-center p-3 rounded-lg bg-green-500/10">
            <div className="text-2xl font-bold text-green-700">{results.success}</div>
            <div className="text-sm text-green-600">{t('merge:results.stats.merged')}</div>
          </div>
          <div className="text-center p-3 rounded-lg bg-red-500/10">
            <div className="text-2xl font-bold text-red-700">{results.failed}</div>
            <div className="text-sm text-red-600">{t('merge:results.stats.failed')}</div>
          </div>
          <div className="text-center p-3 rounded-lg bg-yellow-500/10">
            <div className="text-2xl font-bold text-yellow-700">{results.skipped}</div>
            <div className="text-sm text-yellow-600">{t('merge:results.stats.skipped')}</div>
          </div>
        </div>

        {/* Results List */}
        <div className="space-y-2">
          {results.results.map((result, index) => (
            <div key={index} className="flex items-start gap-3 p-3 rounded-lg border">
              {getStatusIcon(result.status)}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-medium truncate">{result.prTitle}</span>
                  <span className="text-sm text-muted-foreground">#{result.prNumber}</span>
                </div>
                <div className="text-sm text-muted-foreground">{result.repositoryName}</div>
                {result.errorMessage && (
                  <div className="text-sm text-red-600 mt-1">{result.errorMessage}</div>
                )}
                {result.mergeSha && (
                  <div className="text-xs text-muted-foreground mt-1 font-mono">
                    {result.mergeSha.substring(0, 8)}
                  </div>
                )}
              </div>
              <span
                className={`text-xs px-2 py-1 rounded capitalize ${getStatusBadgeClass(
                  result.status
                )}`}
              >
                {getStatusLabel(result.status)}
              </span>
            </div>
          ))}
        </div>

        <div className="flex justify-end pt-4">
          <Button onClick={() => onOpenChange(false)}>{t('merge:results.close')}</Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
