type OverlayProps = {
  message: string;
  visible: boolean;
};

export const Overlay = ({ message, visible }: OverlayProps) => {
  if (!visible) {
    return null;
  }

  return (
    <div className="overlay">
      <div className="overlay__content">
        <div className="overlay__title">Privacy Alert</div>
        <div className="overlay__message">{message}</div>
      </div>
    </div>
  );
};
