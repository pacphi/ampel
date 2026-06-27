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

/**
 * Phase 3 — Observability & UX DTOs (Run history/detail + SSE).
 *
 * Field naming matches the backend JSON contract (camelCase). Enum *values*
 * are snake_case where they come from serde `rename_all = "snake_case"`;
 * disposition variants are PascalCase (externally-tagged Rust enum).
 */

/** Remediation run state machine. */
export type RunState =
  | 'created'
  | 'selecting'
  | 'consolidating'
  | 'verifying'
  | 'awaiting_approval'
  | 'merging'
  | 'finalizing'
  | 'agent_fixing'
  | 'completed'
  | 'handoff_human'
  | 'failed'
  | 'cancelled'
  | 'no_op';

/** Aggregate CI status. Loosely typed — providers report varying labels. */
export type CiStatus = string;

/** Run summary (list endpoint). */
export interface RemediationRun {
  id: string;
  repositoryId: string;
  policyId: string;
  state: RunState;
  autonomyLevel: AutonomyLevel;
  consolidatedPrNumber?: number | null;
  merged: boolean;
  branchName: string;
  ciStatus: CiStatus;
  attempts: number;
  errorMessage?: string | null;
  startedAt: string;
  completedAt?: string | null;
  createdAt: string;
  updatedAt: string;
}

/**
 * Per-source-PR disposition. Externally-tagged to mirror the Rust enum:
 * the unit variant serializes as a bare string, struct variants as objects.
 */
export type Disposition =
  | 'Consolidated'
  | { ClosedWithRef: { consolidatedPrNumber: number } }
  | { SkippedConflict: { reason: string } }
  | { LeftOpen: { reason: string } };

export interface PrDisposition {
  prNumber: number;
  disposition: Disposition;
}

export interface CiMatrix {
  ciStatus: CiStatus;
  ciLogsUrl?: string | null;
  headSha: string;
  predictedConflicts: string[];
}

export interface RemediationConflict {
  prNumber: number;
  reason: string;
}

export interface RemediationSkipped {
  prNumber: number;
  reason: string;
}

export interface ConflictReport {
  conflicts: RemediationConflict[];
  skipped: RemediationSkipped[];
}

/** Run detail (single-run endpoint): summary + dispositions + matrices. */
export interface RunDetail extends RemediationRun {
  dispositions: PrDisposition[];
  ciMatrix: CiMatrix | null;
  conflictReport: ConflictReport | null;
}

export interface ListRunsFilters {
  repositoryId?: string;
  state?: RunState;
  since?: string;
  until?: string;
  limit?: number;
  offset?: number;
}

/** Short-lived token used to authenticate the EventSource SSE stream. */
export interface SseToken {
  token: string;
  expiresAt: string;
}

/** SSE: `run_state_changed`. */
export interface RunStateChangedEvent {
  runId: string;
  state: RunState;
  previousState: RunState;
  ciStatus: CiStatus;
  ts: string;
}

/** SSE: `ci_status_updated`. */
export interface CiStatusUpdatedEvent {
  runId: string;
  ciStatus: CiStatus;
}

/** SSE: `run_finished`. */
export interface RunFinishedEvent {
  runId: string;
  outcome: string;
  ts: string;
}
