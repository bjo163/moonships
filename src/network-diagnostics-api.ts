import { invoke } from "@tauri-apps/api/core";

export type NetworkDiagnosticKind = "dns_lookup" | "tcp_connectivity";

export type NetworkDiagnosticResult = {
  id: string;
  kind: NetworkDiagnosticKind;
  targetHost: string;
  targetPort: number | null;
  observedAtEpochMs: number;
  durationMs: number;
  source: string;
  outcome: string;
  errorCode: string | null;
  resolvedAddresses: string[];
};

export function runNetworkDiagnostic(input: {
  kind: NetworkDiagnosticKind;
  targetHost: string;
  targetPort: number | null;
}): Promise<NetworkDiagnosticResult> {
  return invoke("run_network_diagnostic", { request: { ...input, timeoutMs: 2_000 } });
}

export function createTaskFromDiagnostic(diagnosticResultId: string): Promise<unknown> {
  return invoke("create_task_from_diagnostic", { request: { diagnosticResultId } });
}

export function networkDiagnosticErrorMessage(error: unknown): string {
  const message = typeof error === "string" ? error : error instanceof Error ? error.message : "";
  const code = message.split(":", 1)[0];
  if (code === "VALIDATION_FAILED")
    return "Enter one valid host and, for TCP, a port from 1 to 65535.";
  if (code === "RATE_LIMITED")
    return "The local diagnostic limit was reached. Wait before trying again.";
  if (code === "TIMED_OUT") return "The diagnostic exceeded its 2-second time limit.";
  if (code === "NOT_FOUND") return "This diagnostic result expired. Run the check again.";
  return "The diagnostic could not be completed. No task was created.";
}
