import { type DragEvent, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listProjects, type ProjectRecord } from "./projects-api";
import { type SortDirection, sortTasks, type TaskSortKey } from "./task-list-sort";
import {
  formatTaskDate,
  isTaskOverdue,
  TASK_PRIORITY_LABELS,
  TASK_STATUS_LABELS,
  TASK_STATUS_ORDER,
} from "./task-presentation";
import {
  listTasks,
  type TaskPriority,
  type TaskRecord,
  type TaskStatus,
  toTaskErrorMessage,
  updateTask,
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
const TASK_PRIORITIES: TaskPriority[] = ["p0", "p1", "p2", "p3"];
const PERSONAL_INBOX_FILTER = "__personal_inbox__";

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
  const [searchQuery, setSearchQuery] = useState("");
  const [normalizedQuery, setNormalizedQuery] = useState("");
  const [projectFilter, setProjectFilter] = useState("");
  const [statusFilter, setStatusFilter] = useState<TaskStatus | "">("");
  const [priorityFilter, setPriorityFilter] = useState<TaskPriority | "">("");
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

  useEffect(() => {
    const timeoutId = window.setTimeout(() => {
      setNormalizedQuery(searchQuery.trim().toLowerCase());
    }, 150);
    return () => window.clearTimeout(timeoutId);
  }, [searchQuery]);

  const projectNames = useMemo(
    () => new Map(projects.map((project) => [project.id, project.name])),
    [projects],
  );
  const projectFilterOptions = useMemo(() => {
    const options = new Map(projects.map((project) => [project.id, project.name]));
    for (const task of tasks) {
      if (task.projectId && !options.has(task.projectId)) {
        options.set(
          task.projectId,
          `Archived or unavailable project · ${task.projectId.slice(0, 8)}`,
        );
      }
    }
    return [...options]
      .map(([id, name]) => ({ id, name }))
      .sort(
        (left, right) =>
          left.name.toLowerCase().localeCompare(right.name.toLowerCase()) ||
          left.id.localeCompare(right.id),
      );
  }, [projects, tasks]);
  const hasActiveFilters = Boolean(projectFilter || statusFilter || priorityFilter);
  const hasSearchOrFilters = Boolean(normalizedQuery || hasActiveFilters);
  const matchingTasks = useMemo(() => {
    return tasks.filter(
      (task) =>
        (!normalizedQuery ||
          task.title.toLowerCase().includes(normalizedQuery) ||
          (task.description?.toLowerCase().includes(normalizedQuery) ?? false)) &&
        (!projectFilter ||
          (projectFilter === PERSONAL_INBOX_FILTER
            ? task.projectId === null
            : task.projectId === projectFilter)) &&
        (!statusFilter || task.status === statusFilter) &&
        (!priorityFilter || task.priority === priorityFilter),
    );
  }, [tasks, normalizedQuery, projectFilter, statusFilter, priorityFilter]);
  const sortedTasks = useMemo(
    () => sortTasks(matchingTasks, sortKey, sortDirection, projectNames),
    [matchingTasks, sortKey, sortDirection, projectNames],
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

  const clearFilters = () => {
    setProjectFilter("");
    setStatusFilter("");
    setPriorityFilter("");
  };
  const clearSearchAndFilters = () => {
    setSearchQuery("");
    clearFilters();
  };
  const noResultsTitle = normalizedQuery
    ? hasActiveFilters
      ? "No tasks match this search and filter combination"
      : `No tasks match “${searchQuery.trim()}”`
    : "No tasks match these filters";
  const noResultsActionLabel = normalizedQuery
    ? hasActiveFilters
      ? "Clear search and filters"
      : "Clear search"
    : "Clear filters";
  const clearNoResults = normalizedQuery ? clearSearchAndFilters : clearFilters;

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
            : hasSearchOrFilters
              ? `${sortedTasks.length} of ${tasks.length} active ${tasks.length === 1 ? "task" : "tasks"}`
              : `${tasks.length} active ${tasks.length === 1 ? "task" : "tasks"}`}
        </span>
        <div className="tasks-search">
          <label className="tasks-search__label" htmlFor="task-search">
            Search tasks
          </label>
          <input
            id="task-search"
            className="ms-input tasks-search__input"
            type="search"
            value={searchQuery}
            placeholder="Title or description"
            disabled={loading || Boolean(loadError)}
            onChange={(event) => setSearchQuery(event.target.value)}
          />
          {searchQuery ? (
            <Button
              variant="ghost"
              size="sm"
              aria-label="Clear task search"
              onClick={() => setSearchQuery("")}
            >
              Clear
            </Button>
          ) : null}
          <fieldset className="tasks-filters">
            <legend className="visually-hidden">Filter tasks</legend>
            <label className="tasks-filter">
              <span>Project</span>
              <select
                className="ms-input"
                aria-label="Filter by project"
                value={projectFilter}
                onChange={(event) => setProjectFilter(event.target.value)}
                disabled={loading || Boolean(loadError)}
              >
                <option value="">All projects</option>
                <option value={PERSONAL_INBOX_FILTER}>Personal inbox</option>
                {projectFilterOptions.map((option) => (
                  <option key={option.id} value={option.id}>
                    {option.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="tasks-filter">
              <span>Status</span>
              <select
                className="ms-input"
                aria-label="Filter by status"
                value={statusFilter}
                onChange={(event) => setStatusFilter(event.target.value as TaskStatus | "")}
                disabled={loading || Boolean(loadError)}
              >
                <option value="">All statuses</option>
                {TASK_STATUSES.map((status) => (
                  <option key={status} value={status}>
                    {TASK_STATUS_LABELS[status]}
                  </option>
                ))}
              </select>
            </label>
            <label className="tasks-filter">
              <span>Priority</span>
              <select
                className="ms-input"
                aria-label="Filter by priority"
                value={priorityFilter}
                onChange={(event) => setPriorityFilter(event.target.value as TaskPriority | "")}
                disabled={loading || Boolean(loadError)}
              >
                <option value="">All priorities</option>
                {TASK_PRIORITIES.map((priority) => (
                  <option key={priority} value={priority}>
                    {TASK_PRIORITY_LABELS[priority]}
                  </option>
                ))}
              </select>
            </label>
            {hasActiveFilters ? (
              <Button
                variant="ghost"
                size="sm"
                aria-label="Clear all task filters"
                onClick={clearFilters}
              >
                Clear filters
              </Button>
            ) : null}
          </fieldset>
        </div>
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
      {!loading && !loadError && tasks.length === 0 && !hasSearchOrFilters ? (
        <EmptyState
          title="No active tasks yet"
          description="Create a task in your Personal inbox or attach it to an active project."
          action={<Button onClick={onCreateTask}>Create your first task</Button>}
        />
      ) : null}
      {!loading && !loadError && sortedTasks.length === 0 && hasSearchOrFilters ? (
        <EmptyState
          title={noResultsTitle}
          description="Adjust the search or filters, or clear them to see all active tasks."
          action={
            <Button variant="secondary" onClick={clearNoResults}>
              {noResultsActionLabel}
            </Button>
          }
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
          {sortedTasks.length > 0 && view === "list" ? (
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
                    const overdue = isTaskOverdue(task.dueAt, task.status);
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
                        <td>
                          <span>{formatTaskDate(task.dueAt)}</span>
                          {overdue ? <Badge variant="danger">Overdue</Badge> : null}
                        </td>
                        <td>{formatUpdatedAt(task.updatedAt)}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          ) : sortedTasks.length > 0 ? (
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
                            const overdue = isTaskOverdue(task.dueAt, task.status);
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
                                    <dd>
                                      <Badge
                                        variant={task.priority === "p0" ? "danger" : "neutral"}
                                      >
                                        {TASK_PRIORITY_LABELS[task.priority]}
                                      </Badge>
                                    </dd>
                                  </div>
                                  <div>
                                    <dt>Due</dt>
                                    <dd>
                                      {formatTaskDate(task.dueAt)}
                                      {overdue ? <Badge variant="danger">Overdue</Badge> : null}
                                    </dd>
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
          ) : null}
        </>
      ) : null}
    </section>
  );
}
