// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TaskListPage } from "./TaskListPage";
import type { TaskRecord } from "./tasks-api";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const workspaceId = "00000000-0000-4000-8000-000000000001";
const project = {
  id: "project-1",
  workspaceId,
  name: "Moonships",
  projectKey: null,
  description: null,
  goal: null,
  status: "active" as const,
  archivedAt: null,
  createdAt: "2026-10-26T00:00:00Z",
  updatedAt: "2026-10-26T00:00:00Z",
};

function makeTask(overrides: Partial<TaskRecord> & Pick<TaskRecord, "id" | "title">): TaskRecord {
  const { id, title, ...rest } = overrides;
  return {
    id,
    workspaceId,
    projectId: null,
    title,
    description: null,
    dueAt: null,
    status: "todo",
    priority: "p2",
    createdAt: "2026-10-26T00:00:00Z",
    updatedAt: "2026-10-26T00:00:00Z",
    archivedAt: null,
    ...rest,
  };
}

const sampleTasks = [
  makeTask({ id: "task-z", title: "Zeta task", priority: "p2", projectId: "project-1" }),
  makeTask({ id: "task-a", title: "Alpha task", priority: "p0", status: "blocked" }),
  makeTask({ id: "task-b", title: "alpha task", priority: "p0" }),
];

function mockResponses(tasks: TaskRecord[] = sampleTasks) {
  invokeMock.mockImplementation(async (command: string) => {
    if (command === "list_tasks") return { tasks };
    if (command === "list_projects") return { projects: [project] };
    if (command === undefined) return { tasks };
    throw new Error(`Unexpected command: ${command}`);
  });
}

