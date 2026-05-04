import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { LicenseGateScreen } from "./LicenseGateScreen";
import { useAppStore } from "../../state/appStore";
import { useLicenseStore } from "../../state/licenseStore";
import * as persistence from "../../state/firstRunPersistence";

describe("LicenseGateScreen", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.spyOn(persistence, "persistLicenseGatePassed").mockResolvedValue(undefined);
    useAppStore.setState({
      activeScreen: "monitoring",
      error: undefined,
      firstRunHydrated: true,
      onboarding: { step: "welcome", completed: false },
    });
    useLicenseStore.setState({ licenseGatePassed: false });
  });

  it("shows validation error for malformed license key", async () => {
    render(<LicenseGateScreen />);
    fireEvent.change(screen.getByLabelText(/License key/i), { target: { value: "not-a-key" } });
    fireEvent.click(screen.getByRole("button", { name: /^Continue$/i }));
    expect(await screen.findByText(/format GG#/i)).toBeInTheDocument();
    expect(useLicenseStore.getState().licenseGatePassed).toBe(false);
  });

  it("passes gate when key matches client-side format", async () => {
    render(<LicenseGateScreen />);
    fireEvent.change(screen.getByLabelText(/License key/i), {
      target: { value: "GG1-ABCD-EFGH-JKMN" },
    });
    fireEvent.click(screen.getByRole("button", { name: /^Continue$/i }));
    await waitFor(() => {
      expect(persistence.persistLicenseGatePassed).toHaveBeenCalledWith(true);
      expect(useLicenseStore.getState().licenseGatePassed).toBe(true);
    });
  });
});
