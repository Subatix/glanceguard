import { Store } from "@tauri-apps/plugin-store";

/** Matches the Phase 11 Crockford-style key shape from DECISIONS.md (client-side format check only until server validation lands). */
export const LICENSE_KEY_PATTERN = /^SP\d-[0-9A-HJKMNP-TV-Z]{4}-[0-9A-HJKMNP-TV-Z]{4}-[0-9A-HJKMNP-TV-Z]{4}$/i;

export function isValidLicenseKeyFormat(key: string): boolean {
  return LICENSE_KEY_PATTERN.test(key.trim());
}

const STORE_FILE = "glanceguard_first_run.json";
const STORE_LEGACY = "screenpeek_first_run.json";

const TRACK_KEYS = ["license_gate_passed", "onboarding_completed", "onboarding_step"] as const;

async function storeHasProgress(store: Store): Promise<boolean> {
  if ((await store.get<boolean>("license_gate_passed")) === true) return true;
  if ((await store.get<boolean>("onboarding_completed")) === true) return true;
  const rawStep = await store.get<string>("onboarding_step");
  return rawStep != null && rawStep !== "";
}

async function loadActiveStore(): Promise<Store> {
  const primary = await Store.load(STORE_FILE);
  if (await storeHasProgress(primary)) return primary;

  const legacy = await Store.load(STORE_LEGACY);
  if (!(await storeHasProgress(legacy))) return primary;

  for (const key of TRACK_KEYS) {
    const v = await legacy.get(key);
    if (v !== undefined && v !== null) {
      await primary.set(key, v as never);
    }
  }
  await primary.save();
  return primary;
}

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
  const store = await loadActiveStore();
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
  const store = await loadActiveStore();
  await store.set("license_gate_passed", value);
  await store.save();
}

export async function persistOnboardingStep(step: PersistedOnboardingStep): Promise<void> {
  const store = await loadActiveStore();
  await store.set("onboarding_step", step);
  await store.save();
}

export async function persistOnboardingCompleted(value: boolean): Promise<void> {
  const store = await loadActiveStore();
  await store.set("onboarding_completed", value);
  if (value) {
    await store.set("onboarding_step", "done");
  }
  await store.save();
}
