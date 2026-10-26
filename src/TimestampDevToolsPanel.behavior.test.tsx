// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { DevToolsPage } from "./DevToolsPage";

afterEach(cleanup);

describe("M3-022 timestamp DevTools panel", () => {
  it("shows UTC, device-local timezone and both Unix units", () => {
    render(<DevToolsPage />);
    fireEvent.click(screen.getByRole("button", { name: "Timestamp" }));
    fireEvent.change(screen.getByLabelText("Unix timestamp input"), { target: { value: "0" } });
    fireEvent.click(screen.getByRole("button", { name: "Convert timestamp" }));

    expect(screen.getByRole("status").textContent).toContain("Timestamp converted");
    expect(screen.getByText("1970-01-01T00:00:00.000Z")).toBeTruthy();
    expect(screen.getByText(/^Local time \(.+\)$/)).toBeTruthy();
    expect(screen.getByText("Unix timestamp in seconds")).toBeTruthy();
    expect(screen.getByText("Unix timestamp in milliseconds")).toBeTruthy();
  });

  it("shows a clear validation error for invalid input and supports ISO with an offset", () => {
    render(<DevToolsPage />);
    fireEvent.click(screen.getByRole("button", { name: "Timestamp" }));
    fireEvent.change(screen.getByLabelText("Unix timestamp input"), { target: { value: "1.2" } });
    fireEvent.click(screen.getByRole("button", { name: "Convert timestamp" }));
    expect(screen.getByRole("alert").textContent).toContain("whole-number Unix timestamp");

    fireEvent.click(screen.getByRole("button", { name: "ISO 8601 date" }));
    fireEvent.change(screen.getByLabelText("ISO 8601 date and time input"), {
      target: { value: "1970-01-01T07:00:00+07:00" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Convert timestamp" }));
    expect(screen.getByText("1970-01-01T00:00:00.000Z")).toBeTruthy();
  });
});
