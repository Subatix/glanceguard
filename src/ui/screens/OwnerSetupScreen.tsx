import { useCallback, useEffect, useRef, useState } from "react";
import {
  clearOwner,
  enrollOwnerFromImage,
  enrollOwnerFromLive,
  getOwnerStatus,
} from "../../cv/ipc";
import { useAppStore } from "../../state/appStore";
import { useOwnerStore } from "../../state/ownerStore";
import { Button } from "../components/Button";
import { ScreenHeader } from "../components/ScreenHeader";
import { Surface } from "../components/Surface";

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
    <div className="screen identity-screen">
      <ScreenHeader title="Owner" align="left">
        <p>Enroll once so GlanceGuard can tell you apart from everyone else.</p>
      </ScreenHeader>

      <Surface className="identity-panel">
        <div className="identity-panel__summary">
          <div>
            <h3>{ownerEnrolled ? "Owner saved" : "Owner missing"}</h3>
            <p>
              {ownerEnrolled
                ? "The encrypted owner profile is stored on this Mac."
                : "Monitoring starts after one local owner profile is saved."}
            </p>
          </div>
          <div className="identity-panel__state" data-state={ownerEnrolled ? "saved" : "missing"}>
            {ownerEnrolled ? "Saved" : "Required"}
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

        {error ? <div className="status__error">{error}</div> : null}

        <div className="identity-panel__actions">
          <Button variant="primary" disabled={loading} onClick={() => quickCapture()}>
            {loading ? "Processing..." : ownerEnrolled ? "Replace with quick capture" : "Quick capture"}
          </Button>
          {!cameraActive ? (
            <Button variant="ghost" disabled={loading} onClick={() => startCamera()}>
              Open preview
            </Button>
          ) : (
            <>
              <Button variant="ghost" disabled={loading} onClick={() => captureAndEnroll()}>
                Capture from preview
              </Button>
              <Button variant="ghost" disabled={loading} onClick={() => stopCamera()}>
                Cancel
              </Button>
            </>
          )}
          <Button
            variant="danger"
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
          </Button>
        </div>

        <p className="identity-panel__privacy">
          Face data stays on this Mac. The encryption key lives in macOS Keychain.
        </p>
      </Surface>
    </div>
  );
};
