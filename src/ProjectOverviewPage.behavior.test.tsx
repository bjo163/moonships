// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ProjectOverviewPage } from "./ProjectOverviewPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const project = {
  id: "project-1",
  workspaceId: "00000000-0000-4000-8000-000000000001",
  name: "Moonships",
  projectKey: null,
  description: "A local project workspace.",
  goal: "Ship a useful local workflow.",
  status: "active" as const,
  archivedAt: null,
  createdAt: "2026-10-26T00:00:00Z",
  updatedAt: "2026-10-26T01:00:00Z",
};

const overview = {
  project,
  activeTaskCount: 0,
  recentActivity: [
    {
      id: "event-project",
      entityType: "project" as const,
      action: "created",
      summary: "Project created",
      occurredAt: "2026-10-26T00:00:00Z",
    },
  ],
};

describe("M2-015 persisted Project overview", () => {
  it("shows the saved goal and status, an accurate zero-task state, and real activity", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "get_project_overview") return overview;
      if (command === "list_milestones") return { milestones: [] };
      if (command === "list_tasks") return { tasks: [] };
      if (command === undefined) return undefined;
      throw new Error(`Unexpected command: ${command}`);
    });
    const onCreateTask = vi.fn();
    render(<ProjectOverviewPage projectId="project-1" onCreateTask={onCreateTask} />);

    expect(await screen.findByRole("heading", { name: "Moonships" })).toBeTruthy();
    expect(screen.getByText("Ship a useful local workflow.")).toBeTruthy();
    expect(screen.getByText("A local project workspace.")).toBeTruthy();
    expect(screen.getByText("Active")).toBeTruthy();
    expect(screen.getByText("No non-archived tasks are linked to this project yet.")).toBeTruthy();
    expect(screen.getByText("Project created")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("get_project_overview", {
      request: { id: "project-1" },
    });

    await user.click(screen.getByRole("button", { name: "Create task for this project" }));
    expect(onCreateTask).toHaveBeenCalledWith("project-1");
  });

  it("distinguishes no recorded activity from a storage failure", async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "get_project_overview") return { ...overview, recentActivity: [] };
      if (command === "list_milestones") return { milestones: [] };
      if (command === "list_tasks") return { tasks: [] };
      if (command === undefined) return undefined;
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<ProjectOverviewPage projectId="project-1" onCreateTask={() => undefined} />);
    expect(await screen.findByText("No activity recorded")).toBeTruthy();
    expect(screen.getByText(/Changes to this project and its tasks/)).toBeTruthy();
  });

  it("shows an unavailable state and retries the persisted overview", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "get_project_overview") {
        if (invokeMock.mock.calls.filter(([name]) => name === command).length === 1) {
          throw new Error("database unavailable");
        }
        return overview;
      }
      if (command === "list_milestones") return { milestones: [] };
      if (command === "list_tasks") return { tasks: [] };
      if (command === undefined) return undefined;
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<ProjectOverviewPage projectId="project-1" onCreateTask={() => undefined} />);

    const error = await screen.findByText("Project overview is unavailable");
    expect(error).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    const heading = await screen.findByRole("heading", { name: "Moonships" });
    expect(heading).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledTimes(6);
  });

  it("creates a milestone and associates a task without changing its status", async () => {
    const user = userEvent.setup();
    const milestone = {
      id: "milestone-1",
      workspaceId: project.workspaceId,
      projectId: project.id,
      title: "Release 1",
      description: null,
      dueDate: "2026-12-31",
      status: "planned",
      sortOrder: 0,
      createdAt: project.createdAt,
      updatedAt: project.updatedAt,
      tasks: [],
    };
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "get_project_overview") return overview;
      if (command === "list_milestones") return { milestones: [] };
      if (command === "list_tasks")
        return { tasks: [{ id: "task-1", title: "Write docs", projectId: project.id }] };
      if (command === "create_milestone") return milestone;
      if (command === "associate_milestone_task")
        return {
          ...milestone,
          tasks: [{ id: "task-1", title: "Write docs", status: "in_progress" }],
        };
      if (command === "update_milestone")
        return {
          ...milestone,
          status: "completed",
          tasks: [{ id: "task-1", title: "Write docs", status: "in_progress" }],
        };
      if (command === undefined) return undefined;
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<ProjectOverviewPage projectId="project-1" onCreateTask={() => undefined} />);
    await user.type(await screen.findByRole("textbox", { name: "Milestone title" }), "Release 1");
    await user.type(screen.getByLabelText("Due date"), "2026-12-31");
    await user.click(screen.getByRole("button", { name: "Add milestone" }));
    await user.selectOptions(await screen.findByLabelText("Associate project task"), "task-1");
    expect(await screen.findByText("Write docs · in progress")).toBeTruthy();
    expect(
      screen.getByText("Task associated with milestone. Task status was not changed."),
    ).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("associate_milestone_task", {
      request: { projectId: "project-1", milestoneId: "milestone-1", taskId: "task-1" },
    });
    await user.selectOptions(screen.getByLabelText("Milestone status"), "completed");
    expect(
      await screen.findByText("Milestone status saved. Associated task statuses were not changed."),
    ).toBeTruthy();
    expect(screen.getByText("Write docs · in progress")).toBeTruthy();
  });
});
