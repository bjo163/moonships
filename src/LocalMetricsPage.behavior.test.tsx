// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { LocalMetricsPage } from "./LocalMetricsPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const snapshot = {
  source: "Local operating system APIs",
  sampledAt: "2026-10-26T12:00:00Z",
  logicalProcessors: { value: 8, status: "available" },
  physicalMemoryTotalBytes: { value: null, status: "unsupported" },
  physicalMemoryAvailableBytes: { value: null, status: "unsupported" },
  windowsVolumeTotalBytes: { value: null, status: "unsupported" },
  windowsVolumeFreeBytes: { value: null, status: "unsupported" },
};

describe("M6-006 local metrics snapshot", () => {
  it("shows local source, sample time and unsupported metrics without remote calls", async () => {
    invokeMock.mockResolvedValue(snapshot);
    render(<LocalMetricsPage />);
    expect(await screen.findByText("Source: Local operating system APIs")).toBeTruthy();
    expect(screen.getByText(/Sampled:/).textContent).toContain("Oct");
    expect(screen.getByRole("row", { name: /Logical processors 8 Available/ })).toBeTruthy();
    expect(screen.getAllByText("Unsupported on this platform")).toHaveLength(4);
    expect(invokeMock.mock.calls).toEqual([["get_local_metrics_snapshot"]]);
  });

  it("refreshes only on the explicit action and reports read failure with retry", async () => {
    invokeMock
      .mockResolvedValueOnce(snapshot)
      .mockRejectedValueOnce(new Error("offline"))
      .mockResolvedValueOnce(snapshot);
    render(<LocalMetricsPage />);
    await screen.findByText("Source: Local operating system APIs");
    fireEvent.click(screen.getByRole("button", { name: "Refresh snapshot" }));
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledTimes(2);
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(await screen.findByText("Source: Local operating system APIs")).toBeTruthy();
  });
});
