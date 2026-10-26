//! Local Task creation service. All writes stay inside the caller-provided,
//! foreign-key-enabled SQLite connection; no UI or global database path access.

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskArchiveFilter {
    Active,
    Archived,
    All,
}

const MAX_TITLE_CHARS: usize = 200;
const MAX_DESCRIPTION_CHARS: usize = 10_000;

/// Normalize explicit-offset RFC 3339 instants to canonical UTC without losing precision.
fn normalize_due_at(value: Option<String>) -> Result<Option<String>, String> {
    value
        .map(|value| {
            let value = value.trim();
            if value.is_empty() {
                return Ok(None);
            }
            let parsed = DateTime::parse_from_rfc3339(value).map_err(|_| {
                "VALIDATION_FAILED: dueAt must be an RFC 3339 timestamp with an explicit timezone".to_string()
            })?;
            Ok(Some(
                parsed
                    .with_timezone(&Utc)
                    .to_rfc3339_opts(SecondsFormat::AutoSi, true),
            ))
        })
        .transpose()
        .map(Option::flatten)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTask {
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Task {
    pub id: String,
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

fn task_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        project_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        status: row.get(5)?,
        priority: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        due_at: row.get(9)?,
        archived_at: row.get(10)?,
    })
}

/// Return persisted tasks with an explicit active/archived filter and stable ordering.
pub fn list_tasks(
    connection: &Connection,
    workspace_id: &str,
    filter: TaskArchiveFilter,
) -> Result<Vec<Task>, String> {
    if workspace_id.trim().is_empty() {
        return Err("VALIDATION_FAILED: workspace required".to_string());
    }
    let archive_clause = match filter {
        TaskArchiveFilter::Active => "AND archived_at IS NULL",
        TaskArchiveFilter::Archived => "AND archived_at IS NOT NULL",
        TaskArchiveFilter::All => "",
    };
    let sql = format!("SELECT id, workspace_id, project_id, title, description, status, priority, created_at, updated_at, due_at, archived_at FROM tasks WHERE workspace_id = ?1 {archive_clause} ORDER BY created_at, id");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|_| "IO_ERROR: could not prepare task list".to_string())?;
    let rows = statement
        .query_map(params![workspace_id.trim()], task_from_row)
        .map_err(|_| "IO_ERROR: could not list tasks".to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "IO_ERROR: could not read task list".to_string())
}

/// Counts non-archived tasks currently linked to a project in the given workspace.
pub fn count_active_project_tasks(
    connection: &Connection,
    workspace_id: &str,
    project_id: &str,
) -> Result<u64, String> {
    let workspace_id = workspace_id.trim();
    let project_id = project_id.trim();
    if workspace_id.is_empty() || project_id.is_empty() {
        return Err("VALIDATION_FAILED: project and workspace IDs are required".to_string());
    }
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM tasks
             WHERE workspace_id = ?1 AND project_id = ?2 AND archived_at IS NULL",
            params![workspace_id, project_id],
            |row| row.get(0),
        )
        .map_err(|_| "IO_ERROR: could not count project tasks".to_string())?;
    u64::try_from(count).map_err(|_| "IO_ERROR: invalid project task count".to_string())
}

