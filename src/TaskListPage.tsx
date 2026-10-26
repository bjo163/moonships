import { useCallback, useEffect, useMemo, useRef, useState, type DragEvent } from "react";
import { listProjects, type ProjectRecord } from "./projects-api";
import {
  formatTaskDate,
  TASK_PRIORITY_LABELS,
  TASK_STATUS_LABELS,
  TASK_STATUS_ORDER,
} from "./task-presentation";
import { sortTasks, type SortDirection, type TaskSortKey } from "./task-list-sort";
import {
  listTasks,
  toTaskErrorMessage,
  updateTask,
  type TaskRecord,
  type TaskStatus,
} from "./tasks-api";
import { Badge, Button, EmptyState, ErrorState, LoadingState } from "./ui-primitives";

const SORT_COLUMNS: Array<{ key: TaskSortKey; label: string }> = [
  { key: "title", label: "Task" },
  { key: "project", label: "Project" },
  { key: "status", label: "Status" },
  { key: "priority", label: "Priority" },
  { key: "dueAt", label: "Due date" },
  { key: "updatedAt", label: "Updated" },
];

const TASK_STATUSES = Object.keys(TASK_STATUS_ORDER) as TaskStatus[];

function formatUpdatedAt(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Date unavailable";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
    date,
  );
}

