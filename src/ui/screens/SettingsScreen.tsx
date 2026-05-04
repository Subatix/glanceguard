import { enable, disable } from "@tauri-apps/plugin-autostart";
import { setSettings } from "../../cv/ipc";
import { cameraSelectionKey } from "../../cv/utils";
import { useAppStore } from "../../state/appStore";
import { useSettingsStore } from "../../state/settingsStore";
import type { AppTheme, NotificationStyle, Sensitivity } from "../../settings/types";
import { SegmentedControl } from "../components/SegmentedControl";
import { SettingsCameraPreview } from "../components/SettingsCameraPreview";

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
    description: 'Uses the full "GlanceGuard Alert" title — easiest to notice.',
  },
  {
    value: "compact",
    label: "Compact",
    description: "Shorter title — slightly quieter while monitoring.",
  },
];

export const SettingsScreen = () => {
  const settings = useSettingsStore((state) => state.settings);
  const cameras = useSettingsStore((state) => state.cameras);
  const setSettingsState = useSettingsStore((state) => state.setSettings);
  const setError = useAppStore((state) => state.setError);

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
    <div className="screen">
      <div className="screen__header">
        <h2>Settings</h2>
        <p>Sensitivity, cooldown, appearance, and startup behavior.</p>
      </div>

      <div className="panel">
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

        <SegmentedControl
          label="Theme"
          name="theme"
          value={settings.theme ?? "system"}
          options={THEME_OPTIONS}
          onChange={(value) => {
            persist({ theme: value }).catch((err) => setError(String(err)));
          }}
        />

        <SegmentedControl
          label="Notification style"
          name="notification"
          value={settings.notificationStyle ?? "native"}
          options={NOTIFY_OPTIONS}
          onChange={(value) => {
            persist({ notificationStyle: value }).catch((err) => setError(String(err)));
          }}
        />

        <div className="field field--row">
          <label className="field__label" htmlFor="settings-start-login">
            Start at login
          </label>
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
        </div>

        <div className="field">
          <label className="field__label" htmlFor="settings-camera-select">
            Camera
          </label>
          <select
            id="settings-camera-select"
            className="field__input"
            value={settings.camera ? cameraSelectionKey(settings.camera) : ""}
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
        </div>

        <SettingsCameraPreview cameras={cameras} selected={settings.camera} />

        <div className="field field--row">
          <label className="field__label" htmlFor="settings-clahe">
            Low-light face boost (CLAHE)
          </label>
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
        </div>

        <div className="field field--row">
          <label className="field__label" htmlFor="settings-debug">
            Debug overlay
          </label>
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
        </div>

        <section className="privacy-panel" aria-labelledby="privacy-panel-title">
          <h3 id="privacy-panel-title" className="privacy-panel__title">
            Privacy &amp; data
          </h3>
          <ul className="privacy-panel__list">
            <li>Face embeddings and enrollment data stay on this Mac inside encrypted storage.</li>
            <li>No frames or thumbnails are uploaded by this app (offline-first).</li>
            <li>
              Telemetry / crash reporting stays opt-in when shipped (see DECISIONS.md D8); this build does not enable it
              by default.
            </li>
          </ul>
        </section>

        <p className="muted settings-shortcut-hint">
          Pause / resume monitoring: ⌘⌥P (global shortcut).
        </p>
      </div>
    </div>
  );
};
