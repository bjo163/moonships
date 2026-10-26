// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
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
    mockResponses();
    const originalOrder = sampleTasks.map((task) => task.id);
    render(<TaskListPage onCreateTask={() => undefined} />);

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
    render(<TaskListPage onCreateTask={() => undefined} />);

    expect(await screen.findByText("Tasks are unavailable")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText("No active tasks yet")).toBeTruthy();
  });

  it("keeps persisted tasks visible when optional project names cannot be read", async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "list_tasks") return { tasks: sampleTasks.slice(0, 1) };
      if (command === "list_projects") throw new Error("projects unavailable");
      if (command === undefined) return { tasks: sampleTasks.slice(0, 1) };
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<TaskListPage onCreateTask={() => undefined} />);

    expect(await screen.findByText("Zeta task")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toContain(
      "Project names are unavailable. Tasks remain visible",
    );
    expect(screen.getByText("Archived or unavailable project")).toBeTruthy();
  });
});
