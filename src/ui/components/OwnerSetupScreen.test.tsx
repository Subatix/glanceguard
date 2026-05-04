import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { OwnerSetupScreen } from "../screens/OwnerSetupScreen";
import { useAppStore } from "../../state/appStore";
import { useOwnerStore } from "../../state/ownerStore";
import { useSettingsStore } from "../../state/settingsStore";
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
      activeScreen: "monitoring",
      error: undefined,
      firstRunHydrated: false,
      onboarding: { step: "welcome", completed: false },
    });
    useOwnerStore.setState({ ownerEnrolled: false });
    useSettingsStore.setState({ settings: defaultSettings, cameras: [] });
  });

  it("renders enrollment actions when IPC is mocked", async () => {
    render(<OwnerSetupScreen />);
    expect(screen.getByRole("heading", { name: /^Owner$/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Quick capture/i })).toBeEnabled();
    expect(screen.getByRole("button", { name: /Clear owner/i })).toBeDisabled();
  });
});
