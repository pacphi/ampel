/**
 * Remediation playbook DTOs (Phase 4 — ADR-006).
 *
 * Field naming matches the backend JSON contract (serde `rename_all = "camelCase"`).
 * Playbooks are DB-stored YAML overrides of the embedded default remediation
 * playbook. The `preview` endpoint renders the assembled prompt with NO model
 * call, so an operator can lint a playbook safely.
 */

import type { ScopeType } from './remediation';

export interface Playbook {
  id: string;
  playbookId: string;
  version: number;
  source: string;
  name: string;
  description: string | null;
  /** YAML playbook body. */
  content: string;
  enabled: boolean;
  scopeType: string;
  scopeId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreatePlaybookRequest {
  playbookId: string;
  version?: number;
  name: string;
  description?: string;
  content: string;
  source?: string;
  enabled?: boolean;
  scopeType?: ScopeType;
  scopeId?: string;
}

export interface UpdatePlaybookRequest {
  name?: string;
  description?: string;
  content?: string;
  enabled?: boolean;
}

export interface PlaybookPreviewRequest {
  failureClass?: string;
  repoFullName?: string;
  baseBranch?: string;
}

export interface PlaybookPreviewResponse {
  failureClass: string;
  role: string;
  /** The fully assembled, prompt-injection-safe trusted `system` instruction. */
  systemInstruction: string;
  outputContract: string;
  /** Tools after the ADR-006 ceiling clamp (an override can only remove). */
  allowedTools: string[];
}
