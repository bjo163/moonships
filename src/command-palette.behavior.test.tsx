// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CommandPalette, type PaletteCommand } from "./CommandPalette";

afterEach(cleanup);

const commands: PaletteCommand[] = [
  {
    id: "navigation.overview",
    title: "Go Home",
    description: "Open the overview",
    category: "Navigation",
    risk: "read-only",
  },
  {
    id: "project.create",
    title: "New Project",
    description: "Preview project creation",
    category: "Create",
    risk: "state-change",
  },
  {
    id: "task.create",
    title: "New Task",
    description: "Preview task creation",
    category: "Create",
    risk: "state-change",
  },
];

describe("M1-031 Command Palette DOM behaviors", () => {
  it("focuses search on opening, filters results and only selects after Enter", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const { rerender } = render(
      <CommandPalette open={false} commands={commands} onClose={() => {}} onSelect={onSelect} />,
    );
    expect(screen.queryByRole("dialog")).toBeNull();
    rerender(<CommandPalette open commands={commands} onClose={() => {}} onSelect={onSelect} />);
    const dialog = screen.getByRole("dialog", { name: "Command Palette" });
    const search = within(dialog).getByRole("searchbox", { name: "Search commands" });
    expect(document.activeElement).toBe(search);

    await user.type(search, "project");
    expect(within(dialog).getAllByRole("option")).toHaveLength(1);
    expect(within(dialog).getByRole("option", { name: /New Project/ }).textContent).toContain(
      "State change",
    );
    expect(onSelect).not.toHaveBeenCalled();

    await user.keyboard("{Enter}");
    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(onSelect).toHaveBeenCalledWith(commands[1]);
  });

  it("cycles via Arrow keys, highlights the active option, and Escape does not select", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const onClose = vi.fn();
    render(<CommandPalette open commands={commands} onClose={onClose} onSelect={onSelect} />);
    const search = screen.getByRole("searchbox");
    expect(search.getAttribute("aria-activedescendant")).toBe("palette-command-navigation.overview");

    await user.keyboard("{ArrowUp}");
    expect(search.getAttribute("aria-activedescendant")).toBe("palette-command-task.create");
    expect(screen.getByRole("option", { name: /New Task/ }).getAttribute("aria-selected")).toBe(
      "true",
    );
    await user.keyboard("{ArrowDown}");
    expect(search.getAttribute("aria-activedescendant")).toBe("palette-command-navigation.overview");
    await user.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(onSelect).not.toHaveBeenCalled();
  });

  it("avoids stale active descendants and suppresses selection while loading or empty", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const { rerender } = render(
      <CommandPalette open loading commands={commands} onClose={() => {}} onSelect={onSelect} />,
    );
    const search = screen.getByRole("searchbox");
    expect(screen.getByRole("status").textContent).toContain("Loading commands");
    expect(search.hasAttribute("aria-activedescendant")).toBe(false);
    expect(screen.queryByRole("option")).toBeNull();
    await user.keyboard("{Enter}");
    expect(onSelect).not.toHaveBeenCalled();

    rerender(<CommandPalette open commands={[]} onClose={() => {}} onSelect={onSelect} />);
    expect(screen.getByRole("status").textContent).toContain("No commands are registered yet");
    expect(search.hasAttribute("aria-activedescendant")).toBe(false);
    await user.type(search, "missing");
    expect(screen.getByRole("status").textContent).toContain("No commands match this search");
    expect(onSelect).not.toHaveBeenCalled();
  });

  it("recovers from stale selected index when the registered command collection shrinks", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const { rerender } = render(
      <CommandPalette open commands={commands} onClose={() => {}} onSelect={onSelect} />,
    );
    const search = screen.getByRole("searchbox");
    await user.keyboard("{ArrowDown}{ArrowDown}");
    expect(search.getAttribute("aria-activedescendant")).toBe("palette-command-task.create");

    rerender(
      <CommandPalette open commands={[commands[0]]} onClose={() => {}} onSelect={onSelect} />,
    );
    expect(search.getAttribute("aria-activedescendant")).toBe("palette-command-navigation.overview");
    await user.keyboard("{Enter}");
    expect(onSelect).toHaveBeenCalledWith(commands[0]);
  });

  it("clears an old search when closed and reopened", async () => {
    const user = userEvent.setup();
    const { rerender } = render(
      <CommandPalette open commands={commands} onClose={() => {}} onSelect={() => {}} />,
    );
    await user.type(screen.getByRole("searchbox"), "missing");
    expect(screen.getByRole("status").textContent).toContain("No commands match");

    rerender(
      <CommandPalette open={false} commands={commands} onClose={() => {}} onSelect={() => {}} />,
    );
    rerender(<CommandPalette open commands={commands} onClose={() => {}} onSelect={() => {}} />);
    const search = screen.getByRole("searchbox") as HTMLInputElement;
    expect(search.value).toBe("");
    expect(document.activeElement).toBe(search);
    expect(screen.getAllByRole("option")).toHaveLength(commands.length);
  });
});
