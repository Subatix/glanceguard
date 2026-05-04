import { test, expect } from "@playwright/test";

/**
 * Smoke: Vite shell renders before or without a working Tauri IPC bridge (invoke fails in plain Chromium).
 * Full first-run (license gate → onboarding wizard → enroll → monitor) needs a packaged app + WebDriver; defer to Phase 7 with tray/E2E harness.
 */
test("startup shell shows model gate or download screen", async ({ page }) => {
  await page.goto("/");
  await expect(
    page.getByText(/Checking model files|Download face models/i),
  ).toBeVisible({
    timeout: 120_000,
  });
});

test.describe("Full-stack onboarding", () => {
  test.skip(
    true,
    "Wire Tauri WebDriver + fixtures when Phase 7 tray/E2E harness lands",
  );
});
