import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Badge, Button } from "./ui-primitives";

type Item = {
  id: string;
  sourceId: string;
  sourceUrl: string;
  originalUrl: string;
  title: string;
  summary: string | null;
  publishedAt: string | null;
  observedAt: string;
  eventTime: string | null;
  category: string;
  freshnessState: "fresh" | "stale" | "unknown" | "blocked" | "error";
  verificationState: "unverified" | "source_attributed" | "independently_verified";
};
type Source = {
  connector: {
    id: string;
    displayName: string;
    enabled: boolean;
    sourceUrl: string;
    health: string;
    lastAttemptAt: string | null;
  };
  freshness: { lastSuccessAt: string | null; isLive: boolean; status: string };
};
const formatTime = (value: string | null) =>
  value ? new Date(value).toLocaleString() : "Not provided";

export function IntelligencePage({ itemId }: { itemId?: string }) {
  const [items, setItems] = useState<Item[]>([]);
  const [item, setItem] = useState<Item | null>(null);
  const [sources, setSources] = useState<Source[]>([]);
  const [search, setSearch] = useState("");
  const [sourceFilter, setSourceFilter] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("");
  const [loading, setLoading] = useState(true);
  const [unavailable, setUnavailable] = useState(false);
  const [busySource, setBusySource] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setUnavailable(false);
    setError(null);
    try {
      const sourceRows = await invoke<Source[]>("list_intelligence_sources");
      setSources(sourceRows);
      if (itemId) {
        setItem(await invoke<Item | null>("get_intelligence_item", { request: { id: itemId } }));
      } else {
        const rows = await invoke<Item[]>("list_intelligence_items", {
          request: { sourceId: null, category: null, search: null, limit: 200 },
        });
        setItems(rows);
      }
    } catch {
      setUnavailable(true);
    } finally {
      setLoading(false);
    }
  }, [itemId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const categories = useMemo(() => [...new Set(items.map((row) => row.category))].sort(), [items]);
  const filtered = items.filter(
    (row) =>
      (!sourceFilter || row.sourceId === sourceFilter) &&
      (!categoryFilter || row.category === categoryFilter) &&
      (!search || `${row.title} ${row.summary ?? ""}`.toLowerCase().includes(search.toLowerCase())),
  );

  const fetchSource = async (source: Source) => {
    setBusySource(source.connector.id);
    setError(null);
    setNotice(null);
    try {
      const result = await invoke<{ fetchedCount: number }>("fetch_intelligence_source", {
        request: { id: source.connector.id },
      });
      setNotice(`Saved ${result.fetchedCount} items from ${source.connector.displayName}.`);
    } catch (reason) {
      setError(
        typeof reason === "string"
          ? reason
          : reason instanceof Error
            ? reason.message
            : `Could not fetch ${source.connector.displayName}.`,
      );
      setBusySource(null);
      window.setTimeout(() => {
        void reload();
      }, 0);
      return;
    }
    setBusySource(null);
    void reload();
  };

  const status = (row: Item) =>
    row.freshnessState === "fresh" ? "Fresh at observation" : row.freshnessState;
  const sourceName = (sourceId: string) =>
    sources.find((source) => source.connector.id === sourceId)?.connector.displayName ?? sourceId;

  return (
    <section className="page intelligence-page" aria-labelledby="intelligence-title">
      <header className="page-header">
        <div>
          <p className="eyebrow">INTELLIGENCE / WORLD MONITOR</p>
          <h2 id="intelligence-title">{itemId ? "Intelligence item" : "Intelligence"}</h2>
          <p className="page-description">
            Source-attributed items saved on this device. Fetches happen only when you request them.
          </p>
        </div>
      </header>
      {error ? (
        <div className="inline-alert" role="alert">
          {error}
        </div>
      ) : null}
      {notice ? (
        <p className="inline-status" role="status">
          {notice}
        </p>
      ) : null}
      {itemId ? (
        loading ? (
          <p role="status">Loading intelligence item…</p>
        ) : unavailable ? (
          <div className="empty-state">
            <h3>Intelligence item is unavailable</h3>
            <p>Saved item data could not be read.</p>
            <Button variant="secondary" onClick={() => void reload()}>
              Retry
            </Button>
          </div>
        ) : !item ? (
          <div className="empty-state">
            <h3>Intelligence item not found</h3>
            <p>It may have been removed from local storage.</p>
            <a href="#/intelligence">Back to Intelligence</a>
          </div>
        ) : (
          <article className="intelligence-detail">
            <p className="eyebrow">
              {item.category} · {status(item)}
            </p>
            <h3>{item.title}</h3>
            {item.summary ? (
              <p>{item.summary}</p>
            ) : (
              <p className="muted">No summary provided by the source.</p>
            )}
            <dl className="intelligence-metadata">
              <div>
                <dt>Source</dt>
                <dd>{sourceName(item.sourceId)}</dd>
              </div>
              <div>
                <dt>Published</dt>
                <dd>{formatTime(item.publishedAt)}</dd>
              </div>
              <div>
                <dt>Observed</dt>
                <dd>{formatTime(item.observedAt)}</dd>
              </div>
              <div>
                <dt>Event time</dt>
                <dd>{formatTime(item.eventTime)}</dd>
              </div>
              <div>
                <dt>Verification</dt>
                <dd>{item.verificationState.replaceAll("_", " ")}</dd>
              </div>
            </dl>
            <p>
              <a href={item.sourceUrl} target="_blank" rel="noreferrer">
                Open source feed
              </a>{" "}
              ·{" "}
              <a href={item.originalUrl} target="_blank" rel="noreferrer">
                Open original item
              </a>
            </p>
            <a href="#/intelligence">Back to Intelligence list</a>
          </article>
        )
      ) : (
        <>
          <section className="intelligence-fetch" aria-labelledby="fetch-sources-title">
            <div className="section-heading">
              <div>
                <p className="eyebrow">USER CONTROLLED</p>
                <h3 id="fetch-sources-title">Fetch a source</h3>
              </div>
            </div>
            <p>
              Only enabled sources can be fetched. Fetching is manual; no background schedule
              starts.
            </p>
            {loading ? (
              <p role="status">Loading sources…</p>
            ) : sources.length === 0 ? (
              <p>
                No sources configured. Add a source in{" "}
                <a href="#/intelligence-sources">Intelligence Sources</a>.
              </p>
            ) : (
              <ul className="intelligence-source-list">
                {sources.map((source) => (
                  <li key={source.connector.id}>
                    <div>
                      <strong>{source.connector.displayName}</strong>
                      <span>
                        {source.freshness.isLive ? "Live" : source.freshness.status} · Last success:{" "}
                        {formatTime(source.freshness.lastSuccessAt)}
                      </span>
                    </div>
                    <Button
                      variant="secondary"
                      size="sm"
                      disabled={!source.connector.enabled || busySource !== null}
                      loading={busySource === source.connector.id}
                      loadingLabel="Fetching…"
                      onClick={() => void fetchSource(source)}
                    >
                      {source.connector.enabled ? "Fetch now" : "Enable source to fetch"}
                    </Button>
                  </li>
                ))}
              </ul>
            )}
          </section>
          <section aria-labelledby="intelligence-items-title">
            <div className="section-heading">
              <div>
                <p className="eyebrow">SAVED LOCALLY</p>
                <h3 id="intelligence-items-title">Items</h3>
              </div>
              <Button
                variant="secondary"
                size="sm"
                disabled={loading}
                onClick={() => void reload()}
              >
                Refresh list
              </Button>
            </div>
            <div className="intelligence-filters">
              <label>
                Search items
                <input
                  aria-label="Search intelligence items"
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                />
              </label>
              <label>
                Source
                <select
                  aria-label="Filter by source"
                  value={sourceFilter}
                  onChange={(event) => setSourceFilter(event.target.value)}
                >
                  <option value="">All sources</option>
                  {sources.map((source) => (
                    <option key={source.connector.id} value={source.connector.id}>
                      {source.connector.displayName}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Category
                <select
                  aria-label="Filter by category"
                  value={categoryFilter}
                  onChange={(event) => setCategoryFilter(event.target.value)}
                >
                  <option value="">All categories</option>
                  {categories.map((category) => (
                    <option key={category}>{category}</option>
                  ))}
                </select>
              </label>
            </div>
            {sources.length > 0 ? (
              <p className="intelligence-health">
                {sources
                  .map(
                    (source) =>
                      `${source.connector.displayName}: ${source.freshness.isLive ? "live" : source.connector.health.toLowerCase().replaceAll("_", " ")} · last attempt ${formatTime(source.connector.lastAttemptAt)} · last success ${formatTime(source.freshness.lastSuccessAt)}`,
                  )
                  .join(" · ")}
              </p>
            ) : null}
            {loading ? (
              <p role="status">Loading saved items…</p>
            ) : unavailable ? (
              <div className="empty-state">
                <h4>Saved intelligence items are unavailable</h4>
                <p>Moonships could not read item or source data.</p>
                <Button variant="secondary" onClick={() => void reload()}>
                  Retry
                </Button>
              </div>
            ) : filtered.length === 0 ? (
              <div className="empty-state">
                <h4>
                  {items.length ? "No items match these filters" : "No intelligence items saved"}
                </h4>
                <p>
                  {items.length
                    ? "Change or clear a filter to see saved items."
                    : "Fetch an enabled source manually to save source-attributed items here."}
                </p>
              </div>
            ) : (
              <div className="intelligence-table-wrap">
                <table className="intelligence-table">
                  <thead>
                    <tr>
                      <th scope="col">Item</th>
                      <th scope="col">Source</th>
                      <th scope="col">Category</th>
                      <th scope="col">Published</th>
                      <th scope="col">Observed</th>
                      <th scope="col">Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filtered.map((row) => (
                      <tr key={row.id}>
                        <th scope="row">
                          <a href={`#/intelligence/items/${encodeURIComponent(row.id)}`}>
                            {row.title}
                          </a>
                          {row.summary ? <span>{row.summary}</span> : null}
                        </th>
                        <td>{sourceName(row.sourceId)}</td>
                        <td>
                          <Badge variant="neutral">{row.category}</Badge>
                        </td>
                        <td>{formatTime(row.publishedAt)}</td>
                        <td>{formatTime(row.observedAt)}</td>
                        <td>{status(row)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>
        </>
      )}
    </section>
  );
}
