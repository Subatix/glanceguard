import type { Settings } from "./types";

export const defaultSettings: Settings = {
  sensitivity: "medium",
  cooldownSec: 30,
  debugOverlay: false,
  theme: "system",
  startAtLogin: false,
  notificationStyle: "native",
  telemetryEnabled: false,
  autoCheckUpdates: true,
};
