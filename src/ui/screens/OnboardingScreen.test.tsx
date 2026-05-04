import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { OnboardingScreen } from "./OnboardingScreen";
import { useAppStore } from "../../state/appStore";
import { useLicenseStore } from "../../state/licenseStore";
import { useOwnerStore } from "../../state/ownerStore";
import { useSettingsStore } from "../../state/settingsStore";
import { defaultSettings } from "../../settings/defaults";
import * as persistence from "../../state/firstRunPersistence";

vi.mock("../../cv/ipc", () => ({
  listCameras: vi.fn().mockResolvedValue([
    { id: { kind: "Index" as const, value: 0 }, name: "FaceTime", description: "" },
  ]),
  setCamera: vi.fn().mockResolvedValue({
    sensitivity: "medium",
    cooldownSec: 30,
    debugOverlay: false,
    theme: "system",
    startAtLogin: false,
    notificationStyle: "native",
    camera: { kind: "Index" as const, value: 0 },
  }),
  getOwnerStatus: vi.fn().mockResolvedValue(true),
}));

describe("OnboardingScreen", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.spyOn(persistence, "persistOnboardingStep").mockResolvedValue(undefined);
    vi.spyOn(persistence, "persistOnboardingCompleted").mockResolvedValue(undefined);
    useAppStore.setState({
      activeScreen: "monitoring",
      error: undefined,
      firstRunHydrated: true,
      onboarding: { step: "welcome", completed: false },
    });
    useLicenseStore.setState({ licenseGatePassed: true });
    useSettingsStore.setState({ settings: defaultSettings, cameras: [] });
    useOwnerStore.setState({ ownerEnrolled: false });
  });

  it("advances from welcome to camera explainer", async () => {
    render(<OnboardingScreen />);
    fireEvent.click(screen.getByRole("button", { name: /^Continue$/i }));
    expect(await screen.findByRole("heading", { name: /Camera access/i })).toBeInTheDocument();
    expect(persistence.persistOnboardingStep).toHaveBeenCalledWith("camera-explainer");
  });
});
