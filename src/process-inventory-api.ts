import { invoke } from "@tauri-apps/api/core";

export type ProcessInventoryStatus =
  | "available"
  | "partial"
  | "permissionDenied"
  | "unsupported"
  | "unavailable";

export type ProcessInventorySnapshot = {
  status: ProcessInventoryStatus;
  observedAtEpochMs: number;
  processes: Array<{
    processId: number;
    parentProcessId: number;
    threadCount: number;
    executableName: string;
  }>;
  errorCode: "accessDenied" | "snapshotFailed" | "enumerationFailed" | "resultLimitReached" | null;
};

export function listProcessInventory(): Promise<ProcessInventorySnapshot> {
  return invoke<ProcessInventorySnapshot>("list_process_inventory");
}
