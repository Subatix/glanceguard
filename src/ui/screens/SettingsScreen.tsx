import { enable, disable } from "@tauri-apps/plugin-autostart";
import { setSettings } from "../../cv/ipc";
import { cameraSelectionKey } from "../../cv/utils";
import { useAppStore } from "../../state/appStore";
import { useSettingsStore } from "../../state/settingsStore";
import type { AppTheme, NotificationStyle, Sensitivity } from "../../settings/types";
import { AlertsPermissionPanel } from "../components/AlertsPermissionPanel";
import { PreferenceGroup, PreferenceRow } from "../components/PreferenceGroup";
import { ScreenHeader } from "../components/ScreenHeader";
import { SegmentedControl } from "../components/SegmentedControl";
import { SettingsCameraPreview } from "../components/SettingsCameraPreview";
import { Surface } from "../components/Surface";

const SENSITIVITY_OPTIONS: {
  value: Sensitivity;
  label: string;
  description: string;
}[] = [
  {
    value: "low",
    label: "Low",
    description: "Stricter match — fewer alerts; may miss quick glances.",
  },
  {
    value: "medium",
    label: "Medium",
    description: "Recommended balance for desks and cafés.",
  },
  {
    value: "high",
    label: "High",
    description: "Catches peripheral attention; expect more false positives.",
  },
];

const COOLDOWN_OPTIONS: {
  value: 15 | 30 | 60;
  label: string;
  description: string;
}[] = [
  { value: 15, label: "15s", description: "Short quiet window between alerts." },
  { value: 30, label: "30s", description: "Default spacing between repeats." },
  { value: 60, label: "60s", description: "Fewer interruptions during sustained conversations." },
];

const THEME_OPTIONS: { value: AppTheme; label: string; description?: string }[] = [
  { value: "system", label: "System", description: "Follow macOS light/dark appearance." },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
];

const NOTIFY_OPTIONS: { value: NotificationStyle; label: string; description: string }[] = [
  {
    value: "native",
    label: "Standard",
    description: 'Uses the full friendly headline (“Someone else may see your screen”) — easiest to notice.',
  },
  {
    value: "compact",
    label: "Compact",
    description: 'Shorter headline (“Heads-up”) — slightly quieter banners.',
  },
];

