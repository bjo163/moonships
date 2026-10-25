// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import App from "./App";

beforeEach(() => {
  window.localStorage.clear();
  window.location.hash = "#/overview";
});
afterEach(cleanup);

describe("M1-034 first-run profile onboarding", () => {
  it("explains local-first boundaries and saves the chosen profile across remounts", async () => {
    const user = userEvent.setup();
    const view = render(<App />);
    const dialog = screen.getByRole("dialog", { name: "Welcome to Moonships Workbench" });

    expect(within(dialog).getByText(/No account, cloud connection, or API key/)).toBeTruthy();
    expect(within(dialog).getByText(/startup with Windows are not enabled/)).toBeTruthy();
    expect(within(dialog).getByText(/Profiles never grant permissions/)).toBeTruthy();
    expect(dialog.contains(document.activeElement)).toBe(true);
    await user.click(within(dialog).getByRole("button", { name: /Network \/ NOC/ }));

    expect(screen.queryByRole("dialog")).toBeNull();
    expect(window.localStorage.getItem("moonships.workspace-profile.v1")).toBe("network_noc");
    expect(screen.getByRole("button", { name: "Change workspace profile" }).textContent).toContain(
      "Network / NOC",
    );

    view.unmount();
    render(<App />);
    expect(screen.queryByRole("dialog", { name: "Welcome to Moonships Workbench" })).toBeNull();
    expect(screen.getByRole("button", { name: "Change workspace profile" }).textContent).toContain(
      "Network / NOC",
    );
  });

  it("reopens for profile changes and Escape restores trigger focus without changing selection", async () => {
    window.localStorage.setItem("moonships.workspace-profile.v1", "developer");
    const user = userEvent.setup();
    render(<App />);
    const trigger = screen.getByRole("button", { name: "Change workspace profile" });
    await user.click(trigger);
    const dialog = screen.getByRole("dialog", { name: "Choose your workspace profile" });
    expect(dialog.contains(document.activeElement)).toBe(true);

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(trigger);
    expect(window.localStorage.getItem("moonships.workspace-profile.v1")).toBe("developer");
  });

  it("keeps keyboard focus in the selector and dismissal on first run does not save a profile", async () => {
    const user = userEvent.setup();
    render(<App />);
    const dialog = screen.getByRole("dialog", { name: "Welcome to Moonships Workbench" });
    const close = within(dialog).getByRole("button", { name: "Close workspace profile selector" });
    const focusable = Array.from(dialog.querySelectorAll<HTMLElement>("*")).filter((element) =>
      element.matches("button:not(:disabled)"),
    );
    expect(document.activeElement).toBe(close);
    expect(fireEvent.keyDown(close, { key: "Tab", shiftKey: true, cancelable: true })).toBe(false);
    expect(document.activeElement).toBe(focusable[focusable.length - 1]);
    expect(
      fireEvent.keyDown(focusable[focusable.length - 1], { key: "Tab", cancelable: true }),
    ).toBe(false);
    expect(document.activeElement).toBe(close);

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(window.localStorage.getItem("moonships.workspace-profile.v1")).toBeNull();
  });
});
