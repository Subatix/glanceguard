import type { MonitorStatus } from "../../state/appStore";

type StatusCardProps = {
  status: MonitorStatus;
  observerScore?: number | null;
  error?: string;
};

const statusLabels: Record<MonitorStatus, string> = {
  idle: "Idle",
  monitoring: "Monitoring",
  alert: "Alert",
  cooldown: "Cooldown",
};

export const StatusCard = ({ status, observerScore, error }: StatusCardProps) => {
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
