import { invoke } from "@tauri-apps/api/core";

export type ProjectTaskImportPreviewResult = {
  cancelled: boolean;
  fileName: string | null;
  schemaVersion: number | null;
  appVersion: string | null;
  exportedAt: string | null;
  projectCount: number;
  taskCount: number;
  activityEventCount: number;
  projectNames: string[];
  taskTitles: string[];
};

export function previewProjectTaskImport(): Promise<ProjectTaskImportPreviewResult> {
  return invoke<ProjectTaskImportPreviewResult>("preview_project_task_import");
}
