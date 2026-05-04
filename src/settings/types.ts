import type { CameraSelection } from "../cv/types";

export type Sensitivity = "low" | "medium" | "high";

export type Settings = {
  sensitivity: Sensitivity;
  cooldownSec: 15 | 30 | 60;
  debugOverlay: boolean;
  /** Optional CLAHE on aligned face crops (off by default). */
  claheFacePreproc?: boolean;
  camera?: CameraSelection;
};

export type SettingsUpdate = {
  sensitivity?: Sensitivity;
  cooldownSec?: 15 | 30 | 60;
  debugOverlay?: boolean;
  claheFacePreproc?: boolean;
  camera?: CameraSelection;
};
