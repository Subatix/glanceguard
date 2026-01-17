import { useState } from "react";
import {
  clearOwner,
  enrollOwnerFromImage,
  enrollOwnerFromLive,
  getOwnerStatus,
} from "../../cv/ipc";
import { useAppStore } from "../../state/appStore";

export const OwnerSetupScreen = () => {
  const ownerEnrolled = useAppStore((state) => state.ownerEnrolled);
  const setOwnerEnrolled = useAppStore((state) => state.setOwnerEnrolled);
  const setError = useAppStore((state) => state.setError);
  const [loading, setLoading] = useState(false);

  const refreshStatus = () => {
    getOwnerStatus()
      .then((status) => setOwnerEnrolled(status))
      .catch((err) => setError(String(err)));
  };

  return (
    <div className="screen">
      <div className="screen__header">
        <h2>Owner setup</h2>
        <p>Enroll yourself so the app can distinguish you from observers.</p>
      </div>

      <div className="panel">
        <div className={`owner-status ${ownerEnrolled ? "is-enrolled" : "is-missing"}`}>
          <div className="owner-status__title">
            {ownerEnrolled ? "Owner enrolled and saved locally" : "Owner not enrolled"}
          </div>
          <div className="owner-status__meta">
            {ownerEnrolled
              ? "Your owner profile is stored on this device. You do not need to upload again unless you clear it."
              : "Enroll once to enable monitoring and owner recognition."}
          </div>
        </div>
        <div className="field">
          <label className="field__label">Upload photo</label>
          <input
            className="field__input"
            type="file"
            accept="image/*"
            disabled={loading}
            onChange={(event) => {
              const file = event.currentTarget.files?.[0];
              if (!file) {
                return;
              }
              setLoading(true);
              file
                .arrayBuffer()
                .then((buffer) => Array.from(new Uint8Array(buffer)))
                .then((bytes) => enrollOwnerFromImage(bytes))
                .then(() => refreshStatus())
                .catch((err) => setError(String(err)))
                .finally(() => setLoading(false));
            }}
          />
        </div>
        <div className="actions">
          <button
            className="button button--primary"
            disabled={loading}
            onClick={() => {
              setLoading(true);
              enrollOwnerFromLive()
                .then(() => refreshStatus())
                .catch((err) => setError(String(err)))
                .finally(() => setLoading(false));
            }}
          >
            Capture from camera
          </button>
          <button
            className="button button--ghost"
            disabled={!ownerEnrolled || loading}
            onClick={() => {
              setLoading(true);
              clearOwner()
                .then(() => refreshStatus())
                .catch((err) => setError(String(err)))
                .finally(() => setLoading(false));
            }}
          >
            Clear owner
          </button>
        </div>
      </div>
    </div>
  );
};
