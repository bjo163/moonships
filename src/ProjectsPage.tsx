import { useCallback, useEffect, useState, type FormEvent } from "react";
import {
  createProject,
  listProjects,
  toProjectErrorMessage,
  updateProject,
  type ProjectInput,
  type ProjectRecord,
  type ProjectStatus,
} from "./projects-api";
import {
  Badge,
  Button,
  Dialog,
  EmptyState,
  ErrorState,
  Input,
  LoadingState,
} from "./ui-primitives";
import { ProjectTaskExportButton } from "./ProjectTaskExportButton";

type ProjectFormState = {
  name: string;
  description: string;
  goal: string;
  status: ProjectStatus;
};

type ProjectDialogState = { mode: "create" } | { mode: "edit"; project: ProjectRecord };

const EMPTY_FORM: ProjectFormState = { name: "", description: "", goal: "", status: "planned" };
const STATUS_LABELS: Record<ProjectStatus, string> = {
  planned: "Planned",
  active: "Active",
  paused: "Paused",
  completed: "Completed",
};

function toForm(project?: ProjectRecord): ProjectFormState {
  return project
    ? {
        name: project.name,
        description: project.description ?? "",
        goal: project.goal ?? "",
        status: project.status,
      }
    : EMPTY_FORM;
}

function formatUpdatedAt(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Unknown";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
    date,
  );
}

function nullableText(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length ? trimmed : null;
}

