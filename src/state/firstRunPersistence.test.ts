import { describe, it, expect } from "vitest";
import { isValidLicenseKeyFormat } from "./firstRunPersistence";

describe("firstRunPersistence", () => {
  it("validates Crockford-style license segments", () => {
    expect(isValidLicenseKeyFormat("GG1-ABCD-EFGH-JKMN")).toBe(true);
    expect(isValidLicenseKeyFormat(" gg1-abcd-efgh-jkmn ")).toBe(true);
    expect(isValidLicenseKeyFormat("gg1-abid-efgh-jkmn")).toBe(false);
    expect(isValidLicenseKeyFormat("")).toBe(false);
  });
});
