import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { LicenseGateScreen } from "./LicenseGateScreen";
import { useAppStore } from "../../state/appStore";
import { defaultSettings } from "../../settings/defaults";
import * as persistence from "../../state/firstRunPersistence";

describe("LicenseGateScreen", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.spyOn(persistence, "persistLicenseGatePassed").mockResolvedValue(undefined);
    useAppStore.setState({
      settings: defaultSettings,
      cameras: [],
      activeScreen: "monitoring",
      ownerEnrolled: false,
      monitor: { status: "idle" },
      debugFrame: undefined,
      error: undefined,
      firstRunHydrated: true,
      licenseGatePassed: false,
      onboarding: { step: "welcome", completed: false },
    });
  });

  it("shows validation error for malformed license key", async () => {
    render(<LicenseGateScreen />);
    fireEvent.change(screen.getByLabelText(/License key/i), { target: { value: "not-a-key" } });
    fireEvent.click(screen.getByRole("button", { name: /^Continue$/i }));
    expect(await screen.findByText(/format SP#/i)).toBeInTheDocument();
    expect(useAppStore.getState().licenseGatePassed).toBe(false);
  });

  it("passes gate when key matches client-side format", async () => {
    render(<LicenseGateScreen />);
    fireEvent.change(screen.getByLabelText(/License key/i), {
      target: { value: "SP1-ABCD-EFGH-JKMN" },
    });
    fireEvent.click(screen.getByRole("button", { name: /^Continue$/i }));
    await waitFor(() => {
      expect(persistence.persistLicenseGatePassed).toHaveBeenCalledWith(true);
      expect(useAppStore.getState().licenseGatePassed).toBe(true);
    });
  });
});
