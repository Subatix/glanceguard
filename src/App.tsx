import { useEffect, useState, type KeyboardEvent } from "react";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import {
  getOwnerStatus,
  getSettings,
  listCameras,
  modelsReady,
  stopMonitoring,
} from "./cv/ipc";
import type { AlertEvent, FrameEvent, ErrorEvent, MonitorStoppedEvent } from "./cv/types";
import { toggleMonitoringPause } from "./monitoring/toggleMonitoringPause";
import { loadFirstRunSnapshot } from "./state/firstRunPersistence";
import { useAppStore } from "./state/appStore";
import { useLicenseStore } from "./state/licenseStore";
import { useMonitorStore } from "./state/monitorStore";
import { useOwnerStore } from "./state/ownerStore";
import { useSettingsStore } from "./state/settingsStore";
import { syncDomTheme } from "./theme/syncDomTheme";
import { monitoringChipLabels } from "./messages/alertExperience";
import { notifyAlert } from "./ui/notifications";
import { MonitoringScreen } from "./ui/screens/MonitoringScreen";
import { OwnerSetupScreen } from "./ui/screens/OwnerSetupScreen";
import { SettingsScreen } from "./ui/screens/SettingsScreen";
import { LicenseGateScreen } from "./ui/screens/LicenseGateScreen";
import { OnboardingScreen } from "./ui/screens/OnboardingScreen";
import { UpdateBanner } from "./ui/components/UpdateBanner";
import { ScreenHeader } from "./ui/components/ScreenHeader";
import { StatusPill } from "./ui/components/StatusPill";
import { Surface } from "./ui/components/Surface";
import { syncBrowserSentry } from "./telemetry/syncBrowserSentry";

const monitorStatusLabels = monitoringChipLabels;

const screenLabels = {
  monitoring: "Protection",
  owner: "Owner",
  settings: "Settings",
} as const;

