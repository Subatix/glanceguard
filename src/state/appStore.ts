import { create } from "zustand";
import type { AlertEvent, CameraInfo, FrameEvent } from "../cv/types";
import type { Settings } from "../settings/types";
import { defaultSettings } from "../settings/defaults";

export type Screen = "monitoring" | "owner" | "settings";

export type MonitorStatus = "idle" | "monitoring" | "alert" | "cooldown";

type MonitorState = {
  status: MonitorStatus;
  lastAlert?: AlertEvent;
  observerScore?: number | null;
};

type AppState = {
  settings: Settings;
  cameras: CameraInfo[];
  activeScreen: Screen;
  ownerEnrolled: boolean;
  monitor: MonitorState;
  debugFrame?: FrameEvent;
  error?: string;
  setSettings: (settings: Settings) => void;
  setCameras: (cameras: CameraInfo[]) => void;
  setActiveScreen: (screen: Screen) => void;
  setOwnerEnrolled: (value: boolean) => void;
  setMonitorStatus: (status: MonitorStatus, observerScore?: number | null) => void;
  setLastAlert: (alert: AlertEvent) => void;
  setDebugFrame: (frame?: FrameEvent) => void;
  setError: (message?: string) => void;
};

export const useAppStore = create<AppState>()((set) => ({
  settings: defaultSettings,
  cameras: [],
  activeScreen: "monitoring",
  ownerEnrolled: false,
  monitor: { status: "idle" },
  debugFrame: undefined,
  error: undefined,
  setSettings: (settings) => set({ settings }),
  setCameras: (cameras) => set({ cameras }),
  setActiveScreen: (screen) => set({ activeScreen: screen }),
  setOwnerEnrolled: (value) => set({ ownerEnrolled: value }),
  setMonitorStatus: (status, observerScore) =>
    set((state) => ({
      monitor: {
        ...state.monitor,
        status,
        observerScore,
      },
    })),
  setLastAlert: (alert) =>
    set((state) => ({
      monitor: {
        ...state.monitor,
        lastAlert: alert,
        status: "alert",
      },
    })),
  setDebugFrame: (frame) => set({ debugFrame: frame }),
  setError: (message) => set({ error: message }),
}));
