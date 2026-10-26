// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { JsonDevToolsPage } from "./JsonDevToolsPage";

afterEach(cleanup);

describe("M3-015 JSON DevTools page", () => {
  it("validates and formats input locally while preserving numeric tokens", () => {
    render(<JsonDevToolsPage />);
    fireEvent.change(screen.getByLabelText("JSON input"), {
      target: { value: '{"n":9007199254740993}' },
    });
    fireEvent.click(screen.getByRole("button", { name: "Validate & format" }));

    expect(screen.getByRole("status").textContent).toContain("Valid JSON");
    expect((screen.getByLabelText("Formatted JSON") as HTMLTextAreaElement).value).toContain(
      '"n": 9007199254740993',
    );
    expect(screen.getByText(/no provider or network requests/i)).toBeTruthy();
  });

  it("shows a clear validation error for malformed input", () => {
    render(<JsonDevToolsPage />);
    fireEvent.change(screen.getByLabelText("JSON input"), { target: { value: "{bad}" } });
    fireEvent.click(screen.getByRole("button", { name: "Validate & format" }));

    expect(screen.getByRole("alert").textContent).toContain("Input is not valid JSON.");
    expect(screen.getByLabelText("JSON input").getAttribute("aria-invalid")).toBe("true");
  });

  it("returns parser codes and source positions for invalid YAML", () => {
    render(<JsonDevToolsPage />);
    fireEvent.click(screen.getByRole("button", { name: "YAML formatter" }));
    fireEvent.change(screen.getByLabelText("YAML input"), {
      target: { value: "items: [unterminated\n" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Validate & format" }));

    expect(screen.getByRole("alert").textContent).toContain("Input is not valid YAML");
    expect(screen.getByRole("list", { name: "YAML parser issues" }).textContent).toMatch(
      /[A-Z_]+ · line \d+, column \d+/,
    );
  });
});
