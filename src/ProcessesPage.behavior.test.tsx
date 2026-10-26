// @vitest-environment jsdom
import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ProcessesPage } from "./ProcessesPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const snapshot = {
  status: "available",
  observedAtEpochMs: 1_800_000_000_000,
  processes: [
    { processId: 42, parentProcessId: 1, threadCount: 3, executableName: "moonships.exe" },
  ],
  errorCode: null,
};

describe("M4-008 read-only process inventory", () => {
  it("shows a bounded process snapshot without exposing process actions", async () => {
    invokeMock.mockResolvedValue(snapshot);
    render(<ProcessesPage />);

    expect(await screen.findByRole("table", { name: "Read-only process inventory" })).toBeTruthy();
    expect(screen.getByText("moonships.exe")).toBeTruthy();
    expect(screen.getByText("42")).toBeTruthy();
    const row = screen.getByRole("row", { name: /moonships\.exe/ });
    expect(within(row).queryByRole("button")).toBeNull();
    expect(screen.getAllByRole("button")).toHaveLength(1);
    expect(screen.getByRole("button", { name: "Refresh snapshot" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /kill|terminate|end process/i })).toBeNull();
  });

  it("surfaces permission denial without requesting elevation", async () => {
    invokeMock.mockResolvedValue({
      ...snapshot,
      status: "permissionDenied",
      processes: [],
      errorCode: "accessDenied",
    });
    render(<ProcessesPage />);

    expect(
      await screen.findByText(
        "Windows did not allow the process snapshot. Moonships did not request elevation.",
      ),
    ).toBeTruthy();
    expect(screen.getByText(/Permission denied/)).toBeTruthy();
  });

  it("distinguishes an empty snapshot from an unsupported platform", async () => {
    invokeMock.mockResolvedValueOnce({ ...snapshot, processes: [] });
    const emptyView = render(<ProcessesPage />);
    expect(
      await screen.findByText("No process entries were returned in this snapshot."),
    ).toBeTruthy();
    emptyView.unmount();

    invokeMock.mockResolvedValueOnce({
      ...snapshot,
      status: "unsupported",
      processes: [],
    });
    render(<ProcessesPage />);
    expect(
      await screen.findByText("Read-only process inventory is unsupported on this platform."),
    ).toBeTruthy();
  });

  it("reports adapter failures and allows an explicit retry", async () => {
    invokeMock.mockRejectedValueOnce(new Error("fixture"));
    invokeMock.mockResolvedValueOnce(snapshot);
    render(<ProcessesPage />);

    expect(await screen.findByRole("alert")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() => expect(screen.getByText("moonships.exe")).toBeTruthy());
    expect(invokeMock).toHaveBeenCalledTimes(2);
  });
});
