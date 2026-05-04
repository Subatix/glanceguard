import { Store } from "@tauri-apps/plugin-store";

/** Matches the Phase 11 Crockford-style key shape from DECISIONS.md (client-side format check only until server validation lands). */
export const LICENSE_KEY_PATTERN = /^SP\d-[0-9A-HJKMNP-TV-Z]{4}-[0-9A-HJKMNP-TV-Z]{4}-[0-9A-HJKMNP-TV-Z]{4}$/i;

export function isValidLicenseKeyFormat(key: string): boolean {
  return LICENSE_KEY_PATTERN.test(key.trim());
}

const STORE_FILE = "screenpeek_first_run.json";

export type PersistedOnboardingStep =
  | "welcome"
  | "camera-explainer"
  | "camera-grant"
  | "keychain-explainer"
  | "enrollment"
  | "done";

export type FirstRunSnapshot = {
  licenseGatePassed: boolean;
  onboardingCompleted: boolean;
  onboardingStep: PersistedOnboardingStep | null;
};

export async function loadFirstRunSnapshot(): Promise<FirstRunSnapshot> {
  const store = await Store.load(STORE_FILE);
  const licenseGatePassed = (await store.get<boolean>("license_gate_passed")) ?? false;
  const onboardingCompleted = (await store.get<boolean>("onboarding_completed")) ?? false;
  const rawStep = await store.get<string>("onboarding_step");
  const onboardingStep =
    rawStep === "welcome" ||
    rawStep === "camera-explainer" ||
    rawStep === "camera-grant" ||
    rawStep === "keychain-explainer" ||
    rawStep === "enrollment" ||
    rawStep === "done"
      ? rawStep
      : null;

  return { licenseGatePassed, onboardingCompleted, onboardingStep };
}

export async function persistLicenseGatePassed(value: boolean): Promise<void> {
  const store = await Store.load(STORE_FILE);
  await store.set("license_gate_passed", value);
  await store.save();
}

export async function persistOnboardingStep(step: PersistedOnboardingStep): Promise<void> {
  const store = await Store.load(STORE_FILE);
  await store.set("onboarding_step", step);
  await store.save();
}

export async function persistOnboardingCompleted(value: boolean): Promise<void> {
  const store = await Store.load(STORE_FILE);
  await store.set("onboarding_completed", value);
  if (value) {
    await store.set("onboarding_step", "done");
  }
  await store.save();
}
