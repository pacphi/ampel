/// <reference types="vitest" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import { execFileSync } from 'child_process';
import { readFileSync } from 'fs';

// Source the version from package.json — release-please's source of truth
// (frontend/package.json `version`, x-release-please-version) — so release
// bumps flow through automatically with no manual edits here.
const pkg = JSON.parse(readFileSync(path.resolve(__dirname, 'package.json'), 'utf-8')) as {
  version: string;
};

// Short git SHA as SemVer build metadata to disambiguate dev builds. Best-effort:
// fall back to "unknown" when git is unavailable (e.g. a tarball build).
const gitSha = (() => {
  try {
    return execFileSync('git', ['rev-parse', '--short', 'HEAD']).toString().trim();
  } catch {
    return 'unknown';
  }
})();

export default defineConfig({
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
    __GIT_SHA__: JSON.stringify(gitSha),
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  test: {
    passWithNoTests: true,
    environment: 'jsdom',
    coverage: {
      provider: 'v8',
    },
  },
});