describe("M2-018 persisted Task List", () => {
  it("renders persisted active tasks and sorts a copy deterministically", async () => {
    const user = userEvent.setup();
    const onOpenTask = vi.fn();
    mockResponses();
    const originalOrder = sampleTasks.map((task) => task.id);
    render(<TaskListPage onCreateTask={() => undefined} onOpenTask={onOpenTask} />);

    const table = await screen.findByRole("table", {
      name: "Active tasks in the Personal workspace",
    });
    expect(invokeMock).toHaveBeenCalledWith("list_tasks", { request: {} });
    expect(
      within(table)
        .getAllByRole("rowheader")
        .map((row) => row.textContent),
    ).toEqual(["Alpha task", "alpha task", "Zeta task"]);
    expect(within(table).getByText("Moonships")).toBeTruthy();
    expect(within(table).getAllByText("Personal inbox")).toHaveLength(2);
    await user.click(within(table).getByRole("button", { name: "Alpha task" }));
    expect(onOpenTask).toHaveBeenCalledWith("task-a");
    expect(sampleTasks.map((task) => task.id)).toEqual(originalOrder);

    await user.click(within(table).getByRole("button", { name: "Sort by priority ascending" }));
    expect(
      within(table)
        .getByRole("columnheader", { name: /Priority/ })
        .getAttribute("aria-sort"),
    ).toBe("ascending");
    expect(
      within(table)
        .getAllByRole("rowheader")
        .map((row) => row.textContent),
    ).toEqual(["Alpha task", "alpha task", "Zeta task"]);
    await user.click(within(table).getByRole("button", { name: "Sort by priority descending" }));
    expect(
      within(table)
        .getAllByRole("rowheader")
        .map((row) => row.textContent),
    ).toEqual(["Zeta task", "Alpha task", "alpha task"]);
    expect(sampleTasks.map((task) => task.id)).toEqual(originalOrder);
  });

  it("groups tasks by canonical status and saves a Board status move through Task IPC", async () => {
    const user = userEvent.setup();
    let tasks = [...sampleTasks];
    invokeMock.mockImplementation(
      async (
        command: string,
        args?: { request?: { id: string; status: TaskRecord["status"] } },
      ) => {
        if (command === "list_tasks") return { tasks };
        if (command === "list_projects") return { projects: [project] };
        if (command === undefined) return { tasks };
        if (command === "update_task") {
          const request = args?.request;
          const saved = {
            ...tasks.find((task) => task.id === request?.id)!,
            status: request!.status,
          };
          tasks = tasks.map((task) => (task.id === saved.id ? saved : task));
          return saved;
        }
        throw new Error(`Unexpected command: ${command}`);
      },
    );
    render(<TaskListPage onCreateTask={() => undefined} onOpenTask={() => undefined} />);

    await user.click(await screen.findByRole("button", { name: "Board" }));
    const board = screen.getByRole("region", { name: "Tasks grouped by status" });
    expect(within(board).getByRole("region", { name: "Blocked" })).toBeTruthy();
    expect(within(board).getByRole("region", { name: "Todo" })).toBeTruthy();
    expect(
      within(screen.getByRole("region", { name: "Blocked" })).getByRole("button", {
        name: "Alpha task",
      }),
    ).toBeTruthy();

    await user.selectOptions(
      screen.getByRole("combobox", { name: "Move Zeta task to status" }),
      "done",
    );
    expect(invokeMock).toHaveBeenCalledWith("update_task", {
      request: {
        id: "task-z",
        projectId: "project-1",
        title: "Zeta task",
        description: null,
        dueAt: null,
        status: "done",
        priority: "p2",
      },
    });
    expect(await screen.findByText("Zeta task moved to Done and saved.")).toBeTruthy();
    expect(
      within(screen.getByRole("region", { name: "Done" })).getByRole("button", {
        name: "Zeta task",
      }),
    ).toBeTruthy();
    expect(tasks.find((task) => task.id === "task-z")?.status).toBe("done");
  });

  it("keeps a task in its saved column and reports an update failure", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "list_tasks") return { tasks: sampleTasks };
      if (command === "list_projects") return { projects: [project] };
      if (command === undefined) return { tasks: sampleTasks };
      if (command === "update_task") throw new Error("database unavailable");
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<TaskListPage onCreateTask={() => undefined} onOpenTask={() => undefined} />);

    await user.click(await screen.findByRole("button", { name: "Board" }));
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Move Zeta task to status" }),
      "done",
    );
    expect((await screen.findByRole("status")).textContent).toContain(
      "The Board stays on Todo until a saved update is confirmed.",
    );
    expect(
      (screen.getByRole("combobox", { name: "Move Zeta task to status" }) as HTMLSelectElement)
        .value,
    ).toBe("todo");
    expect(
      within(screen.getByRole("region", { name: "Todo" })).getByRole("button", {
        name: "Zeta task",
      }),
    ).toBeTruthy();
  });

  it("persists a drag-and-drop move through the same Task update command", async () => {
    let tasks = [...sampleTasks];
    invokeMock.mockImplementation(
      async (
        command: string,
        args?: { request?: { id: string; status: TaskRecord["status"] } },
      ) => {
        if (command === "list_tasks" || command === undefined) return { tasks };
        if (command === "list_projects") return { projects: [project] };
        if (command === "update_task") {
          const request = args?.request;
          const saved = {
            ...tasks.find((task) => task.id === request?.id)!,
            status: request!.status,
          };
          tasks = tasks.map((task) => (task.id === saved.id ? saved : task));
          return saved;
        }
        throw new Error(`Unexpected command: ${command}`);
      },
    );
    render(<TaskListPage onCreateTask={() => undefined} onOpenTask={() => undefined} />);

    await userEvent.setup().click(await screen.findByRole("button", { name: "Board" }));
    const dataTransfer = { getData: () => "task-z" };
    fireEvent.drop(screen.getByRole("region", { name: "Done" }), { dataTransfer });

    expect(await screen.findByText("Zeta task moved to Done and saved.")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith(
      "update_task",
      expect.objectContaining({
        request: expect.objectContaining({ id: "task-z", status: "done" }),
      }),
    );
    expect(tasks.find((task) => task.id === "task-z")?.status).toBe("done");
  });

  it("searches titles and descriptions case-insensitively and clears distinct no-results state", async () => {
    const user = userEvent.setup();
    mockResponses([
      ...sampleTasks,
      makeTask({
        id: "task-description",
        title: "Migration notes",
        description: "Prepare the MOONSHIPS handoff",
      }),
    ]);
    render(<TaskListPage onCreateTask={() => undefined} onOpenTask={() => undefined} />);

    const search = await screen.findByRole("searchbox", { name: "Search tasks" });
    await user.type(search, "zEtA");
    await screen.findByText("1 of 4 active tasks");
    expect(screen.getAllByRole("rowheader").map((row) => row.textContent?.trim())).toEqual([
      "Zeta task",
    ]);
    expect(screen.queryByText("Alpha task")).toBeNull();
    expect(screen.getByText("1 of 4 active tasks")).toBeTruthy();

    await user.clear(search);
    await user.type(search, "moonships");
    await screen.findByText("Prepare the MOONSHIPS handoff");
    expect(screen.getAllByRole("rowheader").map((row) => row.textContent?.trim())).toEqual([
      "Migration notesPrepare the MOONSHIPS handoff",
    ]);
    expect(screen.getByText("Prepare the MOONSHIPS handoff")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Board" }));
    expect(
      within(screen.getByRole("region", { name: "Todo" })).getByRole("button", {
        name: "Migration notes",
      }),
    ).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Alpha task" })).toBeNull();

    await user.clear(search);
    await user.type(search, "not present");
    expect(
      await screen.findByRole("heading", { name: "No tasks match “not present”" }),
    ).toBeTruthy();
    expect(screen.getByText("0 of 4 active tasks")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Clear search" }));
    await screen.findByText("4 active tasks");
    await user.click(screen.getByRole("button", { name: "List" }));
    expect(screen.getAllByRole("rowheader").map((row) => row.textContent?.trim())).toEqual([
      "Alpha task",
      "alpha task",
      "Migration notesPrepare the MOONSHIPS handoff",
      "Zeta task",
    ]);
    expect(screen.queryByText("No tasks match “not present”")).toBeNull();
  });

  it("distinguishes an empty persisted list from an unavailable list and retries", async () => {
    const user = userEvent.setup();
    let taskAttempt = 0;
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "list_tasks") {
        taskAttempt += 1;
        if (taskAttempt === 1) throw new Error("database unavailable");
        return { tasks: [] };
      }
      if (command === "list_projects") return { projects: [project] };
      if (command === undefined) return taskAttempt === 1 ? { projects: [project] } : { tasks: [] };
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<TaskListPage onCreateTask={() => undefined} onOpenTask={() => undefined} />);

    expect(await screen.findByText("Tasks are unavailable")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText("No active tasks yet")).toBeTruthy();
    await user.type(screen.getByRole("searchbox", { name: "Search tasks" }), "missing");
    expect(await screen.findByRole("heading", { name: "No tasks match “missing”" })).toBeTruthy();
    expect(screen.queryByText("Tasks are unavailable")).toBeNull();
  });

  it("keeps persisted tasks visible when optional project names cannot be read", async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "list_tasks") return { tasks: sampleTasks.slice(0, 1) };
      if (command === "list_projects") throw new Error("projects unavailable");
      if (command === undefined) return { tasks: sampleTasks.slice(0, 1) };
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<TaskListPage onCreateTask={() => undefined} onOpenTask={() => undefined} />);

    expect(await screen.findByText("Zeta task")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toContain(
      "Project names are unavailable. Tasks remain visible",
    );
    expect(screen.getByText("Archived or unavailable project")).toBeTruthy();
  });
});
