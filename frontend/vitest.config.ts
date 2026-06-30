import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';
import { execFileSync } from 'child_process';
import { readFileSync } from 'fs';

// Mirror vite.config.ts's build-time version injection so __APP_VERSION__ /
// __GIT_SHA__ resolve under the test runner too (vitest uses THIS config, not
// vite.config.ts). Version comes from package.json (release-please source of truth).
const pkg = JSON.parse(readFileSync(path.resolve(__dirname, 'package.json'), 'utf-8')) as {
  version: string;
};
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
  test: {
    // Use jsdom environment for React component testing
    environment: 'jsdom',

    // Global test setup
    globals: true,

    // Setup files to run before tests
    setupFiles: ['./tests/setup.ts'],

    // Coverage configuration
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      exclude: [
        'node_modules/',
        'tests/',
        '**/*.d.ts',
        '**/*.config.*',
        '**/mockData/**',
        'dist/',
        'src/components/ui/**', // shadcn/ui components - third-party library code
        'src/main.tsx', // Entry point - just renders the app
        'src/vite-env.d.ts',
      ],
      all: true,
      lines: 80,
      functions: 75,
      branches: 75,
      statements: 80,
    },

    // Test isolation
    isolate: true,

    // Parallel execution
    threads: true,
    maxThreads: 4,
    minThreads: 1,

    // Test timeout - CI environments need longer timeouts due to shared resources
    testTimeout: process.env.CI ? 30000 : 10000,
    hookTimeout: process.env.CI ? 20000 : 10000,

    // File patterns
    include: ['src/**/*.{test,spec}.{js,ts,jsx,tsx}', 'tests/**/*.{test,spec}.{js,ts,jsx,tsx}'],
    exclude: ['node_modules', 'dist', '.idea', '.git', '.cache'],

    // Mock reset behavior
    clearMocks: true,
    mockReset: true,
    restoreMocks: true,

    // Reporter configuration
    reporters: process.env.CI ? ['verbose', 'junit'] : ['verbose'],
    outputFile: {
      junit: './test-results/junit.xml',
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/lib': path.resolve(__dirname, './src/lib'),
      '@/hooks': path.resolve(__dirname, './src/hooks'),
      '@/api': path.resolve(__dirname, './src/api'),
      '@/types': path.resolve(__dirname, './src/types'),
      '@/tests': path.resolve(__dirname, './tests'),
    },
  },
});