export function TaskListPage({
  onCreateTask,
  onOpenTask,
}: {
  onCreateTask: () => void;
  onOpenTask: (taskId: string) => void;
}) {
  const [tasks, setTasks] = useState<TaskRecord[]>([]);
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [projectLoadError, setProjectLoadError] = useState(false);
  const [sortKey, setSortKey] = useState<TaskSortKey>("title");
  const [sortDirection, setSortDirection] = useState<SortDirection>("ascending");
  const [view, setView] = useState<"list" | "board">("list");
  const [updatingTaskIds, setUpdatingTaskIds] = useState<Set<string>>(() => new Set());
  const updatingTaskIdsRef = useRef(new Set<string>());
  const [statusMessage, setStatusMessage] = useState("");
  const [statusError, setStatusError] = useState("");

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

  const changeTaskStatus = async (taskId: string, nextStatus: TaskStatus) => {
    const task = tasks.find((item) => item.id === taskId);
    if (!task || task.status === nextStatus || updatingTaskIdsRef.current.has(taskId)) return;
    updatingTaskIdsRef.current.add(taskId);
    setUpdatingTaskIds((current) => new Set(current).add(taskId));
    setStatusMessage("");
    setStatusError("");
    try {
      const saved = await updateTask({
        id: task.id,
        projectId: task.projectId,
        title: task.title,
        description: task.description,
        dueAt: task.dueAt,
        status: nextStatus,
        priority: task.priority,
      });
      setTasks((current) => current.map((item) => (item.id === saved.id ? saved : item)));
      setStatusMessage(`${saved.title} moved to ${TASK_STATUS_LABELS[saved.status]} and saved.`);
    } catch (error) {
      setStatusError(
        `${toTaskErrorMessage(error, "update")} The Board stays on ${TASK_STATUS_LABELS[task.status]} until a saved update is confirmed.`,
      );
    } finally {
      updatingTaskIdsRef.current.delete(taskId);
      setUpdatingTaskIds((current) => {
        const next = new Set(current);
        next.delete(taskId);
        return next;
      });
    }
  };

  const handleDrop = (event: DragEvent<HTMLElement>, status: TaskStatus) => {
    event.preventDefault();
    const taskId = event.dataTransfer.getData("text/plain");
    if (taskId) void changeTaskStatus(taskId, status);
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
        <div className="tasks-page__toolbar-actions">
          <fieldset className="tasks-view-toggle">
            <legend className="visually-hidden">Task view</legend>
            <Button
              variant={view === "list" ? "secondary" : "ghost"}
              size="sm"
              aria-pressed={view === "list"}
              onClick={() => setView("list")}
            >
              List
            </Button>
            <Button
              variant={view === "board" ? "secondary" : "ghost"}
              size="sm"
              aria-pressed={view === "board"}
              onClick={() => setView("board")}
            >
              Board
            </Button>
          </fieldset>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => void reload()}
            disabled={loading || updatingTaskIds.size > 0}
          >
            Refresh
          </Button>
        </div>
      </div>
      <p className="visually-hidden" aria-live="polite">
        {statusMessage}
      </p>
      {statusError ? (
        <p className="tasks-page__notice tasks-page__notice--error" role="status">
          {statusError}
        </p>
      ) : null}

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
          {view === "list" ? (
            <div className="tasks-table-wrap">
              <table className="tasks-table">
                <caption className="visually-hidden">
                  Active tasks in the Personal workspace
                </caption>
                <thead>
                  <tr>
                    {SORT_COLUMNS.map(({ key, label }) => (
                      <th
                        key={key}
                        scope="col"
                        aria-sort={sortKey === key ? sortDirection : "none"}
                      >
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
                          <button
                            type="button"
                            className="tasks-table__title-link"
                            onClick={() => onOpenTask(task.id)}
                          >
                            {task.title}
                          </button>
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
          ) : (
            <>
              <p className="tasks-page__board-help">
                Drag a task between status columns, or use its status menu to move it with the
                keyboard.
              </p>
              <section className="task-board-scroll" aria-label="Tasks grouped by status">
                <div className="task-board">
                  {TASK_STATUSES.map((status) => {
                    const columnTasks = sortedTasks.filter((task) => task.status === status);
                    const columnId = `task-board-${status}`;
                    return (
                      <section
                        className="task-board__column"
                        key={status}
                        aria-labelledby={columnId}
                        onDragOver={(event) => event.preventDefault()}
                        onDrop={(event) => handleDrop(event, status)}
                      >
                        <header className="task-board__column-header">
                          <h3 id={columnId}>{TASK_STATUS_LABELS[status]}</h3>
                          <span className="task-board__count" aria-hidden="true">
                            {columnTasks.length}
                          </span>
                          <span className="visually-hidden">
                            {columnTasks.length} {columnTasks.length === 1 ? "task" : "tasks"}
                          </span>
                        </header>
                        <div className="task-board__cards">
                          {columnTasks.length === 0 ? (
                            <p className="task-board__empty">No tasks in this status.</p>
                          ) : null}
                          {columnTasks.map((task) => {
                            const projectName = task.projectId
                              ? (projectNames.get(task.projectId) ??
                                "Archived or unavailable project")
                              : "Personal inbox";
                            const updating = updatingTaskIds.has(task.id);
                            return (
                              <article
                                className={`task-board-card${updating ? " task-board-card--updating" : ""}`}
                                key={task.id}
                                draggable={!updating}
                                onDragStart={(event) => {
                                  event.dataTransfer.setData("text/plain", task.id);
                                  event.dataTransfer.effectAllowed = "move";
                                }}
                              >
                                <button
                                  type="button"
                                  className="tasks-table__title-link task-board-card__title"
                                  onClick={() => onOpenTask(task.id)}
                                >
                                  {task.title}
                                </button>
                                {task.description ? (
                                  <p className="task-board-card__description">{task.description}</p>
                                ) : null}
                                <dl className="task-board-card__details">
                                  <div>
                                    <dt>Project</dt>
                                    <dd>{projectName}</dd>
                                  </div>
                                  <div>
                                    <dt>Priority</dt>
                                    <dd>{TASK_PRIORITY_LABELS[task.priority]}</dd>
                                  </div>
                                  <div>
                                    <dt>Due</dt>
                                    <dd>{formatTaskDate(task.dueAt)}</dd>
                                  </div>
                                </dl>
                                <div className="task-board-card__move">
                                  <label
                                    className="visually-hidden"
                                    htmlFor={`task-status-${task.id}`}
                                  >
                                    Move {task.title} to status
                                  </label>
                                  <select
                                    id={`task-status-${task.id}`}
                                    className="ms-input"
                                    value={task.status}
                                    disabled={updating}
                                    onChange={(event) =>
                                      void changeTaskStatus(
                                        task.id,
                                        event.target.value as TaskStatus,
                                      )
                                    }
                                  >
                                    {TASK_STATUSES.map((value) => (
                                      <option key={value} value={value}>
                                        {TASK_STATUS_LABELS[value]}
                                      </option>
                                    ))}
                                  </select>
                                  {updating ? <span role="status">Saving status…</span> : null}
                                </div>
                              </article>
                            );
                          })}
                        </div>
                      </section>
                    );
                  })}
                </div>
              </section>
            </>
          )}
        </>
      ) : null}
    </section>
  );
}
