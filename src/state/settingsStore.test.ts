import { describe, it, expect, beforeEach } from "vitest";
import { useSettingsStore } from "./settingsStore";
import { defaultSettings } from "../settings/defaults";

describe("useSettingsStore", () => {
  beforeEach(() => {
    useSettingsStore.setState({
      settings: defaultSettings,
      cameras: [],
    });
  });

  it("setSettings replaces settings", () => {
    useSettingsStore.getState().setSettings({
      ...defaultSettings,
      cooldownSec: 60,
      debugOverlay: true,
    });
    expect(useSettingsStore.getState().settings.cooldownSec).toBe(60);
    expect(useSettingsStore.getState().settings.debugOverlay).toBe(true);
  });

  it("setCameras stores camera list", () => {
    const cameras = [
      {
        id: { kind: "Index" as const, value: 0 },
        name: "FaceTime",
        description: "",
      },
    ];
    useSettingsStore.getState().setCameras(cameras);
    expect(useSettingsStore.getState().cameras).toHaveLength(1);
  });
});
