import type { ReactNode } from "react";

type PreferenceGroupProps = {
  title: string;
  description?: string;
  children: ReactNode;
};

type PreferenceRowProps = {
  label: string;
  hint?: string;
  children: ReactNode;
};

const titleId = (title: string) =>
  `${title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/(^-|-$)/g, "")}-title`;

export const PreferenceGroup = ({ title, description, children }: PreferenceGroupProps) => {
  const headingId = titleId(title);

  return (
    <section className="preference-group" aria-labelledby={headingId}>
      <div className="preference-group__header">
        <h3 id={headingId}>{title}</h3>
        {description ? <p>{description}</p> : null}
      </div>
      <div className="preference-group__rows">{children}</div>
    </section>
  );
};

export const PreferenceRow = ({ label, hint, children }: PreferenceRowProps) => (
  <div className="preference-row">
    <div className="preference-row__copy">
      <div className="preference-row__label">{label}</div>
      {hint ? <p>{hint}</p> : null}
    </div>
    <div className="preference-row__control">{children}</div>
  </div>
);
