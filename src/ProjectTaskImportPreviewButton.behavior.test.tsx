// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ProjectTaskImportPreviewButton } from "./ProjectTaskImportPreviewButton";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M2-030 Project and Task import preview", () => {
  it("shows a validated summary and confirms that no records were imported", async () => {
    const user = userEvent.setup();
    invokeMock.mockResolvedValue({
      cancelled: false,
      fileName: "backup.json",
      schemaVersion: 1,
      appVersion: "0.1.0-2",
      exportedAt: "2026-10-26T00:00:00Z",
      projectCount: 2,
      taskCount: 3,
      activityEventCount: 4,
      projectNames: ["Research"],
      taskTitles: ["Review notes"],
    });
    render(<ProjectTaskImportPreviewButton />);

    await user.click(screen.getByRole("button", { name: "Preview import" }));

    expect((await screen.findByRole("status")).textContent).toContain(
      "Valid export preview: 2 projects, 3 tasks, and 4 activity events.",
    );
    expect(screen.getByRole("status").textContent).toContain("No records were imported.");
    expect(invokeMock).toHaveBeenCalledWith("preview_project_task_import");
  });

  it("reports cancellation and malformed or oversized-file failures without claiming an import", async () => {
    const user = userEvent.setup();
    invokeMock.mockResolvedValueOnce({ cancelled: true });
    render(<ProjectTaskImportPreviewButton />);
    await user.click(screen.getByRole("button", { name: "Preview import" }));
    expect((await screen.findByRole("status")).textContent).toBe(
      "Preview cancelled. No records were imported.",
    );

    invokeMock.mockRejectedValueOnce("VALIDATION_FAILED: malformed");
    await user.click(screen.getByRole("button", { name: "Preview import" }));
    expect((await screen.findByRole("alert")).textContent).toContain(
      "malformed or too large. No records were imported.",
    );
  });
});
