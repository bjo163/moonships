import { Badge, EmptyState } from "./ui-primitives";
import type { CleanupCategory, CleanupPreviewDto, CleanupSkipReason } from "./cleanup-preview";

const categoryLabels: Record<CleanupCategory, string> = {
  temporary_files: "Temporary files",
  application_cache: "Application cache",
};

const skipReasonLabels: Record<CleanupSkipReason, string> = {
  invalid_rule: "Rule is invalid",
  wrong_root: "Outside the allowed folder",
  not_regular_file: "Not a regular file",
  modified_time_in_future: "Modified time is invalid",
  too_recent: "File is too recent",
  item_too_large: "File exceeds the item limit",
};

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "Size unavailable";
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 }).format(value)} ${units[unit]}`;
}

function formatModifiedAt(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? "Date unavailable"
    : new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(date);
}

export function CleanerPreviewPage({ preview }: { preview: CleanupPreviewDto | null }) {
  const items = preview?.items ?? [];
  const eligibleItems = items.filter((item) => item.disposition === "eligible");
  const skippedItems = items.filter((item) => item.disposition === "skipped");
  const estimatedBytes = eligibleItems.reduce((total, item) => total + item.sizeBytes, 0);

  return (
    <section className="cleaner-preview" aria-labelledby="cleaner-preview-title">
      <header className="home-dashboard__header cleaner-preview__header">
        <div>
          <p className="eyebrow">DAILY TOOLS / CLEAN &amp; STORAGE</p>
          <h2 id="cleaner-preview-title">Clean &amp; Storage</h2>
          <p className="muted">
            Review candidate files and estimated space before any future action.
          </p>
        </div>
        <Badge variant="info">Preview only · no files changed</Badge>
      </header>

      {preview === null ? (
        <EmptyState
          title="No scan preview available"
          description="The scan workflow is not available yet. No preview data was supplied, and no files have been changed."
        />
      ) : items.length === 0 ? (
        <EmptyState
          title="No cleanup candidates"
          description="This completed preview contains no candidate files. No files have been changed."
        />
      ) : (
        <>
          <section aria-label="Preview summary" className="cleaner-preview__summary-region">
            <dl className="cleaner-preview__summary">
              <div>
                <dt>Eligible items</dt>
                <dd>{eligibleItems.length}</dd>
              </div>
              <div>
                <dt>Estimated space</dt>
                <dd>{formatBytes(estimatedBytes)}</dd>
              </div>
              <div>
                <dt>Skipped items</dt>
                <dd>{skippedItems.length}</dd>
              </div>
            </dl>
          </section>

          <div className="home-dashboard__panel cleaner-preview__panel">
            <div className="home-dashboard__panel-heading">
              <div>
                <p className="eyebrow">PREVIEW / READ ONLY</p>
                <h3>Candidate files</h3>
              </div>
              <time dateTime={preview.generatedAt}>
                Preview created {formatModifiedAt(preview.generatedAt)}
              </time>
            </div>
            <div className="cleaner-preview__table-wrap">
              <table className="cleaner-preview__table">
                <caption>Cleanup candidates; this view does not change files.</caption>
                <thead>
                  <tr>
                    <th scope="col">Item</th>
                    <th scope="col">Category</th>
                    <th scope="col">Size</th>
                    <th scope="col">Last modified</th>
                    <th scope="col">Review status</th>
                  </tr>
                </thead>
                <tbody>
                  {items.map((item) => (
                    <tr key={item.candidateId}>
                      <th scope="row" title={item.displayName}>
                        {item.displayName}
                      </th>
                      <td>{categoryLabels[item.category]}</td>
                      <td>{formatBytes(item.sizeBytes)}</td>
                      <td>{formatModifiedAt(item.modifiedAt)}</td>
                      <td>
                        {item.disposition === "eligible" ? (
                          <Badge variant="success">Eligible for review</Badge>
                        ) : (
                          <Badge variant="warning">
                            Skipped:{" "}
                            {item.skipReason ? skipReasonLabels[item.skipReason] : "Policy check"}
                          </Badge>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <p className="cleaner-preview__estimate-note">
              Estimated space is not a promise of space recovered. This preview has no clean action.
            </p>
          </div>
        </>
      )}
    </section>
  );
}
