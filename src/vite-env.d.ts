/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Optional default base URL for `download_models` (must serve `<sha256>.onnx`). */
  readonly VITE_SCREENPEEK_MODELS_BASE_URL: string | undefined;
  /** Sentry frontend DSN; only used when telemetry is enabled in Settings. */
  readonly VITE_GLANCEGUARD_SENTRY_DSN: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
