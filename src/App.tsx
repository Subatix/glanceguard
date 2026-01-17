import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import {
  getOwnerStatus,
  getSettings,
  listCameras,
} from "./cv/ipc";
import type { AlertEvent, FrameEvent, ErrorEvent } from "./cv/types";
import { useAppStore } from "./state/appStore";
import { notifyAlert } from "./ui/notifications";
import { MonitoringScreen } from "./ui/screens/MonitoringScreen";
import { OwnerSetupScreen } from "./ui/screens/OwnerSetupScreen";
import { SettingsScreen } from "./ui/screens/SettingsScreen";

const App = () => {
  const activeScreen = useAppStore((state) => state.activeScreen);
  const setActiveScreen = useAppStore((state) => state.setActiveScreen);
  const setSettings = useAppStore((state) => state.setSettings);
  const setCameras = useAppStore((state) => state.setCameras);
  const setOwnerEnrolled = useAppStore((state) => state.setOwnerEnrolled);
  const setMonitorStatus = useAppStore((state) => state.setMonitorStatus);
  const setLastAlert = useAppStore((state) => state.setLastAlert);
  const setDebugFrame = useAppStore((state) => state.setDebugFrame);
  const setError = useAppStore((state) => state.setError);

  useEffect(() => {
    getSettings()
      .then((settings) => setSettings(settings))
      .catch((err) => setError(String(err)));

    listCameras()
      .then((cameras) => setCameras(cameras))
      .catch((err) => setError(String(err)));

    getOwnerStatus()
      .then((status) => setOwnerEnrolled(status))
      .catch((err) => setError(String(err)));

    const frameListener = listen<FrameEvent>("cv:frame", (event) => {
      setDebugFrame(event.payload);
      const status = (event.payload.state || "monitoring") as
        | "idle"
        | "monitoring"
        | "alert"
        | "cooldown";
      setMonitorStatus(status, event.payload.observerScore);
    });

    const alertListener = listen<AlertEvent>("cv:alert", (event) => {
      setLastAlert(event.payload);
      notifyAlert("Someone may be looking at your screen.").catch((err) =>
        setError(String(err))
      );
    });

    const errorListener = listen<ErrorEvent>("cv:error", (event) => {
      setError(event.payload.message);
    });

    return () => {
      frameListener.then((unlisten) => unlisten()).catch(() => undefined);
      alertListener.then((unlisten) => unlisten()).catch(() => undefined);
      errorListener.then((unlisten) => unlisten()).catch(() => undefined);
    };
  }, [setCameras, setDebugFrame, setError, setLastAlert, setMonitorStatus, setOwnerEnrolled, setSettings]);

  return (
    <div className="app">
      <header className="app__header">
        <div className="brand">
          <div className="brand__title">Screen Peek Alert</div>
          <div className="brand__subtitle">Privacy assistant</div>
        </div>
        <nav className="nav">
          <button
            className={`nav__item ${activeScreen === "monitoring" ? "is-active" : ""}`}
            onClick={() => setActiveScreen("monitoring")}
          >
            Monitoring
          </button>
          <button
            className={`nav__item ${activeScreen === "owner" ? "is-active" : ""}`}
            onClick={() => setActiveScreen("owner")}
          >
            Owner setup
          </button>
          <button
            className={`nav__item ${activeScreen === "settings" ? "is-active" : ""}`}
            onClick={() => setActiveScreen("settings")}
          >
            Settings
          </button>
        </nav>
      </header>

      <main className="app__main">
        {activeScreen === "monitoring" ? <MonitoringScreen /> : null}
        {activeScreen === "owner" ? <OwnerSetupScreen /> : null}
        {activeScreen === "settings" ? <SettingsScreen /> : null}
      </main>
    </div>
  );
};

export default App;
