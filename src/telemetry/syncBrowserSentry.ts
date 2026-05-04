import * as Sentry from "@sentry/react";

/** Injected via Vercel/host — never bake a real DSN into the committed tree. */
const dsnRaw = import.meta.env.VITE_GLANCEGUARD_SENTRY_DSN;

const sanitizeDsn = (): string =>
  typeof dsnRaw === "string" && dsnRaw.trim().length > 0 ? dsnRaw.trim() : "";

export function syncBrowserSentry(enabled: boolean): void {
  const dsn = sanitizeDsn();
  if (!enabled || !dsn) {
    Sentry.close();
    return;
  }
  Sentry.init({
    dsn,
    enabled: true,
    environment: import.meta.env.MODE,
    sendDefaultPii: false,
    integrations: [],
    tracesSampleRate: 0,
  });
}
