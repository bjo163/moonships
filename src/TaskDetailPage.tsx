import { useCallback, useEffect, useState, type FormEvent } from "react";
import { listProjects, type ProjectRecord } from "./projects-api";
import { formatTaskDate, TASK_PRIORITY_LABELS, TASK_STATUS_LABELS } from "./task-presentation";
import {
  getTask,
  toTaskErrorMessage,
  updateTask,
  type TaskPriority,
  type TaskRecord,
  type TaskStatus,
} from "./tasks-api";
import { Button, ErrorState, Input, LoadingState } from "./ui-primitives";

const STATUSES = Object.keys(TASK_STATUS_LABELS) as TaskStatus[];
const PRIORITIES = Object.keys(TASK_PRIORITY_LABELS) as TaskPriority[];

function toLocalInputValue(value: string | null): string {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  const local = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}

function localInputToUtc(value: string): string | null {
  if (!value) return null;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date.toISOString();
}

export function TaskDetailPage({ taskId }: { taskId: string }) {
  const [task, setTask] = useState<TaskRecord | null>(null);
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [projectLoadError, setProjectLoadError] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [dueAt, setDueAt] = useState("");
  const [status, setStatus] = useState<TaskStatus>("todo");
  const [priority, setPriority] = useState<TaskPriority>("p2");
  const [titleError, setTitleError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    const [taskResult, projectResult] = await Promise.allSettled([getTask(taskId), listProjects()]);
    if (taskResult.status === "rejected") {
      setTask(null);
      setLoadError(toTaskErrorMessage(taskResult.reason, "load"));
    } else {
      const savedTask = taskResult.value;
      setTask(savedTask);
      setTitle(savedTask.title);
      setDescription(savedTask.description ?? "");
      setDueAt(toLocalInputValue(savedTask.dueAt));
      setStatus(savedTask.status);
      setPriority(savedTask.priority);
      setProjectLoadError(projectResult.status === "rejected");
      setProjects(projectResult.status === "fulfilled" ? projectResult.value : []);
    }
    setLoading(false);
  }, [taskId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const projectName = task?.projectId
    ? (projects.find((project) => project.id === task.projectId)?.name ??
      (projectLoadError ? "Project name unavailable" : "Archived or unavailable project"))
    : "Personal inbox";

  const save = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const trimmedTitle = title.trim();
    if (!trimmedTitle) {
      setTitleError("Enter a task title.");
      setSaveMessage(null);
      return;
    }
    if (!task) return;
    setTitleError(null);
    setSaving(true);
    setSaveError(null);
    setSaveMessage(null);
    try {
      const saved = await updateTask({
        id: task.id,
        projectId: task.projectId,
        title: trimmedTitle,
        description: description.trim() || null,
        dueAt: localInputToUtc(dueAt),
        status,
        priority,
      });
      setTask(saved);
      setTitle(saved.title);
      setDescription(saved.description ?? "");
      setDueAt(toLocalInputValue(saved.dueAt));
      setSaveMessage("Task saved on this device.");
    } catch (error) {
      setSaveError(toTaskErrorMessage(error, "update"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="task-detail-page" aria-labelledby="task-detail-title">
      <a className="task-detail-page__back" href="#/tasks">
        ← Back to Tasks
      </a>
      {loading ? <LoadingState label="Loading task from this device" /> : null}
      {!loading && loadError ? (
        <ErrorState
          title="Task is unavailable"
          description={loadError}
          onRetry={() => void reload()}
        />
      ) : null}
      {!loading && !loadError && task ? (
        <>
          <header className="task-detail-page__header">
            <div>
              <p className="eyebrow">WORKSPACE / TASKS / DETAIL</p>
              <h2 id="task-detail-title">Task details</h2>
              <p className="muted">
                Saved in {projectName} · Updated {formatTaskDate(task.updatedAt)}
              </p>
              {projectLoadError ? (
                <p className="muted" role="status">
                  Project names could not be loaded. Task details remain available.
                </p>
              ) : null}
            </div>
          </header>
          <form className="task-detail-form" onSubmit={(event) => void save(event)}>
            <Input
              label="Title"
              value={title}
              onChange={(event) => {
                setTitle(event.target.value);
                if (event.target.value.trim()) setTitleError(null);
              }}
              required
              disabled={saving}
              maxLength={200}
              autoComplete="off"
              error={titleError ?? undefined}
            />
            <div className="task-detail-form__field">
              <label className="ms-label" htmlFor="task-description">
                Description
              </label>
              <textarea
                id="task-description"
                className="ms-input task-detail-form__description"
                value={description}
                disabled={saving}
                onChange={(event) => setDescription(event.target.value)}
                maxLength={10_000}
              />
            </div>
            <div className="task-detail-form__grid">
              <div className="task-detail-form__field">
                <label className="ms-label" htmlFor="task-status">
                  Status
                </label>
                <select
                  id="task-status"
                  className="ms-input"
                  value={status}
                  disabled={saving}
                  onChange={(event) => setStatus(event.target.value as TaskStatus)}
                >
                  {STATUSES.map((value) => (
                    <option key={value} value={value}>
                      {TASK_STATUS_LABELS[value]}
                    </option>
                  ))}
                </select>
              </div>
              <div className="task-detail-form__field">
                <label className="ms-label" htmlFor="task-priority">
                  Priority
                </label>
                <select
                  id="task-priority"
                  className="ms-input"
                  value={priority}
                  disabled={saving}
                  onChange={(event) => setPriority(event.target.value as TaskPriority)}
                >
                  {PRIORITIES.map((value) => (
                    <option key={value} value={value}>
                      {TASK_PRIORITY_LABELS[value]}
                    </option>
                  ))}
                </select>
              </div>
            </div>
            <Input
              label="Due date and time"
              type="datetime-local"
              value={dueAt}
              disabled={saving}
              onChange={(event) => setDueAt(event.target.value)}
              hint={
                task.dueAt
                  ? `Currently saved as ${formatTaskDate(task.dueAt)} in your local timezone.`
                  : "Optional. Stored as a UTC instant."
              }
            />
            <div className="task-detail-form__footer">
              <span aria-live="polite" className={saveError ? "task-detail-form__error" : "muted"}>
                {saveError ?? saveMessage ?? "Changes are saved only when you select Save changes."}
              </span>
              <Button type="submit" loading={saving} loadingLabel="Saving…">
                Save changes
              </Button>
            </div>
          </form>
        </>
      ) : null}
    </section>
  );
}
