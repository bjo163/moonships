import { useCallback, useEffect, useState, type FormEvent } from "react";
import {
  createTask,
  toTaskErrorMessage,
  type TaskPriority,
  type TaskRecord,
  type TaskStatus,
} from "./tasks-api";
import { TASK_PRIORITY_LABELS, TASK_STATUS_LABELS } from "./task-presentation";
import { listProjects, type ProjectRecord } from "./projects-api";
import { Badge, Button, Dialog, Input } from "./ui-primitives";

export function TaskCreateDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [title, setTitle] = useState("");
  const [projectId, setProjectId] = useState("");
  const [status, setStatus] = useState<TaskStatus>("todo");
  const [priority, setPriority] = useState<TaskPriority>("p2");
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [projectsLoading, setProjectsLoading] = useState(false);
  const [projectsError, setProjectsError] = useState(false);
  const [titleError, setTitleError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [createdTask, setCreatedTask] = useState<TaskRecord | null>(null);

  const loadProjects = useCallback(async () => {
    setProjectsLoading(true);
    setProjectsError(false);
    try {
      setProjects(await listProjects());
    } catch {
      setProjects([]);
      setProjectsError(true);
    } finally {
      setProjectsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!open) return;
    setTitle("");
    setProjectId("");
    setStatus("todo");
    setPriority("p2");
    setTitleError(null);
    setSaveError(null);
    setCreatedTask(null);
  }, [open]);

  useEffect(() => {
    if (open) void loadProjects();
  }, [open, loadProjects]);

  const close = () => {
    if (saving) return;
    onClose();
  };

  const save = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (saving || createdTask) return;
    if (!title.trim()) {
      setTitleError("Enter a task title.");
      document.getElementById("task-title")?.focus();
      return;
    }

    setTitleError(null);
    setSaveError(null);
    setSaving(true);
    try {
      const created = await createTask({
        title,
        projectId: projectId || null,
        status,
        priority,
      });
      setCreatedTask(created);
    } catch (error) {
      setSaveError(toTaskErrorMessage(error));
    } finally {
      setSaving(false);
    }
  };

  const selectedProject = projects.find((project) => project.id === createdTask?.projectId);

  return (
    <Dialog
      open={open}
      title={createdTask ? "Task created" : "Create task"}
      description={
        createdTask
          ? "The task was saved to your local workspace."
          : "Create a task in your Personal inbox or attach it to an active project."
      }
      onClose={close}
    >
      {createdTask ? (
        <section className="task-create-success" aria-live="polite">
          <Badge variant="success">Saved</Badge>
          <h3>{createdTask.title}</h3>
          <p>
            {selectedProject ? `Project: ${selectedProject.name}` : "Personal inbox"} ·{" "}
            {TASK_STATUS_LABELS[createdTask.status]} · {TASK_PRIORITY_LABELS[createdTask.priority]}
          </p>
          <div className="task-create-form__actions">
            <Button onClick={close}>Done</Button>
          </div>
        </section>
      ) : (
        <form className="task-create-form" onSubmit={(event) => void save(event)}>
          <Input
            id="task-title"
            label="Task title"
            value={title}
            onChange={(event) => {
              setTitle(event.target.value);
              if (event.target.value.trim()) setTitleError(null);
            }}
            required
            maxLength={200}
            autoComplete="off"
            error={titleError ?? undefined}
            disabled={saving}
          />
          <label className="ms-field" htmlFor="task-project">
            <span className="ms-label">Project</span>
            <select
              id="task-project"
              className="ms-input"
              value={projectId}
              onChange={(event) => setProjectId(event.target.value)}
              disabled={saving || projectsLoading || projectsError}
            >
              <option value="">No project · Personal inbox</option>
              {projects.map((project) => (
                <option key={project.id} value={project.id}>
                  {project.name}
                </option>
              ))}
            </select>
            {projectsLoading ? (
              <span className="ms-hint" role="status">
                Loading active projects…
              </span>
            ) : null}
            {projectsError ? (
              <span className="task-create-project-error" role="status">
                Projects could not be loaded. You can still save to the Personal inbox.
                <Button size="sm" variant="secondary" onClick={() => void loadProjects()}>
                  Retry project list
                </Button>
              </span>
            ) : null}
          </label>
          <div className="task-create-form__row">
            <label className="ms-field" htmlFor="task-status">
              <span className="ms-label">Status</span>
              <select
                id="task-status"
                className="ms-input"
                value={status}
                onChange={(event) => setStatus(event.target.value as TaskStatus)}
                disabled={saving}
              >
                {Object.entries(TASK_STATUS_LABELS).map(([value, label]) => (
                  <option key={value} value={value}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
            <label className="ms-field" htmlFor="task-priority">
              <span className="ms-label">Priority</span>
              <select
                id="task-priority"
                className="ms-input"
                value={priority}
                onChange={(event) => setPriority(event.target.value as TaskPriority)}
                disabled={saving}
              >
                {Object.entries(TASK_PRIORITY_LABELS).map(([value, label]) => (
                  <option key={value} value={value}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
          </div>
          {saveError ? (
            <p className="ms-error" role="alert">
              {saveError}
            </p>
          ) : null}
          <div className="task-create-form__actions">
            <Button variant="secondary" onClick={close} disabled={saving}>
              Cancel
            </Button>
            <Button type="submit" loading={saving} loadingLabel="Saving task…">
              Create task
            </Button>
          </div>
        </form>
      )}
    </Dialog>
  );
}
