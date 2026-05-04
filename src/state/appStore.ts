import { create } from "zustand";
import type { AlertEvent, CameraInfo, FrameEvent } from "../cv/types";
import type { Settings } from "../settings/types";
import { defaultSettings } from "../settings/defaults";
import type { FirstRunSnapshot } from "./firstRunPersistence";

export type Screen = "monitoring" | "owner" | "settings";

export type MonitorStatus = "idle" | "monitoring" | "alert" | "cooldown";

export type OnboardingWizardStep =
  | "welcome"
  | "camera-explainer"
  | "camera-grant"
  | "keychain-explainer"
  | "enrollment"
  | "done";

type MonitorState = {
  status: MonitorStatus;
  lastAlert?: AlertEvent;
  observerScore?: number | null;
};

type AppState = {
  settings: Settings;
  cameras: CameraInfo[];
  activeScreen: Screen;
  ownerEnrolled: boolean;
  monitor: MonitorState;
  debugFrame?: FrameEvent;
  error?: string;
  firstRunHydrated: boolean;
  licenseGatePassed: boolean;
  onboarding: {
    step: OnboardingWizardStep;
    completed: boolean;
  };
  setSettings: (settings: Settings) => void;
  setCameras: (cameras: CameraInfo[]) => void;
  setActiveScreen: (screen: Screen) => void;
  setOwnerEnrolled: (value: boolean) => void;
  setMonitorStatus: (status: MonitorStatus, observerScore?: number | null) => void;
  setLastAlert: (alert: AlertEvent) => void;
  setDebugFrame: (frame?: FrameEvent) => void;
  setError: (message?: string) => void;
  hydrateFirstRun: (snapshot: FirstRunSnapshot) => void;
  setLicenseGatePassed: (value: boolean) => void;
  setOnboardingStep: (step: OnboardingWizardStep) => void;
  setOnboardingCompleted: (value: boolean) => void;
};

function pickInitialStep(snapshot: FirstRunSnapshot): OnboardingWizardStep {
  if (snapshot.onboardingCompleted) {
    return "done";
  }
  if (
    snapshot.onboardingStep === "welcome" ||
    snapshot.onboardingStep === "camera-explainer" ||
    snapshot.onboardingStep === "camera-grant" ||
    snapshot.onboardingStep === "keychain-explainer" ||
    snapshot.onboardingStep === "enrollment" ||
    snapshot.onboardingStep === "done"
  ) {
    return snapshot.onboardingStep;
  }
  return "welcome";
}

export const useAppStore = create<AppState>()((set) => ({
  settings: defaultSettings,
  cameras: [],
  activeScreen: "monitoring",
  ownerEnrolled: false,
  monitor: { status: "idle" },
  debugFrame: undefined,
  error: undefined,
  firstRunHydrated: false,
  licenseGatePassed: false,
  onboarding: { step: "welcome", completed: false },
  setSettings: (settings) => set({ settings }),
  setCameras: (cameras) => set({ cameras }),
  setActiveScreen: (screen) => set({ activeScreen: screen }),
  setOwnerEnrolled: (value) => set({ ownerEnrolled: value }),
  setMonitorStatus: (status, observerScore) =>
    set((state) => ({
      monitor: {
        ...state.monitor,
        status,
        observerScore,
      },
    })),
  setLastAlert: (alert) =>
    set((state) => ({
      monitor: {
        ...state.monitor,
        lastAlert: alert,
        status: "alert",
      },
    })),
  setDebugFrame: (frame) => set({ debugFrame: frame }),
  setError: (message) => set({ error: message }),
  hydrateFirstRun: (snapshot) =>
    set({
      firstRunHydrated: true,
      licenseGatePassed: snapshot.licenseGatePassed,
      onboarding: {
        completed: snapshot.onboardingCompleted,
        step: pickInitialStep(snapshot),
      },
    }),
  setLicenseGatePassed: (value) => set({ licenseGatePassed: value }),
  setOnboardingStep: (step) =>
    set((state) => ({
      onboarding: {
        ...state.onboarding,
        step,
      },
    })),
  setOnboardingCompleted: (value) =>
    set((state) => ({
      onboarding: {
        ...state.onboarding,
        completed: value,
        step: value ? "done" : state.onboarding.step,
      },
    })),
}));
