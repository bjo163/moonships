// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ServicesPage } from "./ServicesPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

const snapshot = {
  status: "available",
  observedAtEpochMs: 1_800_000_000_000,
  services: [{ serviceName: "Example", displayName: "Example Service", state: "running" }],
  errorCode: null,
};

describe("M4-009 read-only Windows service inventory", () => {
  it("shows service status with no service-control action", async () => {
    invokeMock.mockResolvedValue(snapshot);
    render(<ServicesPage />);

    const table = await screen.findByRole("table", { name: "Read-only Windows service inventory" });
    expect(within(table).getByText("Example Service")).toBeTruthy();
    expect(within(table).getByText("Example")).toBeTruthy();
    expect(within(table).getByText("Running")).toBeTruthy();
    expect(screen.getAllByRole("button")).toHaveLength(1);
    expect(screen.getByRole("button", { name: "Refresh snapshot" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /start|stop|restart|change service/i })).toBeNull();
  });

  it("makes permission denial visible without requesting elevation", async () => {
    invokeMock.mockResolvedValue({
      ...snapshot,
      status: "permissionDenied",
      services: [],
      errorCode: "accessDenied",
    });
    render(<ServicesPage />);

    expect(
      await screen.findByText(
        "Windows did not allow service enumeration. Moonships did not request elevation.",
      ),
    ).toBeTruthy();
    expect(screen.getByText(/Permission denied/)).toBeTruthy();
  });

  it("distinguishes an empty service list from an unavailable query", async () => {
    invokeMock.mockResolvedValueOnce({ ...snapshot, services: [] });
    const emptyView = render(<ServicesPage />);
    expect(
      await screen.findByText("No service entries were returned in this snapshot."),
    ).toBeTruthy();
    emptyView.unmount();

    invokeMock.mockResolvedValueOnce({
      ...snapshot,
      status: "unavailable",
      services: [],
      errorCode: "managerUnavailable",
    });
    render(<ServicesPage />);
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByText("Windows could not provide a service snapshot.")).toBeTruthy();
  });

  it("does not describe a partial empty page as a successful empty snapshot", async () => {
    invokeMock.mockResolvedValue({
      ...snapshot,
      status: "partial",
      services: [],
      errorCode: "enumerationFailed",
    });
    render(<ServicesPage />);

    expect(
      await screen.findByText(
        "Enumeration was incomplete; no service entries were included in this snapshot.",
      ),
    ).toBeTruthy();
    expect(screen.queryByText("No service entries were returned in this snapshot.")).toBeNull();
  });
});
