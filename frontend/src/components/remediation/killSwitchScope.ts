import type { RemediationPolicy, ScopeType } from '@/types/remediation';

/** Broadest scope wins — that policy acts as the fleet-wide kill-switch. */
const SCOPE_PRIORITY: Record<ScopeType, number> = {
  org: 0,
  team: 1,
  repository: 2,
  user: 3,
};

/** Pick the top-scope policy (broadest scope, tie-broken by oldest). */
export function selectTopScopePolicy(
  policies: RemediationPolicy[] | undefined
): RemediationPolicy | undefined {
  if (!policies || policies.length === 0) return undefined;
  return [...policies].sort((a, b) => {
    const byScope = SCOPE_PRIORITY[a.scopeType] - SCOPE_PRIORITY[b.scopeType];
    if (byScope !== 0) return byScope;
    return a.createdAt.localeCompare(b.createdAt);
  })[0];
}
