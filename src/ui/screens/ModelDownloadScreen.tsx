import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import { downloadModels, modelsReady } from "../../cv/ipc";

export type ModelDownloadProgress = {
  filename: string;
  downloadedBytes: number;
  totalBytes?: number | null;
};

type Props = {
  onReady: () => void;
};

export const ModelDownloadScreen = ({ onReady }: Props) => {
  const defaultBase =
    typeof import.meta.env.VITE_SCREENPEEK_MODELS_BASE_URL === "string"
      ? import.meta.env.VITE_SCREENPEEK_MODELS_BASE_URL
      : "";
  const [baseUrl, setBaseUrl] = useState(defaultBase);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<ModelDownloadProgress | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const recheck = useCallback(() => {
    setMessage(null);
    modelsReady()
      .then((ok) => {
        if (ok) {
          onReady();
        } else {
          setMessage("Models are still missing or failed verification.");
        }
      })
      .catch((err) => setMessage(String(err)));
  }, [onReady]);

  useEffect(() => {
    const unsubs: Array<Promise<() => void>> = [
      listen<ModelDownloadProgress>("model-download-progress", (e) => {
        setProgress(e.payload);
      }),
      listen<{ message: string }>("model-download-error", (e) => {
        setBusy(false);
        setMessage(e.payload.message);
      }),
      listen("model-download-done", () => {
        setBusy(false);
        setProgress(null);
        recheck();
      }),
    ];
    return () => {
      Promise.all(unsubs)
        .then((list) => {
          list.forEach((u) => u());
        })
        .catch(() => undefined);
    };
  }, [recheck]);

  const startDownload = () => {
    setMessage(null);
    setBusy(true);
    setProgress(null);
    downloadModels(baseUrl.trim()).catch((err) => {
      setBusy(false);
      setMessage(String(err));
    });
  };

  return (
    <div className="screen model-download">
      <div className="screen__header">
        <h2>Download face models</h2>
        <p>
          ONNX weights are not shipped in the app bundle. Enter the base URL that serves{" "}
          <code className="inline-code">{"<sha256>.onnx"}</code> files (see{" "}
          <code className="inline-code">src-tauri/src/models/mod.rs</code> for hashes), or run{" "}
          <code className="inline-code">./scripts/download-buffalo-s-models.sh</code> into{" "}
          <code className="inline-code">src-tauri/models/</code> for local development.
        </p>
      </div>

      <div className="panel model-download__panel">
        <label className="field">
          <span className="field__label">Models base URL</span>
          <input
            className="field__input"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            placeholder="https://updates.example.com/models"
            disabled={busy}
          />
        </label>

        <div className="actions">
          <button type="button" className="button button--primary" disabled={busy} onClick={startDownload}>
            {busy ? "Downloading…" : "Download"}
          </button>
          <button type="button" className="button" disabled={busy} onClick={recheck}>
            Check again
          </button>
        </div>

        {progress ? (
          <div className="model-download__progress">
            <div>
              <strong>{progress.filename}</strong>
            </div>
            <div className="muted">
              {progress.downloadedBytes}
              {progress.totalBytes != null ? ` / ${progress.totalBytes} bytes` : " bytes"}
            </div>
          </div>
        ) : null}

        {message ? <div className="status__error">{message}</div> : null}
      </div>
    </div>
  );
};
