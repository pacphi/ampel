import { execFileSync } from 'child_process';
import { readFileSync } from 'fs';
import path from 'path';

/**
 * Build-time version values for Vite/Vitest `define`.
 *
 * Shared by vite.config.ts AND vitest.config.ts so the build and the test runner
 * derive the SAME version + SHA — any drift between the two would make tests
 * green while the shipped build differs. The version is read from package.json
 * (release-please's source of truth); the short git SHA is best-effort (falls
 * back to "unknown" when git is unavailable, e.g. a tarball build).
 *
 * Returns the raw values; each config maps them onto the `__APP_VERSION__` /
 * `__GIT_SHA__` define tokens itself (keeping those tokens visible at the
 * injection site).
 */
export function versionInfo(rootDir: string): { appVersion: string; gitSha: string } {
  const pkg = JSON.parse(readFileSync(path.resolve(rootDir, 'package.json'), 'utf-8')) as {
    version: string;
  };

  const gitSha = (() => {
    try {
      return execFileSync('git', ['rev-parse', '--short', 'HEAD']).toString().trim();
    } catch {
      return 'unknown';
    }
  })();

  return { appVersion: pkg.version, gitSha };
}
