import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { Button } from "./ui-primitives";

type TermsReview = "unreviewed" | "reviewed" | "restricted";
type Freshness =
  | "fresh"
  | "stale"
  | "unknown"
  | "blocked"
  | "rate_limited"
  | "unavailable"
  | "error"
  | "disabled";

type Source = {
  connector: {
    id: string;
    displayName: string;
    sourceUrl: string;
    termsUrl: string | null;
    termsNotes: string;
    termsReview: TermsReview;
    minimumPollIntervalSeconds: number;
    enabled: boolean;
    health: string;
  };
  freshness: { status: Freshness; lastSuccessAt: string | null; isLive: boolean };
};

type Draft = {
  id?: string;
  displayName: string;
  sourceUrl: string;
  termsUrl: string;
  termsNotes: string;
  termsReview: TermsReview;
  minimumPollIntervalSeconds: number;
};

const emptyDraft: Draft = {
  displayName: "",
  sourceUrl: "https://",
  termsUrl: "",
  termsNotes: "",
  termsReview: "unreviewed",
  minimumPollIntervalSeconds: 900,
};

const nwsAlertAreas = [
  "AL",
  "AK",
  "AZ",
  "AR",
  "CA",
  "CO",
  "CT",
  "DE",
  "DC",
  "FL",
  "GA",
  "HI",
  "ID",
  "IL",
  "IN",
  "IA",
  "KS",
  "KY",
  "LA",
  "ME",
  "MD",
  "MA",
  "MI",
  "MN",
  "MS",
  "MO",
  "MT",
  "NE",
  "NV",
  "NH",
  "NJ",
  "NM",
  "NY",
  "NC",
  "ND",
  "OH",
  "OK",
  "OR",
  "PA",
  "RI",
  "SC",
  "SD",
  "TN",
  "TX",
  "UT",
  "VT",
  "VA",
  "WA",
  "WV",
  "WI",
  "WY",
];

function cadenceLabel(seconds: number): string {
  if (seconds % 86_400 === 0)
    return `Every ${seconds / 86_400} day${seconds === 86_400 ? "" : "s"}`;
  if (seconds % 3_600 === 0) return `Every ${seconds / 3_600} hour${seconds === 3_600 ? "" : "s"}`;
  if (seconds % 60 === 0) return `Every ${seconds / 60} minute${seconds === 60 ? "" : "s"}`;
  return `Every ${seconds} seconds`;
}

function freshnessLabel(status: string): string {
  return status.replaceAll("_", " ").replace(/^./, (letter) => letter.toUpperCase());
}

