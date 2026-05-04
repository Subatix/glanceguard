import { create } from "zustand";
import type { AlertEvent, FrameEvent } from "../cv/types";

export type MonitorStatus = "idle" | "monitoring" | "alert" | "cooldown";

type MonitorSlice = {
  status: MonitorStatus;
  lastAlert?: AlertEvent;
  observerScore?: number | null;
};

type MonitorState = {
  monitor: MonitorSlice;
  debugFrame?: FrameEvent;
  setMonitorStatus: (status: MonitorStatus, observerScore?: number | null) => void;
  setLastAlert: (alert: AlertEvent) => void;
  setDebugFrame: (frame?: FrameEvent) => void;
};

export const useMonitorStore = create<MonitorState>()((set) => ({
  monitor: { status: "idle" },
  debugFrame: undefined,
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
}));
