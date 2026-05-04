import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import {
  alertHeadline,
  alertOverlaySupporting,
} from "../../messages/alertExperience";
import { Overlay } from "./Overlay";

describe("Overlay", () => {
  it("renders nothing when not visible", () => {
    const { container } = render(<Overlay visible={false} />);
    expect(container.firstChild).toBeNull();
  });

  it("renders friendly alert copy when visible", () => {
    render(<Overlay visible />);
    expect(screen.getByText(alertHeadline)).toBeInTheDocument();
    expect(screen.getByText(alertOverlaySupporting)).toBeInTheDocument();
  });
});
