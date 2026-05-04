import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "./appStore";
import { useLicenseStore } from "./licenseStore";

describe("useAppStore", () => {
  beforeEach(() => {
    useAppStore.setState({
      activeScreen: "monitoring",
      error: undefined,
      firstRunHydrated: false,
      onboarding: { step: "welcome", completed: false },
    });
    useLicenseStore.setState({ licenseGatePassed: false });
  });

  it("setActiveScreen switches screens", () => {
    useAppStore.getState().setActiveScreen("settings");
    expect(useAppStore.getState().activeScreen).toBe("settings");
  });

  it("setError clears and sets messages", () => {
    useAppStore.getState().setError("boom");
    expect(useAppStore.getState().error).toBe("boom");
    useAppStore.getState().setError(undefined);
    expect(useAppStore.getState().error).toBeUndefined();
  });

  it("hydrateFirstRun restores onboarding snapshot", () => {
    useAppStore.getState().hydrateFirstRun({
      licenseGatePassed: true,
      onboardingCompleted: false,
      onboardingStep: "keychain-explainer",
    });
    const s = useAppStore.getState();
    expect(s.firstRunHydrated).toBe(true);
    expect(s.onboarding.completed).toBe(false);
    expect(s.onboarding.step).toBe("keychain-explainer");
  });

  it("hydrateFirstRun restores alerts step from snapshot", () => {
    useAppStore.getState().hydrateFirstRun({
      licenseGatePassed: true,
      onboardingCompleted: false,
      onboardingStep: "alerts",
    });
    expect(useAppStore.getState().onboarding.step).toBe("alerts");
  });
});
