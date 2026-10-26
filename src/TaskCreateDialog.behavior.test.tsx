// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TaskCreateDialog } from "./TaskCreateDialog";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const project = {
  id: "project-1",
  workspaceId: "00000000-0000-4000-8000-000000000001",
  name: "Moonships",
  projectKey: null,
  description: null,
  goal: null,
  status: "active" as const,
  archivedAt: null,
  createdAt: "2026-10-26T00:00:00Z",
  updatedAt: "2026-10-26T00:00:00Z",
};

function task(request: Record<string, unknown>) {
  return {
    id: "task-1",
    workspaceId: project.workspaceId,
    projectId: request.projectId,
    title: request.title,
    description: null,
    dueAt: null,
    status: request.status,
    priority: request.priority,
    createdAt: "2026-10-26T00:00:00Z",
    updatedAt: "2026-10-26T00:00:00Z",
    archivedAt: null,
  };
}

describe("M2-016 persisted Task creation dialog", () => {
  it("validates a required title accessibly and Escape never saves", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "list_projects") return { projects: [project] };
      if (command === undefined) return { projects: [project] };
      throw new Error(`Unexpected command: ${command}`);
    });
    const onClose = vi.fn();
    render(<TaskCreateDialog open onClose={onClose} />);
    const dialog = await screen.findByRole("dialog", { name: "Create task" });

    expect(
      (within(dialog).getByRole("combobox", { name: "Status" }) as HTMLSelectElement).value,
    ).toBe("todo");
    expect(
      (within(dialog).getByRole("combobox", { name: "Priority" }) as HTMLSelectElement).value,
    ).toBe("p2");
    await user.type(within(dialog).getByRole("textbox", { name: "Task title" }), "   ");
    await user.click(within(dialog).getByRole("button", { name: "Create task" }));
    expect((await within(dialog).findByRole("alert")).textContent).toBe("Enter a task title.");
    expect(invokeMock).not.toHaveBeenCalledWith("create_task", expect.anything());

    await user.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledOnce();
    expect(invokeMock).not.toHaveBeenCalledWith("create_task", expect.anything());
  });

  it("persists a task with an optional project and confirms the saved record", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(
      async (command: string, payload?: { request?: Record<string, unknown> }) => {
        if (command === "list_projects") return { projects: [project] };
        if (command === "create_task") return task(payload?.request ?? {});
        if (command === undefined) return { projects: [project] };
        throw new Error(`Unexpected command: ${command}`);
      },
    );
    render(<TaskCreateDialog open onClose={() => undefined} />);
    const dialog = await screen.findByRole("dialog", { name: "Create task" });
    await user.type(
      within(dialog).getByRole("textbox", { name: "Task title" }),
      "Write first workflow",
    );
    await user.selectOptions(
      within(dialog).getByRole("combobox", { name: "Project" }),
      "project-1",
    );
    await user.click(within(dialog).getByRole("button", { name: "Create task" }));

    expect(await screen.findByRole("dialog", { name: "Task created" })).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("create_task", {
      request: {
        title: "Write first workflow",
        projectId: "project-1",
        status: "todo",
        priority: "p2",
      },
    });
    expect(screen.getByText("Project: Moonships · Todo · P2 · Normal")).toBeTruthy();
  });

  it("saves to the Personal inbox when no project is selected", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(
      async (command: string, payload?: { request?: Record<string, unknown> }) => {
        if (command === "list_projects") return { projects: [] };
        if (command === "create_task") return task(payload?.request ?? {});
        if (command === undefined) return { projects: [] };
        throw new Error(`Unexpected command: ${command}`);
      },
    );
    render(<TaskCreateDialog open onClose={() => undefined} />);
    const dialog = await screen.findByRole("dialog", { name: "Create task" });
    await user.type(within(dialog).getByRole("textbox", { name: "Task title" }), "Unassigned work");
    await user.click(within(dialog).getByRole("button", { name: "Create task" }));

    expect(await screen.findByText("Personal inbox · Todo · P2 · Normal")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("create_task", {
      request: { title: "Unassigned work", projectId: null, status: "todo", priority: "p2" },
    });
  });

  it("allows inbox creation and explains when the optional project list is unavailable", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(
      async (command: string, payload?: { request?: Record<string, unknown> }) => {
        if (command === "list_projects") throw new Error("database unavailable");
        if (command === "create_task") return task(payload?.request ?? {});
        if (command === undefined) return { projects: [] };
        throw new Error(`Unexpected command: ${command}`);
      },
    );
    render(<TaskCreateDialog open onClose={() => undefined} />);
    const dialog = await screen.findByRole("dialog", { name: "Create task" });
    expect(
      await within(dialog).findByText(/You can still save to the Personal inbox/),
    ).toBeTruthy();
    await user.type(within(dialog).getByRole("textbox", { name: "Task title" }), "Offline task");
    await user.click(within(dialog).getByRole("button", { name: "Create task" }));
    expect(await screen.findByText("Personal inbox · Todo · P2 · Normal")).toBeTruthy();
  });

  it("preselects the Project when launched from its overview", async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "list_projects") return { projects: [project] };
      if (command === undefined) return { projects: [project] };
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<TaskCreateDialog open initialProjectId="project-1" onClose={() => undefined} />);
    const dialog = await screen.findByRole("dialog", { name: "Create task" });
    expect(
      (within(dialog).getByRole("combobox", { name: "Project" }) as HTMLSelectElement).value,
    ).toBe("project-1");
  });
});
