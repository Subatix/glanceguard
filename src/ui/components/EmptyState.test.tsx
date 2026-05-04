import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { EmptyState, emptyStatePresets } from "./EmptyState";

describe("EmptyState", () => {
  it("renders owner-not-enrolled preset with CTA", () => {
    const onClick = () => undefined;
    render(<EmptyState {...emptyStatePresets.ownerNotEnrolled({ label: "Go", onClick })} />);
    expect(screen.getByText(/Owner not enrolled/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Go" })).toBeInTheDocument();
  });
});
