// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CleanerPreviewPage } from "./CleanerPreviewPage";
import type { CleanupPreviewDto } from "./cleanup-preview";

afterEach(cleanup);

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
    expect(screen.getByText(/cleanup executor is not available yet/i)).toBeTruthy();
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
