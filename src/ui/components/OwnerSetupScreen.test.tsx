import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { OwnerSetupScreen } from "../screens/OwnerSetupScreen";
import { useAppStore } from "../../state/appStore";
import { defaultSettings } from "../../settings/defaults";

vi.mock("../../cv/ipc", () => ({
  clearOwner: vi.fn().mockResolvedValue(undefined),
  enrollOwnerFromImage: vi.fn().mockResolvedValue(undefined),
  enrollOwnerFromImageBatch: vi.fn().mockResolvedValue(undefined),
  enrollOwnerFromLive: vi.fn().mockResolvedValue(undefined),
  getOwnerStatus: vi.fn().mockResolvedValue(false),
}));

describe("OwnerSetupScreen", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    useAppStore.setState({
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
    });
  });

  it("renders enrollment actions when IPC is mocked", async () => {
    render(<OwnerSetupScreen />);
    expect(screen.getByRole("heading", { name: /Owner setup/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Quick capture/i })).toBeEnabled();
    expect(screen.getByRole("button", { name: /Clear owner/i })).toBeDisabled();
  });
});
