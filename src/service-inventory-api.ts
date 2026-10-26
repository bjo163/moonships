import { invoke } from "@tauri-apps/api/core";

export type ServiceInventoryStatus =
  | "available"
  | "partial"
  | "permissionDenied"
  | "unsupported"
  | "unavailable";

export type WindowsServiceState =
  | "stopped"
  | "startPending"
  | "stopPending"
  | "running"
  | "continuePending"
  | "pausePending"
  | "paused"
  | "unknown";

export type ServiceInventorySnapshot = {
  status: ServiceInventoryStatus;
  observedAtEpochMs: number;
  services: Array<{
    serviceName: string;
    displayName: string;
    state: WindowsServiceState;
  }>;
  errorCode:
    | "accessDenied"
    | "managerUnavailable"
    | "enumerationFailed"
    | "resultLimitReached"
    | null;
};

export function listServiceInventory(): Promise<ServiceInventorySnapshot> {
  return invoke<ServiceInventorySnapshot>("list_service_inventory");
}
