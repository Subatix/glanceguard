import { create } from "zustand";
import type { CameraInfo } from "../cv/types";
import type { Settings } from "../settings/types";
import { defaultSettings } from "../settings/defaults";

type SettingsState = {
  settings: Settings;
  cameras: CameraInfo[];
  setSettings: (settings: Settings) => void;
  setCameras: (cameras: CameraInfo[]) => void;
};

export const useSettingsStore = create<SettingsState>()((set) => ({
  settings: defaultSettings,
  cameras: [],
  setSettings: (settings) => set({ settings }),
  setCameras: (cameras) => set({ cameras }),
}));
