//! Versioned, secret-free export contract for the Personal Projects & Tasks data.

use chrono::DateTime;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const EXPORT_FORMAT: &str = "moonships.project-task-export";
pub const EXPORT_SCHEMA_VERSION: u32 = 1;
pub const MAX_EXPORT_SIZE_BYTES: usize = 50 * 1024 * 1024;
pub const MAX_EXPORT_RECORDS: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportProject {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub project_key: Option<String>,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: String,
    pub color_token: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportTask {
    pub id: String,
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_at: Option<String>,
    pub estimate_minutes: Option<i64>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportActivityEvent {
    pub id: String,
    pub actor_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub action: String,
    pub occurred_at: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectTaskExport {
    pub format: String,
    pub schema_version: u32,
    pub app_version: String,
    pub exported_at: String,
    pub projects: Vec<ExportProject>,
    pub tasks: Vec<ExportTask>,
    pub activity_events: Vec<ExportActivityEvent>,
}

impl ProjectTaskExport {
    pub fn new(
        app_version: &str,
        exported_at: &str,
        projects: Vec<ExportProject>,
        tasks: Vec<ExportTask>,
        activity_events: Vec<ExportActivityEvent>,
    ) -> Result<Self, String> {
        let document = Self {
            format: EXPORT_FORMAT.to_string(),
            schema_version: EXPORT_SCHEMA_VERSION,
            app_version: app_version.to_string(),
            exported_at: exported_at.to_string(),
            projects,
            tasks,
            activity_events,
        };
        document.validate()?;
        Ok(document)
    }

    pub fn validate(&self) -> Result<(), String> {
        let record_count = self
            .projects
            .len()
            .saturating_add(self.tasks.len())
            .saturating_add(self.activity_events.len());
        if record_count > MAX_EXPORT_RECORDS {
            return Err("VALIDATION_FAILED: export contains too many records".to_string());
        }
        if self.format != EXPORT_FORMAT || self.schema_version != EXPORT_SCHEMA_VERSION {
            return Err(
                "VALIDATION_FAILED: unsupported Project and Task export format".to_string(),
            );
        }
        if self.app_version.trim().is_empty()
            || DateTime::parse_from_rfc3339(&self.exported_at).is_err()
        {
            return Err("VALIDATION_FAILED: invalid export manifest metadata".to_string());
        }

        let project_ids: BTreeSet<&str> =
            self.projects.iter().map(|item| item.id.as_str()).collect();
        let task_ids: BTreeSet<&str> = self.tasks.iter().map(|item| item.id.as_str()).collect();
        if project_ids.len() != self.projects.len()
            || task_ids.len() != self.tasks.len()
            || project_ids.iter().any(|id| id.trim().is_empty())
            || task_ids.iter().any(|id| id.trim().is_empty())
        {
            return Err("VALIDATION_FAILED: duplicate or empty export record ID".to_string());
        }

        for project in &self.projects {
            if project.workspace_id != crate::PERSONAL_WORKSPACE_ID
                || project.name.trim().is_empty()
                || project.name.chars().count() > 120
                || project
                    .description
                    .as_ref()
                    .is_some_and(|text| text.chars().count() > 10_000)
                || project
                    .goal
                    .as_ref()
                    .is_some_and(|text| text.chars().count() > 4_000)
                || project
                    .project_key
                    .as_ref()
                    .is_some_and(|key| key.chars().count() > 40)
                || !matches!(
                    project.status.as_str(),
                    "planned" | "active" | "paused" | "completed" | "archived"
                )
                || (project.status == "archived") != project.archived_at.is_some()
                || DateTime::parse_from_rfc3339(&project.created_at).is_err()
                || DateTime::parse_from_rfc3339(&project.updated_at).is_err()
                || project
                    .archived_at
                    .as_deref()
                    .is_some_and(|date| DateTime::parse_from_rfc3339(date).is_err())
                || project.start_date.as_deref().is_some_and(|date| {
                    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err()
                })
                || project.target_date.as_deref().is_some_and(|date| {
                    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err()
                })
            {
                return Err("VALIDATION_FAILED: invalid exported Project record".to_string());
            }
        }
        for task in &self.tasks {
            if task.workspace_id != crate::PERSONAL_WORKSPACE_ID {
                return Err("VALIDATION_FAILED: exported Task has an invalid workspace".to_string());
            }
            crate::task_status::validate_status(&task.status)?;
            crate::task_status::validate_priority(&task.priority)?;
            if task.title.trim().is_empty()
                || task.title.chars().count() > 200
                || task
                    .description
                    .as_ref()
                    .is_some_and(|text| text.chars().count() > 10_000)
                || task.estimate_minutes.is_some_and(|value| value < 0)
                || task
                    .project_id
                    .as_deref()
                    .is_some_and(|id| !project_ids.contains(id))
                || task
                    .parent_task_id
                    .as_deref()
                    .is_some_and(|id| !task_ids.contains(id))
                || task.parent_task_id.as_deref() == Some(task.id.as_str())
                || DateTime::parse_from_rfc3339(&task.created_at).is_err()
                || DateTime::parse_from_rfc3339(&task.updated_at).is_err()
                || task
                    .due_at
                    .as_deref()
                    .is_some_and(|date| DateTime::parse_from_rfc3339(date).is_err())
                || task
                    .completed_at
                    .as_deref()
                    .is_some_and(|date| DateTime::parse_from_rfc3339(date).is_err())
                || task
                    .archived_at
                    .as_deref()
                    .is_some_and(|date| DateTime::parse_from_rfc3339(date).is_err())
            {
                return Err(
                    "VALIDATION_FAILED: invalid exported Task relationship or title".to_string(),
                );
            }
        }

        let mut event_ids = BTreeSet::new();
        for event in &self.activity_events {
            let valid_entity = match event.entity_type.as_str() {
                "project" => {
                    project_ids.contains(event.entity_id.as_str())
                        && event.project_id.as_deref() == Some(event.entity_id.as_str())
                        && event.task_id.is_none()
                }
                "task" => {
                    task_ids.contains(event.entity_id.as_str())
                        && event.task_id.as_deref() == Some(event.entity_id.as_str())
                        && event.project_id.is_none()
                }
                _ => false,
            };
            if event.id.trim().is_empty()
                || !event_ids.insert(event.id.as_str())
                || !matches!(event.actor_type.as_str(), "local" | "system" | "tool")
                || event.action.trim().is_empty()
                || event.summary.trim().is_empty()
                || !valid_entity
                || DateTime::parse_from_rfc3339(&event.occurred_at).is_err()
                || event
                    .project_id
                    .as_deref()
                    .is_some_and(|id| !project_ids.contains(id))
                || event
                    .task_id
                    .as_deref()
                    .is_some_and(|id| !task_ids.contains(id))
            {
                return Err("VALIDATION_FAILED: invalid exported activity record".to_string());
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|_| "IO_ERROR: could not serialize Project and Task export".to_string())?;
        if json.len() > MAX_EXPORT_SIZE_BYTES {
            return Err("VALIDATION_FAILED: export exceeds the supported size limit".to_string());
        }
        Self::from_json(&json).map_err(|_| {
            "VALIDATION_FAILED: generated export did not match its schema".to_string()
        })?;
        Ok(json)
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        Self::from_json_with_limit(json, MAX_EXPORT_SIZE_BYTES)
    }

    fn from_json_with_limit(json: &str, max_bytes: usize) -> Result<Self, String> {
        if json.len() > max_bytes {
            return Err("VALIDATION_FAILED: import file exceeds the 50 MiB size limit".to_string());
        }
        let document: Self = serde_json::from_str(json)
            .map_err(|_| "VALIDATION_FAILED: import file is not valid export JSON".to_string())?;
        document.validate()?;
        Ok(document)
    }
}

pub fn load_personal_export(
    connection: &Connection,
    app_version: &str,
) -> Result<ProjectTaskExport, String> {
    let projects = {
        let mut statement = connection
            .prepare(
                "SELECT id, workspace_id, name, project_key, description, goal, status,
                        color_token, start_date, target_date, archived_at, created_at, updated_at
                 FROM projects WHERE workspace_id = ?1 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|_| "IO_ERROR: could not prepare Project export".to_string())?;
        let rows = statement
            .query_map([crate::PERSONAL_WORKSPACE_ID], |row| {
                Ok(ExportProject {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    name: row.get(2)?,
                    project_key: row.get(3)?,
                    description: row.get(4)?,
                    goal: row.get(5)?,
                    status: row.get(6)?,
                    color_token: row.get(7)?,
                    start_date: row.get(8)?,
                    target_date: row.get(9)?,
                    archived_at: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })
            .map_err(|_| "IO_ERROR: could not read Projects for export".to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "IO_ERROR: could not read Projects for export".to_string())?
    };

    let tasks = {
        let mut statement = connection
            .prepare(
                "SELECT id, workspace_id, project_id, parent_task_id, title, description,
                        status, priority, due_at, estimate_minutes, completed_at,
                        created_at, updated_at, archived_at
                 FROM tasks WHERE workspace_id = ?1 ORDER BY created_at ASC, id ASC",
            )
            .map_err(|_| "IO_ERROR: could not prepare Task export".to_string())?;
        let rows = statement
            .query_map([crate::PERSONAL_WORKSPACE_ID], |row| {
                Ok(ExportTask {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    project_id: row.get(2)?,
                    parent_task_id: row.get(3)?,
                    title: row.get(4)?,
                    description: row.get(5)?,
                    status: row.get(6)?,
                    priority: row.get(7)?,
                    due_at: row.get(8)?,
                    estimate_minutes: row.get(9)?,
                    completed_at: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    archived_at: row.get(13)?,
                })
            })
            .map_err(|_| "IO_ERROR: could not query Tasks for export".to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "IO_ERROR: could not read Tasks for export".to_string())?
    };

    let activity_events = {
        let mut statement = connection
            .prepare(
                "SELECT id, actor_type, entity_type, entity_id, project_id, task_id,
                        action, occurred_at, summary
                 FROM activity_events
                 WHERE workspace_id = ?1 AND sensitivity = 'normal'
                 ORDER BY occurred_at ASC, id ASC",
            )
            .map_err(|_| "IO_ERROR: could not prepare activity export".to_string())?;
        let rows = statement
            .query_map([crate::PERSONAL_WORKSPACE_ID], |row| {
                Ok(ExportActivityEvent {
                    id: row.get(0)?,
                    actor_type: row.get(1)?,
                    entity_type: row.get(2)?,
                    entity_id: row.get(3)?,
                    project_id: row.get(4)?,
                    task_id: row.get(5)?,
                    action: row.get(6)?,
                    occurred_at: row.get(7)?,
                    summary: row.get(8)?,
                })
            })
            .map_err(|_| "IO_ERROR: could not query activity for export".to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "IO_ERROR: could not read activity for export".to_string())?
    };

    let exported_at: String = connection
        .query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now')", [], |row| {
            row.get(0)
        })
        .map_err(|_| "IO_ERROR: could not timestamp Project and Task export".to_string())?;
    ProjectTaskExport::new(app_version, &exported_at, projects, tasks, activity_events)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_document() -> ProjectTaskExport {
        ProjectTaskExport::new(
            "0.1.0-2",
            "2026-10-26T00:00:00Z",
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .expect("valid empty export")
    }

    #[test]
    fn export_includes_manifest_and_round_trips_against_strict_schema() {
        let document = empty_document();
        let json = document.to_json().expect("serialize schema-valid export");
        let parsed: ProjectTaskExport = serde_json::from_str(&json).expect("strict schema parse");
        assert_eq!(parsed.format, EXPORT_FORMAT);
        assert_eq!(parsed.schema_version, EXPORT_SCHEMA_VERSION);
        assert_eq!(parsed.app_version, "0.1.0-2");
        assert!(json.contains("\"projects\": []"));
        assert!(json.contains("\"tasks\": []"));
        assert!(!json.contains("credential"));
        assert!(!json.contains("appConfig"));
    }

    #[test]
    fn export_rejects_unknown_or_future_schema_versions() {
        let mut document = empty_document();
        document.schema_version += 1;
        assert!(document
            .validate()
            .unwrap_err()
            .contains("unsupported Project and Task export"));
        assert!(serde_json::from_str::<ProjectTaskExport>(
            r#"{"format":"moonships.project-task-export","schemaVersion":1,"appVersion":"1","exportedAt":"2026-10-26T00:00:00Z","projects":[],"tasks":[],"activityEvents":[],"unknown":true}"#
        ).is_err());
    }

    #[test]
    fn export_rejects_invalid_manifest_timestamps() {
        assert!(
            ProjectTaskExport::new("1", "not-a-time", Vec::new(), Vec::new(), Vec::new()).is_err()
        );
    }

    #[test]
    fn import_preview_parser_rejects_malformed_and_oversized_content() {
        assert!(ProjectTaskExport::from_json("{").is_err());
        assert!(ProjectTaskExport::from_json_with_limit("{}", 1)
            .unwrap_err()
            .contains("50 MiB size limit"));
    }
}
