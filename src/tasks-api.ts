import { invoke } from "@tauri-apps/api/core";

export type TaskStatus =
  | "backlog"
  | "todo"
  | "in_progress"
  | "blocked"
  | "in_review"
  | "done"
  | "cancelled";

export type TaskPriority = "p0" | "p1" | "p2" | "p3";

export type TaskRecord = {
  id: string;
  workspaceId: string;
  projectId: string | null;
  title: string;
  description: string | null;
  dueAt: string | null;
  status: TaskStatus;
  priority: TaskPriority;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
};

export type CreateTaskInput = {
  title: string;
  projectId: string | null;
  status: TaskStatus;
  priority: TaskPriority;
};

export function listTasks(): Promise<TaskRecord[]> {
  return invoke<{ tasks: TaskRecord[] }>("list_tasks", { request: {} }).then(({ tasks }) => tasks);
}

export function createTask(request: CreateTaskInput): Promise<TaskRecord> {
  return invoke<TaskRecord>("create_task", { request });
}

export function toTaskErrorMessage(error: unknown, action: "load" | "save" = "save"): string {
  const code = typeof error === "string" ? error.split(":", 1)[0] : "";
  if (code === "VALIDATION_FAILED")
    return "Check the task title, status and priority, then try again.";
  if (code === "NOT_FOUND")
    return "That project is no longer available. Choose another project or the Personal inbox.";
  if (action === "load") return "Tasks could not be read from this device. Try again.";
  return "The task could not be saved. Your entered details are still here; try again.";
}
