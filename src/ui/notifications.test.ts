import { describe, it, expect, vi, beforeEach } from "vitest";

const hoisted = vi.hoisted(() => ({
  isPermissionGranted: vi.fn(),
  requestPermission: vi.fn(),
  sendNotification: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: () => hoisted.isPermissionGranted(),
  requestPermission: () => hoisted.requestPermission(),
  sendNotification: hoisted.sendNotification,
}));

describe("ensureNotificationPermission", () => {
  beforeEach(() => {
    vi.resetModules();
    hoisted.isPermissionGranted.mockResolvedValue(false);
    hoisted.requestPermission.mockResolvedValue("granted");
    hoisted.sendNotification.mockClear();
  });

  it("reuses a single in-flight permission workflow", async () => {
    const { ensureNotificationPermission } = await import("./notifications");
    const first = ensureNotificationPermission();
    const second = ensureNotificationPermission();
    await Promise.all([first, second]);
    expect(hoisted.isPermissionGranted).toHaveBeenCalledTimes(1);
    expect(hoisted.requestPermission).toHaveBeenCalledTimes(1);
    await expect(first).resolves.toBe(true);
    await expect(second).resolves.toBe(true);
  });
});
