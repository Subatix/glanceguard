type OverlayProps = {
  message: string;
  visible: boolean;
};

export const Overlay = ({ message, visible }: OverlayProps) => {
  if (!visible) {
    return null;
  }

  return (
    <div className="overlay" role="alert" aria-live="assertive">
      <div className="overlay__content">
        <div className="overlay__title">GlanceGuard Alert</div>
        <div className="overlay__message">{message}</div>
      </div>
    </div>
  );
};
