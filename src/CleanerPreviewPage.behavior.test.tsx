// @vitest-environment jsdom
import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CleanerPreviewPage } from "./CleanerPreviewPage";
import type { CleanupReportSummary } from "./cleanup-reports-api";
import type { CleanupPreviewDto } from "./cleanup-preview";

const { listCleanupReportsMock } = vi.hoisted(() => ({ listCleanupReportsMock: vi.fn() }));

vi.mock("./cleanup-reports-api", () => ({
  listCleanupReports: listCleanupReportsMock,
  toCleanupReportErrorMessage: () =>
    "Saved cleanup results could not be loaded from this device. Try again.",
}));

afterEach(cleanup);
beforeEach(() => listCleanupReportsMock.mockResolvedValue([]));

const preview: CleanupPreviewDto = {
  previewId: "preview-1",
  generatedAt: "2026-10-26T07:00:00Z",
  items: [
    {
      candidateId: "candidate-1",
      category: "temporary_files",
      displayName: "cache-fragment.bin",
      sizeBytes: 2048,
      modifiedAt: "2026-10-25T07:00:00Z",
      disposition: "eligible",
      skipReason: null,
    },
    {
      candidateId: "candidate-2",
      category: "application_cache",
      displayName: "recent-index.db",
      sizeBytes: 512,
      modifiedAt: "2026-10-26T06:50:00Z",
      disposition: "skipped",
      skipReason: "too_recent",
    },
  ],
};

describe("M3-007 cleanup preview", () => {
  it("shows a truthful unavailable state when there is no scan preview", () => {
    render(<CleanerPreviewPage preview={null} />);

    expect(screen.getByRole("heading", { name: "Clean & Storage" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "No scan preview available" })).toBeTruthy();
    expect(screen.getByText(/scan workflow is not available yet/i)).toBeTruthy();
  });

  it("shows counts, estimated size and skip reasons without exposing a clean action", () => {
    render(<CleanerPreviewPage preview={preview} />);

    const summary = screen.getByRole("region", { name: "Preview summary" });
    expect(within(summary).getAllByText("1")).toHaveLength(2);
    expect(within(summary).getByText("2 KB")).toBeTruthy();
    expect(screen.getByText("cache-fragment.bin")).toBeTruthy();
    expect(screen.getByText("Temporary files")).toBeTruthy();
    expect(screen.getByText("Skipped: File is too recent")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /clean/i })).toBeNull();
    expect(screen.getByText(/approval alone does not change files/i)).toBeTruthy();
  });

  it("requires a separate confirmation before recording approval and never executes cleanup", async () => {
    const user = userEvent.setup();
    const onApprove = vi.fn();
    render(<CleanerPreviewPage preview={preview} onApprove={onApprove} />);

    const skipped = screen.getByRole("checkbox", { name: "Select recent-index.db" });
    expect((skipped as HTMLInputElement).disabled).toBe(true);
    const eligible = screen.getByRole("checkbox", { name: "Select cache-fragment.bin" });
    const review = screen.getByRole("button", { name: "Review selected actions" });
    expect((review as HTMLButtonElement).disabled).toBe(true);

    await user.click(eligible);
    expect((review as HTMLButtonElement).disabled).toBe(false);
    expect(onApprove).not.toHaveBeenCalled();
    await user.click(review);
    expect(screen.getByRole("dialog", { name: "Approve selected cleanup items" })).toBeTruthy();
    expect(screen.getByText(/cleanup is not wired to this screen yet/i)).toBeTruthy();
    expect(onApprove).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onApprove).not.toHaveBeenCalled();
    await user.click(review);
    await user.click(screen.getByRole("button", { name: "Approve selected items" }));

    expect(onApprove).toHaveBeenCalledWith({
      previewId: "preview-1",
      candidateIds: ["candidate-1"],
      estimatedBytes: 2048,
    });
    expect(screen.getByRole("status").textContent).toMatch(/cleanup has not started/i);
    expect(screen.queryByRole("button", { name: /^clean$/i })).toBeNull();
  });
});

describe("M3-014 cleanup report history", () => {
  const savedReport: CleanupReportSummary = {
    id: "report-1",
    previewId: "preview-1",
    taskId: "00000000-0000-4000-8000-000000000099",
    status: "partial",
    completedAt: "2026-10-26T07:00:00Z",
    selectedCount: 3,
    deletedCount: 1,
    skippedCount: 1,
    failedCount: 1,
    estimatedBytes: 20_480,
    actualDeletedBytes: 7_168,
  };

  it("shows persisted estimates, actual deleted file size and an explicit task link", async () => {
    listCleanupReportsMock.mockResolvedValueOnce([savedReport]);
    render(<CleanerPreviewPage preview={null} />);

    const row = await screen.findByRole("row", { name: /partial/i });
    expect(within(row).getByText("20 KB")).toBeTruthy();
    expect(within(row).getByText("7 KB")).toBeTruthy();
    expect(within(row).getByText(/1 deleted · 1 skipped · 1 failed/i)).toBeTruthy();
    expect(within(row).getByRole("link", { name: "Open linked task" }).getAttribute("href")).toBe(
      "#/tasks/00000000-0000-4000-8000-000000000099",
    );
    expect(screen.queryByText(/candidate path|relative path/i)).toBeNull();
  });

  it("offers a retry when local report history cannot be read", async () => {
    const user = userEvent.setup();
    listCleanupReportsMock.mockRejectedValueOnce(new Error("db busy"));
    render(<CleanerPreviewPage preview={null} />);

    expect(
      await screen.findByRole("heading", { name: "Cleanup history unavailable" }),
    ).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByRole("heading", { name: "No saved cleanup results" })).toBeTruthy();
    await waitFor(() => expect(listCleanupReportsMock).toHaveBeenCalledTimes(2));
  });
});
