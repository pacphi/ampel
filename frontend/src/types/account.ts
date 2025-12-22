import type { GitProvider } from './index';

export type ValidationStatus = 'pending' | 'valid' | 'invalid' | 'expired';

export interface ProviderAccount {
  id: string;
  provider: GitProvider;
  instanceUrl: string | null;
  accountLabel: string;
  providerUsername: string;
  providerEmail: string | null;
  avatarUrl: string | null;
  scopes: string[];
  tokenExpiresAt: string | null;
  validationStatus: ValidationStatus;
  lastValidatedAt: string | null;
  isActive: boolean;
  isDefault: boolean;
  repositoryCount: number;
  createdAt: string;
}

export interface AddAccountRequest {
  provider: GitProvider;
  instanceUrl?: string;
  accountLabel: string;
  accessToken: string;
  username?: string; // Required for Bitbucket
}

export interface UpdateAccountRequest {
  accountLabel?: string;
  accessToken?: string;
  isActive?: boolean;
  isDefault?: boolean;
}

export interface ValidateAccountResponse {
  isValid: boolean;
  validationStatus: ValidationStatus;
  errorMessage?: string;
}
