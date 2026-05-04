import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Overlay } from "./Overlay";

describe("Overlay", () => {
  it("renders nothing when not visible", () => {
    const { container } = render(<Overlay visible={false} message="Hi" />);
    expect(container.firstChild).toBeNull();
  });

  it("renders alert copy when visible", () => {
    render(<Overlay visible message="Someone may be observing" />);
    expect(screen.getByText("Privacy Alert")).toBeInTheDocument();
    expect(screen.getByText("Someone may be observing")).toBeInTheDocument();
  });
});
