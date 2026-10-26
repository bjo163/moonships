import { invoke } from "@tauri-apps/api/core";
import { type FormEvent, useCallback, useEffect, useState } from "react";
import { Button, Dialog } from "./ui-primitives";

type Watchlist = {
  id: string;
  name: string;
  keywords: string[];
  categories: string[];
  paused: boolean;
};

type Draft = { id?: string; name: string; keywords: string; categories: string };
const emptyDraft: Draft = { name: "", keywords: "", categories: "" };

function splitFilters(value: string): string[] {
  return value
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
}

export function TopicWatchlistsPage() {
  const [watchlists, setWatchlists] = useState<Watchlist[]>([]);
  const [draft, setDraft] = useState<Draft>(emptyDraft);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<Watchlist | null>(null);
  const [unavailable, setUnavailable] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setWatchlists(await invoke<Watchlist[]>("list_intelligence_watchlists"));
      setUnavailable(false);
    } catch {
      setUnavailable(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const save = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSaving(true);
    setError(null);
    setNotice(null);
    try {
      await invoke<Watchlist>("save_intelligence_watchlist", {
        request: {
          id: draft.id ?? null,
          name: draft.name.trim(),
          keywords: splitFilters(draft.keywords),
          categories: splitFilters(draft.categories),
        },
      });
      setDraft(emptyDraft);
      setNotice("Watchlist saved on this device.");
      await reload();
    } catch (reason) {
      setError(typeof reason === "string" ? reason : "Watchlist could not be saved.");
    } finally {
      setSaving(false);
    }
  };

  const togglePaused = async (watchlist: Watchlist) => {
    setBusyId(watchlist.id);
    setError(null);
    setNotice(null);
    try {
      await invoke<Watchlist>("set_intelligence_watchlist_paused", {
        request: { id: watchlist.id, paused: !watchlist.paused },
      });
      setNotice(watchlist.paused ? "Watchlist resumed." : "Watchlist paused.");
      await reload();
    } catch (reason) {
      setError(typeof reason === "string" ? reason : "Watchlist state could not be changed.");
    } finally {
      setBusyId(null);
    }
  };

  const remove = async () => {
    if (!deleteTarget) return;
    setBusyId(deleteTarget.id);
    setError(null);
    setNotice(null);
    try {
      const deleted = await invoke<boolean>("delete_intelligence_watchlist", {
        request: { id: deleteTarget.id },
      });
      if (!deleted) throw new Error("Watchlist no longer exists.");
      setDeleteTarget(null);
      if (draft.id === deleteTarget.id) setDraft(emptyDraft);
      setNotice("Watchlist deleted. Source data was not changed.");
      await reload();
    } catch (reason) {
      setError(typeof reason === "string" ? reason : "Watchlist could not be deleted.");
    } finally {
      setBusyId(null);
    }
  };

  const edit = (watchlist: Watchlist) => {
    setDraft({
      id: watchlist.id,
      name: watchlist.name,
      keywords: watchlist.keywords.join(", "),
      categories: watchlist.categories.join(", "),
    });
    setNotice("Editing saved filters.");
  };

  return (
    <section className="page topic-watchlists-page" aria-labelledby="topic-watchlists-title">
      <header className="page-header">
        <div>
          <p className="eyebrow">INTELLIGENCE / WATCHLISTS</p>
          <h2 id="topic-watchlists-title">Topic Watchlists</h2>
          <p className="page-description">
            Save keyword and category filters for attributed intelligence items.
          </p>
        </div>
      </header>

      <div className="notice-card" role="note">
        Keywords use case-insensitive title or summary matches; categories use exact matches. When
        both groups are set, a match must satisfy both groups. Either group may be empty, but each
        watchlist needs at least one filter. Saved filters do not start feed fetches, background
        monitoring or notifications.
      </div>
      <p className="inline-status" role="status">
        Watchlist matching is ready for the Intelligence item view. No item records are stored by
        this milestone.
      </p>
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

      <section className="watchlist-list" aria-labelledby="watchlist-list-title">
        <div className="section-heading">
          <div>
            <p className="eyebrow">SAVED LOCALLY</p>
            <h3 id="watchlist-list-title">Watchlists</h3>
          </div>
          <Button variant="secondary" size="sm" disabled={loading} onClick={() => void reload()}>
            Refresh list
          </Button>
        </div>
        {loading ? (
          <p role="status">Loading saved watchlists…</p>
        ) : unavailable ? (
          <div className="empty-state">
            <h4>Saved watchlists are unavailable</h4>
            <p>Moonships could not confirm which filters are stored on this device.</p>
            <Button variant="secondary" size="sm" onClick={() => void reload()}>
              Retry loading watchlists
            </Button>
          </div>
        ) : watchlists.length === 0 ? (
          <div className="empty-state">
            <h4>No watchlists saved</h4>
            <p>Create a keyword or category filter below.</p>
          </div>
        ) : (
          <div className="watchlist-cards">
            {watchlists.map((watchlist) => (
              <article className="watchlist-card" key={watchlist.id}>
                <div className="watchlist-card-heading">
                  <div>
                    <h4>{watchlist.name}</h4>
                    <p>{watchlist.paused ? "Paused" : "Active filter"}</p>
                  </div>
                  <span
                    className={`watchlist-state ${watchlist.paused ? "watchlist-state-paused" : "watchlist-state-active"}`}
                  >
                    {watchlist.paused ? "Paused" : "Active"}
                  </span>
                </div>
                <dl className="watchlist-details">
                  <div>
                    <dt>Keywords</dt>
                    <dd>
                      {watchlist.keywords.length ? watchlist.keywords.join(", ") : "Any keyword"}
                    </dd>
                  </div>
                  <div>
                    <dt>Categories</dt>
                    <dd>
                      {watchlist.categories.length
                        ? watchlist.categories.join(", ")
                        : "Any category"}
                    </dd>
                  </div>
                  <div>
                    <dt>Match behavior</dt>
                    <dd>Any keyword AND any selected category</dd>
                  </div>
                </dl>
                <div className="source-actions">
                  <Button variant="secondary" size="sm" onClick={() => edit(watchlist)}>
                    Edit
                  </Button>
                  <Button
                    variant="secondary"
                    size="sm"
                    disabled={busyId === watchlist.id}
                    onClick={() => void togglePaused(watchlist)}
                  >
                    {watchlist.paused ? "Resume" : "Pause"}
                  </Button>
                  <Button variant="danger" size="sm" onClick={() => setDeleteTarget(watchlist)}>
                    Delete
                  </Button>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="source-form-section" aria-labelledby="watchlist-form-title">
        <div className="section-heading">
          <div>
            <p className="eyebrow">FILTER SETTINGS</p>
            <h3 id="watchlist-form-title">{draft.id ? "Edit watchlist" : "Add watchlist"}</h3>
          </div>
          {draft.id ? (
            <Button variant="secondary" size="sm" onClick={() => setDraft(emptyDraft)}>
              Cancel edit
            </Button>
          ) : null}
        </div>
        <form className="watchlist-form" onSubmit={save}>
          <label>
            Name
            <input
              required
              maxLength={120}
              value={draft.name}
              onChange={(event) => setDraft({ ...draft, name: event.target.value })}
            />
          </label>
          <label>
            Keywords, separated by commas
            <input
              value={draft.keywords}
              onChange={(event) => setDraft({ ...draft, keywords: event.target.value })}
              aria-describedby="watchlist-keyword-help"
            />
          </label>
          <p id="watchlist-keyword-help" className="field-help">
            Up to 20 keywords, 100 characters each. Matching ignores letter case and checks title or
            summary.
          </p>
          <label>
            Categories, separated by commas
            <input
              value={draft.categories}
              onChange={(event) => setDraft({ ...draft, categories: event.target.value })}
              aria-describedby="watchlist-category-help"
            />
          </label>
          <p id="watchlist-category-help" className="field-help">
            Use normalized category IDs such as general or advisory. Matching is exact.
          </p>
          <p className="field-help">
            Add at least one keyword or category. When both groups are set, both groups must match.
          </p>
          <div className="source-form-actions">
            <Button type="submit" disabled={saving}>
              {saving ? "Saving…" : draft.id ? "Save changes" : "Save watchlist"}
            </Button>
          </div>
        </form>
      </section>

      <Dialog
        open={deleteTarget !== null}
        title="Delete topic watchlist?"
        description="Only this saved filter will be removed. Source declarations and source data are not affected."
        onClose={() => setDeleteTarget(null)}
      >
        <div className="watchlist-delete-confirmation">
          <p>{deleteTarget?.name}</p>
          <Button
            variant="danger"
            disabled={busyId === deleteTarget?.id}
            onClick={() => void remove()}
          >
            {busyId === deleteTarget?.id ? "Deleting…" : "Delete watchlist"}
          </Button>
        </div>
      </Dialog>
    </section>
  );
}
