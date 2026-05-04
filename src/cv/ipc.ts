import { invoke } from "@tauri-apps/api/core";
import type { AlertEvent, CameraInfo, CameraSelection, FrameEvent } from "./types";
import type { Settings, SettingsUpdate } from "../settings/types";

export const listCameras = async (): Promise<CameraInfo[]> => {
  return invoke<CameraInfo[]>("list_cameras");
};

export const setCamera = async (selection: CameraSelection): Promise<Settings> => {
  return invoke<Settings>("set_camera", { selection });
};

export const getSettings = async (): Promise<Settings> => {
  return invoke<Settings>("get_settings");
};

export const setSettings = async (update: SettingsUpdate): Promise<Settings> => {
  return invoke<Settings>("set_settings", { update });
};

export const startMonitoring = async (): Promise<void> => {
  await invoke("start_monitoring");
};

export const stopMonitoring = async (): Promise<void> => {
  await invoke("stop_monitoring");
};

export const enrollOwnerFromImage = async (imageBytes: number[]): Promise<void> => {
  await invoke("enroll_owner_from_image", { imageBytes });
};

export const enrollOwnerFromImageBatch = async (imageBytesList: number[][]): Promise<void> => {
  await invoke("enroll_owner_from_image_batch", { imageBytesList });
};

/** Runs detector + quality gate on-device (same face picking as enrollment). Call after each wizard snapshot. */
export const validateEnrollmentSnapshot = async (imageBytes: number[]): Promise<void> => {
  await invoke("validate_enrollment_snapshot", { imageBytes });
};

export const enrollOwnerFromLive = async (): Promise<void> => {
  await invoke("enroll_owner_from_live");
};

export const clearOwner = async (): Promise<void> => {
  await invoke("clear_owner");
};

export const getOwnerStatus = async (): Promise<boolean> => {
  return invoke<boolean>("get_owner_status");
};

export const modelsReady = async (): Promise<boolean> => {
  return invoke<boolean>("models_ready");
};

export type { AlertEvent, FrameEvent };
