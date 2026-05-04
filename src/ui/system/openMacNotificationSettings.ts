import { openUrl } from "@tauri-apps/plugin-opener";

function isProbablyMacOs(): boolean {
  if (typeof navigator === "undefined") {
    return false;
  }
  return /Mac/i.test(navigator.platform ?? navigator.userAgent);
}

/** Opens macOS Notifications system settings when running in the desktop app on a Mac. */
export async function openMacNotificationSettings(): Promise<void> {
  if (!isProbablyMacOs()) {
    throw new Error("Open System Settings manually and allow notifications for GlanceGuard.");
  }
  await openUrl("x-apple.systempreferences:com.apple.Notifications-Settings.extension");
}
