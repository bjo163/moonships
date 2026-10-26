// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { HttpInspectorPage } from "./HttpInspectorPage";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

afterEach(cleanup);
beforeEach(() => invokeMock.mockReset());

describe("M6-002 HTTP Inspector", () => {
  it("loads saved history and inspects only after explicit submit", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "list_http_inspection_history") return Promise.resolve([]);
      if (command === "inspect_http_response")
        return Promise.resolve({
          history: {
            id: "inspection-1",
            requestedUrl: "https://example.test/status?token=%5BREDACTED%5D",
            observedAt: "2026-10-26T12:00:00Z",
            status: 200,
            requestHeaders: [{ name: "accept", value: "*/*" }],
            responseHeaders: [
              { name: "set-cookie", value: "[REDACTED]" },
              { name: "content-type", value: "application/json" },
            ],
            responseSizeBytes: 18,
            errorCode: null,
          },
          bodyText: '{"status":"ok"}',
          bodyIsUtf8: true,
        });
      if (command === undefined) return Promise.resolve([]);
      if (command === "create_task_from_http_inspection") return Promise.resolve({ id: "task-1" });
      return Promise.reject(new Error(`Unexpected ${command}`));
    });

    render(<HttpInspectorPage />);
    expect(await screen.findByText("No inspections saved.")).toBeTruthy();
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual([
      "list_http_inspection_history",
    ]);
    fireEvent.change(screen.getByLabelText("Public HTTPS URL"), {
      target: { value: "https://example.test/status?token=secret" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Inspect response" }));
    expect(await screen.findByText('{"status":"ok"}')).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("inspect_http_response", {
      request: { url: "https://example.test/status?token=secret" },
    });
    expect(await screen.findByText("HTTP 200")).toBeTruthy();
    expect(screen.getAllByText("[REDACTED]").length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: "Create follow-up Task" }));
    expect(await screen.findByText("Task created: task-1")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("create_task_from_http_inspection", {
      request: { historyId: "inspection-1" },
    });
  });

  it("shows history read failures instead of a false empty state", async () => {
    invokeMock.mockImplementation((command: string | undefined) =>
      command === "list_http_inspection_history"
        ? Promise.reject(new Error("offline"))
        : Promise.resolve([]),
    );
    render(<HttpInspectorPage />);
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.queryByText("No inspections saved.")).toBeNull();
  });
});
