/**
 * Fleet Remediation DTOs (Phase 1 — Policy CRUD + Dry-Run).
 *
 * Field naming matches the backend JSON contract:
 *  - Policy/Fleet object fields are camelCase (serde `rename_all = "camelCase"`).
 *  - Enum *values* are snake_case (serde `rename_all = "snake_case"`).
 *  - `ConsolidationPlan` fields are snake_case (no serde rename on that struct).
 */

export type ScopeType = 'repository' | 'team' | 'org' | 'user';

export type AutonomyLevel =
  | 'dry_run_only'
  | 'suggest_only'
  | 'auto_with_approval'
  | 'fully_autonomous';

export type RemediationTier = 'consolidate_only' | 'fix_and_consolidate' | 'full_remediation';

export type PolicyState =
  | 'none'
  | 'disabled'
  | 'dry_run'
  | 'suggest'
  | 'auto_with_approval'
  | 'auto_merge';

/**
 * Strategy for choosing which open PRs a run operates on. Externally tagged to
 * mirror the Rust `PrSelectionStrategy` enum serialization.
 */
export type PrSelectionStrategy =
  | 'all_open'
  | { oldest_first: { max: number } }
  | { by_label: { labels: string[] } }
  | { explicit_ids: { ids: number[] } };

export interface RemediationPolicy {
  id: string;
  scopeType: ScopeType;
  scopeId: string;
  enabled: boolean;
  minOpenPrs: number;
  prSelection: PrSelectionStrategy;
  autonomyLevel: AutonomyLevel;
  remediationTier: RemediationTier;
  maxPrsPerRun: number;
  allowedTargets: string[];
  skipDraft: boolean;
  requireGreenBeforeMerge: boolean;
  airGapped: boolean;
  autoMergeEnabled: boolean;
  autoMergeRule: string | null;
  requireHumanApproval: boolean;
  agentBudget: unknown | null;
  notificationConfig: unknown | null;
  playbookRef: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreatePolicyRequest {
  scopeType: ScopeType;
  scopeId: string;
  enabled?: boolean;
  minOpenPrs: number;
  prSelection?: PrSelectionStrategy;
  autonomyLevel: AutonomyLevel;
  remediationTier?: RemediationTier;
  maxPrsPerRun: number;
  allowedTargets?: string[];
  skipDraft?: boolean;
  requireGreenBeforeMerge?: boolean;
  airGapped?: boolean;
  autoMergeEnabled?: boolean;
  autoMergeRule?: string | null;
  requireHumanApproval?: boolean;
  playbookRef?: string | null;
}

export type UpdatePolicyRequest = Partial<Omit<CreatePolicyRequest, 'scopeType' | 'scopeId'>>;

export interface FleetRow {
  repositoryId: string;
  name: string;
  openPrCount: number;
  eligible: boolean;
  policyState: PolicyState;
  airGapped: boolean;
}

export interface ConsolidationPrRef {
  number: number;
  title: string;
  branch: string;
}

export interface ConsolidationPlan {
  would_select: ConsolidationPrRef[];
  pr_count: number;
  predicted_conflicts: string[];
  estimated_duration_secs: number;
  air_gapped: boolean;
  blocked_by_air_gap: boolean;
}
