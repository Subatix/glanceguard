import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { CameraSelect } from "./CameraSelect";

const cameras = [
  {
    id: { kind: "Index" as const, value: 0 },
    name: "Built-in",
    description: "",
  },
  {
    id: { kind: "StableId" as const, value: "usb-1" },
    name: "USB Cam",
    description: "",
  },
];

describe("CameraSelect", () => {
  it("shows empty state when no cameras and retry is available", () => {
    const onRetry = vi.fn();
    render(<CameraSelect cameras={[]} selected={undefined} onChange={vi.fn()} onRetryList={onRetry} />);
    expect(screen.getByText(/No camera found/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Refresh camera list/i }));
    expect(onRetry).toHaveBeenCalled();
  });

  it("lists cameras and emits selection on change", () => {
    const onChange = vi.fn();
    render(
      <CameraSelect
        cameras={cameras}
        selected={{ kind: "Index", value: 0 }}
        onChange={onChange}
      />,
    );

    expect(screen.getByRole("option", { name: "Built-in" })).toBeInTheDocument();
    fireEvent.change(screen.getByRole("combobox"), {
      target: { value: "stable:usb-1" },
    });
    expect(onChange).toHaveBeenCalledWith({ kind: "StableId", value: "usb-1" });
  });
});
