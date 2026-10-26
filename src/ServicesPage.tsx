import { useCallback, useEffect, useState } from "react";
import {
  listServiceInventory,
  type ServiceInventorySnapshot,
  type WindowsServiceState,
} from "./service-inventory-api";
import { Button } from "./ui-primitives";

const STATE_LABELS: Record<WindowsServiceState, string> = {
  stopped: "Stopped",
  startPending: "Starting",
  stopPending: "Stopping",
  running: "Running",
  continuePending: "Resuming",
  pausePending: "Pausing",
  paused: "Paused",
  unknown: "Unknown",
};

function formatObservedAt(value: number): string {
  const date = new Date(value);
  if (!Number.isFinite(value) || Number.isNaN(date.getTime())) return "Time unavailable";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "medium" }).format(
    date,
  );
}

function statusLabel(snapshot: ServiceInventorySnapshot): string {
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

export function ServicesPage() {
  const [snapshot, setSnapshot] = useState<ServiceInventorySnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(false);
    try {
      setSnapshot(await listServiceInventory());
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
    <section className="tasks-page" aria-labelledby="services-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">SYSOPS / READ ONLY</p>
          <h2 id="services-title">Windows services</h2>
          <p className="muted">
            A local service status snapshot. Moonships does not start, stop or change services here.
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
          Loading service snapshot…
        </p>
      ) : loadError ? (
        <div className="task-detail-error" role="alert">
          <p>The service snapshot could not be loaded.</p>
          <Button variant="secondary" onClick={() => void reload()}>
            Retry
          </Button>
        </div>
      ) : snapshot ? (
        <>
          <div className="tasks-page__toolbar">
            <span role="status" aria-live="polite">
              {statusLabel(snapshot)} · {snapshot.services.length}{" "}
              {snapshot.services.length === 1 ? "service" : "services"}
            </span>
            <span className="muted">Snapshot: {formatObservedAt(snapshot.observedAtEpochMs)}</span>
          </div>
          {snapshot.status === "permissionDenied" ? (
            <p role="status">
              Windows did not allow service enumeration. Moonships did not request elevation.
            </p>
          ) : snapshot.status === "unsupported" ? (
            <p role="status">Service inventory is supported only on Windows.</p>
          ) : snapshot.status === "unavailable" ? (
            <p role="alert">Windows could not provide a service snapshot.</p>
          ) : snapshot.status === "partial" && snapshot.services.length === 0 ? (
            <p role="status">
              Enumeration was incomplete; no service entries were included in this snapshot.
            </p>
          ) : snapshot.services.length === 0 ? (
            <p role="status">No service entries were returned in this snapshot.</p>
          ) : (
            <div className="tasks-table-wrap">
              <table className="tasks-table" aria-label="Read-only Windows service inventory">
                <thead>
                  <tr>
                    <th scope="col">Display name</th>
                    <th scope="col">Service name</th>
                    <th scope="col">Status</th>
                  </tr>
                </thead>
                <tbody>
                  {snapshot.services.map((service) => (
                    <tr key={service.serviceName}>
                      <th scope="row">{service.displayName}</th>
                      <td>{service.serviceName}</td>
                      <td>{STATE_LABELS[service.state]}</td>
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
