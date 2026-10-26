import { useCallback, useEffect, useRef, useState } from "react";
import {
  listCleanupReports,
  toCleanupReportErrorMessage,
  type CleanupReportStatus,
  type CleanupReportSummary,
} from "./cleanup-reports-api";
import { Badge, Button, Dialog, EmptyState, ErrorState, LoadingState } from "./ui-primitives";
import type {
  CleanupApprovalRequest,
  CleanupCategory,
  CleanupPreviewDto,
  CleanupSkipReason,
} from "./cleanup-preview";

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

function reportStatusLabel(status: CleanupReportStatus): string {
  if (status === "succeeded") return "Completed";
  if (status === "partial") return "Partial";
  return "Failed";
}

function reportStatusVariant(status: CleanupReportStatus): "success" | "warning" | "danger" {
  if (status === "succeeded") return "success";
  if (status === "partial") return "warning";
  return "danger";
}

export function CleanerPreviewPage({
  preview,
  onApprove,
}: {
  preview: CleanupPreviewDto | null;
  onApprove?: (request: CleanupApprovalRequest) => void;
}) {
  return (
    <CleanerPreviewContent
      key={preview?.previewId ?? "no-preview"}
      preview={preview}
      onApprove={onApprove}
    />
  );
}

function CleanerPreviewContent({
  preview,
  onApprove,
}: {
  preview: CleanupPreviewDto | null;
  onApprove?: (request: CleanupApprovalRequest) => void;
}) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => new Set());
  const [approvalOpen, setApprovalOpen] = useState(false);
  const [approvalRecorded, setApprovalRecorded] = useState<CleanupApprovalRequest | null>(null);
  const [reports, setReports] = useState<CleanupReportSummary[]>([]);
  const [reportsLoading, setReportsLoading] = useState(true);
  const [reportsError, setReportsError] = useState<string | null>(null);
  const reportsRequestId = useRef(0);

  const loadReports = useCallback(() => {
    const requestId = ++reportsRequestId.current;
    setReportsLoading(true);
    setReportsError(null);
    listCleanupReports()
      .then((nextReports) => {
        if (reportsRequestId.current === requestId) setReports(nextReports);
      })
      .catch(() => {
        if (reportsRequestId.current === requestId) {
          setReportsError(toCleanupReportErrorMessage());
        }
      })
      .finally(() => {
        if (reportsRequestId.current === requestId) setReportsLoading(false);
      });
  }, []);

  useEffect(() => {
    loadReports();
    return () => {
      reportsRequestId.current += 1;
    };
  }, [loadReports]);

  const items = preview?.items ?? [];
  const eligibleItems = items.filter((item) => item.disposition === "eligible");
  const skippedItems = items.filter((item) => item.disposition === "skipped");
  const estimatedBytes = eligibleItems.reduce((total, item) => total + item.sizeBytes, 0);
  const selectedItems = eligibleItems.filter((item) => selectedIds.has(item.candidateId));
  const selectedBytes = selectedItems.reduce((total, item) => total + item.sizeBytes, 0);

  const recordApproval = () => {
    if (!preview || selectedItems.length === 0) return;
    const request: CleanupApprovalRequest = {
      previewId: preview.previewId,
      candidateIds: selectedItems.map((item) => item.candidateId),
      estimatedBytes: selectedBytes,
    };
    onApprove?.(request);
    setApprovalRecorded(request);
    setApprovalOpen(false);
  };

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
                    <th scope="col">Select</th>
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
                      <td>
                        <input
                          type="checkbox"
                          aria-label={`Select ${item.displayName}`}
                          checked={selectedIds.has(item.candidateId)}
                          disabled={item.disposition !== "eligible"}
                          onChange={(event) => {
                            setSelectedIds((current) => {
                              const next = new Set(current);
                              if (event.target.checked) next.add(item.candidateId);
                              else next.delete(item.candidateId);
                              return next;
                            });
                            setApprovalRecorded(null);
                          }}
                        />
                      </td>
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
            <div className="cleaner-preview__approval-actions">
              <span aria-live="polite">
                {selectedItems.length} eligible item{selectedItems.length === 1 ? "" : "s"} selected
              </span>
              <Button
                variant="secondary"
                disabled={selectedItems.length === 0}
                onClick={() => setApprovalOpen(true)}
              >
                Review selected actions
              </Button>
            </div>
            {approvalRecorded ? (
              <p className="cleaner-preview__approval-note" role="status">
                Approval recorded for {approvalRecorded.candidateIds.length} item(s) in this
                preview. Cleanup has not started; no files have been changed.
              </p>
            ) : null}
            <p className="cleaner-preview__estimate-note">
              Estimated space is not a promise of space recovered. Approval alone does not change
              files.
            </p>
          </div>
        </>
      )}
      <section
        className="home-dashboard__panel cleaner-preview__reports"
        aria-labelledby="cleanup-reports-title"
      >
        <div className="home-dashboard__panel-heading">
          <div>
            <p className="eyebrow">LOCAL HISTORY</p>
            <h3 id="cleanup-reports-title">Recent cleanup results</h3>
          </div>
        </div>
        {reportsLoading ? (
          <LoadingState label="Loading saved cleanup results" />
        ) : reportsError ? (
          <ErrorState
            title="Cleanup history unavailable"
            description={reportsError}
            onRetry={loadReports}
          />
        ) : reports.length === 0 ? (
          <EmptyState
            title="No saved cleanup results"
            description="Completed cleanup summaries will appear here. This history does not store file paths."
          />
        ) : (
          <div className="cleaner-preview__table-wrap">
            <table className="cleaner-preview__table cleaner-preview__reports-table">
              <caption>Saved summaries only; candidate file paths are not stored.</caption>
              <thead>
                <tr>
                  <th scope="col">Result</th>
                  <th scope="col">Completed</th>
                  <th scope="col">Items</th>
                  <th scope="col">Estimated</th>
                  <th scope="col">Deleted file size</th>
                  <th scope="col">Task</th>
                </tr>
              </thead>
              <tbody>
                {reports.map((report) => (
                  <tr key={report.id}>
                    <td>
                      <Badge variant={reportStatusVariant(report.status)}>
                        {reportStatusLabel(report.status)}
                      </Badge>
                    </td>
                    <td>
                      <time dateTime={report.completedAt}>
                        {formatModifiedAt(report.completedAt)}
                      </time>
                    </td>
                    <td>
                      {report.deletedCount} deleted · {report.skippedCount} skipped ·{" "}
                      {report.failedCount} failed
                    </td>
                    <td>{formatBytes(report.estimatedBytes)}</td>
                    <td>{formatBytes(report.actualDeletedBytes)}</td>
                    <td>
                      {report.taskId ? (
                        <a href={`#/tasks/${encodeURIComponent(report.taskId)}`}>
                          Open linked task
                        </a>
                      ) : (
                        <span className="muted">None</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <p className="cleaner-preview__estimate-note">
              “Deleted file size” totals logical sizes of removed files; physical disk space
              reclaimed may differ.
            </p>
          </div>
        )}
      </section>
      <Dialog
        open={approvalOpen}
        title="Approve selected cleanup items"
        description="Confirm the exact files and estimated space covered by this approval."
        onClose={() => setApprovalOpen(false)}
      >
        <div className="cleaner-preview__approval-dialog">
          <p>
            {selectedItems.length} eligible item(s) · {formatBytes(selectedBytes)} estimated
          </p>
          <ul>
            {selectedItems.map((item) => (
              <li key={item.candidateId}>{item.displayName}</li>
            ))}
          </ul>
          <p>
            This approval is limited to the selected IDs in this preview. Cleanup is not wired to
            this screen yet; approving will not delete files.
          </p>
          <div className="cleaner-preview__approval-dialog-actions">
            <Button variant="secondary" onClick={() => setApprovalOpen(false)}>
              Cancel
            </Button>
            <Button onClick={recordApproval}>Approve selected items</Button>
          </div>
        </div>
      </Dialog>
    </section>
  );
}
