import {
  sendNotification,
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import type { NotificationStyle } from "../settings/types";

let permissionPromise: Promise<boolean> | undefined;

export const ensureNotificationPermission = async (): Promise<boolean> => {
  if (!permissionPromise) {
    permissionPromise = (async () => {
      let granted = await isPermissionGranted();
      if (!granted) {
        const permission = await requestPermission();
        granted = permission === "granted";
      }
      return granted;
    })();
  }
  return permissionPromise;
};

export const notifyAlert = async (
  body: string,
  style: NotificationStyle = "native",
): Promise<void> => {
  const allowed = await ensureNotificationPermission();
  if (!allowed) {
    throw new Error("Notifications permission not granted");
  }
  const title =
    style === "compact" ? "Peek alert" : "Privacy Alert";
  sendNotification({
    title,
    body,
  });
};
