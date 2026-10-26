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

export type UpdateTaskInput = {
  id: string;
  projectId: string | null;
  title: string;
  description: string | null;
  dueAt: string | null;
  status: TaskStatus;
  priority: TaskPriority;
};

export function listTasks(): Promise<TaskRecord[]> {
  return invoke<{ tasks: TaskRecord[] }>("list_tasks", { request: {} }).then(({ tasks }) => tasks);
}

export function createTask(request: CreateTaskInput): Promise<TaskRecord> {
  return invoke<TaskRecord>("create_task", { request });
}

export function getTask(id: string): Promise<TaskRecord> {
  return invoke<TaskRecord>("get_task", { request: { id } });
}

export function updateTask(request: UpdateTaskInput): Promise<TaskRecord> {
  return invoke<TaskRecord>("update_task", { request });
}

export function toTaskErrorMessage(
  error: unknown,
  action: "load" | "save" | "update" = "save",
): string {
  const code = typeof error === "string" ? error.split(":", 1)[0] : "";
  if (code === "VALIDATION_FAILED")
    return "Check the task title, status and priority, then try again.";
  if (code === "NOT_FOUND")
    return action === "load"
      ? "This task is unavailable or has been archived. Return to the task list and refresh."
      : "That project or task is no longer available. Refresh and try again.";
  if (action === "load") return "Tasks could not be read from this device. Try again.";
  if (action === "update")
    return "The task could not be saved. Your entered details are still here; try again.";
  return "The task could not be saved. Your entered details are still here; try again.";
}
