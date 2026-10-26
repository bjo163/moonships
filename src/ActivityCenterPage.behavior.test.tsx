// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ActivityCenterPage } from "./ActivityCenterPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M4-012 Activity Center", () => {
  it("shows saved state, progress, duration and result", async () => {
    invokeMock.mockResolvedValue([
      {
        id: "job-1",
        title: "Index workspace",
        status: "succeeded",
        createdAt: "2026-10-26T09:00:00Z",
        startedAt: "2026-10-26T09:00:05Z",
        finishedAt: "2026-10-26T09:01:05Z",
        stage: "Complete",
        progressPercent: 100,
        resultSummary: "12 files indexed",
      },
    ]);
    render(<ActivityCenterPage />);
    const row = await screen.findByRole("row", { name: /Index workspace/ });
    expect(within(row).getByText("Succeeded")).toBeTruthy();
    expect(within(row).getByText("100%")).toBeTruthy();
    expect(within(row).getByText("1m 0s")).toBeTruthy();
    expect(within(row).getByText("12 files indexed")).toBeTruthy();
  });

  it("shows a truthful empty state and a retryable storage error", async () => {
    invokeMock.mockResolvedValueOnce([]).mockRejectedValueOnce(new Error("unavailable"));
    const view = render(<ActivityCenterPage />);
    expect(await screen.findByText("No job activity yet")).toBeTruthy();
    view.unmount();
    render(<ActivityCenterPage />);
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Retry" })).toBeTruthy();
  });
});
