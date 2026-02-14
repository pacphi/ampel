import { useTranslation } from 'react-i18next';
import { Progress } from '@/components/ui/progress';
import { CheckCircle2, Loader2 } from 'lucide-react';
import type { RefreshJobStatus } from '@/types';

interface RefreshProgressProps {
  status: RefreshJobStatus;
}

export default function RefreshProgress({ status }: RefreshProgressProps) {
  const { t } = useTranslation(['dashboard', 'common']);

  const progressPercentage =
    status.totalRepositories > 0 ? (status.completed / status.totalRepositories) * 100 : 0;

  return (
    <div className="space-y-4">
      {/* Progress Bar */}
      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm">
          <span className="text-muted-foreground">
            {t('dashboard:refresh.progress', {
              completed: status.completed,
              total: status.totalRepositories,
            })}
          </span>
          <span className="font-medium">{Math.round(progressPercentage)}%</span>
        </div>
        <Progress value={progressPercentage} className="h-2" />
      </div>

      {/* Current Repository */}
      {status.currentRepository && !status.isComplete && (
        <div className="flex items-center gap-2 text-sm">
          <Loader2 className="h-4 w-4 animate-spin text-primary" />
          <span className="text-muted-foreground">
            {t('dashboard:refresh.currentRepository')}:{' '}
            <span className="font-medium text-foreground">{status.currentRepository}</span>
          </span>
        </div>
      )}

      {/* Completion Status */}
      {status.isComplete && (
        <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
          <CheckCircle2 className="h-4 w-4" />
          <span className="font-medium">{t('dashboard:refresh.complete')}</span>
        </div>
      )}

      {/* Time Information */}
      <div className="text-xs text-muted-foreground">
        {t('dashboard:refresh.started')}: {new Date(status.startedAt).toLocaleTimeString()}
        {status.completedAt && (
          <>
            {' • '}
            {t('dashboard:refresh.completed')}: {new Date(status.completedAt).toLocaleTimeString()}
          </>
        )}
      </div>
    </div>
  );
}
