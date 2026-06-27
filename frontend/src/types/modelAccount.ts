/**
 * Model-provider account DTOs (Phase 4 — Agentic Remediation Tier, ADR-007/008).
 *
 * Field naming matches the backend JSON contract (serde `rename_all = "camelCase"`).
 *
 * Security: the API key is **write-only**. It is accepted on create/update via
 * `apiKey` but is NEVER returned. The response carries `hasCredentials` (a bool)
 * so the UI can show whether a key is on file without ever exposing the secret.
 */

export type ProviderKind = 'claude' | 'gemini' | 'ollama' | 'onnx';

export type EgressClass = 'external' | 'local_only';

export type ModelValidationStatus = 'unvalidated' | 'valid' | 'invalid';

export interface ModelAccount {
  id: string;
  organizationId: string | null;
  userId: string | null;
  providerKind: ProviderKind;
  displayName: string;
  endpointUrl: string | null;
  egressClass: EgressClass;
  modelId: string | null;
  modelPath: string | null;
  authType: string;
  validationStatus: ModelValidationStatus;
  spendCapUsd: string | null;
  spendUsedUsd: string;
  lastValidatedAt: string | null;
  enabled: boolean;
  isDefault: boolean;
  /** `true` if a credential is on file. The key itself is never returned. */
  hasCredentials: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateModelAccountRequest {
  providerKind: ProviderKind;
  displayName: string;
  /** Write-only hosted-API bearer key. Never echoed back by the server. */
  apiKey?: string;
  endpointUrl?: string;
  modelId?: string;
  modelPath?: string;
  organizationId?: string;
  egressClass?: EgressClass;
  spendCapUsd?: string;
  enabled?: boolean;
  isDefault?: boolean;
}

export interface UpdateModelAccountRequest {
  displayName?: string;
  /** Replacement write-only key; re-encrypted server-side, resets validation. */
  apiKey?: string;
  endpointUrl?: string;
  modelId?: string;
  modelPath?: string;
  spendCapUsd?: string;
  enabled?: boolean;
  isDefault?: boolean;
}

export interface ModelValidationResult {
  isValid: boolean;
  validationStatus: ModelValidationStatus;
  errorMessage?: string;
  lastValidatedAt: string;
}
