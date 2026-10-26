import { invoke } from "@tauri-apps/api/core";

export type CleanupReportStatus = "succeeded" | "partial" | "failed";

export type CleanupReportSummary = {
  id: string;
  previewId: string;
  taskId: string | null;
  status: CleanupReportStatus;
  completedAt: string;
  selectedCount: number;
  deletedCount: number;
  skippedCount: number;
  failedCount: number;
  estimatedBytes: number;
  actualDeletedBytes: number;
};

export function listCleanupReports(): Promise<CleanupReportSummary[]> {
  return invoke<CleanupReportSummary[]>("list_cleanup_reports");
}

export function toCleanupReportErrorMessage(): string {
  return "Saved cleanup results could not be loaded from this device. Try again.";
}
