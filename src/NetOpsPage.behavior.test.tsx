// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { NetOpsPage } from "./NetOpsPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M4-014 network diagnostic follow-up", () => {
  it("creates an inbox Task from an actual diagnostic result handle", async () => {
    invokeMock.mockResolvedValueOnce({
      id: "7f7eae13-c3a8-4370-8b79-b637fdd6b443",
      kind: "dns_lookup",
      targetHost: "internal.example",
      targetPort: null,
      observedAtEpochMs: 1_800_000_000_000,
      durationMs: 18,
      source: "SystemResolver",
      outcome: "Failed",
      errorCode: "ResolverError",
      resolvedAddresses: [],
    });
    invokeMock.mockResolvedValueOnce({ id: "task-1" });
    render(<NetOpsPage />);

    await userEvent.type(
      screen.getByRole("textbox", { name: "Hostname or IP address" }),
      "internal.example",
    );
    await userEvent.click(screen.getByRole("button", { name: "Run diagnostic" }));
    expect(await screen.findByText("Failed · ResolverError")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Create follow-up task" }));
    expect((await screen.findByRole("status")).textContent).toMatch(
      /saved to your personal inbox/i,
    );
    expect(invokeMock).toHaveBeenNthCalledWith(1, "run_network_diagnostic", {
      request: {
        kind: "dns_lookup",
        targetHost: "internal.example",
        targetPort: null,
        timeoutMs: 2_000,
      },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "create_task_from_diagnostic", {
      request: { diagnosticResultId: "7f7eae13-c3a8-4370-8b79-b637fdd6b443" },
    });
  });

  it("does not offer a Task for an unsupported diagnostic result", async () => {
    invokeMock.mockResolvedValue({
      id: "c307965a-8995-44d0-a30d-0d24fb9094d4",
      kind: "tcp_connectivity",
      targetHost: "192.0.2.4",
      targetPort: 443,
      observedAtEpochMs: 1_800_000_000_000,
      durationMs: 0,
      source: "OperatingSystemTcpConnect",
      outcome: "Unsupported",
      errorCode: "Unsupported",
      resolvedAddresses: [],
    });
    render(<NetOpsPage />);
    await userEvent.type(
      screen.getByRole("textbox", { name: "Hostname or IP address" }),
      "example.test",
    );
    await userEvent.click(screen.getByRole("button", { name: "Run diagnostic" }));
    expect(await screen.findByText("Unsupported · Unsupported")).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Create follow-up task" }).hasAttribute("disabled"),
    ).toBe(true);
  });
});
