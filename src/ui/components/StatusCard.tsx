import type { MonitorStatus } from "../../state/monitorStore";
import { StatusPill } from "./StatusPill";
import { Surface } from "./Surface";

type StatusCardProps = {
  variant?: "default" | "skeleton";
  status?: MonitorStatus;
  observerScore?: number | null;
  error?: string;
};

const statusLabels: Record<MonitorStatus, string> = {
  idle: "Idle",
  monitoring: "Monitoring",
  alert: "Alert",
  cooldown: "Cooldown",
};

const statusTones: Record<MonitorStatus, "neutral" | "success" | "warning" | "danger"> = {
  idle: "neutral",
  monitoring: "success",
  alert: "danger",
  cooldown: "warning",
};

export const StatusCard = ({ variant = "default", status, observerScore, error }: StatusCardProps) => {
  if (variant === "skeleton") {
    return (
      <Surface tone="card" className="card card--skeleton" aria-busy="true" aria-label="Loading status">
        <div className="skeleton-line skeleton-line--title" />
        <div className="skeleton-line skeleton-line--hero" />
        <div className="skeleton-line skeleton-line--row" />
        <div className="skeleton-line skeleton-line--row-short" />
      </Surface>
    );
  }

  if (!status) {
    return null;
  }

  return (
    <Surface tone="card" className="card status-card">
      <div className="card__title">Status</div>
      <StatusPill tone={statusTones[status]} className={`status status--${status}`}>
        {statusLabels[status]}
      </StatusPill>
      <div className="status__row">
        <span>Observer score</span>
        <span>{typeof observerScore === "number" ? observerScore.toFixed(2) : "—"}</span>
      </div>
      {error ? <div className="status__error">{error}</div> : null}
    </Surface>
  );
};
