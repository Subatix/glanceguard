/**
 * Prefer an HD-ish capture size so SCRFD face boxes stay in pixel space above quality thresholds.
 * Many browsers ignore `ideal` alone and default to VGA (~352×288).
 */
export async function requestUserFaceStream(): Promise<MediaStream> {
  try {
    return await navigator.mediaDevices.getUserMedia({
      audio: false,
      video: {
        facingMode: "user",
        width: { ideal: 1280, min: 640 },
        height: { ideal: 720, min: 480 },
      },
    });
  } catch {
    return await navigator.mediaDevices.getUserMedia({
      audio: false,
      video: {
        facingMode: "user",
        width: { ideal: 1280 },
        height: { ideal: 720 },
      },
    });
  }
}