fn mutate_archive(
    connection: &mut Connection,
    workspace_id: &str,
    id: &str,
    archive: bool,
) -> Result<Task, String> {
    let workspace_id = workspace_id.trim();
    let id = id.trim();
    if workspace_id.is_empty() || id.is_empty() {
        return Err("VALIDATION_FAILED: task and workspace required".to_string());
    }
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin task archive transaction".to_string())?;
    let (existing, archived_at) = read_task(&transaction, workspace_id, id)?;
    if archive == archived_at.is_some() {
        return Ok(existing);
    }
    let workspace_active: Option<i64> = transaction
        .query_row(
            "SELECT 1 FROM workspaces WHERE id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read task workspace".to_string())?;
    if workspace_active.is_none() {
        return Err("NOT_FOUND: workspace is archived or unavailable".to_string());
    }
    if !archive {
        if let Some(project_id) = existing.project_id.as_deref() {
            let project_active: Option<i64> = transaction.query_row("SELECT 1 FROM projects WHERE id = ?1 AND workspace_id = ?2 AND archived_at IS NULL AND status != 'archived'", params![project_id, workspace_id], |row| row.get(0)).optional().map_err(|_| "IO_ERROR: could not read task project".to_string())?;
            if project_active.is_none() {
                return Err(
                    "CONFLICT: restore the task's project before restoring this task".to_string(),
                );
            }
        }
    }
    transaction.execute("UPDATE tasks SET archived_at = CASE WHEN ?1 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE NULL END, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2 AND workspace_id = ?3 AND archived_at IS ?4", params![archive, id, workspace_id, archived_at]).map_err(|_| "IO_ERROR: could not change task archive state".to_string())?;
    crate::activity::record_change(
        &transaction,
        workspace_id,
        crate::activity::ActivityEntityKind::Task,
        id,
        if archive {
            crate::activity::ActivityAction::Archived
        } else {
            crate::activity::ActivityAction::Restored
        },
    )?;
    let task = transaction.query_row("SELECT id, workspace_id, project_id, title, description, status, priority, created_at, updated_at, due_at, archived_at FROM tasks WHERE id = ?1 AND workspace_id = ?2", params![id, workspace_id], task_from_row).map_err(|_| "IO_ERROR: could not read changed task".to_string())?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit task archive state".to_string())?;
    Ok(task)
}

pub fn archive_task(
    connection: &mut Connection,
    workspace_id: &str,
    id: &str,
) -> Result<Task, String> {
    mutate_archive(connection, workspace_id, id, true)
}
pub fn restore_task(
    connection: &mut Connection,
    workspace_id: &str,
    id: &str,
) -> Result<Task, String> {
    mutate_archive(connection, workspace_id, id, false)
}

