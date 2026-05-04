import { create } from "zustand";

import type { FirstRunSnapshot } from "./firstRunPersistence";

export type Screen = "monitoring" | "owner" | "settings";

export type OnboardingWizardStep =
  | "welcome"
  | "camera-explainer"
  | "camera-grant"
  | "keychain-explainer"
  | "enrollment"
  | "done";

type AppUiState = {
  activeScreen: Screen;
  error?: string;
  firstRunHydrated: boolean;
  onboarding: {
    step: OnboardingWizardStep;
    completed: boolean;
  };
  setActiveScreen: (screen: Screen) => void;
  setError: (message?: string) => void;
  hydrateFirstRun: (snapshot: FirstRunSnapshot) => void;
  setOnboardingStep: (step: OnboardingWizardStep) => void;
  setOnboardingCompleted: (value: boolean) => void;
};

function pickInitialStep(
  onboardingCompleted: boolean,
  snapshotStep: FirstRunSnapshot["onboardingStep"],
): OnboardingWizardStep {
  if (onboardingCompleted) {
    return "done";
  }
  if (
    snapshotStep === "welcome" ||
    snapshotStep === "camera-explainer" ||
    snapshotStep === "camera-grant" ||
    snapshotStep === "keychain-explainer" ||
    snapshotStep === "enrollment" ||
    snapshotStep === "done"
  ) {
    return snapshotStep;
  }
  return "welcome";
}

export const useAppStore = create<AppUiState>()((set) => ({
  activeScreen: "monitoring",
  error: undefined,
  firstRunHydrated: false,
  onboarding: { step: "welcome", completed: false },
  setActiveScreen: (screen) => set({ activeScreen: screen }),
  setError: (message) => set({ error: message }),
  hydrateFirstRun: (snapshot) =>
    set({
      firstRunHydrated: true,
      onboarding: {
        completed: snapshot.onboardingCompleted,
        step: pickInitialStep(snapshot.onboardingCompleted, snapshot.onboardingStep),
      },
    }),
  setOnboardingStep: (step) =>
    set((state) => ({
      onboarding: { ...state.onboarding, step },
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
