import { invoke } from "@tauri-apps/api/core";

export type LocalMetricStatus = "available" | "unsupported" | "unavailable";

export type LocalMetricValue = {
  value: number | null;
  status: LocalMetricStatus;
};

export type LocalMetricsSnapshot = {
  source: string;
  sampledAt: string;
  logicalProcessors: LocalMetricValue;
  physicalMemoryTotalBytes: LocalMetricValue;
  physicalMemoryAvailableBytes: LocalMetricValue;
  windowsVolumeTotalBytes: LocalMetricValue;
  windowsVolumeFreeBytes: LocalMetricValue;
};

export function getLocalMetricsSnapshot(): Promise<LocalMetricsSnapshot> {
  return invoke<LocalMetricsSnapshot>("get_local_metrics_snapshot");
}
