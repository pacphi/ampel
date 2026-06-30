/// <reference types="vite/client" />

// Injected at build time by Vite `define` (see vite.config.ts).
declare const __APP_VERSION__: string;
declare const __GIT_SHA__: string;

interface ImportMetaEnv {
  readonly VITE_API_URL: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
