import type { MonitorStatus } from "../../state/appStore";

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

export const StatusCard = ({ variant = "default", status, observerScore, error }: StatusCardProps) => {
  if (variant === "skeleton") {
    return (
      <div className="card card--skeleton" aria-busy="true" aria-label="Loading status">
        <div className="skeleton-line skeleton-line--title" />
        <div className="skeleton-line skeleton-line--hero" />
        <div className="skeleton-line skeleton-line--row" />
        <div className="skeleton-line skeleton-line--row-short" />
      </div>
    );
  }

  if (!status) {
    return null;
  }

  return (
    <div className="card">
      <div className="card__title">Status</div>
      <div className={`status status--${status}`}>{statusLabels[status]}</div>
      <div className="status__row">
        <span>Observer score</span>
        <span>{typeof observerScore === "number" ? observerScore.toFixed(2) : "—"}</span>
      </div>
      {error ? <div className="status__error">{error}</div> : null}
    </div>
  );
};
