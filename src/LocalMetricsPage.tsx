import { useCallback, useEffect, useState } from "react";
import {
  getLocalMetricsSnapshot,
  type LocalMetricsSnapshot,
  type LocalMetricValue,
} from "./local-metrics-api";
import { Button } from "./ui-primitives";

function formatSampleTime(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? "Time unavailable"
    : new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "medium" }).format(date);
}

function formatBytes(value: number | null): string {
  if (value === null || !Number.isFinite(value) || value < 0) return "—";
  const units = ["bytes", "KiB", "MiB", "GiB", "TiB"];
  let amount = value;
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  return `${amount.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function metricState(metric: LocalMetricValue): string {
  if (metric.status === "unsupported") return "Unsupported on this platform";
  if (metric.status === "unavailable") return "Unavailable";
  return "Available";
}

function metricValue(metric: LocalMetricValue, bytes = false): string {
  if (metric.status !== "available" || metric.value === null) return "—";
  return bytes ? formatBytes(metric.value) : String(metric.value);
}

export function LocalMetricsPage() {
  const [snapshot, setSnapshot] = useState<LocalMetricsSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);

  const reload = useCallback(async () => {
    setLoading(true);
    setLoadError(false);
    try {
      setSnapshot(await getLocalMetricsSnapshot());
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

  const metrics = snapshot
    ? ([
        ["Logical processors", snapshot.logicalProcessors, false],
        ["Physical memory · total", snapshot.physicalMemoryTotalBytes, true],
        ["Physical memory · available", snapshot.physicalMemoryAvailableBytes, true],
        ["Windows volume · total", snapshot.windowsVolumeTotalBytes, true],
        ["Windows volume · free", snapshot.windowsVolumeFreeBytes, true],
      ] as const)
    : [];

  return (
    <section className="tasks-page" aria-labelledby="local-metrics-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">OPERATIONS / LOCAL SNAPSHOT</p>
          <h2 id="local-metrics-title">Local metrics</h2>
          <p className="muted">
            One-time system snapshot. Nothing is sent to a remote telemetry service or collected in
            the background.
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
          Reading local metrics…
        </p>
      ) : loadError ? (
        <div className="task-detail-error" role="alert">
          <p>The local metrics snapshot could not be loaded.</p>
          <Button variant="secondary" onClick={() => void reload()}>
            Retry
          </Button>
        </div>
      ) : snapshot ? (
        <>
          <div className="tasks-page__toolbar">
            <span>Source: {snapshot.source}</span>
            <span className="muted">Sampled: {formatSampleTime(snapshot.sampledAt)}</span>
          </div>
          <div className="tasks-table-wrap">
            <table className="tasks-table" aria-label="Local system metrics snapshot">
              <thead>
                <tr>
                  <th scope="col">Metric</th>
                  <th scope="col">Value</th>
                  <th scope="col">State</th>
                </tr>
              </thead>
              <tbody>
                {metrics.map(([label, metric, bytes]) => (
                  <tr key={label}>
                    <th scope="row">{label}</th>
                    <td>{metricValue(metric, bytes)}</td>
                    <td>{metricState(metric)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      ) : null}
    </section>
  );
}
