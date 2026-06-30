/// <reference types="vitest" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import { versionInfo } from './build/version';

// Version + short git SHA, derived once (see build/version.ts) and shared with
// vitest.config.ts so build and tests inject identical values.
const { appVersion, gitSha } = versionInfo(__dirname);

export default defineConfig({
  plugins: [react()],
  // Inject the version at build time, sourced from package.json (release-please
  // source of truth) — so release bumps flow through with no edits here.
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
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
