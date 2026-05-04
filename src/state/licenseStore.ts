import { create } from "zustand";

/** Minimal slice until Phase 11 JWT verification (see DECISIONS.md D7). */
type LicenseState = {
  licenseGatePassed: boolean;
  setLicenseGatePassed: (value: boolean) => void;
};

export const useLicenseStore = create<LicenseState>()((set) => ({
  licenseGatePassed: false,
  setLicenseGatePassed: (licenseGatePassed) => set({ licenseGatePassed }),
}));
