import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { Button } from "./ui-primitives";

type JobRecord = {
  id: string;
  title: string;
  status:
    | "queued"
    | "running"
    | "waiting_approval"
    | "succeeded"
    | "partial"
    | "failed"
    | "cancelled"
    | "timed_out";
  createdAt: string;
  startedAt: string | null;
  finishedAt: string | null;
  stage: string | null;
  progressPercent: number | null;
  resultSummary: string | null;
};

const statusLabels: Record<JobRecord["status"], string> = {
  queued: "Queued",
  running: "Running",
  waiting_approval: "Waiting for approval",
  succeeded: "Succeeded",
  partial: "Partial",
  failed: "Failed",
  cancelled: "Cancelled",
  timed_out: "Timed out",
};

function dateLabel(value: string | null): string {
  if (!value) return "—";
  const date = new Date(value);
  return Number.isNaN(date.valueOf())
    ? "Unknown"
    : new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(date);
}

function durationLabel(job: JobRecord): string {
  const start = job.startedAt ? Date.parse(job.startedAt) : Date.parse(job.createdAt);
  const end = job.finishedAt ? Date.parse(job.finishedAt) : Date.now();
  if (!Number.isFinite(start) || !Number.isFinite(end) || end < start) return "Unknown";
  const seconds = Math.floor((end - start) / 1000);
  return seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}

export function ActivityCenterPage() {
  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setJobs(await invoke<JobRecord[]>("list_jobs"));
    } catch {
      setError("Activity history could not be loaded from local storage.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <section className="tasks-page" aria-labelledby="activity-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">OPERATIONS</p>
          <h2 id="activity-title">Activity Center</h2>
          <p className="muted">
            Long-running job status, progress and results saved on this device.
          </p>
        </div>
      </header>
      {loading ? (
        <p role="status">Loading activity history…</p>
      ) : error ? (
        <div className="task-detail-error" role="alert">
          <p>{error}</p>
          <Button variant="secondary" onClick={() => void load()}>
            Retry
          </Button>
        </div>
      ) : jobs.length === 0 ? (
        <div className="activity-state">
          <h3>No job activity yet</h3>
          <p>
            When a supported long-running job is connected, its status and result will appear here.
          </p>
        </div>
      ) : (
        <div className="projects-table-wrap activity-table-wrap">
          <table className="projects-table activity-table">
            <caption>
              Latest {jobs.length} saved jobs. Durations use recorded start and finish times.
            </caption>
            <thead>
              <tr>
                <th scope="col">Job</th>
                <th scope="col">Status</th>
                <th scope="col">Progress</th>
                <th scope="col">Duration</th>
                <th scope="col">Result</th>
              </tr>
            </thead>
            <tbody>
              {jobs.map((job) => (
                <tr key={job.id}>
                  <th scope="row">
                    <span className="projects-table__name">{job.title}</span>
                    <span className="projects-table__description">
                      {job.stage || `Created ${dateLabel(job.createdAt)}`}
                    </span>
                  </th>
                  <td>{statusLabels[job.status]}</td>
                  <td>
                    {job.progressPercent === null ? (
                      "Not reported"
                    ) : (
                      <>
                        <progress
                          aria-label={`${job.title} progress`}
                          max={100}
                          value={job.progressPercent}
                        />{" "}
                        <span>{job.progressPercent}%</span>
                      </>
                    )}
                  </td>
                  <td>{durationLabel(job)}</td>
                  <td>
                    {job.resultSummary || (job.finishedAt ? "No result summary" : "In progress")}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
