import type { SVGProps } from "react";

type GlanceGuardLogoProps = SVGProps<SVGSVGElement>;

export const GlanceGuardLogo = (props: GlanceGuardLogoProps) => (
  <svg viewBox="0 0 32 32" fill="none" {...props}>
    <path
      d="M13.4 13.42 A 5.32 5.32 0 1 1 13.4 18.58"
      stroke="currentColor"
      strokeWidth="2.21"
      strokeLinecap="round"
    />
    <path
      d="M8.8 16 L 14.42 16"
      stroke="currentColor"
      strokeWidth="2.21"
      strokeLinecap="round"
    />
    <path
      d="M18.6 13.42 A 5.32 5.32 0 1 0 18.6 18.58"
      stroke="currentColor"
      strokeWidth="2.21"
      strokeLinecap="round"
    />
    <path
      d="M17.58 16 L 23.2 16"
      stroke="currentColor"
      strokeWidth="2.21"
      strokeLinecap="round"
    />
  </svg>
);
