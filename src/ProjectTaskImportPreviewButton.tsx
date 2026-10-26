import { useState } from "react";
import {
  previewProjectTaskImport,
  type ProjectTaskImportPreviewResult,
} from "./project-task-import-preview-api";
import { Button } from "./ui-primitives";

function summarize(result: ProjectTaskImportPreviewResult): string {
  const projects = result.projectNames.length
    ? ` Projects: ${result.projectNames.join(", ")}.`
    : "";
  const tasks = result.taskTitles.length ? ` Tasks: ${result.taskTitles.join(", ")}.` : "";
  return `Valid export preview: ${result.projectCount} projects, ${result.taskCount} tasks, and ${result.activityEventCount} activity events. Schema v${result.schemaVersion}, Moonships ${result.appVersion}.${projects}${tasks} No records were imported.`;
}

export function ProjectTaskImportPreviewButton() {
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<{ text: string; error: boolean } | null>(null);

  const preview = async () => {
    if (loading) return;
    setLoading(true);
    setMessage(null);
    try {
      const result = await previewProjectTaskImport();
      setMessage(
        result.cancelled
          ? { text: "Preview cancelled. No records were imported.", error: false }
          : { text: summarize(result), error: false },
      );
    } catch {
      setMessage({
        text: "Could not preview this file. It may be malformed or too large. No records were imported.",
        error: true,
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="project-task-export">
      <Button variant="secondary" size="sm" onClick={() => void preview()} disabled={loading}>
        {loading ? "Checking file…" : "Preview import"}
      </Button>
      <p className="project-task-export__note">
        Previews a Moonships JSON export only. Nothing is imported or changed.
      </p>
      {message ? <span role={message.error ? "alert" : "status"}>{message.text}</span> : null}
    </div>
  );
}
