/**
 * Fleet-preview-before-first-auto-merge gate (Phase 4 UX safeguard).
 *
 * Records (in localStorage) whether the operator has run at least one fleet
 * dry-run preview. The PolicyEditor uses this to require a preview before a
 * policy is moved to Auto-merge for the FIRST time — a guardrail against
 * enabling autonomous merges without first seeing what a run would do.
 */

const STORAGE_KEY = 'ampel-fleet-previewed';

/** `true` once any fleet preview has been run on this device. */
export function hasFleetPreviewed(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === 'true';
  } catch {
    return false;
  }
}

/** Mark that a fleet preview has been run (idempotent). */
export function markFleetPreviewed(): void {
  try {
    localStorage.setItem(STORAGE_KEY, 'true');
  } catch {
    // Non-fatal: in environments without localStorage the gate simply stays closed.
  }
}