export function IntelligenceSourcesPage() {
  const [sources, setSources] = useState<Source[]>([]);
  const [draft, setDraft] = useState<Draft>(emptyDraft);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [busySource, setBusySource] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [sourcesUnavailable, setSourcesUnavailable] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [nwsArea, setNwsArea] = useState("");

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setSources(await invoke<Source[]>("list_intelligence_sources"));
      setSourcesUnavailable(false);
    } catch {
      setSourcesUnavailable(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const save = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSaving(true);
    setError(null);
    setNotice(null);
    try {
      await invoke<Source>("save_intelligence_source", {
        request: {
          id: draft.id ?? null,
          displayName: draft.displayName.trim(),
          sourceUrl: draft.sourceUrl.trim(),
          termsUrl: draft.termsUrl.trim() || null,
          termsNotes: draft.termsNotes.trim(),
          termsReview: draft.termsReview,
          minimumPollIntervalSeconds: draft.minimumPollIntervalSeconds,
        },
      });
      setDraft(emptyDraft);
      setNotice("Source settings saved locally.");
      await reload();
    } catch (reason) {
      setError(typeof reason === "string" ? reason : "Source settings could not be saved.");
    } finally {
      setSaving(false);
    }
  };

  const addNwsAlerts = async () => {
    if (!nwsArea) {
      setError("Choose a state or district before adding its NWS alert feed.");
      return;
    }
    setSaving(true);
    setError(null);
    setNotice(null);
    try {
      await invoke<Source>("save_intelligence_source", {
        request: {
          id: null,
          displayName: `NWS Active Alerts (${nwsArea})`,
          sourceUrl: `https://api.weather.gov/alerts/active.atom?area=${nwsArea}`,
          termsUrl: "https://www.weather.gov/documentation/services-web-alerts",
          termsNotes:
            "Official National Weather Service active-alert Atom feed. Review NWS access guidance before enabling. Manual fetch only; configured minimum interval is 30 minutes.",
          termsReview: "unreviewed",
          minimumPollIntervalSeconds: 1_800,
        },
      });
      setNotice(
        `Added the NWS ${nwsArea} feed disabled. Review the linked NWS guidance before enabling it.`,
      );
      await reload();
    } catch (reason) {
      setError(typeof reason === "string" ? reason : "NWS alert source could not be added.");
    } finally {
      setSaving(false);
    }
  };

  const toggle = async (source: Source) => {
    setBusySource(source.connector.id);
    setError(null);
    setNotice(null);
    try {
      await invoke("set_intelligence_source_enabled", {
        request: { id: source.connector.id, enabled: !source.connector.enabled },
      });
      setNotice(source.connector.enabled ? "Source disabled." : "Source enabled for manual fetch.");
      await reload();
    } catch (reason) {
      setError(typeof reason === "string" ? reason : "Source state could not be changed.");
    } finally {
      setBusySource(null);
    }
  };

  const edit = (source: Source) => {
    setDraft({
      id: source.connector.id,
      displayName: source.connector.displayName,
      sourceUrl: source.connector.sourceUrl,
      termsUrl: source.connector.termsUrl ?? "",
      termsNotes: source.connector.termsNotes,
      termsReview: source.connector.termsReview,
      minimumPollIntervalSeconds: source.connector.minimumPollIntervalSeconds,
    });
    setNotice("Editing source settings. Saving a changed URL will disable this source.");
  };

  return (
    <section
      className="page intelligence-sources-page"
      aria-labelledby="intelligence-sources-title"
    >
      <header className="page-header">
        <div>
          <p className="eyebrow">INTELLIGENCE / SOURCES</p>
          <h2 id="intelligence-sources-title">Intelligence Sources</h2>
          <p className="page-description">
            Manage RSS and Atom source declarations and review their terms before allowing a manual
            fetch.
          </p>
        </div>
      </header>

      <div className="notice-card" role="note">
        Enabled sources are available only for an explicit manual fetch. Moonships does not schedule
        background fetches or fetch sources when this screen opens.
      </div>
      {error ? (
        <div className="inline-alert" role="alert">
          {error}{" "}
          <Button variant="secondary" size="sm" onClick={() => void reload()}>
            Retry
          </Button>
        </div>
      ) : null}
      {notice ? (
        <p className="inline-status" role="status">
          {notice}
        </p>
      ) : null}

      <section className="source-list" aria-labelledby="source-list-title">
        <div className="section-heading">
          <div>
            <p className="eyebrow">SAVED LOCALLY</p>
            <h3 id="source-list-title">Sources</h3>
          </div>
          <Button variant="secondary" size="sm" onClick={() => void reload()} disabled={loading}>
            Refresh list
          </Button>
        </div>
        {loading ? (
          <p role="status">Loading saved sources…</p>
        ) : sourcesUnavailable ? (
          <div className="empty-state">
            <h4>Saved sources are unavailable</h4>
            <p>Moonships could not confirm whether any sources are stored on this device.</p>
            <Button variant="secondary" size="sm" onClick={() => void reload()}>
              Retry loading sources
            </Button>
          </div>
        ) : sources.length === 0 ? (
          <div className="empty-state">
            <h4>No sources configured</h4>
            <p>Add an RSS feed below. New sources start disabled.</p>
          </div>
        ) : (
          <div className="source-cards">
            {sources.map(({ connector, freshness }) => (
              <article className="source-card" key={connector.id}>
                <div className="source-card-heading">
                  <div>
                    <h4>{connector.displayName}</h4>
                    <a href={connector.sourceUrl} target="_blank" rel="noreferrer">
                      {connector.sourceUrl}
                    </a>
                  </div>
                  <span className={`source-status source-status-${freshness.status}`}>
                    {freshnessLabel(freshness.status)}
                  </span>
                </div>
                <dl className="source-details">
                  <div>
                    <dt>State</dt>
                    <dd>{connector.enabled ? "Enabled for manual fetch" : "Disabled"}</dd>
                  </div>
                  <div>
                    <dt>Minimum interval</dt>
                    <dd>{cadenceLabel(connector.minimumPollIntervalSeconds)}</dd>
                  </div>
                  <div>
                    <dt>Terms review</dt>
                    <dd>{freshnessLabel(connector.termsReview)}</dd>
                  </div>
                  <div>
                    <dt>Last successful fetch</dt>
                    <dd>
                      {freshness.lastSuccessAt
                        ? new Date(freshness.lastSuccessAt).toLocaleString()
                        : "No successful fetch recorded"}
                    </dd>
                  </div>
                </dl>
                <div className="source-actions">
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => edit({ connector, freshness })}
                  >
                    Edit settings
                  </Button>
                  <Button
                    variant={connector.enabled ? "secondary" : "primary"}
                    size="sm"
                    disabled={busySource === connector.id}
                    onClick={() => void toggle({ connector, freshness })}
                  >
                    {busySource === connector.id
                      ? "Saving…"
                      : connector.enabled
                        ? "Disable"
                        : "Enable"}
                  </Button>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="source-form-section" aria-labelledby="nws-preset-title">
        <div className="section-heading">
          <div>
            <p className="eyebrow">OFFICIAL ALERT FEED</p>
            <h3 id="nws-preset-title">National Weather Service</h3>
          </div>
        </div>
        <p className="field-help">
          Add a state or district active-alert feed. It starts disabled and requires you to review
          NWS access guidance. Fetching stays manual; Moonships waits at least 30 minutes between
          requests for this preset.
        </p>
        <div className="source-form-actions">
          <label>
            State or district code
            <select value={nwsArea} onChange={(event) => setNwsArea(event.target.value)}>
              <option value="">Choose an area</option>
              {nwsAlertAreas.map((area) => (
                <option key={area} value={area}>
                  {area}
                </option>
              ))}
            </select>
          </label>
          <Button type="button" onClick={() => void addNwsAlerts()} disabled={saving}>
            {saving ? "Saving…" : "Add NWS alert feed"}
          </Button>
        </div>
      </section>

      <section className="source-form-section" aria-labelledby="source-form-title">
        <div className="section-heading">
          <div>
            <p className="eyebrow">SOURCE SETTINGS</p>
            <h3 id="source-form-title">{draft.id ? "Edit source" : "Add RSS or Atom source"}</h3>
          </div>
          {draft.id ? (
            <Button variant="secondary" size="sm" onClick={() => setDraft(emptyDraft)}>
              Cancel edit
            </Button>
          ) : null}
        </div>
        <form className="source-form" onSubmit={save}>
          <label>
            Display name
            <input
              required
              maxLength={120}
              value={draft.displayName}
              onChange={(event) => setDraft({ ...draft, displayName: event.target.value })}
            />
          </label>
          <label>
            RSS or Atom feed URL
            <input
              required
              type="url"
              inputMode="url"
              value={draft.sourceUrl}
              onChange={(event) => setDraft({ ...draft, sourceUrl: event.target.value })}
              aria-describedby="source-url-help"
            />
          </label>
          <p id="source-url-help" className="field-help">
            HTTPS URLs only. Changing a saved URL disables the source and clears its previous
            freshness.
          </p>
          <label>
            Terms URL (optional)
            <input
              type="url"
              inputMode="url"
              value={draft.termsUrl}
              onChange={(event) => setDraft({ ...draft, termsUrl: event.target.value })}
            />
          </label>
          <label>
            Terms and access notes
            <textarea
              required
              maxLength={1000}
              rows={3}
              value={draft.termsNotes}
              onChange={(event) => setDraft({ ...draft, termsNotes: event.target.value })}
            />
          </label>
          <label>
            Terms review
            <select
              value={draft.termsReview}
              onChange={(event) =>
                setDraft({ ...draft, termsReview: event.target.value as TermsReview })
              }
            >
              <option value="unreviewed">Not reviewed</option>
              <option value="reviewed">Reviewed and allowed</option>
              <option value="restricted">Restricted</option>
            </select>
          </label>
          <label>
            Minimum poll interval
            <select
              value={draft.minimumPollIntervalSeconds}
              onChange={(event) =>
                setDraft({ ...draft, minimumPollIntervalSeconds: Number(event.target.value) })
              }
            >
              <option value={300}>5 minutes</option>
              <option value={900}>15 minutes</option>
              <option value={1800}>30 minutes</option>
              <option value={3600}>1 hour</option>
              <option value={21600}>6 hours</option>
              <option value={86400}>1 day</option>
            </select>
            <span className="field-help">
              This records the source’s minimum cadence. It does not create a background schedule.
            </span>
          </label>
          <div className="source-form-actions">
            <Button type="submit" disabled={saving}>
              {saving ? "Saving…" : draft.id ? "Save changes" : "Save source"}
            </Button>
          </div>
        </form>
      </section>
    </section>
  );
}
