import {
  sendNotification,
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import type { NotificationStyle } from "../settings/types";
import {
  alertDetailForNotification,
  alertNotificationTitleCompact,
  alertNotificationTitleNative,
} from "../messages/alertExperience";

let inFlightPermission: Promise<boolean> | undefined;

/** Current OS-granted notification access (no system prompt). */
export const getNotificationPermissionGranted = (): Promise<boolean> => isPermissionGranted();

/**
 * Ensures notifications are allowed before showing an alert. Re-checks the OS each time so
 * grants from System Settings apply without restarting the app. Coalesces concurrent prompts.
 */
export const ensureNotificationPermission = async (): Promise<boolean> => {
  if (await isPermissionGranted()) {
    return true;
  }
  if (!inFlightPermission) {
    inFlightPermission = (async () => {
      const permission = await requestPermission();
      return permission === "granted";
    })().finally(() => {
      inFlightPermission = undefined;
    });
  }
  return inFlightPermission;
};

/**
 * Call from Settings after the user reads why alerts matter. Runs the permission flow immediately
 * (may show macOS prompt). Does not share the in-flight coalescer with background alert paths.
 */
export const requestNotificationAccessFromUser = async (): Promise<boolean> => {
  if (await isPermissionGranted()) {
    return true;
  }
  const permission = await requestPermission();
  return permission === "granted";
};

export const notifyAlert = async (style: NotificationStyle = "native"): Promise<void> => {
  const allowed = await ensureNotificationPermission();
  if (!allowed) {
    throw new Error("Notifications permission not granted");
  }
  const title =
    style === "compact" ? alertNotificationTitleCompact : alertNotificationTitleNative;
  sendNotification({
    title,
    body: alertDetailForNotification,
  });
};
