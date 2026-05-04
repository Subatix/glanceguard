export type SegmentedOption<T extends string | number> = {
  value: T;
  label: string;
  description?: string;
};

type SegmentedControlProps<T extends string | number> = {
  label: string;
  labelledBy?: string;
  name: string;
  value: T;
  options: SegmentedOption<T>[];
  onChange: (value: T) => void;
};

export function SegmentedControl<T extends string | number>({
  label,
  labelledBy,
  name,
  value,
  options,
  onChange,
}: SegmentedControlProps<T>) {
  const groupId = labelledBy ?? `${name}-segment-label`;
  return (
    <div className="segmented" role="group" aria-labelledby={groupId}>
      <div id={groupId} className="segmented__label">
        {label}
      </div>
      <div className="segmented__tabs" role="tablist" aria-label={label}>
        {options.map((opt) => {
          const selected = opt.value === value;
          const suffix = String(opt.value).replace(/\s+/g, "-");
          const tabId = `${name}-tab-${suffix}`;
          const panelId = `${name}-panel-${suffix}`;
          return (
            <button
              key={suffix}
              type="button"
              role="tab"
              id={tabId}
              aria-selected={selected}
              aria-controls={panelId}
              className={`segmented__tab ${selected ? "is-selected" : ""}`}
              onClick={() => onChange(opt.value)}
            >
              {opt.label}
            </button>
          );
        })}
      </div>
      {options.map((opt) => {
        const selected = opt.value === value;
        if (!selected || !opt.description) {
          return null;
        }
        const suffix = String(opt.value).replace(/\s+/g, "-");
        const panelId = `${name}-panel-${suffix}`;
        const tabId = `${name}-tab-${suffix}`;
        return (
          <p
            key={`desc-${suffix}`}
            id={panelId}
            role="tabpanel"
            aria-labelledby={tabId}
            className="segmented__hint muted"
          >
            {opt.description}
          </p>
        );
      })}
    </div>
  );
}
