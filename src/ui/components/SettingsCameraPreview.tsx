import { useCallback, useEffect, useRef, useState } from "react";
import type { CameraInfo, CameraSelection } from "../../cv/types";
import { cameraSelectionKey } from "../../cv/utils";

type SettingsCameraPreviewProps = {
  cameras: CameraInfo[];
  selected?: CameraSelection;
};

/** Settings-only framing preview; monitoring still uses the selected native camera. */
export const SettingsCameraPreview = ({ cameras, selected }: SettingsCameraPreviewProps) => {
  const videoRef = useRef<HTMLVideoElement>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const [active, setActive] = useState(false);

  const selectedName = selected
    ? cameras.find((c) => cameraSelectionKey(c.id) === cameraSelectionKey(selected))?.name
    : undefined;

  const stop = useCallback(() => {
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
      streamRef.current = null;
    }
    if (videoRef.current) {
      videoRef.current.srcObject = null;
    }
    setActive(false);
  }, []);

  useEffect(() => {
    stop();
    if (!selected || cameras.length === 0) {
      return;
    }

    let cancelled = false;

    const run = async () => {
      const devices = await navigator.mediaDevices.enumerateDevices();
      const videos = devices.filter((d) => d.kind === "videoinput");
      let deviceId: string | undefined;
      if (selectedName) {
        const match = videos.find(
          (d) => d.label && selectedName && d.label.includes(selectedName.split("(")[0].trim()),
        );
        deviceId = match?.deviceId;
      }

      const constraints: MediaStreamConstraints = {
        video: deviceId
          ? { deviceId: { exact: deviceId } }
          : { facingMode: "user", width: { ideal: 640 }, height: { ideal: 480 } },
        audio: false,
      };

      const stream = await navigator.mediaDevices.getUserMedia(constraints);
      if (cancelled) {
        stream.getTracks().forEach((t) => t.stop());
        return;
      }
      streamRef.current = stream;
      const video = videoRef.current;
      if (video) {
        video.srcObject = stream;
        await video.play().catch(() => undefined);
      }
      setActive(true);
    };

    run().catch(() => {
      setActive(false);
    });

    return () => {
      cancelled = true;
      stop();
    };
  }, [selected, selectedName, cameras.length, stop]);

  return (
    <div className="field">
      <span className="field__label" id="settings-camera-preview-label">
        Live preview
      </span>
      <div
        className={`camera-preview camera-preview--settings ${active ? "" : "camera-preview--hidden"}`}
        aria-labelledby="settings-camera-preview-label"
      >
        <video ref={videoRef} autoPlay playsInline muted className="camera-preview__video" />
      </div>
      {!active ? (
        <p className="muted">
          Choose a camera above to check framing before you start monitoring.
        </p>
      ) : null}
    </div>
  );
};
