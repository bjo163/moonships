// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { DevToolsPage } from "./DevToolsPage";

afterEach(cleanup);

describe("M3-021 text diff DevTools panel", () => {
  it("shows inserted and deleted lines from local text inputs", () => {
    render(<DevToolsPage />);
    fireEvent.click(screen.getByRole("button", { name: "Text diff" }));
    fireEvent.change(screen.getByLabelText("Original text"), { target: { value: "old\n" } });
    fireEvent.change(screen.getByLabelText("Updated text"), { target: { value: "new\n" } });
    fireEvent.click(screen.getByRole("button", { name: "Compare text" }));

    expect(screen.getByRole("status").textContent).toContain("Differences found");
    expect((screen.getByLabelText("Unified text diff") as HTMLTextAreaElement).value).toBe(
      "-old\n+new\n",
    );
    expect(screen.getByText(/no provider or network requests/i)).toBeTruthy();
  });

  it("reports that empty inputs match", () => {
    render(<DevToolsPage />);
    fireEvent.click(screen.getByRole("button", { name: "Text diff" }));
    fireEvent.click(screen.getByRole("button", { name: "Compare text" }));

    expect(screen.getByRole("status").textContent).toContain("Inputs match");
    expect((screen.getByLabelText("Unified text diff") as HTMLTextAreaElement).value).toBe("");
  });
});
