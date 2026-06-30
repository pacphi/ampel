import { displayVersion } from '@/lib/version';

/**
 * App footer — shows the build version unobtrusively in the bottom-left.
 *
 * In dev it renders the SemVer pre-release form (vX.Y.Z-dev+<sha>); in a
 * production build the clean release (vX.Y.Z) — see src/lib/version.ts. The
 * version string is self-explanatory, so no i18n label is rendered.
 */
export default function Footer() {
  return (
    <footer className="flex h-8 shrink-0 items-center border-t bg-card px-6">
      <span className="text-xs text-muted-foreground" data-testid="app-version">
        {displayVersion()}
      </span>
    </footer>
  );
}
