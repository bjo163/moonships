// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { HomeDashboardPage } from "./HomeDashboardPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const EMPTY_DASHBOARD = {
  asOf: "2026-06-20T12:00:00.000Z",
  activeProjectCount: 0,
  activeTaskCount: 0,
  counts: { openTasks: 0, overdueTasks: 0, dueSoonTasks: 0, blockedTasks: 0 },
  recentProjects: [],
  overdueTasks: [],
  upcomingTasks: [],
  recentActivity: [],
};

const props = {
  onCreateTask: vi.fn(),
  onCreateProject: vi.fn(),
  onOpenProjects: vi.fn(),
  onOpenTasks: vi.fn(),
  onOpenProject: vi.fn(),
  onOpenTask: vi.fn(),
};

describe("M2-033 persisted Home dashboard", () => {
  it("shows saved counts, recent records, activity and navigates to their details", async () => {
    const user = userEvent.setup();
    invokeMock.mockResolvedValue({
      ...EMPTY_DASHBOARD,
      activeProjectCount: 1,
      activeTaskCount: 3,
      counts: { openTasks: 3, overdueTasks: 1, dueSoonTasks: 1, blockedTasks: 1 },
      recentProjects: [
        {
          id: "project-1",
          name: "Local project",
          status: "active",
          updatedAt: "2026-06-20T11:00:00Z",
        },
      ],
      overdueTasks: [
        {
          id: "task-1",
          projectId: "project-1",
          projectName: "Local project",
          title: "Late work",
          status: "todo",
          priority: "p1",
          dueAt: "2026-06-19T11:00:00Z",
        },
      ],
      upcomingTasks: [
        {
          id: "task-2",
          projectId: null,
          projectName: null,
          title: "Inbox task",
          status: "todo",
          priority: "p2",
          dueAt: "2026-06-25T11:00:00Z",
        },
      ],
      recentActivity: [
        {
          id: "event-1",
          entityType: "task",
          entityId: "task-1",
          action: "created",
          summary: "Task created",
          occurredAt: "2026-06-20T11:30:00Z",
        },
      ],
    });
    render(<HomeDashboardPage {...props} />);

    expect(await screen.findByText("Late work")).toBeTruthy();
    const counts = screen.getByRole("region", { name: "My Work" });
    expect(within(counts).getByText("3", { selector: "strong" })).toBeTruthy();
    expect(within(counts).getAllByText("1", { selector: "strong" })).toHaveLength(3);
    expect(screen.getByText("Task created")).toBeTruthy();
    expect(screen.getByText(/Waiting approvals are not available yet/)).toBeTruthy();

    const projectPanel = screen.getByRole("region", { name: "Projects" });
    await user.click(within(projectPanel).getByRole("button", { name: /Local project/ }));
    expect(props.onOpenProject).toHaveBeenCalledWith("project-1");
    const overduePanel = screen.getByRole("region", { name: "Overdue tasks" });
    await user.click(within(overduePanel).getByRole("button", { name: /Late work/ }));
    expect(props.onOpenTask).toHaveBeenCalledWith("task-1");
    expect(invokeMock).toHaveBeenCalledWith("get_home_dashboard");
  });

  it("shows a truthful first-run empty state and a retryable read failure", async () => {
    const user = userEvent.setup();
    invokeMock.mockResolvedValueOnce(EMPTY_DASHBOARD);
    render(<HomeDashboardPage {...props} />);
    expect(await screen.findByText("Your workspace is ready")).toBeTruthy();
    expect(screen.getByText("No activity has been recorded yet.")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Tasks" }));
    await user.click(screen.getByRole("button", { name: "Create a project" }));
    expect(props.onCreateProject).toHaveBeenCalledOnce();

    cleanup();
    invokeMock.mockReset();
    invokeMock.mockRejectedValueOnce("IO_ERROR: database unavailable");
    render(<HomeDashboardPage {...props} />);
    expect(await screen.findByRole("alert")).toBeTruthy();
    invokeMock.mockResolvedValueOnce(EMPTY_DASHBOARD);
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText("Your workspace is ready")).toBeTruthy();
  });
});