const App = () => {
  const activeScreen = useAppStore((state) => state.activeScreen);
  const setActiveScreen = useAppStore((state) => state.setActiveScreen);
  const setSettingsState = useSettingsStore((state) => state.setSettings);
  const setCameras = useSettingsStore((state) => state.setCameras);
  const settings = useSettingsStore((state) => state.settings);
  const setOwnerEnrolled = useOwnerStore((state) => state.setOwnerEnrolled);
  const setMonitorStatus = useMonitorStore((state) => state.setMonitorStatus);
  const setLastAlert = useMonitorStore((state) => state.setLastAlert);
  const setDebugFrame = useMonitorStore((state) => state.setDebugFrame);
  const setError = useAppStore((state) => state.setError);
  const monitorStatus = useMonitorStore((state) => state.monitor.status);

  const firstRunHydrated = useAppStore((state) => state.firstRunHydrated);
  const licenseGatePassed = useLicenseStore((state) => state.licenseGatePassed);
  const onboardingCompleted = useAppStore((state) => state.onboarding.completed);
  const mainUnlocked = licenseGatePassed && onboardingCompleted;

  const [modelsOk, setModelsOk] = useState<boolean | null>(null);

  useEffect(() => {
    syncDomTheme(settings.theme ?? "system");
  }, [settings.theme]);

  useEffect(() => {
    syncBrowserSentry(Boolean(settings.telemetryEnabled));
  }, [settings.telemetryEnabled]);

  useEffect(() => {
    modelsReady()
      .then((ok) => setModelsOk(ok))
      .catch((err) => {
        setError(String(err));
        setModelsOk(false);
      });
  }, [setError]);

  useEffect(() => {
    if (modelsOk !== true) {
      return;
    }
    let cancelled = false;
    loadFirstRunSnapshot()
      .then((snapshot) => {
        if (!cancelled) {
          useLicenseStore.getState().setLicenseGatePassed(snapshot.licenseGatePassed);
          useAppStore.getState().hydrateFirstRun(snapshot);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(String(err));
          useLicenseStore.getState().setLicenseGatePassed(false);
          useAppStore.getState().hydrateFirstRun({
            licenseGatePassed: false,
            onboardingCompleted: false,
            onboardingStep: null,
          });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [modelsOk, setError]);

  useEffect(() => {
    if (!mainUnlocked || modelsOk !== true) {
      return;
    }

    getSettings()
      .then((s) => setSettingsState(s))
      .catch((err) => setError(String(err)));

    listCameras()
      .then((cams) => setCameras(cams))
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
      const style = useSettingsStore.getState().settings.notificationStyle ?? "native";
      notifyAlert(style).catch((err) =>
        setError(String(err)),
      );
    });

    const errorListener = listen<ErrorEvent>("cv:error", (event) => {
      setError(event.payload.message);
    });

    const stopListener = listen<MonitorStoppedEvent>("cv:monitor-stopped", (event) => {
      setError(event.payload.reason);
      setMonitorStatus("idle", null);
      stopMonitoring().catch(() => undefined);
    });

    const trayStateListener = listen<{ idle: boolean }>(
      "glanceguard-monitor-state",
      (event) => {
        useMonitorStore
          .getState()
          .setMonitorStatus(event.payload.idle ? "idle" : "monitoring", null);
      },
    );

    const trayErrListener = listen<{ message: string }>("glanceguard-tray-error", (event) => {
      setError(event.payload.message);
    });

    const navListener = listen<{ screen: string }>("glanceguard-navigate", (event) => {
      const s = event.payload.screen;
      if (s === "settings" || s === "monitoring" || s === "owner") {
        setActiveScreen(s);
      }
    });

    return () => {
      frameListener.then((unlisten) => unlisten()).catch(() => undefined);
      alertListener.then((unlisten) => unlisten()).catch(() => undefined);
      errorListener.then((unlisten) => unlisten()).catch(() => undefined);
      stopListener.then((unlisten) => unlisten()).catch(() => undefined);
      trayStateListener.then((unlisten) => unlisten()).catch(() => undefined);
      trayErrListener.then((unlisten) => unlisten()).catch(() => undefined);
      navListener.then((unlisten) => unlisten()).catch(() => undefined);
    };
  }, [
    mainUnlocked,
    modelsOk,
    setActiveScreen,
    setCameras,
    setDebugFrame,
    setError,
    setLastAlert,
    setMonitorStatus,
    setOwnerEnrolled,
    setSettingsState,
  ]);

  const headerStatusClick = () => {
    toggleMonitoringPause(setError).catch(() => undefined);
  };

  const headerStatusKeyDown = (e: KeyboardEvent<HTMLButtonElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      headerStatusClick();
    }
  };

  if (modelsOk === null) {
    return (
      <div className="app">
        <main className="app__main app__main--center">
          <Surface className="panel boot-panel">
            <StatusPill tone="info">Starting</StatusPill>
            <p className="muted">Checking bundled face model files…</p>
          </Surface>
        </main>
      </div>
    );
  }

  if (!modelsOk) {
    return (
      <div className="app">
        <main className="app__main app__main--center">
          <Surface className="panel model-fatal">
            <ScreenHeader eyebrow="Cannot continue" title="Face models could not be loaded">
              <p>
                The bundled detection files are missing or do not match the expected fingerprints.
                Install a fresh GlanceGuard build, or rebuild your development app with verified bundled model assets.
              </p>
            </ScreenHeader>
          </Surface>
        </main>
      </div>
    );
  }

  if (!firstRunHydrated) {
    return (
      <div className="app">
        <main className="app__main app__main--center">
          <Surface className="panel boot-panel">
            <StatusPill tone="info">Preparing</StatusPill>
            <p className="muted">Loading local preferences…</p>
          </Surface>
        </main>
      </div>
    );
  }

  if (!licenseGatePassed) {
    return (
      <div className="app">
        <main className="app__main">
          <LicenseGateScreen />
        </main>
      </div>
    );
  }

  if (!onboardingCompleted) {
    return (
      <div className="app">
        <main className="app__main">
          <OnboardingScreen />
        </main>
      </div>
    );
  }

  const monitoringRunning = monitorStatus !== "idle";

  return (
    <div className="app">
      <header className="app__header">
        <nav className="nav" role="tablist" aria-label="Primary">
          {(["monitoring", "owner", "settings"] as const).map((screen) => (
            <button
              key={screen}
              type="button"
              role="tab"
              id={`tab-${screen}`}
              aria-selected={activeScreen === screen}
              aria-controls="panel-main"
              className={`nav__item ${activeScreen === screen ? "is-active" : ""}`}
              onClick={() => setActiveScreen(screen)}
            >
              {screenLabels[screen]}
            </button>
          ))}
        </nav>
        <div className="header-status-slot">
          <button
            type="button"
            className="header-status"
            data-state={monitorStatus}
            aria-label={monitoringRunning ? "Pause monitoring" : "Resume monitoring"}
            aria-pressed={monitoringRunning}
            onClick={() => headerStatusClick()}
            onKeyDown={headerStatusKeyDown}
          >
            <span className="header-status__dot" aria-hidden />
            <span className="header-status__label" aria-live="polite">
              {monitorStatusLabels[monitorStatus]}
            </span>
          </button>
        </div>
      </header>

      <UpdateBanner autoCheck={Boolean(settings.autoCheckUpdates)} />

      <main id="panel-main" role="tabpanel" className="app__main">
        {activeScreen === "monitoring" ? <MonitoringScreen /> : null}
        {activeScreen === "owner" ? <OwnerSetupScreen /> : null}
        {activeScreen === "settings" ? <SettingsScreen /> : null}
      </main>
    </div>
  );
};

export default App;
