import { useCallback, useEffect, useState } from "react";
import {
  getProjectOverview,
  toProjectErrorMessage,
  type ProjectOverviewRecord,
  type ProjectStatus,
} from "./projects-api";
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

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      setOverview(await getProjectOverview(projectId));
    } catch (error) {
      setOverview(null);
      setLoadError(toProjectErrorMessage(error, "load"));
    } finally {
      setLoading(false);
    }
  }, [projectId]);

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
        </>
      ) : null}
    </section>
  );
}
