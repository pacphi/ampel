import { useState } from 'react';
import type { ProviderAccount } from '@/types/account';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { GithubIcon, GitlabIcon, BitbucketIcon } from '@/components/icons/ProviderIcons';
import {
  CheckCircle2,
  XCircle,
  AlertCircle,
  Clock,
  Star,
  Pencil,
  Trash2,
  RefreshCw,
} from 'lucide-react';

interface AccountCardProps {
  account: ProviderAccount;
  onEdit: (account: ProviderAccount) => void;
  onDelete: (account: ProviderAccount) => void;
  onValidate: (account: ProviderAccount) => void;
  onSetDefault: (account: ProviderAccount) => void;
  isLoading?: boolean;
}

export function AccountCard({
  account,
  onEdit,
  onDelete,
  onValidate,
  onSetDefault,
  isLoading = false,
}: AccountCardProps) {
  const [isValidating, setIsValidating] = useState(false);

  const ProviderIcon =
    account.provider === 'github'
      ? GithubIcon
      : account.provider === 'gitlab'
        ? GitlabIcon
        : BitbucketIcon;

  const getStatusBadge = () => {
    switch (account.validationStatus) {
      case 'valid':
        return (
          <Badge variant="success" className="gap-1">
            <CheckCircle2 className="h-3 w-3" />
            Valid
          </Badge>
        );
      case 'invalid':
        return (
          <Badge variant="destructive" className="gap-1">
            <XCircle className="h-3 w-3" />
            Invalid
          </Badge>
        );
      case 'expired':
        return (
          <Badge variant="warning" className="gap-1">
            <AlertCircle className="h-3 w-3" />
            Expired
          </Badge>
        );
      case 'pending':
        return (
          <Badge variant="secondary" className="gap-1">
            <Clock className="h-3 w-3" />
            Pending
          </Badge>
        );
    }
  };

  const handleValidate = async () => {
    setIsValidating(true);
    try {
      await onValidate(account);
    } finally {
      setIsValidating(false);
    }
  };

  return (
    <div className="flex items-start justify-between p-4 rounded-lg border bg-card">
      <div className="flex items-start gap-4 flex-1">
        <ProviderIcon className="h-10 w-10 mt-1" />
        <div className="flex-1 space-y-2">
          <div className="flex items-start justify-between gap-4">
            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <h3 className="font-medium">{account.accountLabel}</h3>
                {account.isDefault && (
                  <Badge variant="outline" className="gap-1">
                    <Star className="h-3 w-3 fill-current" />
                    Default
                  </Badge>
                )}
              </div>
              <p className="text-sm text-muted-foreground">@{account.providerUsername}</p>
              {account.instanceUrl && (
                <p className="text-xs text-muted-foreground">{account.instanceUrl}</p>
              )}
            </div>
            <div className="flex items-center gap-2">{getStatusBadge()}</div>
          </div>

          <div className="flex items-center gap-4 text-sm text-muted-foreground">
            <span>{account.repositoryCount} repositories</span>
            {account.lastValidatedAt && (
              <span>
                Validated{' '}
                {new Date(account.lastValidatedAt).toLocaleDateString('en-US', {
                  month: 'short',
                  day: 'numeric',
                })}
              </span>
            )}
            {account.tokenExpiresAt && (
              <span>
                Expires{' '}
                {new Date(account.tokenExpiresAt).toLocaleDateString('en-US', {
                  month: 'short',
                  day: 'numeric',
                  year: 'numeric',
                })}
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2 ml-4">
        <Button
          variant="ghost"
          size="sm"
          onClick={handleValidate}
          disabled={isLoading || isValidating}
          title="Validate token"
        >
          <RefreshCw className={`h-4 w-4 ${isValidating ? 'animate-spin' : ''}`} />
        </Button>
        {!account.isDefault && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => onSetDefault(account)}
            disabled={isLoading}
            title="Set as default"
          >
            <Star className="h-4 w-4" />
          </Button>
        )}
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onEdit(account)}
          disabled={isLoading}
          title="Edit account"
        >
          <Pencil className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onDelete(account)}
          disabled={isLoading}
          title="Delete account"
        >
          <Trash2 className="h-4 w-4 text-destructive" />
        </Button>
      </div>
    </div>
  );
}
