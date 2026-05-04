import { useAppStore } from "../../state/appStore";
import { useMonitorStore } from "../../state/monitorStore";
import type { MonitorStatus } from "../../state/monitorStore";
import { useOwnerStore } from "../../state/ownerStore";
import { useSettingsStore } from "../../state/settingsStore";
import { cameraSelectionKey } from "../../cv/utils";
import { Button } from "../components/Button";
import { CameraSelect } from "../components/CameraSelect";
import { DebugOverlayCanvas as LiveDetectionPreview } from "../components/DebugOverlayCanvas";
import { EmptyState, emptyStatePresets } from "../components/EmptyState";
import { Overlay } from "../components/Overlay";
import { Surface } from "../components/Surface";
import {
  listCameras,
  setCamera,
  setSettings as setSettingsCommand,
  startMonitoring,
  stopMonitoring,
} from "../../cv/ipc";
import { monitoringChipLabels, monitoringProtectionCopy } from "../../messages/alertExperience";

const protectionCopy: Record<MonitorStatus, { title: string; body: string }> =
  monitoringProtectionCopy;

const monitorStatusLabels: Record<MonitorStatus, string> = monitoringChipLabels;

export const MonitoringScreen = () => {
  const cameras = useSettingsStore((state) => state.cameras);
  const settings = useSettingsStore((state) => state.settings);
  const monitor = useMonitorStore((state) => state.monitor);
  const error = useAppStore((state) => state.error);
  const ownerEnrolled = useOwnerStore((state) => state.ownerEnrolled);
  const debugFrame = useMonitorStore((state) => state.debugFrame);
  const setSettingsState = useSettingsStore((state) => state.setSettings);
  const setActiveScreen = useAppStore((state) => state.setActiveScreen);
  const setCameras = useSettingsStore((state) => state.setCameras);
  const setError = useAppStore((state) => state.setError);
  const setMonitorStatus = useMonitorStore((state) => state.setMonitorStatus);

  const isMonitoring = monitor.status !== "idle";
  const readyToMonitor = Boolean(settings.camera && ownerEnrolled);
  const selectedCamera = settings.camera;
  const selectedCameraName = selectedCamera
    ? cameras.find((camera) => cameraSelectionKey(camera.id) === cameraSelectionKey(selectedCamera))?.name ??
      "Camera selected"
    : "No camera";
  const statusCopy = protectionCopy[monitor.status];

  return (
    <div className="screen protection-screen">
      <Surface className="protection-panel" aria-live="polite">
        <div className="protection-panel__status" data-state={monitor.status}>
          <span className="protection-panel__dot" aria-hidden />
          <span>{monitorStatusLabels[monitor.status]}</span>
        </div>

        <div className="protection-panel__copy">
          <h2>{statusCopy.title}</h2>
          <p>{statusCopy.body}</p>
        </div>

        <div className="protection-panel__meta" aria-label="Setup status">
          <span>{selectedCameraName}</span>
          <span>{ownerEnrolled ? "Owner enrolled" : "Owner missing"}</span>
          <span>On-device</span>
        </div>

        {!ownerEnrolled ? (
          <div className="protection-panel__setup">
            <EmptyState
              {...emptyStatePresets.ownerNotEnrolled({
                label: "Set up owner",
                onClick: () => setActiveScreen("owner"),
                variant: "primary",
              })}
            />
          </div>
        ) : null}

        {ownerEnrolled && !settings.camera ? (
          <div className="protection-panel__setup">
            <CameraSelect
              cameras={cameras}
              selected={settings.camera}
              onChange={(selection) => {
                setCamera(selection)
                  .then((updated) => setSettingsState(updated))
                  .catch((err) => setError(String(err)));
              }}
              onRetryList={() => {
                listCameras()
                  .then((c) => setCameras(c))
                  .catch((err) => setError(String(err)));
              }}
            />
          </div>
        ) : null}

        <div className="protection-panel__actions">
          {readyToMonitor ? (
            isMonitoring ? (
              <Button
                variant="secondary"
                onClick={() => {
                  stopMonitoring()
                    .then(() => setMonitorStatus("idle"))
                    .catch((err) => setError(String(err)));
                }}
              >
                Stop monitoring
              </Button>
            ) : (
              <Button
                variant="primary"
                onClick={() => {
                  startMonitoring()
                    .then(() => setMonitorStatus("monitoring"))
                    .catch((err) => setError(String(err)));
                }}
              >
                Start monitoring
              </Button>
            )
          ) : null}
        </div>

        {error ? <div className="status__error">{error}</div> : null}

        <details className="protection-details">
          <summary>Details</summary>
          <div className="protection-details__body">
            <div className="detail-row">
              <span>Observer score</span>
              <strong>{typeof monitor.observerScore === "number" ? monitor.observerScore.toFixed(2) : "—"}</strong>
            </div>
            <CameraSelect
              cameras={cameras}
              selected={settings.camera}
              onChange={(selection) => {
                setCamera(selection)
                  .then((updated) => setSettingsState(updated))
                  .catch((err) => setError(String(err)));
              }}
              onRetryList={() => {
                listCameras()
                  .then((c) => setCameras(c))
                  .catch((err) => setError(String(err)));
              }}
            />
            <div className="field field--row">
              <div>
                <label className="field__label" htmlFor="monitoring-live-preview">
                  Live detection preview
                </label>
                <p className="field__hint">Shows local frames and face boxes only while monitoring.</p>
              </div>
              <input
                id="monitoring-live-preview"
                type="checkbox"
                checked={settings.debugOverlay}
                onChange={(event) => {
                  setSettingsCommand({ debugOverlay: event.currentTarget.checked })
                    .then((updated) => setSettingsState(updated))
                    .catch((err) => setError(String(err)));
                }}
              />
            </div>
            {settings.debugOverlay ? (
              <div
                className={`debug-panel__canvas ${
                  isMonitoring && !debugFrame ? "debug-panel__canvas--shimmer" : ""
                }`}
              >
                <LiveDetectionPreview frame={debugFrame} />
              </div>
            ) : null}
          </div>
        </details>
      </Surface>

      <Overlay visible={monitor.status === "alert"} />
    </div>
  );
};
