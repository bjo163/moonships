// @vitest-environment jsdom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TaskDetailPage } from "./TaskDetailPage";
import type { TaskRecord } from "./tasks-api";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const workspaceId = "00000000-0000-4000-8000-000000000001";
const savedTask: TaskRecord = {
  id: "task-1",
  workspaceId,
  projectId: null,
  title: "Write release notes",
  description: "Summarize the changes",
  dueAt: "2026-10-27T12:30:00Z",
  status: "in_progress",
  priority: "p1",
  createdAt: "2026-10-26T00:00:00Z",
  updatedAt: "2026-10-26T00:00:00Z",
  archivedAt: null,
};

describe("M2-017 persisted Task detail", () => {
  it("loads, edits and confirms the persisted task fields", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "get_task") return savedTask;
      if (command === "list_projects") return { projects: [] };
      if (command === "update_task")
        return {
          ...savedTask,
          title: "Publish release notes",
          description: null,
          dueAt: null,
          status: "done",
          priority: "p2",
        };
      if (command === undefined) return undefined;
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<TaskDetailPage taskId="task-1" />);

    const title = await screen.findByRole("textbox", { name: "Title" });
    expect(title).toHaveProperty("value", "Write release notes");
    expect(screen.getByRole("textbox", { name: "Description" })).toHaveProperty(
      "value",
      "Summarize the changes",
    );
    expect(screen.getByLabelText("Status")).toHaveProperty("value", "in_progress");
    expect(screen.getByLabelText("Priority")).toHaveProperty("value", "p1");

    await user.clear(title);
    await user.type(title, "Publish release notes");
    await user.clear(screen.getByRole("textbox", { name: "Description" }));
    await user.selectOptions(screen.getByLabelText("Status"), "done");
    await user.selectOptions(screen.getByLabelText("Priority"), "p2");
    await user.clear(screen.getByLabelText("Due date and time"));
    await user.click(screen.getByRole("button", { name: "Save changes" }));

    await screen.findByText("Task saved on this device.");
    expect(invokeMock).toHaveBeenCalledWith("update_task", {
      request: {
        id: "task-1",
        projectId: null,
        title: "Publish release notes",
        description: null,
        dueAt: null,
        status: "done",
        priority: "p2",
      },
    });
  });

  it("keeps task details usable when the supplemental Project list is unavailable", async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "get_task") return { ...savedTask, projectId: "project-1" };
      if (command === "list_projects") throw new Error("offline");
      if (command === undefined) return undefined;
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<TaskDetailPage taskId="task-1" />);
    expect(await screen.findByRole("textbox", { name: "Title" })).toBeTruthy();
    expect(screen.getByText(/Project name unavailable/)).toBeTruthy();
    expect(screen.getByText(/Project names could not be loaded/)).toBeTruthy();
  });

  it("shows retryable read failure instead of an empty detail form", async () => {
    const user = userEvent.setup();
    invokeMock.mockImplementation(async (command: string) => {
      if (command === "get_task") throw "NOT_FOUND: task is archived";
      if (command === "list_projects") return { projects: [] };
      if (command === undefined) return undefined;
      throw new Error(`Unexpected command: ${command}`);
    });
    render(<TaskDetailPage taskId="task-1" />);
    expect(await screen.findByText(/unavailable or has been archived/)).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("get_task", { request: { id: "task-1" } }),
    );
  });
});
