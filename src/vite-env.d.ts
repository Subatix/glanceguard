/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Sentry frontend DSN; only used when telemetry is enabled in Settings. */
  readonly VITE_GLANCEGUARD_SENTRY_DSN: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
