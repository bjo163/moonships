import { useCallback, useEffect, useMemo, useState } from "react";
import { listProjects, type ProjectRecord } from "./projects-api";
import { formatTaskDate, TASK_PRIORITY_LABELS, TASK_STATUS_LABELS } from "./task-presentation";
import { sortTasks, type SortDirection, type TaskSortKey } from "./task-list-sort";
import { listTasks, toTaskErrorMessage, type TaskRecord } from "./tasks-api";
import { Badge, Button, EmptyState, ErrorState, LoadingState } from "./ui-primitives";

const SORT_COLUMNS: Array<{ key: TaskSortKey; label: string }> = [
  { key: "title", label: "Task" },
  { key: "project", label: "Project" },
  { key: "status", label: "Status" },
  { key: "priority", label: "Priority" },
  { key: "dueAt", label: "Due date" },
  { key: "updatedAt", label: "Updated" },
];

function formatUpdatedAt(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Date unavailable";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
    date,
  );
}

export function TaskListPage({ onCreateTask }: { onCreateTask: () => void }) {
  const [tasks, setTasks] = useState<TaskRecord[]>([]);
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [projectLoadError, setProjectLoadError] = useState(false);
  const [sortKey, setSortKey] = useState<TaskSortKey>("title");
  const [sortDirection, setSortDirection] = useState<SortDirection>("ascending");

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    const [taskResult, projectResult] = await Promise.allSettled([listTasks(), listProjects()]);
    if (taskResult.status === "fulfilled") {
      setTasks(taskResult.value);
    } else {
      setTasks([]);
      setLoadError(toTaskErrorMessage(taskResult.reason, "load"));
    }
    if (projectResult.status === "fulfilled") {
      setProjects(projectResult.value);
      setProjectLoadError(false);
    } else {
      setProjects([]);
      setProjectLoadError(true);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const projectNames = useMemo(
    () => new Map(projects.map((project) => [project.id, project.name])),
    [projects],
  );
  const sortedTasks = useMemo(
    () => sortTasks(tasks, sortKey, sortDirection, projectNames),
    [tasks, sortKey, sortDirection, projectNames],
  );

  const changeSort = (key: TaskSortKey) => {
    if (key === sortKey) {
      setSortDirection((direction) => (direction === "ascending" ? "descending" : "ascending"));
    } else {
      setSortKey(key);
      setSortDirection("ascending");
    }
  };

  return (
    <section className="tasks-page" aria-labelledby="tasks-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">WORKSPACE / TASKS</p>
          <h2 id="tasks-title">Tasks</h2>
          <p className="muted">Active tasks stored on this device in your Personal workspace.</p>
        </div>
        <Button onClick={onCreateTask}>New task</Button>
      </header>

      <div className="tasks-page__toolbar">
        <span className="muted" aria-live="polite">
          {loading
            ? "Loading tasks…"
            : `${tasks.length} active ${tasks.length === 1 ? "task" : "tasks"}`}
        </span>
        <Button variant="secondary" size="sm" onClick={() => void reload()} disabled={loading}>
          Refresh
        </Button>
      </div>

      {loading ? <LoadingState label="Loading tasks from this device" /> : null}
      {!loading && loadError ? (
        <ErrorState
          title="Tasks are unavailable"
          description={loadError}
          onRetry={() => void reload()}
        />
      ) : null}
      {!loading && !loadError && tasks.length === 0 ? (
        <EmptyState
          title="No active tasks yet"
          description="Create a task in your Personal inbox or attach it to an active project."
          action={<Button onClick={onCreateTask}>Create your first task</Button>}
        />
      ) : null}
      {!loading && !loadError && tasks.length > 0 ? (
        <>
          {projectLoadError ? (
            <p className="tasks-page__notice" role="status">
              Project names are unavailable. Tasks remain visible; refresh to try loading project
              names again.
            </p>
          ) : null}
          <div className="tasks-table-wrap">
            <table className="tasks-table">
              <caption className="visually-hidden">Active tasks in the Personal workspace</caption>
              <thead>
                <tr>
                  {SORT_COLUMNS.map(({ key, label }) => (
                    <th key={key} scope="col" aria-sort={sortKey === key ? sortDirection : "none"}>
                      <Button
                        variant="ghost"
                        size="sm"
                        aria-label={`Sort by ${label.toLowerCase()} ${
                          sortKey === key && sortDirection === "ascending"
                            ? "descending"
                            : "ascending"
                        }`}
                        onClick={() => changeSort(key)}
                      >
                        {label}
                        <span aria-hidden="true">
                          {sortKey === key ? (sortDirection === "ascending" ? " ↑" : " ↓") : ""}
                        </span>
                      </Button>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {sortedTasks.map((task) => {
                  const projectName = task.projectId
                    ? (projectNames.get(task.projectId) ?? "Archived or unavailable project")
                    : "Personal inbox";
                  return (
                    <tr key={task.id}>
                      <th scope="row">
                        <span className="tasks-table__title">{task.title}</span>
                        {task.description ? (
                          <span className="tasks-table__description">{task.description}</span>
                        ) : null}
                      </th>
                      <td>{projectName}</td>
                      <td>
                        <Badge
                          variant={
                            task.status === "done"
                              ? "success"
                              : task.status === "blocked"
                                ? "warning"
                                : "neutral"
                          }
                        >
                          {TASK_STATUS_LABELS[task.status]}
                        </Badge>
                      </td>
                      <td>
                        <Badge variant={task.priority === "p0" ? "danger" : "neutral"}>
                          {TASK_PRIORITY_LABELS[task.priority]}
                        </Badge>
                      </td>
                      <td>{formatTaskDate(task.dueAt)}</td>
                      <td>{formatUpdatedAt(task.updatedAt)}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </>
      ) : null}
    </section>
  );
}
