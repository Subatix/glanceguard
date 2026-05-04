import type { ReactNode } from "react";
import { Button } from "./Button";

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
        <Button variant={variant} onClick={primaryCta.onClick}>
          {primaryCta.label}
        </Button>
      ) : null}
    </div>
  );
};

export const emptyStatePresets = {
  noCamera: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>Camera</span>,
    title: "No camera found",
    body: "Connect a camera or grant access, then try again.",
    primaryCta: cta,
  }),
  cameraBusy: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>Busy</span>,
    title: "Camera may be in use",
    body: "Quit other apps using the camera (FaceTime, Zoom, etc.), then retry.",
    primaryCta: cta,
  }),
  noPermission: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>Permission</span>,
    title: "Camera permission needed",
    body: "Allow GlanceGuard in System Settings → Privacy & Security → Camera, then return here.",
    primaryCta: cta,
  }),
  modelFailed: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>Models</span>,
    title: "Models unavailable",
    body: "Bundled face models are missing or did not pass verification. Install a fresh build of GlanceGuard.",
    primaryCta: cta,
  }),
  ownerNotEnrolled: (cta: EmptyStateCta): EmptyStateProps => ({
    icon: <span aria-hidden>Owner</span>,
    title: "Owner not enrolled",
    body: "Enroll your face once so monitoring can tell you apart from observers.",
    primaryCta: cta,
  }),
};
