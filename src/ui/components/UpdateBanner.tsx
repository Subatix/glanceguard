import { relaunch } from "@tauri-apps/plugin-process";
import type { DownloadEvent } from "@tauri-apps/plugin-updater";
import { check } from "@tauri-apps/plugin-updater";
import { useCallback, useEffect, useState } from "react";

type Phase = "idle" | "available" | "working" | "error";

type Props = {
  /** When false, the banner does not run automatic update checks on mount. */
  autoCheck: boolean;
};

export const UpdateBanner = ({ autoCheck }: Props) => {
  const [phase, setPhase] = useState<Phase>("idle");
  const [latestVersion, setLatestVersion] = useState<string | null>(null);
  const [bytes, setBytes] = useState({ done: 0, total: null as number | null });
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (!autoCheck || import.meta.env.DEV) {
      return undefined;
    }
    let cancelled = false;
    (async () => {
      const update = await check();
      if (cancelled || !update) {
        return;
      }
      setLatestVersion(update.version);
      setPhase("available");
    })().catch((err: unknown) => {
      if (cancelled) {
        return;
      }
      setErrorMessage(err instanceof Error ? err.message : String(err));
      setPhase("error");
    });
    return () => {
      cancelled = true;
    };
  }, [autoCheck]);

  const refresh = useCallback(() => {
    setErrorMessage(null);
    setPhase("idle");
    setLatestVersion(null);
    setBytes({ done: 0, total: null });
    (async () => {
      const update = await check();
      if (!update) {
        setPhase("idle");
        return;
      }
      setLatestVersion(update.version);
      setPhase("available");
    })().catch((err: unknown) => {
      setErrorMessage(err instanceof Error ? err.message : String(err));
      setPhase("error");
    });
  }, []);

  const downloadAndRestart = async () => {
    const update = await check();
    if (!update) {
      setPhase("idle");
      setLatestVersion(null);
      return;
    }
    setPhase("working");
    setBytes({ done: 0, total: null });
    const onEvt = (ev: DownloadEvent) => {
      if (ev.event === "Started") {
        setBytes({ done: 0, total: ev.data.contentLength ?? null });
        return;
      }
      if (ev.event === "Progress") {
        setBytes((b) => ({ done: b.done + ev.data.chunkLength, total: b.total }));
        return;
      }
      if (ev.event === "Finished") {
        setBytes((b) => ({ done: b.total ?? b.done, total: b.total }));
      }
    };
    await update.downloadAndInstall(onEvt);
    await relaunch();
  };

  if (phase === "idle" && !errorMessage && !latestVersion) {
    return null;
  }

  return (
    <div className="update-banner" role="status">
      {phase === "available" ? (
        <div className="update-banner__row">
          <span>
            GlanceGuard {latestVersion} is available.&nbsp;
            <button type="button" className="button button--quiet" onClick={() => refresh()}>
              Refresh check
            </button>
          </span>
          <button
            type="button"
            className="button"
            onClick={() =>
              downloadAndRestart().catch((e: unknown) => {
                setErrorMessage(e instanceof Error ? e.message : String(e));
                setPhase("error");
              })
            }
          >
            Download &amp; restart
          </button>
        </div>
      ) : null}
      {phase === "working" ? (
        <div className="update-banner__row">
          <span>
            Updating…{" "}
            {bytes.total != null
              ? `${Math.round((bytes.done / bytes.total) * 100)}%`
              : `${Math.round(bytes.done / (1024 * 1024))} MB`}
          </span>
        </div>
      ) : null}
      {phase === "error" && errorMessage ? (
        <div className="update-banner__row">
          <span>Update error: {errorMessage}</span>
          <button type="button" className="button" onClick={() => refresh()}>
            Retry
          </button>
        </div>
      ) : null}
    </div>
  );
};
