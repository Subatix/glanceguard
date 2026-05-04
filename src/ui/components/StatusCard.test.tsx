import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatusCard } from "./StatusCard";

describe("StatusCard", () => {
  it("renders observer score or dash", () => {
    render(<StatusCard status="monitoring" observerScore={0.712} />);
    expect(screen.getByText("Monitoring")).toBeInTheDocument();
    expect(screen.getByText("0.71")).toBeInTheDocument();
  });

  it("shows error text when provided", () => {
    render(<StatusCard status="idle" error="Camera blocked" />);
    expect(screen.getByText("Camera blocked")).toBeInTheDocument();
  });
});
