import { useCallback, useEffect, useState } from "react";
import { listProcessInventory, type ProcessInventorySnapshot } from "./process-inventory-api";
import { Button } from "./ui-primitives";

function formatObservedAt(value: number): string {
  const date = new Date(value);
  if (!Number.isFinite(value) || Number.isNaN(date.getTime())) return "Time unavailable";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "medium" }).format(
    date,
  );
}

function statusLabel(snapshot: ProcessInventorySnapshot): string {
  switch (snapshot.status) {
    case "available":
      return "Available";
    case "partial":
      return snapshot.errorCode === "resultLimitReached"
        ? "Partial · display limit reached"
        : "Partial · enumeration interrupted";
    case "permissionDenied":
      return "Permission denied";
    case "unsupported":
      return "Unsupported on this platform";
    case "unavailable":
      return "Unavailable";
  }
}

export function ProcessesPage() {
  const [snapshot, setSnapshot] = useState<ProcessInventorySnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(false);
    try {
      setSnapshot(await listProcessInventory());
    } catch {
      setSnapshot(null);
      setLoadError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  return (
    <section className="tasks-page" aria-labelledby="processes-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">SYSOPS / READ ONLY</p>
          <h2 id="processes-title">Processes</h2>
          <p className="muted">
            A one-time snapshot of process names and IDs. Moonships does not offer process controls
            here.
          </p>
        </div>
        <div className="tasks-page__header-actions">
          <Button variant="secondary" onClick={() => void reload()} disabled={loading}>
            {loading ? "Refreshing…" : "Refresh snapshot"}
          </Button>
        </div>
      </header>

      {loading && !snapshot ? (
        <p role="status" aria-live="polite">
          Loading process snapshot…
        </p>
      ) : loadError ? (
        <div className="task-detail-error" role="alert">
          <p>The process snapshot could not be loaded.</p>
          <Button variant="secondary" onClick={() => void reload()}>
            Retry
          </Button>
        </div>
      ) : snapshot ? (
        <>
          <div className="tasks-page__toolbar">
            <span role="status" aria-live="polite">
              {statusLabel(snapshot)} · {snapshot.processes.length}{" "}
              {snapshot.processes.length === 1 ? "process" : "processes"}
            </span>
            <span className="muted">Snapshot: {formatObservedAt(snapshot.observedAtEpochMs)}</span>
          </div>
          {snapshot.status === "permissionDenied" ? (
            <p role="status">
              Windows did not allow the process snapshot. Moonships did not request elevation.
            </p>
          ) : snapshot.status === "unsupported" ? (
            <p role="status">Read-only process inventory is unsupported on this platform.</p>
          ) : snapshot.status === "unavailable" ? (
            <p role="alert">The operating system could not provide a process snapshot.</p>
          ) : snapshot.processes.length === 0 ? (
            <p role="status">No process entries were returned in this snapshot.</p>
          ) : (
            <div className="tasks-table-wrap">
              <table className="tasks-table" aria-label="Read-only process inventory">
                <thead>
                  <tr>
                    <th scope="col">Process</th>
                    <th scope="col">Process ID</th>
                    <th scope="col">Parent ID</th>
                    <th scope="col">Threads</th>
                  </tr>
                </thead>
                <tbody>
                  {snapshot.processes.map((process) => (
                    <tr key={process.processId}>
                      <th scope="row">{process.executableName}</th>
                      <td>{process.processId}</td>
                      <td>{process.parentProcessId}</td>
                      <td>{process.threadCount}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </>
      ) : null}
    </section>
  );
}
