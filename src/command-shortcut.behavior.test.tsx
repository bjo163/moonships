// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import App from "./App";

beforeEach(() => {
  window.localStorage.clear();
  window.localStorage.setItem("moonships.workspace-profile.v1", "developer");
  window.location.hash = "#/overview";
});
afterEach(cleanup);

describe("M1-033 shortcut and focus lifecycle", () => {
  it("opens with Ctrl+K, focuses search and restores the previous control on Escape", async () => {
    const user = userEvent.setup();
    render(<App />);
    const opener = screen.getByRole("button", { name: "Change workspace profile" });
    opener.focus();
    await user.keyboard("{Control>}k{/Control}");
    const dialog = screen.getByRole("dialog", { name: "Command Palette" });
    expect(document.activeElement).toBe(within(dialog).getByRole("searchbox"));

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Command Palette" })).toBeNull();
    expect(document.activeElement).toBe(opener);
  });

  it("closes on Escape even when focus is on the close button", async () => {
    const user = userEvent.setup();
    render(<App />);
    const opener = screen.getByRole("button", { name: "Open Command Palette" });
    await user.click(opener);
    const dialog = screen.getByRole("dialog", { name: "Command Palette" });
    const close = within(dialog).getByRole("button", { name: "Close Command Palette" });
    close.focus();
    expect(document.activeElement).toBe(close);
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Command Palette" })).toBeNull();
    expect(document.activeElement).toBe(opener);
  });

  it("ignores Ctrl+K in editable inputs and textareas", async () => {
    const user = userEvent.setup();
    render(<App />);
    for (const tag of ["input", "textarea"] as const) {
      const target = document.createElement(tag);
      document.body.appendChild(target);
      target.focus();
      await user.keyboard("{Control>}k{/Control}");
      expect(screen.queryByRole("dialog", { name: "Command Palette" })).toBeNull();
      target.remove();
    }
  });

  it("ignores Ctrl+K in Settings and focuses the new dialog rather than the old palette", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Open Command Palette" }));
    const palette = screen.getByRole("dialog", { name: "Command Palette" });
    await user.click(within(palette).getByRole("option", { name: /Open Settings/ }));
    const settings = screen.getByRole("dialog", { name: "Settings" });
    expect(document.activeElement).toBe(
      within(settings).getByRole("button", { name: "Close settings" }),
    );
    await user.keyboard("{Control>}k{/Control}");
    expect(screen.queryByRole("dialog", { name: "Command Palette" })).toBeNull();
  });

  it("focuses the Task creation dialog after leaving the palette", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Open Command Palette" }));
    const palette = screen.getByRole("dialog", { name: "Command Palette" });
    await user.click(within(palette).getByRole("option", { name: /New Task/ }));
    const preview = await screen.findByRole("dialog", { name: "Create task" });
    expect(document.activeElement).toBe(
      within(preview).getByRole("button", { name: "Close dialog" }),
    );
  });

  it("opens Task creation with Ctrl+Alt+N and exposes the shortcut on its button", async () => {
    const user = userEvent.setup();
    render(<App />);
    const createButton = screen.getByRole("button", { name: /New task/ });
    expect(createButton.getAttribute("aria-keyshortcuts")).toBe("Control+Alt+N Meta+Alt+N");
    expect(createButton.textContent).toContain("Ctrl+Alt+N");

    await user.keyboard("{Control>}{Alt>}n{/Alt}{/Control}");
    const dialog = await screen.findByRole("dialog", { name: "Create task" });
    expect(document.activeElement).toBe(
      within(dialog).getByRole("button", { name: "Close dialog" }),
    );
  });

  it("does not open Task creation from an editable field or while a dialog is open", async () => {
    const user = userEvent.setup();
    render(<App />);
    const input = document.createElement("input");
    document.body.appendChild(input);
    input.focus();
    await user.keyboard("{Control>}{Alt>}n{/Alt}{/Control}");
    expect(screen.queryByRole("dialog", { name: "Create task" })).toBeNull();
    input.remove();

    await user.click(screen.getByRole("button", { name: "Open Command Palette" }));
    await user.keyboard("{Control>}{Alt>}n{/Alt}{/Control}");
    expect(screen.getByRole("dialog", { name: "Command Palette" })).toBeTruthy();
    expect(screen.queryByRole("dialog", { name: "Create task" })).toBeNull();
  });
});
