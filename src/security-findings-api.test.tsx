import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTaskFromSecurityFinding } from "./security-findings-api";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

beforeEach(() => invokeMock.mockReset());

describe("security finding Task handoff API", () => {
  it("sends the typed finding, scope, remediation and evidence URL to the native validator", async () => {
    invokeMock.mockResolvedValue({ id: "task-1" });
    const request = {
      finding: {
        id: "b72f34e5-57c3-4fea-a09f-27faf6ab25ae",
        title: "Certificate expiry",
        summary: "Certificate expires soon",
        severity: "medium" as const,
        likelihood: "likely" as const,
        state: "open" as const,
        target: { kind: "hostname" as const, value: "api.example.test" },
        observedAt: "2026-10-26T12:00:00Z",
      },
      scope: {
        authorizedTargets: [{ kind: "hostname" as const, value: "api.example.test" }],
        authorizationNote: "Ticket SEC-42",
        excludedTargets: [],
        expiresAt: "2026-10-27T12:00:00Z",
      },
      remediation: "Rotate certificate",
      evidenceUrl: "https://evidence.example.test/report",
    };
    await createTaskFromSecurityFinding(request);
    expect(invokeMock).toHaveBeenCalledWith("create_task_from_security_finding", { request });
  });
});
