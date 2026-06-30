/**
 * App version display helper.
 *
 * The version and short git SHA are injected at build time by Vite `define`
 * (see vite.config.ts) — the version is read from package.json, which is
 * release-please's source of truth, so release bumps flow through automatically
 * with no edits here.
 */

/**
 * Pure formatter for the version string. Kept separate from `displayVersion`
 * so both the production (release) and dev (pre-release) branches are testable
 * without stubbing `import.meta.env`.
 *
 * - Production: a clean SemVer release, e.g. `vX.Y.Z`.
 * - Dev: a SemVer pre-release with build metadata, e.g. `vX.Y.Z-dev+<sha>`
 *   (pre-release per SemVer §9, SHA as build metadata per §10). A pre-release
 *   sorts BELOW the release, the correct signal that a dev build is ahead of —
 *   but not yet — the next release.
 */
export function formatVersion(version: string, sha: string, isProd: boolean): string {
  if (isProd) {
    return `v${version}`;
  }
  return sha ? `v${version}-dev+${sha}` : `v${version}-dev`;
}

/** The version string to display in the UI for the current build. */
export function displayVersion(): string {
  return formatVersion(__APP_VERSION__, __GIT_SHA__, import.meta.env.PROD);
}
