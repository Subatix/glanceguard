import type { HTMLAttributes } from "react";

type SurfaceTone = "panel" | "card" | "quiet";

type SurfaceProps = HTMLAttributes<HTMLDivElement> & {
  tone?: SurfaceTone;
};

export const Surface = ({ tone = "panel", className, ...props }: SurfaceProps) => {
  const classes = ["surface", `surface--${tone}`, className ?? ""].filter(Boolean).join(" ");

  return <div className={classes} {...props} />;
};
