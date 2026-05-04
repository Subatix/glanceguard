import { startMonitoring, stopMonitoring } from "../cv/ipc";
import { useMonitorStore } from "../state/monitorStore";
import { useOwnerStore } from "../state/ownerStore";
import { useSettingsStore } from "../state/settingsStore";

export async function toggleMonitoringPause(setError: (message?: string) => void): Promise<void> {
  const monitor = useMonitorStore.getState().monitor;
  const isRunning = monitor.status !== "idle";
  const ownerEnrolled = useOwnerStore.getState().ownerEnrolled;
  const camera = useSettingsStore.getState().settings.camera;

  if (!isRunning && (!camera || !ownerEnrolled)) {
    return;
  }

  try {
    if (isRunning) {
      await stopMonitoring();
      useMonitorStore.getState().setMonitorStatus("idle", null);
      return;
    }
    await startMonitoring();
    useMonitorStore.getState().setMonitorStatus("monitoring");
  } catch (e) {
    setError(String(e));
  }
}
