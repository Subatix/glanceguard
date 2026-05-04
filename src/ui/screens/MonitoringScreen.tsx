import { useAppStore } from "../../state/appStore";
import { CameraSelect } from "../components/CameraSelect";
import { DebugOverlayCanvas } from "../components/DebugOverlayCanvas";
import { EmptyState, emptyStatePresets } from "../components/EmptyState";
import { Overlay } from "../components/Overlay";
import { StatusCard } from "../components/StatusCard";
import {
  listCameras,
  setCamera,
  setSettings as setSettingsCommand,
  startMonitoring,
  stopMonitoring,
} from "../../cv/ipc";

export const MonitoringScreen = () => {
  const cameras = useAppStore((state) => state.cameras);
  const settings = useAppStore((state) => state.settings);
  const monitor = useAppStore((state) => state.monitor);
  const error = useAppStore((state) => state.error);
  const ownerEnrolled = useAppStore((state) => state.ownerEnrolled);
  const debugFrame = useAppStore((state) => state.debugFrame);
  const setSettingsState = useAppStore((state) => state.setSettings);
  const setActiveScreen = useAppStore((state) => state.setActiveScreen);
  const setCameras = useAppStore((state) => state.setCameras);
  const setError = useAppStore((state) => state.setError);
  const setMonitorStatus = useAppStore((state) => state.setMonitorStatus);

  const isMonitoring = monitor.status !== "idle";

  return (
    <div className="screen">
      <div className="screen__header">
        <h2>Monitoring</h2>
        <p>Detects a second face and alerts when someone looks at your screen.</p>
      </div>

      <div className="grid">
        <div className="panel">
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
          {!ownerEnrolled ? (
            <div className="monitoring-empty-owner">
              <EmptyState
                {...emptyStatePresets.ownerNotEnrolled({
                  label: "Open Owner setup",
                  onClick: () => setActiveScreen("owner"),
                  variant: "primary",
                })}
              />
            </div>
          ) : null}
          <div className="actions">
            <button
              className="button button--primary"
              disabled={!settings.camera || !ownerEnrolled}
              onClick={() => {
                startMonitoring()
                  .then(() => setMonitorStatus("monitoring"))
                  .catch((err) => setError(String(err)));
              }}
            >
              Start monitoring
            </button>
            <button
              className="button button--ghost"
              disabled={!isMonitoring}
              onClick={() => {
                stopMonitoring()
                  .then(() => setMonitorStatus("idle"))
                  .catch((err) => setError(String(err)));
              }}
            >
              Stop
            </button>
          </div>
        </div>

        <StatusCard
          variant={isMonitoring && settings.debugOverlay && !debugFrame ? "skeleton" : "default"}
          status={monitor.status}
          observerScore={monitor.observerScore}
          error={error}
        />
      </div>

      {isMonitoring && !settings.debugOverlay ? (
        <div className="callout">
          <div className="callout__title">Monitoring is active</div>
          <div className="callout__body">
            Debug overlay is off, so you will see a blank panel. Turn it on to view face boxes
            and scores in real time.
          </div>
          <button
            className="button button--primary button--small"
            onClick={() => {
              setSettingsCommand({ debugOverlay: true })
                .then((updated) => setSettingsState(updated))
                .catch((err) => setError(String(err)));
            }}
          >
            Enable debug overlay
          </button>
        </div>
      ) : null}

      <div className="debug-panel">
        <div className="debug-panel__header">
          <span>Debug overlay</span>
          <span>{settings.debugOverlay ? "On" : "Off"}</span>
        </div>
        <div
          className={`debug-panel__canvas ${
            isMonitoring && settings.debugOverlay && !debugFrame ? "debug-panel__canvas--shimmer" : ""
          }`}
        >
          {settings.debugOverlay ? <DebugOverlayCanvas frame={debugFrame} /> : null}
          {!settings.debugOverlay ? <div className="debug-panel__empty">Enable debug overlay in settings.</div> : null}
        </div>
      </div>

      <Overlay visible={monitor.status === "alert"} message="Someone may be looking at your screen." />
    </div>
  );
};
