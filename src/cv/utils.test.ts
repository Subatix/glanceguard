import { describe, it, expect } from "vitest";
import { cameraSelectionKey } from "./utils";

describe("cameraSelectionKey", () => {
  it("prefixes index selections", () => {
    expect(cameraSelectionKey({ kind: "Index", value: 2 })).toBe("index:2");
  });

  it("prefixes stable id selections", () => {
    expect(cameraSelectionKey({ kind: "StableId", value: "device-abc" })).toBe(
      "stable:device-abc",
    );
  });
});
