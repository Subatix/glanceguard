import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";

afterEach(() => {
  cleanup();
});

vi.mock("@tauri-apps/plugin-store", () => {
  const buildStore = async () => {
    const data = new Map<string, unknown>();
    return {
      get: async <T,>(key: string) => {
        const v = data.get(key);
        return v === undefined ? null : (v as T);
      },
      set: async (key: string, value: unknown) => {
        data.set(key, value);
      },
      save: async () => undefined,
    };
  };
  return {
    Store: {
      load: vi.fn(buildStore),
    },
  };
});
