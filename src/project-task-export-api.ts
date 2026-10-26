import { invoke } from "@tauri-apps/api/core";

export type ProjectTaskExportResult = {
  cancelled: boolean;
  fileName: string | null;
  projectCount: number;
  taskCount: number;
  activityEventCount: number;
};

export function exportProjectTaskData(): Promise<ProjectTaskExportResult> {
  return invoke<ProjectTaskExportResult>("export_project_task_data");
}