pub fn create_task(connection: &mut Connection, request: CreateTask) -> Result<Task, String> {
    let workspace_id = request.workspace_id.trim();
    let title = request.title.trim();
    if workspace_id.is_empty() || title.is_empty() || title.chars().count() > MAX_TITLE_CHARS {
        return Err("VALIDATION_FAILED: valid workspace and task title required".to_string());
    }

    let status = request.status.as_deref().unwrap_or("backlog");
    crate::task_status::validate_status(status)?;
    let priority = request.priority.as_deref().unwrap_or("p2");
    crate::task_status::validate_priority(priority)?;

    let project_id = request.project_id.map(|id| id.trim().to_string());
    if project_id.as_deref() == Some("") {
        return Err("VALIDATION_FAILED: empty project ID".to_string());
    }
    let description = request
        .description
        .map(|description| {
            let description = description.trim();
            if description.chars().count() > MAX_DESCRIPTION_CHARS {
                Err("VALIDATION_FAILED: task description is too long".to_string())
            } else {
                Ok((!description.is_empty()).then(|| description.to_string()))
            }
        })
        .transpose()?
        .flatten();
    let due_at = normalize_due_at(request.due_at)?;

    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin task transaction".to_string())?;
    let workspace_exists: Option<i64> = transaction
        .query_row(
            "SELECT 1 FROM workspaces WHERE id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read task workspace".to_string())?;
    if workspace_exists.is_none() {
        return Err("NOT_FOUND: workspace does not exist or is archived".to_string());
    }
    if let Some(ref project_id) = project_id {
        let project_exists: Option<i64> = transaction
            .query_row(
                "SELECT 1 FROM projects
                 WHERE id = ?1 AND workspace_id = ?2 AND archived_at IS NULL",
                params![project_id, workspace_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "IO_ERROR: could not read task project".to_string())?;
        if project_exists.is_none() {
            return Err("NOT_FOUND: project is unavailable in this workspace".to_string());
        }
    }

    let id = Uuid::new_v4().to_string();
    transaction
        .execute(
            "INSERT INTO tasks
                (id, workspace_id, project_id, title, description, status, priority, due_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                workspace_id,
                project_id,
                title,
                description,
                status,
                priority,
                due_at
            ],
        )
        .map_err(|_| "IO_ERROR: could not create task".to_string())?;
    crate::activity::record_creation(
        &transaction,
        workspace_id,
        crate::activity::ActivityEntityKind::Task,
        &id,
    )?;

    let created = transaction
        .query_row(
            "SELECT id, workspace_id, project_id, title, description,
                    status, priority, created_at, updated_at, due_at, archived_at
             FROM tasks WHERE id = ?1 AND workspace_id = ?2",
            params![id, workspace_id],
            task_from_row,
        )
        .map_err(|_| "IO_ERROR: could not read created task".to_string())?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit created task".to_string())?;
    Ok(created)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateTask {
    pub workspace_id: String,
    pub id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<String>,
    pub status: String,
    pub priority: String,
}

fn read_task(
    connection: &Connection,
    workspace_id: &str,
    id: &str,
) -> Result<(Task, Option<String>), String> {
    connection
        .query_row(
            "SELECT id, workspace_id, project_id, title, description,
                    status, priority, created_at, updated_at, due_at, archived_at
             FROM tasks WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, id],
            |row| {
                Ok((
                    Task {
                        id: row.get(0)?,
                        workspace_id: row.get(1)?,
                        project_id: row.get(2)?,
                        title: row.get(3)?,
                        description: row.get(4)?,
                        status: row.get(5)?,
                        priority: row.get(6)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                        due_at: row.get(9)?,
                        archived_at: row.get(10)?,
                    },
                    row.get(10)?,
                ))
            },
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read task".to_string())?
        .ok_or_else(|| "NOT_FOUND: task not found in workspace".to_string())
}

pub fn update_task(connection: &mut Connection, request: UpdateTask) -> Result<Task, String> {
    let workspace_id = request.workspace_id.trim();
    let id = request.id.trim();
    let title = request.title.trim();
    if workspace_id.is_empty()
        || id.is_empty()
        || title.is_empty()
        || title.chars().count() > MAX_TITLE_CHARS
    {
        return Err("VALIDATION_FAILED: task, workspace and valid title required".to_string());
    }
    crate::task_status::validate_status(&request.status)?;
    crate::task_status::validate_priority(&request.priority)?;
    let project_id = request.project_id.map(|value| value.trim().to_string());
    if project_id.as_deref() == Some("") {
        return Err("VALIDATION_FAILED: empty project ID".to_string());
    }
    let description = request
        .description
        .map(|value| {
            let trimmed = value.trim();
            if trimmed.chars().count() > MAX_DESCRIPTION_CHARS {
                Err("VALIDATION_FAILED: task description is too long".to_string())
            } else {
                Ok((!trimmed.is_empty()).then(|| trimmed.to_string()))
            }
        })
        .transpose()?
        .flatten();
    let due_at = normalize_due_at(request.due_at)?;

    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin task update".to_string())?;
    let (existing, archived_at) = read_task(&transaction, workspace_id, id)?;
    if archived_at.is_some() {
        return Err("NOT_FOUND: task is archived".to_string());
    }
    let workspace_active: Option<i64> = transaction
        .query_row(
            "SELECT 1 FROM workspaces WHERE id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read task workspace".to_string())?;
    if workspace_active.is_none() {
        return Err("NOT_FOUND: workspace is archived or unavailable".to_string());
    }
    if let Some(ref project_id) = project_id {
        let project_active: Option<i64> = transaction
            .query_row(
                "SELECT 1 FROM projects
                 WHERE id = ?1 AND workspace_id = ?2 AND archived_at IS NULL
                   AND status != 'archived'",
                params![project_id, workspace_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "IO_ERROR: could not read task project".to_string())?;
        if project_active.is_none() {
            return Err("NOT_FOUND: project is unavailable in this workspace".to_string());
        }
    }
    crate::task_status::validate_transition(&existing.status, &request.status)?;
    if existing.project_id == project_id
        && existing.title == title
        && existing.description == description
        && existing.due_at == due_at
        && existing.status == request.status
        && existing.priority == request.priority
    {
        return Ok(existing);
    }

    let changed = transaction
        .execute(
            "UPDATE tasks
             SET project_id = ?1, title = ?2, description = ?3, due_at = ?4,
                 status = ?5, priority = ?6,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?7 AND workspace_id = ?8 AND archived_at IS NULL",
            params![
                project_id,
                title,
                description,
                due_at,
                request.status,
                request.priority,
                id,
                workspace_id
            ],
        )
        .map_err(|_| "IO_ERROR: could not update task".to_string())?;
    if changed == 0 {
        return Err("NOT_FOUND: task is unavailable".to_string());
    }
    crate::activity::record_change(
        &transaction,
        workspace_id,
        crate::activity::ActivityEntityKind::Task,
        id,
        crate::activity::ActivityAction::Updated,
    )?;
    let (updated, _) = read_task(&transaction, workspace_id, id)?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit task update".to_string())?;
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERSONAL: &str = "00000000-0000-4000-8000-000000000001";

    fn fixture() -> Connection {
        let mut connection = Connection::open_in_memory().expect("disposable memory database");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
        crate::run_migrations(&mut connection).expect("schema");
        connection
    }

    fn request() -> CreateTask {
        CreateTask {
            workspace_id: PERSONAL.to_string(),
            project_id: None,
            title: "  Write tests  ".to_string(),
            description: None,
            due_at: None,
            status: None,
            priority: None,
        }
    }

    #[test]
    fn title_only_task_creates_personal_inbox_record_with_defaults() {
        let mut connection = fixture();
        let created = create_task(&mut connection, request()).expect("create inbox task");
        assert!(Uuid::parse_str(&created.id).is_ok());
        assert_eq!(created.workspace_id, PERSONAL);
        assert_eq!(created.project_id, None);
        assert_eq!(created.title, "Write tests");
        assert_eq!(created.status, "backlog");
        assert_eq!(created.priority, "p2");
        assert!(!created.created_at.is_empty());
        assert!(!created.updated_at.is_empty());
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn task_links_only_to_a_valid_project_in_its_own_workspace() {
        let mut connection = fixture();
        connection
            .execute_batch(
                "INSERT INTO projects (id, workspace_id, name)
                 VALUES ('project-a', '00000000-0000-4000-8000-000000000001', 'Allowed');
                 INSERT INTO workspaces (id, name) VALUES ('other-workspace', 'Other');
                 INSERT INTO projects (id, workspace_id, name)
                 VALUES ('project-b', 'other-workspace', 'Other project');
                 INSERT INTO projects (id, workspace_id, name, archived_at)
                 VALUES ('archived-project', '00000000-0000-4000-8000-000000000001',
                         'Archived', '2026-01-01');",
            )
            .unwrap();
        let mut valid = request();
        valid.project_id = Some("project-a".to_string());
        valid.priority = Some("p1".to_string());
        let created = create_task(&mut connection, valid).expect("task in owned project");
        assert_eq!(created.project_id.as_deref(), Some("project-a"));
        assert_eq!(created.priority, "p1");

        for unavailable_project in ["project-b", "missing", "archived-project"] {
            let mut invalid = request();
            invalid.project_id = Some(unavailable_project.to_string());
            assert!(create_task(&mut connection, invalid)
                .unwrap_err()
                .starts_with("NOT_FOUND:"));
        }
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn project_task_count_includes_only_non_archived_links_in_workspace_scope() {
        let mut connection = fixture();
        connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name) VALUES ('project-a', ?1, 'Project')",
                params![PERSONAL],
            )
            .expect("project fixture");
        let mut linked = request();
        linked.project_id = Some("project-a".to_string());
        let first = create_task(&mut connection, linked.clone()).expect("first linked task");
        linked.title = "Second linked task".to_string();
        create_task(&mut connection, linked).expect("second linked task");
        create_task(&mut connection, request()).expect("inbox task");
        assert_eq!(
            count_active_project_tasks(&connection, PERSONAL, "project-a").unwrap(),
            2
        );

        archive_task(&mut connection, PERSONAL, &first.id).expect("archive one linked task");
        assert_eq!(
            count_active_project_tasks(&connection, PERSONAL, "project-a").unwrap(),
            1
        );
        assert_eq!(
            count_active_project_tasks(&connection, "other-workspace", "project-a").unwrap(),
            0
        );
    }

    #[test]
    fn invalid_task_fields_and_workspace_fail_without_writes() {
        let mut connection = fixture();
        let mut invalid = request();
        invalid.title = "   ".to_string();
        assert!(create_task(&mut connection, invalid)
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));
        let mut invalid = request();
        invalid.status = Some("unknown".to_string());
        assert!(create_task(&mut connection, invalid)
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));
        let mut invalid = request();
        invalid.priority = Some("critical".to_string());
        assert!(create_task(&mut connection, invalid)
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));
        let mut invalid = request();
        invalid.workspace_id = "not-found".to_string();
        assert!(create_task(&mut connection, invalid)
            .unwrap_err()
            .starts_with("NOT_FOUND:"));
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn update_task_persists_fields_and_safe_activity_in_one_workspace() {
        let mut connection = fixture();
        connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name)
                 VALUES ('project-a', ?1, 'Active project')",
                params![PERSONAL],
            )
            .expect("project");
        let created = create_task(&mut connection, request()).expect("inbox task");
        let updated = update_task(
            &mut connection,
            UpdateTask {
                workspace_id: PERSONAL.to_string(),
                id: created.id.clone(),
                project_id: Some("project-a".to_string()),
                title: "  Write real task tests  ".to_string(),
                description: Some("sensitive task details".to_string()),
                due_at: None,
                status: "in_progress".to_string(),
                priority: "p0".to_string(),
            },
        )
        .expect("update");
        assert_eq!(updated.project_id.as_deref(), Some("project-a"));
        assert_eq!(updated.title, "Write real task tests");
        assert_eq!(updated.status, "in_progress");
        assert_eq!(updated.priority, "p0");
        assert_eq!(updated.created_at, created.created_at);
        let (events, action, summary): (i64, String, String) = connection
            .query_row(
                "SELECT (SELECT COUNT(*) FROM activity_events WHERE entity_id = ?1),
                        action, summary
                 FROM activity_events WHERE entity_id = ?1
                 ORDER BY rowid DESC LIMIT 1",
                params![created.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("task activity");
        assert_eq!(events, 2);
        assert_eq!(action, "updated");
        assert_eq!(summary, "Task updated");
        assert!(!summary.contains("sensitive"));
    }

    #[test]
    fn task_update_no_op_preserves_timestamps_and_event_count() {
        let mut connection = fixture();
        let created = create_task(&mut connection, request()).expect("task");
        let unchanged = update_task(
            &mut connection,
            UpdateTask {
                workspace_id: PERSONAL.to_string(),
                id: created.id.clone(),
                project_id: None,
                title: " Write tests ".to_string(),
                description: None,
                due_at: None,
                status: "backlog".to_string(),
                priority: "p2".to_string(),
            },
        )
        .expect("no-op");
        assert_eq!(unchanged, created);
        let events: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM activity_events WHERE entity_id = ?1",
                params![created.id],
                |row| row.get(0),
            )
            .expect("events");
        assert_eq!(events, 1);
    }

    #[test]
    fn task_update_rejects_bad_values_missing_records_and_cross_workspace() {
        let mut connection = fixture();
        let created = create_task(&mut connection, request()).expect("task");
        connection
            .execute_batch(
                "INSERT INTO workspaces (id, name) VALUES ('other-workspace', 'Other');
                 INSERT INTO projects (id, workspace_id, name)
                 VALUES ('other-project', 'other-workspace', 'Other');",
            )
            .expect("second workspace");
        let mut change = UpdateTask {
            workspace_id: PERSONAL.to_string(),
            id: created.id.clone(),
            project_id: None,
            title: "Edited".to_string(),
            description: None,
            due_at: None,
            status: "todo".to_string(),
            priority: "p1".to_string(),
        };
        change.status = "invalid".to_string();
        assert!(update_task(&mut connection, change.clone())
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));
        change.status = "todo".to_string();
        change.priority = "urgent".to_string();
        assert!(update_task(&mut connection, change.clone())
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));
        change.priority = "p1".to_string();
        change.project_id = Some("other-project".to_string());
        assert!(update_task(&mut connection, change.clone())
            .unwrap_err()
            .starts_with("NOT_FOUND:"));
        change.project_id = None;
        change.id = "missing-task".to_string();
        assert!(update_task(&mut connection, change.clone())
            .unwrap_err()
            .starts_with("NOT_FOUND:"));
        change.id = created.id.clone();
        change.workspace_id = "other-workspace".to_string();
        assert!(update_task(&mut connection, change)
            .unwrap_err()
            .starts_with("NOT_FOUND:"));
        let (saved, _) = read_task(&connection, PERSONAL, &created.id).expect("unchanged");
        assert_eq!(saved, created);
    }

    #[test]
    fn failed_task_activity_write_rolls_back_update() {
        let mut connection = fixture();
        let created = create_task(&mut connection, request()).expect("task");
        connection
            .execute_batch(
                "CREATE TRIGGER reject_task_activity
                 BEFORE INSERT ON activity_events
                 BEGIN
                     SELECT RAISE(ABORT, 'synthetic activity failure');
                 END;",
            )
            .expect("failure injector");
        let error = update_task(
            &mut connection,
            UpdateTask {
                workspace_id: PERSONAL.to_string(),
                id: created.id.clone(),
                project_id: None,
                title: "Should not persist".to_string(),
                description: None,
                due_at: None,
                status: "done".to_string(),
                priority: "p0".to_string(),
            },
        )
        .unwrap_err();
        assert!(error.starts_with("IO_ERROR:"));
        let (unchanged, _) = read_task(&connection, PERSONAL, &created.id).expect("rollback");
        assert_eq!(unchanged, created);
        let events: i64 = connection
            .query_row("SELECT COUNT(*) FROM activity_events", [], |row| row.get(0))
            .expect("activity count");
        assert_eq!(events, 1);
    }

    #[test]
    fn due_at_normalizes_rfc3339_offsets_to_utc_and_round_trips_json() {
        let mut connection = fixture();
        let mut create = request();
        create.due_at = Some("2026-10-26T12:34:56.123456789+07:00".to_string());
        let created = create_task(&mut connection, create).expect("create with due time");
        let expected = "2026-10-26T05:34:56.123456789Z";
        assert_eq!(created.due_at.as_deref(), Some(expected));

        let stored: Option<String> = connection
            .query_row(
                "SELECT due_at FROM tasks WHERE id = ?1",
                params![created.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored.as_deref(), Some(expected));
        let serialized = serde_json::to_value(&created).expect("serialize task DTO");
        assert_eq!(serialized["dueAt"], expected);
        assert!(serialized.get("due_at").is_none());
        let round_tripped: Task = serde_json::from_value(serialized).expect("deserialize task DTO");
        assert_eq!(round_tripped, created);
        assert_eq!(
            list_tasks(&connection, PERSONAL, TaskArchiveFilter::Active).unwrap(),
            vec![created]
        );
    }

    #[test]
    fn due_at_update_normalizes_equivalent_instants_and_no_op_adds_no_activity() {
        let mut connection = fixture();
        let created = create_task(&mut connection, request()).unwrap();
        let mut change = UpdateTask {
            workspace_id: PERSONAL.to_string(),
            id: created.id.clone(),
            project_id: None,
            title: created.title.clone(),
            description: None,
            due_at: Some("2026-10-26T12:00:00+07:00".to_string()),
            status: created.status.clone(),
            priority: created.priority.clone(),
        };
        let updated = update_task(&mut connection, change.clone()).expect("set due time");
        assert_eq!(updated.due_at.as_deref(), Some("2026-10-26T05:00:00Z"));
        change.due_at = Some("2026-10-26T05:00:00Z".to_string());
        assert_eq!(update_task(&mut connection, change).unwrap(), updated);
        let events: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM activity_events WHERE task_id = ?1",
                params![created.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(events, 2);
    }

    #[test]
    fn invalid_due_at_values_are_rejected_without_writes() {
        let mut connection = fixture();
        for value in [
            "2026-10-26",
            "2026-10-26T12:30:00",
            "2026-10-26T12:30:00+25:00",
            "not-a-time",
        ] {
            let mut invalid = request();
            invalid.due_at = Some(value.to_string());
            assert!(create_task(&mut connection, invalid)
                .unwrap_err()
                .starts_with("VALIDATION_FAILED:"));
        }
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn empty_due_at_clears_due_time_and_invalid_update_preserves_saved_value() {
        let mut connection = fixture();
        let mut create = request();
        create.due_at = Some("2026-10-26T05:00:00Z".to_string());
        let created = create_task(&mut connection, create).unwrap();
        let mut change = UpdateTask {
            workspace_id: PERSONAL.to_string(),
            id: created.id.clone(),
            project_id: None,
            title: created.title.clone(),
            description: None,
            due_at: Some("2026-10-26T12:00:00".to_string()),
            status: created.status.clone(),
            priority: created.priority.clone(),
        };
        assert!(update_task(&mut connection, change.clone())
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));
        assert_eq!(
            read_task(&connection, PERSONAL, &created.id).unwrap().0,
            created
        );
        change.due_at = Some("  ".to_string());
        let cleared = update_task(&mut connection, change).expect("clear due time");
        assert_eq!(cleared.due_at, None);
    }

    #[test]
    fn archive_restore_are_idempotent_preserve_data_and_filter_records() {
        let mut connection = fixture();
        let mut request = request();
        request.description = Some("private description remains on task".to_string());
        request.status = Some("in_progress".to_string());
        request.priority = Some("p1".to_string());
        let created = create_task(&mut connection, request).expect("task");
        let archived = archive_task(&mut connection, PERSONAL, &created.id).expect("archive");
        assert!(archived.archived_at.is_some());
        assert_eq!(archived.status, created.status);
        assert_eq!(archived.priority, created.priority);
        assert_eq!(archived.created_at, created.created_at);
        assert_eq!(archived.description, created.description);
        let repeated =
            archive_task(&mut connection, PERSONAL, &created.id).expect("idempotent archive");
        assert_eq!(repeated, archived);
        assert!(list_tasks(&connection, PERSONAL, TaskArchiveFilter::Active)
            .unwrap()
            .is_empty());
        assert_eq!(
            list_tasks(&connection, PERSONAL, TaskArchiveFilter::Archived).unwrap(),
            vec![archived.clone()]
        );
        let restored = restore_task(&mut connection, PERSONAL, &created.id).expect("restore");
        assert!(restored.archived_at.is_none());
        assert_eq!(restored.status, created.status);
        assert_eq!(restored.priority, created.priority);
        assert_eq!(restored.created_at, created.created_at);
        assert_eq!(restored.description, created.description);
        assert_eq!(
            restore_task(&mut connection, PERSONAL, &created.id).unwrap(),
            restored
        );
        assert_eq!(
            list_tasks(&connection, PERSONAL, TaskArchiveFilter::Active).unwrap(),
            vec![restored.clone()]
        );
        assert!(
            list_tasks(&connection, PERSONAL, TaskArchiveFilter::Archived)
                .unwrap()
                .is_empty()
        );
        let history: Vec<(String, String)> = connection.prepare("SELECT action, summary FROM activity_events WHERE task_id = ?1 ORDER BY occurred_at, rowid").unwrap().query_map(params![created.id], |row| Ok((row.get(0)?, row.get(1)?))).unwrap().map(Result::unwrap).collect();
        assert_eq!(
            history
                .iter()
                .map(|event| event.0.as_str())
                .collect::<Vec<_>>(),
            vec!["created", "archived", "restored"]
        );
        assert_eq!(history[1].1, "Task archived");
        assert_eq!(history[2].1, "Task restored");
        assert!(!history
            .iter()
            .any(|event| event.1.contains("private description")));
    }

    #[test]
    fn archive_restore_reject_missing_and_wrong_workspace_without_writes() {
        let mut connection = fixture();
        let created = create_task(&mut connection, request()).unwrap();
        for result in [
            archive_task(&mut connection, PERSONAL, "missing"),
            restore_task(&mut connection, PERSONAL, "missing"),
            archive_task(&mut connection, "wrong-workspace", &created.id),
            restore_task(&mut connection, "wrong-workspace", &created.id),
        ] {
            assert!(result.unwrap_err().starts_with("NOT_FOUND:"));
        }
        assert_eq!(
            list_tasks(&connection, PERSONAL, TaskArchiveFilter::All).unwrap(),
            vec![created]
        );
    }

    #[test]
    fn restore_waits_for_archived_project_to_be_restored() {
        let mut connection = fixture();
        connection.execute("INSERT INTO projects (id, workspace_id, name) VALUES ('archived-project', ?1, 'Project')", params![PERSONAL]).unwrap();
        let mut request = request();
        request.project_id = Some("archived-project".into());
        let task = create_task(&mut connection, request).unwrap();
        crate::projects::archive_project(&mut connection, PERSONAL, "archived-project").unwrap();
        archive_task(&mut connection, PERSONAL, &task.id).unwrap();
        let error = restore_task(&mut connection, PERSONAL, &task.id).unwrap_err();
        assert!(error.starts_with("CONFLICT:"));
        assert!(read_task(&connection, PERSONAL, &task.id)
            .unwrap()
            .1
            .is_some());
    }

    #[test]
    fn failed_archive_activity_rolls_back_and_task_survives_reopen() {
        let mut connection = fixture();
        let created = create_task(&mut connection, request()).unwrap();
        connection.execute_batch("CREATE TRIGGER reject_task_archive BEFORE INSERT ON activity_events WHEN NEW.action = 'archived' BEGIN SELECT RAISE(ABORT, 'synthetic activity failure'); END;").unwrap();
        assert!(archive_task(&mut connection, PERSONAL, &created.id)
            .unwrap_err()
            .starts_with("IO_ERROR:"));
        let (_, archived_at) = read_task(&connection, PERSONAL, &created.id).unwrap();
        assert!(archived_at.is_none());
        drop(connection);
        let mut path = std::env::temp_dir();
        path.push(format!("moonships-m2-011-{}.sqlite", Uuid::new_v4()));
        let mut disk = Connection::open(&path).unwrap();
        disk.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        crate::run_migrations(&mut disk).unwrap();
        let disk_task = create_task(&mut disk, request()).unwrap();
        archive_task(&mut disk, PERSONAL, &disk_task.id).unwrap();
        drop(disk);
        let reopened = Connection::open(&path).unwrap();
        let stored = list_tasks(&reopened, PERSONAL, TaskArchiveFilter::Archived).unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id, disk_task.id);
        let events: i64 = reopened
            .query_row(
                "SELECT COUNT(*) FROM activity_events WHERE task_id = ?1",
                params![disk_task.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(events, 2);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn restore_of_active_task_is_a_no_op() {
        let mut connection = fixture();
        let task = create_task(&mut connection, request()).unwrap();
        assert_eq!(
            restore_task(&mut connection, PERSONAL, &task.id).unwrap(),
            task
        );
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM activity_events WHERE task_id = ?1",
                params![task.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
