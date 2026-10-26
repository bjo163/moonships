// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ProjectsPage } from "./ProjectsPage";
import type { ProjectRecord } from "./projects-api";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const project = {
  id: "project-1",
  workspaceId: "00000000-0000-4000-8000-000000000001",
  name: "Moonships",
  projectKey: null,
  description: "A local workbench",
  goal: "Ship the first workflow",
  status: "active" as const,
  archivedAt: null,
  createdAt: "2026-10-25T00:00:00Z",
  updatedAt: "2026-10-26T00:00:00Z",
};

describe("M2-014 persisted Projects screen", () => {
  it("loads persisted projects and edits then reloads the saved record", async () => {
    const user = userEvent.setup();
    let saved: ProjectRecord = project;
    invokeMock.mockImplementation(
      async (command: string, payload?: { request?: Record<string, unknown> }) => {
        if (command === "list_projects") return { projects: [saved] };
        if (command === "update_project") {
          saved = { ...saved, ...(payload?.request as object), updatedAt: "2026-10-26T01:00:00Z" };
          return saved;
        }
        if (command === undefined) return { projects: [saved] };
        throw new Error(
          `Unexpected command: ${String(command)}; calls: ${JSON.stringify(invokeMock.mock.calls)}; stack: ${new Error().stack}`,
        );
      },
    );

    render(<ProjectsPage />);
    expect(await screen.findByRole("row", { name: /Moonships/ })).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("list_projects", { request: {} });

    await user.click(screen.getByRole("button", { name: "Edit" }));
    const dialog = screen.getByRole("dialog", { name: "Edit project" });
    expect(within(dialog).getByRole("button", { name: "Close dialog" }).getAttribute("title")).toBe(
      "Close dialog",
    );
    const name = within(dialog).getByRole("textbox", { name: "Project name" });
    await user.clear(name);
    await user.type(name, "Moonships MVP");
    await user.click(within(dialog).getByRole("button", { name: "Save changes" }));

    expect(await screen.findByRole("row", { name: /Moonships MVP/ })).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("update_project", {
      request: {
        id: "project-1",
        name: "Moonships MVP",
        description: "A local workbench",
        goal: "Ship the first workflow",
        status: "active",
      },
    });
    expect(invokeMock.mock.calls.filter(([command]) => command === "list_projects")).toHaveLength(
      2,
    );
  });

  it("creates a project from the form and confirms it by reading the saved list", async () => {
    const user = userEvent.setup();
    let saved: ProjectRecord | null = null;
    invokeMock.mockImplementation(
      async (command: string, payload?: { request?: Record<string, unknown> }) => {
        if (command === "list_projects") return { projects: saved ? [saved] : [] };
        if (command === "create_project") {
          saved = { ...project, ...(payload?.request as object), status: "planned" };
          return saved;
        }
        if (command === undefined) return { projects: saved ? [saved] : [] };
        throw new Error(
          `Unexpected command: ${String(command)}; calls: ${JSON.stringify(invokeMock.mock.calls)}; stack: ${new Error().stack}`,
        );
      },
    );

    render(<ProjectsPage />);
    await user.click(await screen.findByRole("button", { name: "Create your first project" }));
    const dialog = screen.getByRole("dialog", { name: "Create project" });
    await user.type(within(dialog).getByRole("textbox", { name: "Project name" }), "New project");
    await user.type(
      within(dialog).getByRole("textbox", { name: "Goal" }),
      "Deliver a usable workflow",
    );
    await user.click(within(dialog).getByRole("button", { name: "Create project" }));

    expect(await screen.findByRole("row", { name: /New project/ })).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("create_project", {
      request: { name: "New project", goal: "Deliver a usable workflow", description: null },
    });
    expect(invokeMock.mock.calls.filter(([command]) => command === "list_projects")).toHaveLength(
      2,
    );
  });

  it("distinguishes unavailable data from an empty list and retries", async () => {
    const user = userEvent.setup();
    invokeMock
      .mockRejectedValueOnce(new Error("not running in desktop"))
      .mockResolvedValueOnce({ projects: [] });
    render(<ProjectsPage />);

    expect(await screen.findByText("Projects are unavailable")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText("No active projects yet")).toBeTruthy();
  });

  it("opens a persisted Project overview from its list row", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "list_projects") return { projects: [project] };
      if (command === undefined) return { projects: [project] };
      throw new Error(`Unexpected command: ${command}`);
    });
    const onOpenProject = vi.fn();
    render(<ProjectsPage onOpenProject={onOpenProject} />);
    const row = await screen.findByRole("row", { name: /Moonships/ });

    await user.click(within(row).getByRole("button", { name: "Open" }));
    expect(onOpenProject).toHaveBeenCalledWith("project-1");
  });
});
