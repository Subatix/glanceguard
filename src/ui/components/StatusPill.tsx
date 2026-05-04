import type { HTMLAttributes, ReactNode } from "react";

type StatusTone = "neutral" | "success" | "warning" | "danger" | "info";

type StatusPillProps = HTMLAttributes<HTMLSpanElement> & {
  tone?: StatusTone;
  children: ReactNode;
};

export const StatusPill = ({ tone = "neutral", className, children, ...props }: StatusPillProps) => {
  const classes = ["status-pill", `status-pill--${tone}`, className ?? ""].filter(Boolean).join(" ");

  return (
    <span className={classes} {...props}>
      <span className="status-pill__dot" aria-hidden />
      {children}
    </span>
  );
};
