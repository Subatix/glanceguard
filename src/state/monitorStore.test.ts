import { describe, it, expect, beforeEach } from "vitest";
import { useMonitorStore } from "./monitorStore";
import type { AlertEvent } from "../cv/types";

describe("useMonitorStore", () => {
  beforeEach(() => {
    useMonitorStore.setState({
      monitor: { status: "idle" },
      debugFrame: undefined,
    });
  });

  it("setMonitorStatus preserves monitor slice and updates status", () => {
    useMonitorStore.getState().setMonitorStatus("monitoring", 0.42);
    expect(useMonitorStore.getState().monitor.status).toBe("monitoring");
    expect(useMonitorStore.getState().monitor.observerScore).toBe(0.42);
  });

  it("setLastAlert forces alert status and stores payload", () => {
    const alert: AlertEvent = {
      score: 0.91,
      reason: "test",
      cooldownSec: 30,
    };
    useMonitorStore.getState().setLastAlert(alert);
    expect(useMonitorStore.getState().monitor.status).toBe("alert");
    expect(useMonitorStore.getState().monitor.lastAlert).toEqual(alert);
  });

  it("setDebugFrame stores frames", () => {
    useMonitorStore.getState().setDebugFrame({
      frameWidth: 640,
      frameHeight: 480,
      faces: [],
      observerScore: null,
      state: "monitoring",
    });
    expect(useMonitorStore.getState().debugFrame?.frameWidth).toBe(640);
  });
});
