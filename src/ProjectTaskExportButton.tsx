import { useState } from "react";
import { exportProjectTaskData } from "./project-task-export-api";
import { Button } from "./ui-primitives";

export function ProjectTaskExportButton() {
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ text: string; error: boolean } | null>(null);

  const exportData = async () => {
    if (saving) return;
    setSaving(true);
    setMessage(null);
    try {
      const result = await exportProjectTaskData();
      setMessage(
        result.cancelled
          ? { text: "Export cancelled. No file was saved.", error: false }
          : {
              text: `Saved ${result.projectCount} projects and ${result.taskCount} tasks${result.fileName ? ` to ${result.fileName}` : ""}.`,
              error: false,
            },
      );
    } catch {
      setMessage({ text: "Export failed. No success was reported; try again.", error: true });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="project-task-export">
      <Button variant="secondary" size="sm" onClick={() => void exportData()} disabled={saving}>
        {saving ? "Preparing export…" : "Export Projects & Tasks"}
      </Button>
      <p className="project-task-export__note">
        Includes saved Project and Task descriptions. App settings and stored credentials are not
        included; review authored content before sharing.
      </p>
      {message ? <span role={message.error ? "alert" : "status"}>{message.text}</span> : null}
    </div>
  );
}
