import { TASK_PRIORITY_ORDER, TASK_STATUS_ORDER } from "./task-presentation";
import type { TaskRecord } from "./tasks-api";

export type TaskSortKey = "title" | "project" | "status" | "priority" | "dueAt" | "updatedAt";
export type SortDirection = "ascending" | "descending";

function compareText(left: string, right: string): number {
  const a = left.toLowerCase();
  const b = right.toLowerCase();
  return a < b ? -1 : a > b ? 1 : 0;
}

function compareTask(
  left: TaskRecord,
  right: TaskRecord,
  key: TaskSortKey,
  projectNames: ReadonlyMap<string, string>,
): number {
  switch (key) {
    case "title":
      return compareText(left.title, right.title);
    case "project":
      return compareText(
        left.projectId ? (projectNames.get(left.projectId) ?? "") : "",
        right.projectId ? (projectNames.get(right.projectId) ?? "") : "",
      );
    case "status":
      return TASK_STATUS_ORDER[left.status] - TASK_STATUS_ORDER[right.status];
    case "priority":
      return TASK_PRIORITY_ORDER[left.priority] - TASK_PRIORITY_ORDER[right.priority];
    case "dueAt":
      if (left.dueAt === null || right.dueAt === null) {
        return left.dueAt === right.dueAt ? 0 : left.dueAt === null ? 1 : -1;
      }
      return left.dueAt < right.dueAt ? -1 : left.dueAt > right.dueAt ? 1 : 0;
    case "updatedAt":
      return left.updatedAt < right.updatedAt ? -1 : left.updatedAt > right.updatedAt ? 1 : 0;
  }
}

export function sortTasks(
  tasks: readonly TaskRecord[],
  key: TaskSortKey,
  direction: SortDirection,
  projectNames: ReadonlyMap<string, string> = new Map(),
): TaskRecord[] {
  return [...tasks].sort((left, right) => {
    const compared = compareTask(left, right, key, projectNames);
    if (compared !== 0) {
      // Tasks without a due date remain at the bottom in either direction.
      if (key === "dueAt" && (left.dueAt === null || right.dueAt === null)) return compared;
      return direction === "ascending" ? compared : -compared;
    }
    return compareText(left.id, right.id);
  });
}
