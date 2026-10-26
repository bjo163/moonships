import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { Button } from "./ui-primitives";

type Note = {
  id: string;
  title: string;
  body: string;
  itemId: string;
  createdAt: string;
  updatedAt: string;
};
type ExportDocument = {
  schemaVersion: number;
  note: Note;
  reference: {
    itemId: string;
    sourceId: string;
    sourceUrl: string;
    originalUrl: string;
    publishedAt: string | null;
    observedAt: string;
    verificationState: string;
    itemAvailable: boolean;
  };
};

export function ResearchDeskPage({ noteId }: { noteId?: string }) {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selected, setSelected] = useState<ExportDocument | null>(null);
  const [loading, setLoading] = useState(true);
  const [unavailable, setUnavailable] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const reload = useCallback(async () => {
    setLoading(true);
    setUnavailable(false);
    try {
      const noteRows = await invoke<Note[]>("list_research_notes");
      if (!Array.isArray(noteRows)) throw new Error("Saved notes response is invalid");
      setNotes(noteRows);
      if (noteId) {
        const note = await invoke<ExportDocument | null>("get_research_note", {
          request: { id: noteId },
        });
        if (!note || typeof note !== "object" || !note.reference)
          throw new Error("Research note response is invalid");
        setSelected(note);
      }
    } catch {
      setUnavailable(true);
    } finally {
      setLoading(false);
    }
  }, [noteId]);
  useEffect(() => {
    void reload();
  }, [reload]);
  const exportNote = async (id: string) => {
    setError(null);
    try {
      const fileName = await invoke<string | null>("export_research_note", { request: { id } });
      setError(fileName ? `Exported ${fileName}.` : "Export cancelled.");
    } catch (reason) {
      setError(typeof reason === "string" ? reason : "Research note export failed.");
    }
  };
  return (
    <section className="page research-desk-page" aria-labelledby="research-desk-title">
      <header className="page-header">
        <div>
          <p className="eyebrow">KNOWLEDGE / RESEARCH</p>
          <h2 id="research-desk-title">Research Desk</h2>
          <p className="page-description">
            Local notes linked to source-attributed intelligence items.
          </p>
        </div>
        <Button variant="secondary" onClick={() => void reload()} disabled={loading}>
          Refresh notes
        </Button>
      </header>
      {error ? (
        <p className="inline-status" role="status">
          {error}
        </p>
      ) : null}
      {loading ? (
        <p role="status">Loading saved research notes…</p>
      ) : unavailable ? (
        <div className="empty-state">
          <h3>Research notes are unavailable</h3>
          <p>Moonships could not load saved notes.</p>
          <Button variant="secondary" onClick={() => void reload()}>
            Retry
          </Button>
        </div>
      ) : noteId && !selected ? (
        <div className="empty-state">
          <h3>Research note not found</h3>
          <p>The requested note is not saved locally.</p>
        </div>
      ) : noteId && selected ? (
        <article className="research-note-card">
          <h3>{selected.note.title}</h3>
          <p>{selected.note.body || "No operator notes added."}</p>
          <dl className="intelligence-metadata">
            <div>
              <dt>Source ID</dt>
              <dd>{selected.reference.sourceId}</dd>
            </div>
            <div>
              <dt>Published</dt>
              <dd>{selected.reference.publishedAt ?? "Not provided"}</dd>
            </div>
            <div>
              <dt>Observed</dt>
              <dd>{selected.reference.observedAt}</dd>
            </div>
            <div>
              <dt>Verification</dt>
              <dd>{selected.reference.verificationState}</dd>
            </div>
            <div>
              <dt>Item retention</dt>
              <dd>
                {selected.reference.itemAvailable
                  ? "Item retained"
                  : "Item unavailable; provenance snapshot retained"}
              </dd>
            </div>
          </dl>
          <p>
            <a href={selected.reference.sourceUrl} target="_blank" rel="noreferrer">
              Open source feed
            </a>{" "}
            ·{" "}
            <a href={selected.reference.originalUrl} target="_blank" rel="noreferrer">
              Open original item
            </a>
          </p>
          <Button variant="secondary" onClick={() => void exportNote(selected.note.id)}>
            Export note with provenance
          </Button>
          <p>
            <a href="#/research">Back to Research Desk</a>
          </p>
        </article>
      ) : notes.length === 0 ? (
        <div className="empty-state">
          <h3>No research notes saved</h3>
          <p>Save an intelligence item to create a local note with a typed source reference.</p>
        </div>
      ) : (
        <div className="research-notes-list">
          {notes.map((note) => (
            <article key={note.id} className="research-note-card">
              <h3>
                <a href={`#/research/notes/${encodeURIComponent(note.id)}`}>{note.title}</a>
              </h3>
              <p>{note.body || "No operator notes added."}</p>
              <p>
                Source reference:{" "}
                <a href={`#/intelligence/items/${encodeURIComponent(note.itemId)}`}>
                  Open linked intelligence item
                </a>
              </p>
              <Button variant="secondary" size="sm" onClick={() => void exportNote(note.id)}>
                Export note with provenance
              </Button>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
