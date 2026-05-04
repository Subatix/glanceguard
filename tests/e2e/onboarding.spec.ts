import { test, expect } from "@playwright/test";

/**
 * Smoke: Vite shell renders before or without a working Tauri IPC bridge (invoke fails in plain Chromium).
 * Full first-run (license gate → onboarding wizard → enroll → monitor) needs a packaged app + WebDriver; Phase 7 landed tray/Rust IPC but Playwright against plain Vite still cannot drive invoke — keep deferred until WebDriver harness exists.
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
    "Wire Tauri WebDriver + fixtures (Phase 7 tray is in-app; Playwright smoke stays Vite-only)",
  );
});
