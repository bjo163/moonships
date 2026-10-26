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
    invokeMock.mockResolvedValue(overview);
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
    invokeMock.mockResolvedValue({ ...overview, recentActivity: [] });
    render(<ProjectOverviewPage projectId="project-1" onCreateTask={() => undefined} />);
    expect(await screen.findByText("No activity recorded")).toBeTruthy();
    expect(screen.getByText(/Changes to this project and its tasks/)).toBeTruthy();
  });

  it("shows an unavailable state and retries the persisted overview", async () => {
    const user = userEvent.setup();
    invokeMock
      .mockRejectedValueOnce(new Error("database unavailable"))
      .mockResolvedValueOnce(overview);
    render(<ProjectOverviewPage projectId="project-1" onCreateTask={() => undefined} />);

    const error = await screen.findByText("Project overview is unavailable");
    expect(error).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    const heading = await screen.findByRole("heading", { name: "Moonships" });
    expect(heading).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledTimes(2);
  });
});
