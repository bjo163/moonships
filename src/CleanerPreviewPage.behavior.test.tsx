// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { CleanerPreviewPage } from "./CleanerPreviewPage";
import type { CleanupPreviewDto } from "./cleanup-preview";

afterEach(cleanup);

const preview: CleanupPreviewDto = {
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
    expect(screen.getByText(/this preview has no clean action/i)).toBeTruthy();
  });
});