export function ProjectsPage({
  createOnOpen = false,
  onCreateHandled,
  onOpenProject,
}: {
  createOnOpen?: boolean;
  onCreateHandled?: () => void;
  onOpenProject?: (projectId: string) => void;
}) {
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [dialog, setDialog] = useState<ProjectDialogState | null>(null);
  const [form, setForm] = useState<ProjectFormState>(EMPTY_FORM);
  const [fieldError, setFieldError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      setProjects(await listProjects());
    } catch (error) {
      setLoadError(toProjectErrorMessage(error, "load"));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  useEffect(() => {
    if (!createOnOpen) return;
    setForm(EMPTY_FORM);
    setFieldError(null);
    setSaveError(null);
    setDialog({ mode: "create" });
    onCreateHandled?.();
  }, [createOnOpen, onCreateHandled]);

  const closeDialog = () => {
    if (saving) return;
    setDialog(null);
    setFieldError(null);
    setSaveError(null);
  };

  const startCreate = () => {
    setForm(EMPTY_FORM);
    setFieldError(null);
    setSaveError(null);
    setDialog({ mode: "create" });
  };

  const startEdit = (project: ProjectRecord) => {
    setForm(toForm(project));
    setFieldError(null);
    setSaveError(null);
    setDialog({ mode: "edit", project });
  };

  const save = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (saving || !dialog) return;
    if (!form.name.trim()) {
      setFieldError("Enter a project name.");
      document.getElementById("project-name")?.focus();
      return;
    }

    setFieldError(null);
    setSaveError(null);
    setSaving(true);
    try {
      const input: ProjectInput = {
        name: form.name,
        description: nullableText(form.description),
        goal: nullableText(form.goal),
      };
      if (dialog.mode === "create") {
        await createProject(input);
      } else {
        await updateProject({ ...input, id: dialog.project.id, status: form.status });
      }
      setDialog(null);
      await reload();
    } catch (error) {
      setSaveError(toProjectErrorMessage(error, "save"));
    } finally {
      setSaving(false);
    }
  };

  const editingProject = dialog?.mode === "edit" ? dialog.project : null;

  return (
    <section className="projects-page" aria-labelledby="projects-title">
      <header className="projects-page__header">
        <div>
          <p className="eyebrow">WORKSPACE / PROJECTS</p>
          <h2 id="projects-title">Projects</h2>
          <p className="muted">Projects are stored on this device in your Personal workspace.</p>
        </div>
        <div className="projects-page__header-actions">
          <ProjectTaskExportButton />
          <Button onClick={startCreate}>New project</Button>
        </div>
      </header>

      <div className="projects-page__toolbar">
        <span className="muted" aria-live="polite">
          {loading
            ? "Loading projects…"
            : `${projects.length} active ${projects.length === 1 ? "project" : "projects"}`}
        </span>
        <Button variant="secondary" size="sm" onClick={() => void reload()} disabled={loading}>
          Refresh
        </Button>
      </div>

      {loading ? <LoadingState label="Loading projects from this device" /> : null}
      {!loading && loadError ? (
        <ErrorState
          title="Projects are unavailable"
          description={loadError}
          onRetry={() => void reload()}
        />
      ) : null}
      {!loading && !loadError && projects.length === 0 ? (
        <EmptyState
          title="No active projects yet"
          description="Create a project to give related tasks a shared goal and place to live."
          action={<Button onClick={startCreate}>Create your first project</Button>}
        />
      ) : null}
      {!loading && !loadError && projects.length > 0 ? (
        <div className="projects-table-wrap">
          <table className="projects-table">
            <caption className="visually-hidden">Active projects in the Personal workspace</caption>
            <thead>
              <tr>
                <th scope="col">Project</th>
                <th scope="col">Status</th>
                <th scope="col">Goal</th>
                <th scope="col">Updated</th>
                <th scope="col">
                  <span className="visually-hidden">Actions</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {projects.map((project) => (
                <tr key={project.id}>
                  <th scope="row">
                    <span className="projects-table__name">{project.name}</span>
                    {project.description ? (
                      <span className="projects-table__description">{project.description}</span>
                    ) : null}
                  </th>
                  <td>
                    <Badge variant={project.status === "completed" ? "success" : "neutral"}>
                      {STATUS_LABELS[project.status]}
                    </Badge>
                  </td>
                  <td>{project.goal || <span className="muted">No goal set</span>}</td>
                  <td>{formatUpdatedAt(project.updatedAt)}</td>
                  <td>
                    <div className="projects-table__actions">
                      {onOpenProject ? (
                        <Button
                          variant="secondary"
                          size="sm"
                          onClick={() => onOpenProject(project.id)}
                        >
                          Open
                        </Button>
                      ) : null}
                      <Button variant="secondary" size="sm" onClick={() => startEdit(project)}>
                        Edit
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}

      <Dialog
        open={dialog !== null}
        title={editingProject ? "Edit project" : "Create project"}
        description="Project details are saved locally on this device."
        onClose={closeDialog}
      >
        <form className="project-form" onSubmit={(event) => void save(event)}>
          <Input
            id="project-name"
            label="Project name"
            value={form.name}
            onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
            required
            maxLength={120}
            autoComplete="off"
            error={fieldError ?? undefined}
            disabled={saving}
          />
          <label className="ms-field">
            <span className="ms-label">Goal</span>
            <textarea
              className="ms-input project-form__textarea"
              value={form.goal}
              onChange={(event) => setForm((current) => ({ ...current, goal: event.target.value }))}
              maxLength={4000}
              rows={3}
              disabled={saving}
            />
          </label>
          <label className="ms-field">
            <span className="ms-label">Description</span>
            <textarea
              className="ms-input project-form__textarea"
              value={form.description}
              onChange={(event) =>
                setForm((current) => ({ ...current, description: event.target.value }))
              }
              maxLength={10000}
              rows={4}
              disabled={saving}
            />
          </label>
          {editingProject ? (
            <label className="ms-field" htmlFor="project-status">
              <span className="ms-label">Status</span>
              <select
                id="project-status"
                className="ms-input"
                value={form.status}
                onChange={(event) =>
                  setForm((current) => ({
                    ...current,
                    status: event.target.value as ProjectStatus,
                  }))
                }
                disabled={saving}
              >
                {Object.entries(STATUS_LABELS).map(([value, label]) => (
                  <option key={value} value={value}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
          ) : null}
          {saveError ? (
            <p className="ms-error" role="alert">
              {saveError}
            </p>
          ) : null}
          <div className="project-form__actions">
            <Button variant="secondary" onClick={closeDialog} disabled={saving}>
              Cancel
            </Button>
            <Button type="submit" loading={saving} loadingLabel="Saving project…">
              {editingProject ? "Save changes" : "Create project"}
            </Button>
          </div>
        </form>
      </Dialog>
    </section>
  );
}
