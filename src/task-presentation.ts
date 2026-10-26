import type { TaskPriority, TaskStatus } from "./tasks-api";

export const TASK_STATUS_LABELS: Record<TaskStatus, string> = {
  backlog: "Backlog",
  todo: "Todo",
  in_progress: "In progress",
  blocked: "Blocked",
  in_review: "In review",
  done: "Done",
  cancelled: "Cancelled",
};

export const TASK_PRIORITY_LABELS: Record<TaskPriority, string> = {
  p0: "P0 · Critical",
  p1: "P1 · High",
  p2: "P2 · Normal",
  p3: "P3 · Low",
};

export const TASK_STATUS_ORDER: Record<TaskStatus, number> = {
  backlog: 0,
  todo: 1,
  in_progress: 2,
  blocked: 3,
  in_review: 4,
  done: 5,
  cancelled: 6,
};

export const TASK_PRIORITY_ORDER: Record<TaskPriority, number> = {
  p0: 0,
  p1: 1,
  p2: 2,
  p3: 3,
};

export function formatTaskDate(value: string | null): string {
  if (!value) return "No due date";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Date unavailable";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium" }).format(date);
}

export function isTaskOverdue(
  dueAt: string | null,
  status: TaskStatus,
  now: Date = new Date(),
): boolean {
  if (!dueAt || status === "done" || status === "cancelled") return false;
  const dueDate = new Date(dueAt);
  return !Number.isNaN(dueDate.getTime()) && dueDate.getTime() < now.getTime();
}
