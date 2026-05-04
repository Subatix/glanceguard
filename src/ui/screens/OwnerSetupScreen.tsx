import { useCallback, useEffect, useRef, useState } from "react";
import {
  clearOwner,
  enrollOwnerFromImage,
  enrollOwnerFromLive,
  getOwnerStatus,
} from "../../cv/ipc";
import { useAppStore } from "../../state/appStore";
import { useOwnerStore } from "../../state/ownerStore";

export const OwnerSetupScreen = () => {
  const ownerEnrolled = useOwnerStore((state) => state.ownerEnrolled);
  const setOwnerEnrolled = useOwnerStore((state) => state.setOwnerEnrolled);
  const error = useAppStore((state) => state.error);
  const setError = useAppStore((state) => state.setError);
  const [loading, setLoading] = useState(false);
  const [cameraActive, setCameraActive] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const streamRef = useRef<MediaStream | null>(null);

  const refreshStatus = () => {
    getOwnerStatus()
      .then((status) => setOwnerEnrolled(status))
      .catch((err) => setError(String(err)));
  };

  const stopCamera = useCallback(() => {
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
      streamRef.current = null;
    }
    if (videoRef.current) {
      videoRef.current.srcObject = null;
    }
    setCameraActive(false);
  }, []);

  const startCamera = useCallback(async () => {
    setError(undefined);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { width: { ideal: 1280 }, height: { ideal: 720 }, facingMode: "user" },
        audio: false,
      });
      streamRef.current = stream;
      const video = videoRef.current;
      if (video) {
        video.srcObject = stream;
        await new Promise<void>((resolve, reject) => {
          const timeout = setTimeout(() => reject(new Error("Camera timed out — no video frames received")), 5000);
          const onPlaying = () => {
            clearTimeout(timeout);
            video.removeEventListener("playing", onPlaying);
            resolve();
          };
          if (video.readyState >= 2 && video.videoWidth > 0) {
            clearTimeout(timeout);
            resolve();
          } else {
            video.addEventListener("playing", onPlaying);
            video.play().catch(reject);
          }
        });
      }
      setCameraActive(true);
    } catch (err) {
      if (streamRef.current) {
        streamRef.current.getTracks().forEach((t) => t.stop());
        streamRef.current = null;
      }
      setError("Browser camera unavailable: " + String(err) + ". Use 'Quick capture' instead.");
    }
  }, [setError]);

  const captureAndEnroll = useCallback(async () => {
    const video = videoRef.current;
    if (!video || !streamRef.current) return;

    if (video.videoWidth === 0 || video.videoHeight === 0) {
      setError("Camera is not ready yet — wait for the preview to appear, then try again.");
      return;
    }

    setLoading(true);
    setError(undefined);
    try {
      const canvas = document.createElement("canvas");
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      const ctx = canvas.getContext("2d");
      if (!ctx) throw new Error("Failed to create canvas context");
      ctx.drawImage(video, 0, 0);

      const blob = await new Promise<Blob>((resolve, reject) => {
        canvas.toBlob(
          (b) => (b ? resolve(b) : reject(new Error("Failed to capture frame — camera may not have permission"))),
          "image/jpeg",
          0.95
        );
      });
      const buffer = await blob.arrayBuffer();
      const bytes = Array.from(new Uint8Array(buffer));

      await enrollOwnerFromImage(bytes);
      refreshStatus();
      stopCamera();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [setError, stopCamera]);

  /** Uses the Rust backend (nokhwa) to capture a single frame and enroll — no browser camera needed */
  const quickCapture = useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      await enrollOwnerFromLive();
      refreshStatus();
      stopCamera();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [setError, stopCamera]);

  // Clean up camera when leaving the screen
  useEffect(() => {
    return () => stopCamera();
  }, [stopCamera]);

  return (
    <div className="screen">
      <div className="screen__header">
        <h2>Owner setup</h2>
        <p>Enroll yourself so the app can distinguish you from observers.</p>
        <p className="owner-setup__privacy-note muted">
          Your face stays on this Mac. The profile is encrypted; keys live in macOS Keychain. No enrollment data is sent
          over the network.
        </p>
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

        <div className={`camera-preview ${cameraActive ? "" : "camera-preview--hidden"}`}>
          <video
            ref={videoRef}
            autoPlay
            playsInline
            muted
            className="camera-preview__video"
          />
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
              if (!file) return;
              setLoading(true);
              setError(undefined);
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

        {error ? <div className="status__error">{error}</div> : null}

        <div className="actions">
          <button
            className="button button--primary"
            disabled={loading}
            onClick={() => quickCapture()}
          >
            {loading ? "Processing..." : "Quick capture"}
          </button>
          {!cameraActive ? (
            <button
              className="button button--ghost"
              disabled={loading}
              onClick={() => startCamera()}
            >
              Open preview
            </button>
          ) : (
            <>
              <button
                className="button button--ghost"
                disabled={loading}
                onClick={() => captureAndEnroll()}
              >
                Capture from preview
              </button>
              <button
                className="button button--ghost"
                disabled={loading}
                onClick={() => stopCamera()}
              >
                Cancel
              </button>
            </>
          )}
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
