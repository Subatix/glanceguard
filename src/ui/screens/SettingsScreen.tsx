import { setSettings } from "../../cv/ipc";
import { useAppStore } from "../../state/appStore";
import type { Sensitivity } from "../../settings/types";

export const SettingsScreen = () => {
  const settings = useAppStore((state) => state.settings);
  const setSettingsState = useAppStore((state) => state.setSettings);
  const setError = useAppStore((state) => state.setError);

  return (
    <div className="screen">
      <div className="screen__header">
        <h2>Settings</h2>
        <p>Adjust sensitivity, cooldown, and debug overlay.</p>
      </div>

      <div className="panel">
        <div className="field">
          <label className="field__label">Sensitivity</label>
          <select
            className="field__input"
            value={settings.sensitivity}
            onChange={(event) => {
              const value = event.currentTarget.value as Sensitivity;
              setSettings({ sensitivity: value })
                .then((updated) => setSettingsState(updated))
                .catch((err) => setError(String(err)));
            }}
          >
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
          </select>
        </div>

        <div className="field">
          <label className="field__label">Cooldown</label>
          <select
            className="field__input"
            value={settings.cooldownSec}
            onChange={(event) => {
              const value = Number(event.currentTarget.value) as 15 | 30 | 60;
              setSettings({ cooldownSec: value })
                .then((updated) => setSettingsState(updated))
                .catch((err) => setError(String(err)));
            }}
          >
            <option value={15}>15s</option>
            <option value={30}>30s</option>
            <option value={60}>60s</option>
          </select>
        </div>

        <div className="field field--row">
          <label className="field__label">Debug overlay</label>
          <input
            type="checkbox"
            checked={settings.debugOverlay}
            onChange={(event) => {
              setSettings({ debugOverlay: event.currentTarget.checked })
                .then((updated) => setSettingsState(updated))
                .catch((err) => setError(String(err)));
            }}
          />
        </div>
      </div>
    </div>
  );
};
