import { useCallback, useEffect, useRef, useState } from "react";
import { enrollOwnerFromImageBatch, validateEnrollmentSnapshot } from "../../cv/ipc";
import { PrivacyOnboardingFooter } from "./PrivacyOnboardingFooter";

const POSES = [
  { id: "center", title: "Center", hint: "Look straight at the camera, neutral expression." },
  { id: "left", title: "Turn left", hint: "Turn your head slightly to your left (like glancing at the menu bar)." },
  { id: "right", title: "Turn right", hint: "Turn your head slightly to the right." },
  { id: "up", title: "Look up", hint: "Tilt your chin slightly up." },
  { id: "down", title: "Look down", hint: "Tilt your chin slightly down toward the keyboard." },
] as const;

const TOTAL = POSES.length;
const RING_R = 44;
const RING_C = 2 * Math.PI * RING_R;

type Props = {
  onComplete: () => void;
  onError: (message: string) => void;
  /** Clears inline onboarding error before each capture attempt */
  onClearError?: () => void;
};

export const EnrollmentWizard = ({ onComplete, onError, onClearError }: Props) => {
  const [poseIndex, setPoseIndex] = useState(0);
  const [captures, setCaptures] = useState<number[][]>([]);
  const [busy, setBusy] = useState(false);
  const [cameraReady, setCameraReady] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const onErrorRef = useRef(onError);
  onErrorRef.current = onError;

  const stopCamera = useCallback(() => {
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
      streamRef.current = null;
    }
    if (videoRef.current) {
      videoRef.current.srcObject = null;
    }
    setCameraReady(false);
  }, []);

  useEffect(() => {
    let cancelled = false;
    navigator.mediaDevices
      .getUserMedia({
        video: { width: { ideal: 1280 }, height: { ideal: 720 }, facingMode: "user" },
        audio: false,
      })
      .then((stream) => {
        if (cancelled) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }
        streamRef.current = stream;
        const video = videoRef.current;
        if (!video) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }
        video.srcObject = stream;
        return new Promise<void>((resolve, reject) => {
          const timeout = window.setTimeout(
            () => reject(new Error("Camera timed out — no video frames received")),
            8000,
          );
          const onPlaying = () => {
            window.clearTimeout(timeout);
            video.removeEventListener("playing", onPlaying);
            resolve();
          };
          if (video.readyState >= 2 && video.videoWidth > 0) {
            window.clearTimeout(timeout);
            resolve();
          } else {
            video.addEventListener("playing", onPlaying);
            void video.play().catch(reject);
          }
        });
      })
      .then(() => {
        if (!cancelled) {
          setCameraReady(true);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          onErrorRef.current(String(err));
        }
      });

    return () => {
      cancelled = true;
      stopCamera();
    };
  }, [stopCamera]);

  const captureFrame = useCallback(async () => {
    const video = videoRef.current;
    if (!video || !streamRef.current || video.videoWidth === 0) {
      onError("Camera is not ready — wait for the preview, then try again.");
      return;
    }

    onClearError?.();
    setBusy(true);
    try {
      const canvas = document.createElement("canvas");
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        throw new Error("Could not read camera frame.");
      }
      ctx.drawImage(video, 0, 0);
      const blob = await new Promise<Blob>((resolve, reject) => {
        canvas.toBlob(
          (b) => (b ? resolve(b) : reject(new Error("Failed to encode frame"))),
          "image/jpeg",
          0.92,
        );
      });
      const buffer = await blob.arrayBuffer();
      const bytes = Array.from(new Uint8Array(buffer));

      const stepTitle = POSES[poseIndex].title;
      try {
        await validateEnrollmentSnapshot(bytes);
      } catch (err) {
        onError(`${stepTitle}: ${String(err)}`);
        return;
      }

      // Recover from older bug: fifth frame was committed before RPC; failed submits left len > TOTAL and Rust rejects len !== 5.
      let base = captures;
      if (base.length >= TOTAL) {
        base = base.slice(0, TOTAL - 1);
        setCaptures(base);
        setPoseIndex(TOTAL - 1);
      }

      const next = [...base, bytes];

      if (next.length < TOTAL) {
        setCaptures(next);
        setPoseIndex(next.length);
      } else {
        try {
          await enrollOwnerFromImageBatch(next);
        } catch (batchErr) {
          setCaptures([]);
          setPoseIndex(0);
          onError(
            `${String(batchErr)} Progress was reset: quality checks run on every pose after you capture the last one. Start again from Center.`,
          );
          return;
        }
        setCaptures(next);
        stopCamera();
        onComplete();
      }
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  }, [captures, onClearError, onComplete, onError, poseIndex, stopCamera]);

  const progress = captures.length / TOTAL;
  const dashOffset = RING_C * (1 - progress);
  const pose = POSES[poseIndex];
  const lastPoseCapture = captures.length === TOTAL - 1;

  return (
    <div className="enrollment-wizard">
      <div className="enrollment-wizard__header">
        <h3 className="enrollment-wizard__title">Enroll your face</h3>
        <p className="enrollment-wizard__subtitle">
          Five quick poses help the app recognize you in different positions. Your face never leaves this device.{" "}
          <span className="muted enrollment-wizard__subtitle-tip">
            Move closer until your face fills much of the preview — each pose is checked before continuing.
          </span>
        </p>
      </div>

      <div className="enrollment-wizard__ring-wrap" aria-hidden>
        <svg className="enrollment-wizard__ring" viewBox="0 0 100 100">
          <circle
            className="enrollment-wizard__ring-bg"
            cx="50"
            cy="50"
            r={RING_R}
            fill="none"
            strokeWidth="8"
          />
          <circle
            className="enrollment-wizard__ring-fg"
            cx="50"
            cy="50"
            r={RING_R}
            fill="none"
            strokeWidth="8"
            strokeDasharray={RING_C}
            strokeDashoffset={dashOffset}
            transform="rotate(-90 50 50)"
          />
        </svg>
        <div className="enrollment-wizard__ring-label">
          {captures.length}/{TOTAL}
        </div>
      </div>

      <div className={`camera-preview ${cameraReady ? "" : "camera-preview--hidden"}`}>
        <video ref={videoRef} autoPlay playsInline muted className="camera-preview__video" />
      </div>

      {!cameraReady ? <p className="muted">Starting camera…</p> : null}

      <div className="enrollment-wizard__pose">
        <div className="enrollment-wizard__pose-title">
          Pose {poseIndex + 1}: {pose.title}
        </div>
        <p className="enrollment-wizard__pose-hint">{pose.hint}</p>
      </div>

      <div className="actions">
        <button
          type="button"
          className="button button--primary"
          disabled={busy || !cameraReady}
          onClick={() => void captureFrame()}
        >
          {busy
            ? lastPoseCapture
              ? "Enrolling on-device…"
              : "Saving pose…"
            : `Capture ${pose.title.toLowerCase()}`}
        </button>
      </div>

      <PrivacyOnboardingFooter />
    </div>
  );
};
