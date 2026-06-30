import { describe, expect, it } from 'vitest';
import { displayVersion, formatVersion } from './version';

describe('version', () => {
  describe('formatVersion', () => {
    it('returns a clean SemVer release in production', () => {
      expect(formatVersion('0.5.1', 'a1b2c3d', true)).toBe('v0.5.1');
    });

    it('returns a SemVer pre-release with build metadata in dev', () => {
      expect(formatVersion('0.5.1', 'a1b2c3d', false)).toBe('v0.5.1-dev+a1b2c3d');
    });

    it('omits build metadata in dev when no SHA is available', () => {
      expect(formatVersion('0.5.1', '', false)).toBe('v0.5.1-dev');
    });

    it('production form carries no prerelease tag', () => {
      expect(formatVersion('1.2.3', 'deadbee', true)).not.toContain('-dev');
    });

    it('dev form carries a prerelease tag', () => {
      expect(formatVersion('1.2.3', 'deadbee', false)).toContain('-dev');
    });
  });

  describe('displayVersion', () => {
    it('renders the build-time version in the DEV pre-release form under the test runner', () => {
      // __APP_VERSION__ / __GIT_SHA__ are substituted by `define`; tests run in
      // DEV mode (import.meta.env.PROD === false). Anchor the `-dev` pre-release
      // marker so a flipped PROD/DEV switch (e.g. passing import.meta.env.DEV, or
      // an inverted ternary) is actually caught — not just the version prefix.
      expect(displayVersion()).toMatch(/^v\d+\.\d+\.\d+-dev/);
    });
  });
});
