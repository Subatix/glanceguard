import type { CameraSelection } from "../cv/types";

export type Sensitivity = "low" | "medium" | "high";

export type Settings = {
  sensitivity: Sensitivity;
  cooldownSec: 15 | 30 | 60;
  debugOverlay: boolean;
  camera?: CameraSelection;
};

export type SettingsUpdate = {
  sensitivity?: Sensitivity;
  cooldownSec?: 15 | 30 | 60;
  debugOverlay?: boolean;
  camera?: CameraSelection;
};
