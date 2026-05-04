import type { CameraSelection } from "../cv/types";

export type Sensitivity = "low" | "medium" | "high";

export type AppTheme = "system" | "light" | "dark";

export type NotificationStyle = "native" | "compact";

export type Settings = {
  sensitivity: Sensitivity;
  cooldownSec: 15 | 30 | 60;
  debugOverlay: boolean;
  /** Optional CLAHE on aligned face crops (off by default). */
  claheFacePreproc?: boolean;
  camera?: CameraSelection;
  theme?: AppTheme;
  startAtLogin?: boolean;
  notificationStyle?: NotificationStyle;
  telemetryEnabled?: boolean;
  autoCheckUpdates?: boolean;
};

export type SettingsUpdate = {
  sensitivity?: Sensitivity;
  cooldownSec?: 15 | 30 | 60;
  debugOverlay?: boolean;
  claheFacePreproc?: boolean;
  camera?: CameraSelection;
  theme?: AppTheme;
  startAtLogin?: boolean;
  notificationStyle?: NotificationStyle;
  telemetryEnabled?: boolean;
  autoCheckUpdates?: boolean;
};
