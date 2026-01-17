import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

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

export const notifyAlert = async (body: string): Promise<void> => {
  const allowed = await ensureNotificationPermission();
  if (!allowed) {
    throw new Error("Notifications permission not granted");
  }
  sendNotification({
    title: "Privacy Alert",
    body,
  });
};