export const SettingsScreen = () => {
  const settings = useSettingsStore((state) => state.settings);
  const cameras = useSettingsStore((state) => state.cameras);
  const setSettingsState = useSettingsStore((state) => state.setSettings);
  const setError = useAppStore((state) => state.setError);
  const selectedCameraKey = settings.camera ? cameraSelectionKey(settings.camera) : "";

  const persist = async (update: Parameters<typeof setSettings>[0]) => {
    const updated = await setSettings(update);
    setSettingsState(updated);
  };

  const syncAutostartToggle = async (next: boolean) => {
    await persist({ startAtLogin: next });
    if (next) {
      await enable();
    } else {
      await disable();
    }
  };

  return (
    <div className="screen settings-screen">
      <ScreenHeader title="Settings" align="left">
        <p>Small adjustments for alerts, camera, startup, and privacy.</p>
      </ScreenHeader>

      <Surface className="preferences-panel">
        <PreferenceGroup
          title="Alerts"
          description="How GlanceGuard gets your attention while monitoring."
        >
          <AlertsPermissionPanel setError={setError} />
          <SegmentedControl
            label="Notification style"
            name="notification"
            value={settings.notificationStyle ?? "native"}
            options={NOTIFY_OPTIONS}
            onChange={(value) => {
              persist({ notificationStyle: value }).catch((err) => setError(String(err)));
            }}
          />
        </PreferenceGroup>

        <PreferenceGroup title="Detection" description="Tune how quickly GlanceGuard reacts.">
          <SegmentedControl
            label="Sensitivity"
            name="sensitivity"
            value={settings.sensitivity}
            options={SENSITIVITY_OPTIONS}
            onChange={(value) => {
              persist({ sensitivity: value }).catch((err) => setError(String(err)));
            }}
          />
          <SegmentedControl
            label="Cooldown between alerts"
            name="cooldown"
            value={settings.cooldownSec}
            options={COOLDOWN_OPTIONS}
            onChange={(value) => {
              persist({ cooldownSec: value }).catch((err) => setError(String(err)));
            }}
          />
          <PreferenceRow label="Low-light face boost" hint="Improves matching when your room is dim.">
            <input
              id="settings-clahe"
              type="checkbox"
              checked={Boolean(settings.claheFacePreproc)}
              onChange={(event) => {
                persist({ claheFacePreproc: event.currentTarget.checked }).catch((err) =>
                  setError(String(err)),
                );
              }}
            />
          </PreferenceRow>
        </PreferenceGroup>

        <PreferenceGroup title="Camera" description="Choose the camera used for local detection.">
          <PreferenceRow
            label="Camera"
            hint={settings.camera ? "Selected for monitoring." : "Required before monitoring can start."}
          >
            <select
              id="settings-camera-select"
              className="field__input"
              value={selectedCameraKey}
              onChange={(event) => {
                const key = event.currentTarget.value;
                const cam = cameras.find((c) => cameraSelectionKey(c.id) === key);
                if (!cam) return;
                persist({ camera: cam.id }).catch((err) => setError(String(err)));
              }}
            >
              <option value="" disabled>
                Select a camera
              </option>
              {cameras.map((camera) => (
                <option key={cameraSelectionKey(camera.id)} value={cameraSelectionKey(camera.id)}>
                  {camera.name}
                </option>
              ))}
            </select>
          </PreferenceRow>
          <PreferenceRow label="Live detection preview" hint="Shows local frames and face boxes while monitoring.">
            <input
              id="settings-debug"
              type="checkbox"
              checked={settings.debugOverlay}
              onChange={(event) => {
                persist({ debugOverlay: event.currentTarget.checked }).catch((err) =>
                  setError(String(err)),
                );
              }}
            />
          </PreferenceRow>
          <details className="settings-preview">
            <summary>Preview framing</summary>
            <SettingsCameraPreview cameras={cameras} selected={settings.camera} />
          </details>
        </PreferenceGroup>

        <PreferenceGroup title="App">
          <SegmentedControl
            label="Theme"
            name="theme"
            value={settings.theme ?? "system"}
            options={THEME_OPTIONS}
            onChange={(value) => {
              persist({ theme: value }).catch((err) => setError(String(err)));
            }}
          />

          <PreferenceRow label="Start at login" hint="Start quietly when you sign in to this Mac.">
            <input
              id="settings-start-login"
              type="checkbox"
              checked={Boolean(settings.startAtLogin)}
              onChange={(event) => {
                syncAutostartToggle(event.currentTarget.checked).catch((err) =>
                  setError(String(err)),
                );
              }}
            />
          </PreferenceRow>
          <PreferenceRow
            label="Check for updates"
            hint="Only checks update metadata; camera frames are not involved."
          >
            <input
              id="settings-check-updates"
              type="checkbox"
              checked={Boolean(settings.autoCheckUpdates ?? true)}
              onChange={(event) => {
                persist({ autoCheckUpdates: event.currentTarget.checked }).catch((err) =>
                  setError(String(err)),
                );
              }}
            />
          </PreferenceRow>
          <PreferenceRow label="Send crash reports" hint="Optional diagnostics only. No face images or thumbnails.">
            <input
              id="settings-telemetry"
              type="checkbox"
              checked={Boolean(settings.telemetryEnabled)}
              onChange={(event) => {
                persist({ telemetryEnabled: event.currentTarget.checked }).catch((err) =>
                  setError(String(err)),
                );
              }}
            />
          </PreferenceRow>

          <p className="muted settings-shortcut-hint">
            Pause / resume monitoring: ⌘⌥P (global shortcut).
          </p>
        </PreferenceGroup>

        <PreferenceGroup title="Privacy & data" description="The camera pipeline is designed to stay local.">
          <ul className="privacy-panel__list">
            <li>Face embeddings and enrollment data stay on this Mac inside encrypted storage.</li>
            <li>No frames or thumbnails are uploaded by this app.</li>
            <li>Crash reports are sent only if you enable them. They do not include face images or thumbnails.</li>
          </ul>
        </PreferenceGroup>
      </Surface>
    </div>
  );
};
