import { useState } from "react";
import {
  type EvidenceManifest,
  saveEvidenceManifest,
  selectEvidenceFiles,
} from "./evidence-manifest-api";
import { Button } from "./ui-primitives";

function formatBytes(value: number): string {
  if (!Number.isSafeInteger(value) || value < 0) return "—";
  if (value < 1024) return `${value} bytes`;
  const units = ["KiB", "MiB", "GiB"];
  let amount = value / 1024;
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  return `${amount.toFixed(1)} ${units[unit]}`;
}

function formatTimestamp(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? "Time unavailable"
    : new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "medium" }).format(date);
}

export function EvidenceManifestPage() {
  const [manifest, setManifest] = useState<EvidenceManifest | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const selectFiles = async () => {
    setBusy(true);
    setError(null);
    setNotice(null);
    setManifest(null);
    try {
      const selected = await selectEvidenceFiles();
      if (selected) setManifest(selected);
      else setNotice("File selection cancelled. No manifest was created.");
    } catch (reason) {
      setError(
        typeof reason === "string"
          ? reason
          : reason instanceof Error
            ? reason.message
            : "Selected evidence could not be hashed.",
      );
    } finally {
      setBusy(false);
    }
  };

  const saveManifest = async () => {
    if (!manifest || busy) return;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const fileName = await saveEvidenceManifest(manifest.manifestId);
      setNotice(
        fileName ? `Manifest saved as ${fileName}.` : "Save cancelled. No manifest was saved.",
      );
    } catch (reason) {
      setError(
        typeof reason === "string"
          ? reason
          : reason instanceof Error
            ? reason.message
            : "Evidence manifest could not be saved.",
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="tasks-page" aria-labelledby="evidence-manifest-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">SECURITY / EVIDENCE</p>
          <h2 id="evidence-manifest-title">Evidence manifest</h2>
          <p className="muted">
            Hash files you explicitly select and save their names, sizes and SHA-256 digests in a
            local JSON manifest. File contents are not copied into the manifest.
          </p>
        </div>
      </header>
      <div className="notice-card" role="note">
        Hashes support later integrity comparison only. They do not prove who created or handled a
        file, establish chain of custody, or provide forensic validity. Up to 50 files and 2 GiB
        total are read locally; nothing is uploaded.
      </div>
      {error ? (
        <div className="inline-alert" role="alert">
          {error}
        </div>
      ) : null}
      {notice ? <p role="status">{notice}</p> : null}
      <div className="tasks-page__toolbar">
        <Button onClick={() => void selectFiles()} disabled={busy}>
          {busy && !manifest ? "Hashing selected files…" : "Choose evidence files and hash"}
        </Button>
        {manifest ? (
          <Button variant="secondary" onClick={() => void saveManifest()} disabled={busy}>
            {busy ? "Saving manifest…" : "Save manifest JSON"}
          </Button>
        ) : null}
      </div>
      {manifest ? (
        <>
          <div className="tasks-page__toolbar">
            <span>
              {manifest.entries.length} files · {formatTimestamp(manifest.createdAt)}
            </span>
            <span className="muted">Source: user-selected local files · SHA-256</span>
          </div>
          <div className="tasks-table-wrap">
            <table className="tasks-table" aria-label="Evidence file hash manifest preview">
              <thead>
                <tr>
                  <th scope="col">File name</th>
                  <th scope="col">Size</th>
                  <th scope="col">SHA-256</th>
                </tr>
              </thead>
              <tbody>
                {manifest.entries.map((entry) => (
                  <tr key={entry.id}>
                    <th scope="row">{entry.fileName}</th>
                    <td>{formatBytes(entry.sizeBytes)}</td>
                    <td>
                      <code>{entry.sha256}</code>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="muted">
            Manifest SHA-256 · <code>{manifest.manifestSha256}</code>
          </p>
          <p className="muted">{manifest.integrityNotice}</p>
        </>
      ) : null}
    </section>
  );
}
