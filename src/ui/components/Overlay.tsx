import { alertHeadline, alertOverlaySupporting } from "../../messages/alertExperience";

type OverlayProps = {
  visible: boolean;
};

export const Overlay = ({ visible }: OverlayProps) => {
  if (!visible) {
    return null;
  }

  return (
    <div className="overlay" role="alert" aria-live="assertive">
      <div className="overlay__content">
        <div className="overlay__title">{alertHeadline}</div>
        <div className="overlay__message">{alertOverlaySupporting}</div>
      </div>
    </div>
  );
};
