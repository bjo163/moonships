// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import App from "./App";
import { COMMAND_REGISTRY, getRegisteredCommand } from "./command-registry";

beforeEach(() => {
  window.localStorage.clear();
  window.localStorage.setItem("moonships.workspace-profile.v1", "developer");
  window.location.hash = "#/overview";
});

afterEach(cleanup);

async function openPalette(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: "Open Command Palette" }));
  return screen.getByRole("dialog", { name: "Command Palette" });
}

describe("M1-032 registered commands in the running shell", () => {
  it("lists exactly the registered commands, never arbitrary system actions", async () => {
    const user = userEvent.setup();
    render(<App />);
    const palette = await openPalette(user);
    const options = within(palette).getAllByRole("option");

    expect(options).toHaveLength(COMMAND_REGISTRY.length);
    for (const command of COMMAND_REGISTRY) {
      expect(within(palette).getByRole("option", { name: new RegExp(command.title) })).toBeTruthy();
    }
    expect(getRegisteredCommand("system.shell")).toBeUndefined();
    expect(within(palette).queryByText(/run shell/i)).toBeNull();
  });

  it("navigates to a known route from an unavailable route", async () => {
    window.location.hash = "#/unsupported";
    const user = userEvent.setup();
    render(<App />);
    expect(screen.getByRole("heading", { name: "This page is not available." })).toBeTruthy();

    const palette = await openPalette(user);
    await user.click(within(palette).getByRole("option", { name: /Go to Home/ }));

    expect(screen.queryByRole("dialog", { name: "Command Palette" })).toBeNull();
    expect(window.location.hash).toBe("#/overview");
    expect(await screen.findByRole("heading", { name: /A clear starting point/ })).toBeTruthy();
  });

  it("opens the persisted Task List from the registered Tasks navigation command", async () => {
    const user = userEvent.setup();
    render(<App />);
    const palette = await openPalette(user);
    await user.click(within(palette).getByRole("option", { name: /Open Tasks/ }));

    expect(window.location.hash).toBe("#/tasks");
    expect(await screen.findByRole("heading", { name: "Tasks" })).toBeTruthy();
    expect(getRegisteredCommand("navigation.tasks")?.kind).toBe("navigate");
  });

  it("opens Settings without invoking a registered tool", async () => {
    const user = userEvent.setup();
    render(<App />);
    const palette = await openPalette(user);
    await user.click(within(palette).getByRole("option", { name: /Open Settings/ }));

    expect(screen.queryByRole("dialog", { name: "Command Palette" })).toBeNull();
    const settings = screen.getByRole("dialog", { name: "Settings" });
    expect(within(settings).getByRole("heading", { name: "Window behavior" })).toBeTruthy();
    expect(within(settings).getByRole("checkbox", { name: /Hide to tray on close/ })).toBeTruthy();
  });

  it("opens the real local project form from the registered create command", async () => {
    const user = userEvent.setup();
    render(<App />);
    const palette = await openPalette(user);
    await user.click(within(palette).getByRole("option", { name: /New Project/ }));

    expect(window.location.hash).toBe("#/projects");
    expect(await screen.findByRole("dialog", { name: "Create project" })).toBeTruthy();
    expect(screen.queryByRole("dialog", { name: "Preview: New Project" })).toBeNull();
    expect(getRegisteredCommand("project.create")?.kind).toBe("create-project");
  });

  it.each([["New Note", "note"]])("keeps %s as a no-execution preview", async (title, entity) => {
    const user = userEvent.setup();
    render(<App />);
    const palette = await openPalette(user);
    await user.click(within(palette).getByRole("option", { name: new RegExp(title) }));

    expect(screen.queryByRole("dialog", { name: "Command Palette" })).toBeNull();
    const preview = screen.getByRole("dialog", { name: `Preview: ${title}` });
    expect(within(preview).getByText("No changes have been made.")).toBeTruthy();
    expect(within(preview).getByText(new RegExp(`create a ${entity}\\.`))).toBeTruthy();
    expect(within(preview).getByText(/no data has been created/i)).toBeTruthy();
    expect(window.location.hash).toBe("#/overview");
    expect(getRegisteredCommand(`${entity}.create`)?.kind).toBe("preview");

    await user.click(within(preview).getByRole("button", { name: "Back to workspace" }));
    expect(screen.queryByRole("dialog", { name: `Preview: ${title}` })).toBeNull();
  });

  it("opens the task form from the registered create command", async () => {
    const user = userEvent.setup();
    render(<App />);
    const palette = await openPalette(user);
    await user.click(within(palette).getByRole("option", { name: /New Task/ }));

    expect(await screen.findByRole("dialog", { name: "Create task" })).toBeTruthy();
    expect(screen.queryByRole("dialog", { name: "Preview: New Task" })).toBeNull();
    expect(getRegisteredCommand("task.create")?.kind).toBe("create-task");
  });
});
