import { type FormEvent, useCallback, useEffect, useState } from "react";
import {
  associateMilestoneTask,
  createMilestone,
  getProjectOverview,
  listMilestones,
  type MilestoneRecord,
  type ProjectOverviewRecord,
  type ProjectStatus,
  toProjectErrorMessage,
  updateMilestone,
} from "./projects-api";
import { listTasks, type TaskRecord } from "./tasks-api";
import { Badge, Button, EmptyState, ErrorState, LoadingState } from "./ui-primitives";

const STATUS_LABELS: Record<ProjectStatus, string> = {
  planned: "Planned",
  active: "Active",
  paused: "Paused",
  completed: "Completed",
};

function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Date unavailable";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
    date,
  );
}

export function ProjectOverviewPage({
  projectId,
  onCreateTask,
}: {
  projectId: string;
  onCreateTask: (projectId: string) => void;
}) {
  const [overview, setOverview] = useState<ProjectOverviewRecord | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [milestones, setMilestones] = useState<MilestoneRecord[]>([]);
  const [tasks, setTasks] = useState<TaskRecord[]>([]);
  const [milestoneLoadError, setMilestoneLoadError] = useState(false);
  const [taskLoadError, setTaskLoadError] = useState(false);
  const [milestoneTitle, setMilestoneTitle] = useState("");
  const [milestoneDueDate, setMilestoneDueDate] = useState("");
  const [milestoneBusy, setMilestoneBusy] = useState(false);
  const [milestoneMessage, setMilestoneMessage] = useState<string | null>(null);
  const [milestoneError, setMilestoneError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const [overviewResult, milestoneResult, taskResult] = await Promise.allSettled([
        getProjectOverview(projectId),
        listMilestones(projectId),
        listTasks(),
      ]);
      if (overviewResult.status === "rejected") throw overviewResult.reason;
      setOverview(overviewResult.value);
      setMilestoneLoadError(milestoneResult.status === "rejected");
      setMilestones(milestoneResult.status === "fulfilled" ? milestoneResult.value : []);
      setTaskLoadError(taskResult.status === "rejected");
      setTasks(taskResult.status === "fulfilled" ? taskResult.value : []);
    } catch (error) {
      setOverview(null);
      setLoadError(toProjectErrorMessage(error, "load"));
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  const saveMilestone = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!milestoneTitle.trim()) return;
    setMilestoneBusy(true);
    setMilestoneError(null);
    setMilestoneMessage(null);
    try {
      const saved = await createMilestone({
        projectId,
        title: milestoneTitle.trim(),
        description: null,
        dueDate: milestoneDueDate || null,
      });
      setMilestones((current) => [...current, saved]);
      setMilestoneTitle("");
      setMilestoneDueDate("");
      setMilestoneMessage("Milestone saved on this device.");
    } catch (error) {
      setMilestoneError(toProjectErrorMessage(error, "save"));
    } finally {
      setMilestoneBusy(false);
    }
  };

  const linkTask = async (milestone: MilestoneRecord, taskId: string) => {
    if (!taskId) return;
    setMilestoneBusy(true);
    setMilestoneError(null);
    setMilestoneMessage(null);
    try {
      const saved = await associateMilestoneTask({ projectId, milestoneId: milestone.id, taskId });
      setMilestones((current) => current.map((item) => (item.id === saved.id ? saved : item)));
      setMilestoneMessage("Task associated with milestone. Task status was not changed.");
    } catch (error) {
      setMilestoneError(toProjectErrorMessage(error, "save"));
    } finally {
      setMilestoneBusy(false);
    }
  };

  const changeMilestoneStatus = async (
    milestone: MilestoneRecord,
    status: MilestoneRecord["status"],
  ) => {
    setMilestoneBusy(true);
    setMilestoneError(null);
    setMilestoneMessage(null);
    try {
      const saved = await updateMilestone({ ...milestone, status });
      setMilestones((current) => current.map((item) => (item.id === saved.id ? saved : item)));
      setMilestoneMessage("Milestone status saved. Associated task statuses were not changed.");
    } catch (error) {
      setMilestoneError(toProjectErrorMessage(error, "save"));
    } finally {
      setMilestoneBusy(false);
    }
  };

  useEffect(() => {
    void reload();
  }, [reload]);

  return (
    <section className="project-overview" aria-labelledby="project-overview-title">
      <header className="project-overview__header">
        <div>
          <a className="project-overview__back" href="#/projects">
            ← Projects
          </a>
          <p className="eyebrow">WORKSPACE / PROJECT OVERVIEW</p>
          <h2 id="project-overview-title">{overview?.project.name ?? "Project overview"}</h2>
          <p className="muted">Project details and activity saved on this device.</p>
        </div>
        {!loading && !loadError && overview ? (
          <Button onClick={() => onCreateTask(overview.project.id)}>New task</Button>
        ) : null}
      </header>

      <div className="project-overview__toolbar">
        <span className="muted" aria-live="polite">
          {loading
            ? "Loading project overview…"
            : overview
              ? `Updated ${formatDate(overview.project.updatedAt)}`
              : ""}
        </span>
        <Button variant="secondary" size="sm" onClick={() => void reload()} disabled={loading}>
          Refresh
        </Button>
      </div>

      {loading ? <LoadingState label="Loading project details and activity" /> : null}
      {!loading && loadError ? (
        <ErrorState
          title="Project overview is unavailable"
          description={loadError}
          onRetry={() => void reload()}
        />
      ) : null}
      {!loading && !loadError && overview ? (
        <>
          <div className="project-overview__summary">
            <article className="project-overview__card">
              <div className="project-overview__card-heading">
                <h3>Project</h3>
                <Badge variant={overview.project.status === "completed" ? "success" : "neutral"}>
                  {STATUS_LABELS[overview.project.status]}
                </Badge>
              </div>
              <dl className="project-overview__details">
                <div>
                  <dt>Goal</dt>
                  <dd>{overview.project.goal || <span className="muted">No goal set</span>}</dd>
                </div>
                <div>
                  <dt>Description</dt>
                  <dd>
                    {overview.project.description || <span className="muted">No description</span>}
                  </dd>
                </div>
              </dl>
            </article>
            <article className="project-overview__card" aria-labelledby="project-task-count-title">
              <div className="project-overview__card-heading">
                <h3 id="project-task-count-title">Tasks</h3>
                <span className="project-overview__count">{overview.activeTaskCount}</span>
              </div>
              {overview.activeTaskCount === 0 ? (
                <p className="muted">No non-archived tasks are linked to this project yet.</p>
              ) : (
                <p className="muted">
                  {overview.activeTaskCount} non-archived{" "}
                  {overview.activeTaskCount === 1 ? "task" : "tasks"} linked to this project.
                </p>
              )}
              <Button
                variant="secondary"
                size="sm"
                onClick={() => onCreateTask(overview.project.id)}
              >
                Create task for this project
              </Button>
            </article>
          </div>

          <section className="project-overview__activity" aria-labelledby="project-activity-title">
            <div className="project-overview__activity-heading">
              <div>
                <p className="eyebrow">RECENT CHANGES</p>
                <h3 id="project-activity-title">Activity</h3>
              </div>
              <span className="muted">Most recent first</span>
            </div>
            {overview.recentActivity.length === 0 ? (
              <EmptyState
                title="No activity recorded"
                description="Changes to this project and its tasks will appear here when they are saved."
              />
            ) : (
              <ol className="project-activity-list">
                {overview.recentActivity.map((event) => (
                  <li key={event.id}>
                    <Badge>{event.entityType === "project" ? "Project" : "Task"}</Badge>
                    <div className="project-activity-list__copy">
                      <strong>{event.summary}</strong>
                      <span>{formatDate(event.occurredAt)}</span>
                    </div>
                  </li>
                ))}
              </ol>
            )}
          </section>

          <section className="project-milestones" aria-labelledby="project-milestones-title">
            <header>
              <div>
                <p className="eyebrow">PROJECT PLAN</p>
                <h3 id="project-milestones-title">Milestones</h3>
              </div>
              <span className="muted">Grouping tasks never changes their status.</span>
            </header>
            {milestoneLoadError ? (
              <ErrorState
                title="Milestones are unavailable"
                description="Milestones could not be loaded."
                onRetry={() => void reload()}
              />
            ) : milestones.length === 0 ? (
              <EmptyState
                title="No milestones yet"
                description="Add a milestone to group this project's tasks by outcome or due date."
              />
            ) : (
              <ul className="project-milestone-list">
                {milestones.map((milestone) => {
                  const availableTasks = tasks.filter(
                    (task) =>
                      task.projectId === projectId &&
                      !milestone.tasks.some((linked) => linked.id === task.id),
                  );
                  return (
                    <li key={milestone.id}>
                      <div className="project-milestone-list__heading">
                        <strong>{milestone.title}</strong>
                        <label className="ms-field">
                          <span className="ms-label">Milestone status</span>
                          <select
                            className="ms-input"
                            value={milestone.status}
                            disabled={milestoneBusy}
                            onChange={(event) =>
                              void changeMilestoneStatus(
                                milestone,
                                event.target.value as MilestoneRecord["status"],
                              )
                            }
                          >
                            <option value="planned">Planned</option>
                            <option value="active">Active</option>
                            <option value="completed">Completed</option>
                          </select>
                        </label>
                      </div>
                      <span className="muted">
                        {milestone.dueDate ? `Due ${milestone.dueDate}` : "No due date"}
                      </span>
                      {milestone.tasks.length ? (
                        <ul>
                          {milestone.tasks.map((task) => (
                            <li key={task.id}>
                              {task.title} · {task.status.replaceAll("_", " ")}
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p className="muted">No tasks associated.</p>
                      )}
                      <label className="ms-field">
                        <span className="ms-label">Associate project task</span>
                        <select
                          className="ms-input"
                          value=""
                          disabled={milestoneBusy || availableTasks.length === 0}
                          onChange={(event) => void linkTask(milestone, event.target.value)}
                        >
                          <option value="">
                            {availableTasks.length
                              ? "Choose a task…"
                              : "No unassociated project tasks"}
                          </option>
                          {availableTasks.map((task) => (
                            <option key={task.id} value={task.id}>
                              {task.title}
                            </option>
                          ))}
                        </select>
                      </label>
                    </li>
                  );
                })}
              </ul>
            )}
            {taskLoadError ? (
              <p className="task-detail-form__error" role="status">
                Project tasks could not be loaded, so task associations may be unavailable. Use
                Refresh to try again.
              </p>
            ) : null}
            <form
              className="project-milestone-form"
              onSubmit={(event) => void saveMilestone(event)}
            >
              <label className="ms-field">
                <span className="ms-label">Milestone title</span>
                <input
                  className="ms-input"
                  value={milestoneTitle}
                  onChange={(event) => setMilestoneTitle(event.target.value)}
                  maxLength={160}
                  required
                  disabled={milestoneBusy}
                />
              </label>
              <label className="ms-field">
                <span className="ms-label">Due date</span>
                <input
                  className="ms-input"
                  type="date"
                  value={milestoneDueDate}
                  onChange={(event) => setMilestoneDueDate(event.target.value)}
                  disabled={milestoneBusy}
                />
              </label>
              <Button
                type="submit"
                loading={milestoneBusy}
                loadingLabel="Saving…"
                disabled={!milestoneTitle.trim()}
              >
                Add milestone
              </Button>
            </form>
            {milestoneError ? (
              <p className="task-detail-form__error" role="alert">
                {milestoneError}
              </p>
            ) : null}
            {milestoneMessage ? (
              <p className="muted" role="status">
                {milestoneMessage}
              </p>
            ) : null}
          </section>
        </>
      ) : null}
    </section>
  );
}
