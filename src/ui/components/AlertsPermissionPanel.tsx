import { useEffect, useState } from "react";
import {
  getNotificationPermissionGranted,
  requestNotificationAccessFromUser,
} from "../notifications";
import { openMacNotificationSettings } from "../system/openMacNotificationSettings";
import { Button } from "./Button";

export type AlertsPermissionPanelProps = {
  /** Sets or clears app-level error (e.g. permission hints). Omit to keep errors local only. */
  setError?: (message?: string) => void;
  className?: string;
};

export const AlertsPermissionPanel = ({ setError, className }: AlertsPermissionPanelProps) => {
  const [notifGranted, setNotifGranted] = useState<boolean | null>(null);
  const [notifBusy, setNotifBusy] = useState(false);

  useEffect(() => {
    getNotificationPermissionGranted()
      .then((g) => setNotifGranted(g))
      .catch(() => setNotifGranted(false));
  }, []);

  const onEnableAlerts = () => {
    setNotifBusy(true);
    setError?.(undefined);
    requestNotificationAccessFromUser()
      .then(() => getNotificationPermissionGranted())
      .then((synced) => {
        setNotifGranted(synced);
        if (!synced) {
          setError?.(
            "Alerts are still off. Choose Open Notifications settings, enable GlanceGuard, then tap Enable alerts again.",
          );
        }
      })
      .catch((err) => setError?.(String(err)))
      .finally(() => setNotifBusy(false));
  };

  const rootClass = ["settings-alerts", className].filter(Boolean).join(" ");

  return (
    <div className={rootClass}>
      <p className="muted settings-alerts__why">
        GlanceGuard uses standard macOS notifications when it thinks someone may be looking at your
        screen, even when this window is in the background. Turn them on so you do not miss that
        signal.
      </p>
      <div className="settings-alerts__actions">
        <Button
          variant="primary"
          size="small"
          disabled={notifBusy || notifGranted === true}
          type="button"
          onClick={() => onEnableAlerts()}
        >
          {notifBusy ? "Checking…" : notifGranted === true ? "Alerts enabled" : "Enable alerts"}
        </Button>
        {notifGranted === false ? (
          <Button
            variant="ghost"
            size="small"
            type="button"
            onClick={() => openMacNotificationSettings().catch((err) => setError?.(String(err)))}
          >
            Open Notifications settings
          </Button>
        ) : null}
        {notifGranted !== null ? (
          <span className="muted settings-alerts__status" aria-live="polite">
            {notifGranted
              ? "Notification access is on for GlanceGuard."
              : "Notifications are still off until macOS allows them for this app."}
          </span>
        ) : null}
      </div>
    </div>
  );
};
