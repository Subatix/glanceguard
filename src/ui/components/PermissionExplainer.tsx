import type { ReactNode } from "react";
import { PrivacyOnboardingFooter } from "./PrivacyOnboardingFooter";

export type PermissionExplainerProps = {
  title: string;
  body: ReactNode;
  primaryLabel: string;
  onPrimary: () => void;
  primaryDisabled?: boolean;
  secondaryLabel?: string;
  onSecondary?: () => void;
  showPrivacyFooter?: boolean;
};

export const PermissionExplainer = ({
  title,
  body,
  primaryLabel,
  onPrimary,
  primaryDisabled,
  secondaryLabel,
  onSecondary,
  showPrivacyFooter = true,
}: PermissionExplainerProps) => {
  return (
    <div className="permission-explainer">
      <h3 className="permission-explainer__title">{title}</h3>
      <div className="permission-explainer__body">{body}</div>
      <div className="permission-explainer__actions">
        {secondaryLabel && onSecondary ? (
          <button type="button" className="button button--ghost" onClick={onSecondary}>
            {secondaryLabel}
          </button>
        ) : null}
        <button
          type="button"
          className="button button--primary"
          disabled={primaryDisabled}
          onClick={onPrimary}
        >
          {primaryLabel}
        </button>
      </div>
      {showPrivacyFooter ? <PrivacyOnboardingFooter /> : null}
    </div>
  );
};
