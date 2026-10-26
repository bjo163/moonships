// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ProjectTaskExportButton } from "./ProjectTaskExportButton";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M2-029 Project and Task export action", () => {
  it("reports saved counts only after the native save completes", async () => {
    const user = userEvent.setup();
    invokeMock.mockResolvedValue({
      cancelled: false,
      fileName: "moonships-projects-tasks.json",
      projectCount: 2,
      taskCount: 5,
      activityEventCount: 9,
    });
    render(<ProjectTaskExportButton />);

    await user.click(screen.getByRole("button", { name: "Export Projects & Tasks" }));

    expect((await screen.findByRole("status")).textContent).toBe(
      "Saved 2 projects and 5 tasks to moonships-projects-tasks.json.",
    );
    expect(invokeMock).toHaveBeenCalledWith("export_project_task_data");
  });

  it("distinguishes cancellation and failure from a successful export", async () => {
    const user = userEvent.setup();
    invokeMock.mockResolvedValueOnce({
      cancelled: true,
      fileName: null,
      projectCount: 0,
      taskCount: 0,
      activityEventCount: 0,
    });
    render(<ProjectTaskExportButton />);
    await user.click(screen.getByRole("button", { name: "Export Projects & Tasks" }));
    expect((await screen.findByRole("status")).textContent).toBe(
      "Export cancelled. No file was saved.",
    );

    invokeMock.mockRejectedValueOnce("IO_ERROR: could not save");
    await user.click(screen.getByRole("button", { name: "Export Projects & Tasks" }));
    expect((await screen.findByRole("alert")).textContent).toBe(
      "Export failed. No success was reported; try again.",
    );
  });
});
