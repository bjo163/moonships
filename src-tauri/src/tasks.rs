//! Local Task creation service. All writes stay inside the caller-provided,
//! foreign-key-enabled SQLite connection; no UI or global database path access.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_TITLE_CHARS: usize = 200;
const MAX_DESCRIPTION_CHARS: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTask {
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
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
    pub status: String,
    pub priority: String,
    pub created_at: String,
    pub updated_at: String,
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
                (id, workspace_id, project_id, title, description, status, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                workspace_id,
                project_id,
                title,
                description,
                status,
                priority
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
                    status, priority, created_at, updated_at
             FROM tasks WHERE id = ?1 AND workspace_id = ?2",
            params![id, workspace_id],
            |row| {
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
                })
            },
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
                    status, priority, created_at, updated_at, archived_at
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
                    },
                    row.get(9)?,
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
        && existing.status == request.status
        && existing.priority == request.priority
    {
        return Ok(existing);
    }

    let changed = transaction
        .execute(
            "UPDATE tasks
             SET project_id = ?1, title = ?2, description = ?3,
                 status = ?4, priority = ?5,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?6 AND workspace_id = ?7 AND archived_at IS NULL",
            params![
                project_id,
                title,
                description,
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
}
