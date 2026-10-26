import { invoke } from "@tauri-apps/api/core";
import { type FormEvent, useCallback, useEffect, useState } from "react";
import { Button } from "./ui-primitives";

type Header = { name: string; value: string };
type History = {
  id: string;
  requestedUrl: string;
  observedAt: string;
  status: number | null;
  requestHeaders: Header[];
  responseHeaders: Header[];
  responseSizeBytes: number;
  errorCode: string | null;
};
type Result = { history: History; bodyText: string; bodyIsUtf8: boolean };

export function HttpInspectorPage() {
  const [url, setUrl] = useState("https://");
  const [history, setHistory] = useState<History[]>([]);
  const [historyUnavailable, setHistoryUnavailable] = useState(false);
  const [result, setResult] = useState<Result | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [taskCreated, setTaskCreated] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      setHistory(await invoke<History[]>("list_http_inspection_history"));
      setHistoryUnavailable(false);
    } catch {
      setHistoryUnavailable(true);
      setError("Saved inspection history is unavailable.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const inspect = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (busy) return;
    setBusy(true);
    setError(null);
    setTaskCreated(null);
    setResult(null);
    try {
      const response = await invoke<Result>("inspect_http_response", { request: { url } });
      setResult(response);
      setHistory((current) =>
        [response.history, ...current.filter((item) => item.id !== response.history.id)].slice(
          0,
          200,
        ),
      );
    } catch (reason) {
      setError(
        typeof reason === "string"
          ? reason
          : reason instanceof Error
            ? reason.message
            : "HTTP inspection failed.",
      );
    } finally {
      setBusy(false);
    }
  };

  const createFollowUpTask = async () => {
    if (!result || busy) return;
    setBusy(true);
    setError(null);
    setTaskCreated(null);
    try {
      const task = await invoke<{ id: string }>("create_task_from_http_inspection", {
        request: { historyId: result.history.id },
      });
      setTaskCreated(task.id);
    } catch (reason) {
      setError(
        typeof reason === "string"
          ? reason
          : reason instanceof Error
            ? reason.message
            : "Could not create a follow-up Task.",
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="tasks-page" aria-labelledby="http-inspector-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">TOOLBOX / READ ONLY</p>
          <h2 id="http-inspector-title">HTTP Inspector</h2>
          <p className="muted">
            Manually inspect one public HTTPS GET response. Credentials and cookies are not sent.
          </p>
        </div>
      </header>
      <div className="notice-card" role="note">
        Moonships sends a single GET only after you choose Inspect. It rejects private/local targets
        and redirects, waits at most 10 seconds, and reads up to 256 KiB. Saved history redacts
        sensitive headers and URL query values; response bodies stay in this view only.
      </div>
      {error ? (
        <div role="alert" className="inline-alert">
          {error}
        </div>
      ) : null}
      <form className="tasks-page__toolbar" onSubmit={(event) => void inspect(event)}>
        <label className="ms-field">
          Public HTTPS URL
          <input
            type="url"
            required
            maxLength={2048}
            value={url}
            onChange={(event) => setUrl(event.target.value)}
          />
        </label>
        <Button type="submit" disabled={busy}>
          {busy ? "Inspecting…" : "Inspect response"}
        </Button>
      </form>
      {result ? (
        <section className="source-form-section" aria-labelledby="http-result-title">
          <h3 id="http-result-title">Response · HTTP {result.history.status}</h3>
          <p>{result.history.requestedUrl}</p>
          <p>
            Observed {new Date(result.history.observedAt).toLocaleString()} ·{" "}
            {result.history.responseSizeBytes} bytes
          </p>
          <p>Saved result ID: {result.history.id}</p>
          <p className="muted">
            A follow-up Task includes only this saved redacted metadata. The response body is never
            attached.
          </p>
          <Button type="button" disabled={busy} onClick={() => void createFollowUpTask()}>
            Create follow-up Task
          </Button>
          {taskCreated ? <p role="status">Task created: {taskCreated}</p> : null}
          <h4>Response headers</h4>
          <dl className="intelligence-metadata">
            {result.history.responseHeaders.map((header) => (
              <div key={`${header.name}-${header.value}`}>
                <dt>{header.name}</dt>
                <dd>{header.value}</dd>
              </div>
            ))}
          </dl>
          <h4>Response body (not saved to history)</h4>
          <pre className="http-inspector-body">
            {result.bodyText || (result.bodyIsUtf8 ? "(empty response)" : "(binary response)")}
          </pre>
        </section>
      ) : null}
      <section className="source-list" aria-labelledby="http-history-title">
        <h3 id="http-history-title">Saved inspection history</h3>
        {loading ? (
          <p role="status">Loading local history…</p>
        ) : historyUnavailable ? (
          <p>Saved inspection history is unavailable.</p>
        ) : history.length === 0 ? (
          <p>No inspections saved.</p>
        ) : (
          <div className="source-cards">
            {history.map((entry) => (
              <article className="source-card" key={entry.id}>
                <h4>HTTP {entry.status ?? entry.errorCode ?? "Failed"}</h4>
                <p>{entry.requestedUrl}</p>
                <p>
                  {new Date(entry.observedAt).toLocaleString()} · {entry.responseSizeBytes} bytes
                </p>
                <details>
                  <summary>Safe saved headers</summary>
                  <dl className="intelligence-metadata">
                    {[...entry.requestHeaders, ...entry.responseHeaders].map((header) => (
                      <div key={`${header.name}-${header.value}`}>
                        <dt>{header.name}</dt>
                        <dd>{header.value}</dd>
                      </div>
                    ))}
                  </dl>
                </details>
              </article>
            ))}
          </div>
        )}
      </section>
    </section>
  );
}
