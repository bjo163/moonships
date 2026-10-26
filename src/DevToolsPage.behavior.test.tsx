// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { DevToolsPage } from "./DevToolsPage";

afterEach(cleanup);

describe("offline DevTools page", () => {
  it("validates and formats input locally while preserving numeric tokens", () => {
    render(<DevToolsPage />);
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
    render(<DevToolsPage />);
    fireEvent.change(screen.getByLabelText("JSON input"), { target: { value: "{bad}" } });
    fireEvent.click(screen.getByRole("button", { name: "Validate & format" }));

    expect(screen.getByRole("alert").textContent).toContain("Input is not valid JSON.");
    expect(screen.getByLabelText("JSON input").getAttribute("aria-invalid")).toBe("true");
  });

  it("returns parser codes and source positions for invalid YAML", () => {
    render(<DevToolsPage />);
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

  it("keeps binary Base64 decoding distinct from UTF-8 text", () => {
    render(<DevToolsPage />);
    fireEvent.click(screen.getByRole("button", { name: "Base64" }));
    fireEvent.click(screen.getByRole("button", { name: "Base64 → bytes" }));
    fireEvent.change(screen.getByLabelText("Base64 input"), {
      target: { value: "AH+A/w==" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Decode Base64" }));

    expect(screen.getByRole("status").textContent).toContain("Decoded as binary");
    expect(screen.getByText(/00 7f 80 ff/).textContent).toContain("00 7f 80 ff");
  });

  it("preserves URL separators in full URL mode", () => {
    render(<DevToolsPage />);
    fireEvent.click(screen.getByRole("button", { name: "URL" }));
    fireEvent.click(screen.getByRole("button", { name: "Encode full URL" }));
    fireEvent.change(screen.getByLabelText("Absolute URL input"), {
      target: { value: "https://example.com/a b?q=one&next=/path#section" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Encode URL" }));

    expect((screen.getByLabelText("URL output") as HTMLTextAreaElement).value).toBe(
      "https://example.com/a%20b?q=one&next=/path#section",
    );
  });
});
