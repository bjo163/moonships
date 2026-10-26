import { useCallback, useEffect, useRef, useState } from "react";
import { getHomeDashboard, type HomeDashboard } from "./home-dashboard-api";
import { Badge, Button, EmptyState, ErrorState, LoadingState } from "./ui-primitives";

function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Date unavailable";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(
    date,
  );
}

function TaskLinks({
  tasks,
  emptyText,
  onOpenTask,
}: {
  tasks: HomeDashboard["overdueTasks"];
  emptyText: string;
  onOpenTask: (id: string) => void;
}) {
  if (!tasks.length) return <p className="home-dashboard__empty-note">{emptyText}</p>;
  return (
    <ul className="home-dashboard__list">
      {tasks.map((task) => (
        <li key={task.id}>
          <button
            type="button"
            className="home-dashboard__item-link"
            onClick={() => onOpenTask(task.id)}
          >
            <span>
              <strong>{task.title}</strong>
              <small>{task.projectName ?? "Personal inbox"}</small>
            </span>
            <time dateTime={task.dueAt}>{formatDate(task.dueAt)}</time>
          </button>
        </li>
      ))}
    </ul>
  );
}

export function HomeDashboardPage({
  onCreateTask,
  onCreateProject,
  onOpenProjects,
  onOpenTasks,
  onOpenProject,
  onOpenTask,
}: {
  onCreateTask: () => void;
  onCreateProject: () => void;
  onOpenProjects: () => void;
  onOpenTasks: () => void;
  onOpenProject: (id: string) => void;
  onOpenTask: (id: string) => void;
}) {
  const [dashboard, setDashboard] = useState<HomeDashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const requestId = useRef(0);

  const reload = useCallback(async () => {
    const currentRequest = ++requestId.current;
    setLoading(true);
    setLoadError(false);
    try {
      const result = await getHomeDashboard();
      if (currentRequest === requestId.current) setDashboard(result);
    } catch {
      if (currentRequest === requestId.current) {
        setDashboard(null);
        setLoadError(true);
      }
    } finally {
      if (currentRequest === requestId.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
    return () => {
      requestId.current += 1;
    };
  }, [reload]);

  return (
    <section className="home-dashboard" aria-labelledby="home-dashboard-title">
      <header className="home-dashboard__header">
        <div>
          <p className="eyebrow">PERSONAL WORKSPACE / HOME</p>
          <h2 id="home-dashboard-title">Welcome back</h2>
          <p className="muted">Your projects, tasks and recorded activity on this device.</p>
        </div>
        <div className="home-dashboard__actions">
          <Button onClick={onCreateTask}>New task</Button>
          <Button variant="secondary" onClick={onOpenProjects}>
            Projects
          </Button>
          <Button variant="secondary" onClick={onOpenTasks}>
            Tasks
          </Button>
        </div>
      </header>

      {loading ? <LoadingState label="Loading your local work…" /> : null}
      {loadError ? (
        <ErrorState
          title="Home data could not be loaded"
          description="Projects, tasks and activity could not be read from this device. Try again."
          onRetry={reload}
        />
      ) : null}
      {!loading &&
      !loadError &&
      dashboard &&
      dashboard.activeProjectCount === 0 &&
      dashboard.activeTaskCount === 0 &&
      dashboard.recentActivity.length === 0 ? (
        <EmptyState
          title="Your workspace is ready"
          description="Create a Project to organize work, or add a Task to your Personal inbox. Home will show real saved records here."
          action={
            <div className="home-dashboard__actions">
              <Button variant="secondary" onClick={onCreateProject}>
                Create a project
              </Button>
              <Button onClick={onCreateTask}>Create a task</Button>
            </div>
          }
        />
      ) : null}
      {!loading && !loadError && dashboard ? (
        <>
          <section className="home-dashboard__work" aria-labelledby="home-work-title">
            <h3 id="home-work-title">My Work</h3>
            <div className="home-dashboard__counts">
              <article className="home-dashboard__count-card">
                <span>Open tasks</span>
                <strong>{dashboard.counts.openTasks}</strong>
              </article>
              <article className="home-dashboard__count-card home-dashboard__count-card--danger">
                <span>Overdue</span>
                <strong>{dashboard.counts.overdueTasks}</strong>
              </article>
              <article className="home-dashboard__count-card">
                <span>Due in 7 days</span>
                <strong>{dashboard.counts.dueSoonTasks}</strong>
              </article>
              <article className="home-dashboard__count-card">
                <span>Blocked</span>
                <strong>{dashboard.counts.blockedTasks}</strong>
              </article>
            </div>
          </section>

          <div className="home-dashboard__grid">
            <section className="home-dashboard__panel" aria-labelledby="home-projects-title">
              <div className="home-dashboard__panel-heading">
                <div>
                  <p className="eyebrow">RECENTLY UPDATED</p>
                  <h3 id="home-projects-title">Projects</h3>
                </div>
                <Button variant="ghost" size="sm" onClick={onOpenProjects}>
                  All projects
                </Button>
              </div>
              {dashboard.recentProjects.length ? (
                <ul className="home-dashboard__list">
                  {dashboard.recentProjects.map((project) => (
                    <li key={project.id}>
                      <button
                        type="button"
                        className="home-dashboard__item-link"
                        onClick={() => onOpenProject(project.id)}
                      >
                        <span>
                          <strong>{project.name}</strong>
                          <small>Updated {formatDate(project.updatedAt)}</small>
                        </span>
                        <Badge>{project.status}</Badge>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="home-dashboard__empty-note">No active Projects yet.</p>
              )}
            </section>

            <section className="home-dashboard__panel" aria-labelledby="home-overdue-title">
              <div className="home-dashboard__panel-heading">
                <div>
                  <p className="eyebrow">NEEDS ATTENTION</p>
                  <h3 id="home-overdue-title">Overdue tasks</h3>
                </div>
              </div>
              <TaskLinks
                tasks={dashboard.overdueTasks}
                emptyText="No overdue tasks."
                onOpenTask={onOpenTask}
              />
            </section>

            <section className="home-dashboard__panel" aria-labelledby="home-upcoming-title">
              <div className="home-dashboard__panel-heading">
                <div>
                  <p className="eyebrow">NEXT 7 DAYS</p>
                  <h3 id="home-upcoming-title">Upcoming tasks</h3>
                </div>
              </div>
              <TaskLinks
                tasks={dashboard.upcomingTasks}
                emptyText="No tasks due in the next 7 days."
                onOpenTask={onOpenTask}
              />
            </section>

            <section className="home-dashboard__panel" aria-labelledby="home-activity-title">
              <div className="home-dashboard__panel-heading">
                <div>
                  <p className="eyebrow">SAVED LOCALLY</p>
                  <h3 id="home-activity-title">Recent activity</h3>
                </div>
              </div>
              {dashboard.recentActivity.length ? (
                <ul className="home-dashboard__list">
                  {dashboard.recentActivity.map((event) => (
                    <li key={event.id}>
                      <button
                        type="button"
                        className="home-dashboard__activity-link"
                        onClick={() =>
                          event.entityType === "project"
                            ? onOpenProject(event.entityId)
                            : onOpenTask(event.entityId)
                        }
                      >
                        <span>
                          <strong>{event.summary}</strong>
                          <small>
                            {event.entityType === "project" ? "Project" : "Task"} · {event.action}
                          </small>
                        </span>
                        <time dateTime={event.occurredAt}>{formatDate(event.occurredAt)}</time>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="home-dashboard__empty-note">No activity has been recorded yet.</p>
              )}
            </section>
          </div>
          <p className="home-dashboard__freshness">
            Counts reflect saved Personal workspace records as of {formatDate(dashboard.asOf)}.
            Waiting approvals are not available yet.
          </p>
        </>
      ) : null}
    </section>
  );
}
