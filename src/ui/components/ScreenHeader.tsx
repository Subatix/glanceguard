import type { ReactNode } from "react";

type ScreenHeaderProps = {
  eyebrow?: string;
  title: string;
  children?: ReactNode;
  align?: "left" | "center";
  className?: string;
};

export const ScreenHeader = ({
  eyebrow,
  title,
  children,
  align = "center",
  className,
}: ScreenHeaderProps) => {
  const classes = ["screen__header", align === "left" ? "screen__header--left" : "", className ?? ""]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={classes}>
      {eyebrow ? <div className="screen__eyebrow">{eyebrow}</div> : null}
      <h2>{title}</h2>
      {children ? <div className="screen__intro">{children}</div> : null}
    </div>
  );
};
