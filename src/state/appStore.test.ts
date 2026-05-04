import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "./appStore";
import { defaultSettings } from "../settings/defaults";
import type { AlertEvent } from "../cv/types";

describe("useAppStore", () => {
  beforeEach(() => {
    useAppStore.setState({
      settings: defaultSettings,
      cameras: [],
      activeScreen: "monitoring",
      ownerEnrolled: false,
      monitor: { status: "idle" },
      debugFrame: undefined,
      error: undefined,
    });
  });

  it("setSettings replaces settings", () => {
    useAppStore.getState().setSettings({
      ...defaultSettings,
      cooldownSec: 60,
      debugOverlay: true,
    });
    expect(useAppStore.getState().settings.cooldownSec).toBe(60);
    expect(useAppStore.getState().settings.debugOverlay).toBe(true);
  });

  it("setActiveScreen switches screens", () => {
    useAppStore.getState().setActiveScreen("settings");
    expect(useAppStore.getState().activeScreen).toBe("settings");
  });

  it("setOwnerEnrolled toggles enrollment flag", () => {
    useAppStore.getState().setOwnerEnrolled(true);
    expect(useAppStore.getState().ownerEnrolled).toBe(true);
  });

  it("setMonitorStatus preserves monitor slice and updates status", () => {
    useAppStore.getState().setMonitorStatus("monitoring", 0.42);
    expect(useAppStore.getState().monitor.status).toBe("monitoring");
    expect(useAppStore.getState().monitor.observerScore).toBe(0.42);
  });

  it("setLastAlert forces alert status and stores payload", () => {
    const alert: AlertEvent = {
      score: 0.91,
      reason: "test",
      cooldownSec: 30,
    };
    useAppStore.getState().setLastAlert(alert);
    expect(useAppStore.getState().monitor.status).toBe("alert");
    expect(useAppStore.getState().monitor.lastAlert).toEqual(alert);
  });

  it("setDebugFrame and setError update transient UI fields", () => {
    useAppStore.getState().setDebugFrame({
      frameWidth: 640,
      frameHeight: 480,
      faces: [],
      observerScore: null,
      state: "monitoring",
    });
    useAppStore.getState().setError("boom");
    expect(useAppStore.getState().debugFrame?.frameWidth).toBe(640);
    expect(useAppStore.getState().error).toBe("boom");
    useAppStore.getState().setError(undefined);
    expect(useAppStore.getState().error).toBeUndefined();
  });

  it("setCameras stores camera list", () => {
    const cameras = [
      {
        id: { kind: "Index" as const, value: 0 },
        name: "FaceTime",
        description: "",
      },
    ];
    useAppStore.getState().setCameras(cameras);
    expect(useAppStore.getState().cameras).toHaveLength(1);
  });
});
