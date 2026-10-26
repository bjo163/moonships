import { useState, type FormEvent } from "react";
import {
  createTaskFromDiagnostic,
  networkDiagnosticErrorMessage,
  runNetworkDiagnostic,
  type NetworkDiagnosticKind,
  type NetworkDiagnosticResult,
} from "./network-diagnostics-api";
import { Button, Input } from "./ui-primitives";

function observedAt(value: number): string {
  const date = new Date(value);
  return Number.isNaN(date.valueOf())
    ? "Time unavailable"
    : new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "medium" }).format(date);
}

export function NetOpsPage() {
  const [kind, setKind] = useState<NetworkDiagnosticKind>("dns_lookup");
  const [targetHost, setTargetHost] = useState("");
  const [targetPort, setTargetPort] = useState("443");
  const [result, setResult] = useState<NetworkDiagnosticResult | null>(null);
  const [taskSaved, setTaskSaved] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (busy) return;
    setBusy(true);
    setError(null);
    setResult(null);
    setTaskSaved(false);
    try {
      const port = kind === "tcp_connectivity" ? Number(targetPort) : null;
      setResult(await runNetworkDiagnostic({ kind, targetHost, targetPort: port }));
    } catch (reason) {
      setError(networkDiagnosticErrorMessage(reason));
    } finally {
      setBusy(false);
    }
  };

  const createTask = async () => {
    if (!result || busy) return;
    setBusy(true);
    setError(null);
    try {
      await createTaskFromDiagnostic(result.id);
      setTaskSaved(true);
    } catch (reason) {
      setError(networkDiagnosticErrorMessage(reason));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="tasks-page" aria-labelledby="netops-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">TOOLBOX / READ ONLY</p>
          <h2 id="netops-title">Network diagnostics</h2>
          <p className="muted">
            One DNS lookup or one TCP host and port check. No scans or automatic retries.
          </p>
        </div>
      </header>
      <form className="tasks-page__toolbar" onSubmit={(event) => void run(event)}>
        <div className="ms-field">
          <label className="ms-label" htmlFor="netops-kind">
            Diagnostic
          </label>
          <select
            id="netops-kind"
            className="ms-input"
            value={kind}
            onChange={(event) => setKind(event.target.value as NetworkDiagnosticKind)}
            disabled={busy}
          >
            <option value="dns_lookup">DNS lookup</option>
            <option value="tcp_connectivity">TCP connectivity</option>
          </select>
        </div>
        <Input
          label="Hostname or IP address"
          value={targetHost}
          onChange={(event) => setTargetHost(event.target.value)}
          maxLength={253}
          required
          disabled={busy}
        />
        {kind === "tcp_connectivity" ? (
          <Input
            label="Port"
            type="number"
            min={1}
            max={65535}
            value={targetPort}
            onChange={(event) => setTargetPort(event.target.value)}
            required
            disabled={busy}
          />
        ) : null}
        <Button type="submit" disabled={busy}>
          {busy ? "Checking…" : "Run diagnostic"}
        </Button>
      </form>
      <p className="muted">
        Timeout: 2 seconds · limit: 10 checks per minute · no command or file access.
      </p>
      {error ? <p role="alert">{error}</p> : null}
      {result ? (
        <section
          className="activity-state"
          aria-labelledby="diagnostic-result-title"
          aria-live="polite"
        >
          <h3 id="diagnostic-result-title">Diagnostic result</h3>
          <dl className="diagnostic-result-grid">
            <dt>Target</dt>
            <dd>
              {result.targetHost}
              {result.targetPort ? `:${result.targetPort}` : ""}
            </dd>
            <dt>Result</dt>
            <dd>
              {result.outcome}
              {result.errorCode ? ` · ${result.errorCode}` : ""}
            </dd>
            <dt>Observed</dt>
            <dd>{observedAt(result.observedAtEpochMs)}</dd>
            <dt>Duration</dt>
            <dd>{result.durationMs} ms</dd>
            <dt>Source</dt>
            <dd>{result.source}</dd>
            {result.resolvedAddresses.length ? (
              <>
                <dt>Resolved addresses</dt>
                <dd>{result.resolvedAddresses.join(", ")}</dd>
              </>
            ) : null}
          </dl>
          <div className="tasks-page__toolbar">
            <Button
              variant="secondary"
              onClick={() => void createTask()}
              disabled={busy || taskSaved || taskFailed(result)}
            >
              {taskSaved ? "Follow-up task saved" : "Create follow-up task"}
            </Button>
            {taskSaved ? (
              <span role="status" aria-live="polite">
                Saved to your Personal inbox with the target, result and observed time.
              </span>
            ) : null}
          </div>
          <p className="muted">
            The task summary omits resolved addresses and raw system messages.
          </p>
        </section>
      ) : null}
    </section>
  );
}

function taskFailed(result: NetworkDiagnosticResult): boolean {
  return result.outcome === "Unsupported";
}
