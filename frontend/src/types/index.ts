export type AmpelStatus = 'green' | 'yellow' | 'red' | 'none';

export type GitProvider = 'github' | 'gitlab' | 'bitbucket';

export interface User {
  id: string;
  email: string;
  displayName?: string;
  avatarUrl?: string;
  createdAt: string;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  tokenType: string;
  expiresIn: number;
}

export interface Repository {
  id: string;
  userId: string;
  provider: GitProvider;
  providerId: string;
  owner: string;
  name: string;
  fullName: string;
  description?: string;
  url: string;
  defaultBranch: string;
  isPrivate: boolean;
  isArchived: boolean;
  pollIntervalSeconds: number;
  lastPolledAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface RepositoryWithStatus extends Repository {
  status: AmpelStatus;
  openPrCount: number;
}

export interface DiscoveredRepository {
  provider: GitProvider;
  providerId: string;
  owner: string;
  name: string;
  fullName: string;
  description?: string;
  url: string;
  defaultBranch: string;
  isPrivate: boolean;
  isArchived: boolean;
}

export interface PullRequest {
  id: string;
  repositoryId: string;
  provider: GitProvider;
  providerId: string;
  number: number;
  title: string;
  description?: string;
  url: string;
  state: 'open' | 'closed' | 'merged';
  sourceBranch: string;
  targetBranch: string;
  author: string;
  authorAvatarUrl?: string;
  isDraft: boolean;
  isMergeable?: boolean;
  hasConflicts: boolean;
  additions: number;
  deletions: number;
  changedFiles: number;
  commitsCount: number;
  commentsCount: number;
  createdAt: string;
  updatedAt: string;
  mergedAt?: string;
  closedAt?: string;
}

export interface PullRequestWithDetails extends PullRequest {
  status: AmpelStatus;
  ciChecks: CICheck[];
  reviews: Review[];
  repositoryName: string;
  repositoryOwner: string;
}

export interface CICheck {
  id: string;
  pullRequestId: string;
  name: string;
  status: 'queued' | 'in_progress' | 'completed';
  conclusion?:
    | 'success'
    | 'failure'
    | 'neutral'
    | 'cancelled'
    | 'skipped'
    | 'timed_out'
    | 'action_required';
  url?: string;
  startedAt?: string;
  completedAt?: string;
  durationSeconds?: number;
}

export interface Review {
  id: string;
  pullRequestId: string;
  reviewer: string;
  reviewerAvatarUrl?: string;
  state: 'approved' | 'changes_requested' | 'commented' | 'pending' | 'dismissed';
  body?: string;
  submittedAt: string;
}

export interface ProviderConnection {
  id: string;
  provider: GitProvider;
  providerUsername: string;
  scopes: string[];
  tokenExpiresAt?: string;
  createdAt: string;
}

export interface DashboardSummary {
  totalRepositories: number;
  totalOpenPrs: number;
  statusCounts: {
    green: number;
    yellow: number;
    red: number;
  };
  providerCounts: {
    github: number;
    gitlab: number;
    bitbucket: number;
  };
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}
