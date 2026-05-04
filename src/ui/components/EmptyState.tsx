import type { ReactNode } from "react";

export type EmptyStateCta = {
  label: string;
  onClick: () => void;
  variant?: "primary" | "ghost";
};

export type EmptyStateProps = {
  icon?: ReactNode;
  title: string;
  body: string;
  primaryCta?: EmptyStateCta;
};

export const EmptyState = ({ icon, title, body, primaryCta }: EmptyStateProps) => {
  const variant = primaryCta?.variant ?? "primary";
  return (
    <div className="empty-state">
      {icon ? <div className="empty-state__icon">{icon}</div> : null}
      <div className="empty-state__title">{title}</div>
      <p className="empty-state__body">{body}</p>
      {primaryCta ? (
        <button
          type="button"
          className={`button ${variant === "primary" ? "button--primary" : "button--ghost"}`}
          onClick={primaryCta.onClick}
        >
          {primaryCta.label}
        </button>
      ) : null}
    </div>
  );
};

export const emptyStatePresets = {
  noCamera: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>📷</span>,
    title: "No camera found",
    body: "Connect a camera or grant access, then try again.",
    primaryCta: cta,
  }),
  cameraBusy: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>⏳</span>,
    title: "Camera may be in use",
    body: "Quit other apps using the camera (FaceTime, Zoom, etc.), then retry.",
    primaryCta: cta,
  }),
  noPermission: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>🚫</span>,
    title: "Camera permission needed",
    body: "Allow GlanceGuard in System Settings → Privacy & Security → Camera, then return here.",
    primaryCta: cta,
  }),
  modelFailed: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>⚠️</span>,
    title: "Models unavailable",
    body: "Face models are missing or did not pass verification. Check your download URL or use local ONNX files.",
    primaryCta: cta,
  }),
  ownerNotEnrolled: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>👤</span>,
    title: "Owner not enrolled",
    body: "Enroll your face once so monitoring can tell you apart from observers.",
    primaryCta: cta,
  }),
};
