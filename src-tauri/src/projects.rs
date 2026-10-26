//! Local-first Project repository/service. UI and IPC adapters must validate input
//! again at their own boundary. The service never accepts a database path.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_NAME_CHARS: usize = 120;
const MAX_KEY_CHARS: usize = 40;
const MAX_DESCRIPTION_CHARS: usize = 10_000;
const MAX_GOAL_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateProject {
    pub workspace_id: String,
    pub name: String,
    pub project_key: Option<String>,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub project_key: Option<String>,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: String,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn nonempty_optional(value: Option<String>, limit: usize) -> Result<Option<String>, String> {
    value
        .map(|value| {
            let trimmed = value.trim();
            if trimmed.chars().count() > limit {
                return Err("VALIDATION_FAILED: project field is too long".to_string());
            }
            Ok((!trimmed.is_empty()).then(|| trimmed.to_string()))
        })
        .transpose()
        .map(|value| value.flatten())
}

pub fn create_project(
    connection: &mut Connection,
    request: CreateProject,
) -> Result<Project, String> {
    let workspace_id = request.workspace_id.trim();
    let name = request.name.trim();
    if workspace_id.is_empty() || name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err("VALIDATION_FAILED: valid workspace and project name required".to_string());
    }

    let status = request.status.as_deref().unwrap_or("planned");
    if !matches!(
        status,
        "planned" | "active" | "paused" | "completed" | "archived"
    ) {
        return Err("VALIDATION_FAILED: unsupported project status".to_string());
    }
    let project_key = nonempty_optional(request.project_key, MAX_KEY_CHARS)?;
    let description = nonempty_optional(request.description, MAX_DESCRIPTION_CHARS)?;
    let goal = nonempty_optional(request.goal, MAX_GOAL_CHARS)?;

    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin project transaction".to_string())?;
    let workspace_exists: Option<i64> = transaction
        .query_row(
            "SELECT 1 FROM workspaces WHERE id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read workspace".to_string())?;
    if workspace_exists.is_none() {
        return Err("NOT_FOUND: workspace does not exist or is archived".to_string());
    }

    let id = Uuid::new_v4().to_string();
    transaction
        .execute(
            "INSERT INTO projects
                (id, workspace_id, name, project_key, description, goal, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                workspace_id,
                name,
                project_key,
                description,
                goal,
                status
            ],
        )
        .map_err(|error| {
            if matches!(
                error,
                rusqlite::Error::SqliteFailure(ref reason, _)
                    if reason.code == rusqlite::ErrorCode::ConstraintViolation
            ) {
                "CONFLICT: project key or record conflicts with existing data".to_string()
            } else {
                "IO_ERROR: could not create project".to_string()
            }
        })?;

    crate::activity::record_creation(
        &transaction,
        workspace_id,
        crate::activity::ActivityEntityKind::Project,
        &id,
    )?;

    let created = transaction
        .query_row(
            "SELECT id, workspace_id, name, project_key, description,
                    goal, status, archived_at, created_at, updated_at
             FROM projects WHERE id = ?1 AND workspace_id = ?2",
            params![id, workspace_id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    name: row.get(2)?,
                    project_key: row.get(3)?,
                    description: row.get(4)?,
                    goal: row.get(5)?,
                    status: row.get(6)?,
                    archived_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .map_err(|_| "IO_ERROR: could not read created project".to_string())?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit project".to_string())?;
    Ok(created)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateProject {
    pub workspace_id: String,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: String,
}

fn read_project(connection: &Connection, workspace_id: &str, id: &str) -> Result<Project, String> {
    connection
        .query_row(
            "SELECT id, workspace_id, name, project_key, description, goal,
                    status, archived_at, created_at, updated_at
             FROM projects WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    name: row.get(2)?,
                    project_key: row.get(3)?,
                    description: row.get(4)?,
                    goal: row.get(5)?,
                    status: row.get(6)?,
                    archived_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read project".to_string())?
        .ok_or_else(|| "NOT_FOUND: project is not available in this workspace".to_string())
}

pub fn update_project(
    connection: &mut Connection,
    request: UpdateProject,
) -> Result<Project, String> {
    let workspace_id = request.workspace_id.trim();
    let id = request.id.trim();
    let name = request.name.trim();
    if workspace_id.is_empty()
        || id.is_empty()
        || name.is_empty()
        || name.chars().count() > MAX_NAME_CHARS
    {
        return Err(
            "VALIDATION_FAILED: project and workspace IDs and name are required".to_string(),
        );
    }
    if !matches!(
        request.status.as_str(),
        "planned" | "active" | "paused" | "completed"
    ) {
        return Err(
            "VALIDATION_FAILED: update status must not bypass archive operation".to_string(),
        );
    }
    let description = nonempty_optional(request.description, MAX_DESCRIPTION_CHARS)?;
    let goal = nonempty_optional(request.goal, MAX_GOAL_CHARS)?;

    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin project update".to_string())?;
    let existing = read_project(&transaction, workspace_id, id)?;
    if existing.archived_at.is_some() {
        return Err("NOT_FOUND: project is archived".to_string());
    }
    let workspace_active: Option<i64> = transaction
        .query_row(
            "SELECT 1 FROM workspaces WHERE id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read project workspace".to_string())?;
    if workspace_active.is_none() {
        return Err("NOT_FOUND: project workspace is archived".to_string());
    }
    if existing.name == name
        && existing.description == description
        && existing.goal == goal
        && existing.status == request.status
    {
        return Ok(existing);
    }

    let updated = transaction
        .execute(
            "UPDATE projects
             SET name = ?1, description = ?2, goal = ?3, status = ?4,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?5 AND workspace_id = ?6 AND archived_at IS NULL
               AND EXISTS (
                   SELECT 1 FROM workspaces w
                   WHERE w.id = ?6 AND w.archived_at IS NULL
               )",
            params![name, description, goal, request.status, id, workspace_id],
        )
        .map_err(|_| "IO_ERROR: could not update project".to_string())?;
    if updated == 0 {
        return Err("NOT_FOUND: project or active workspace not found".to_string());
    }
    crate::activity::record_change(
        &transaction,
        workspace_id,
        crate::activity::ActivityEntityKind::Project,
        id,
        crate::activity::ActivityAction::Updated,
    )?;
    let project = read_project(&transaction, workspace_id, id)?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit project update".to_string())?;
    Ok(project)
}

pub fn archive_project(
    connection: &mut Connection,
    workspace_id: &str,
    id: &str,
) -> Result<Project, String> {
    let workspace_id = workspace_id.trim();
    let id = id.trim();
    if workspace_id.is_empty() || id.is_empty() {
        return Err("VALIDATION_FAILED: project and workspace IDs are required".to_string());
    }
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin project archive".to_string())?;
    let archived = transaction
        .execute(
            "UPDATE projects
             SET status = 'archived',
                 archived_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1 AND workspace_id = ?2 AND archived_at IS NULL
               AND EXISTS (
                   SELECT 1 FROM workspaces w
                   WHERE w.id = ?2 AND w.archived_at IS NULL
               )",
            params![id, workspace_id],
        )
        .map_err(|_| "IO_ERROR: could not archive project".to_string())?;
    if archived == 0 {
        return Err("NOT_FOUND: project not found or already archived".to_string());
    }
    crate::activity::record_change(
        &transaction,
        workspace_id,
        crate::activity::ActivityEntityKind::Project,
        id,
        crate::activity::ActivityAction::Archived,
    )?;
    let project = read_project(&transaction, workspace_id, id)?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit project archive".to_string())?;
    Ok(project)
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

    fn request() -> CreateProject {
        CreateProject {
            workspace_id: PERSONAL.to_string(),
            name: "  Moonships MVP  ".to_string(),
            project_key: Some("MS-MVP".to_string()),
            description: Some(" Local project ".to_string()),
            goal: None,
            status: None,
        }
    }

    #[test]
    fn create_project_validates_and_persists_canonical_defaults() {
        let mut connection = fixture();
        let saved = create_project(&mut connection, request()).expect("create project");
        assert!(Uuid::parse_str(&saved.id).is_ok());
        assert_eq!(saved.workspace_id, PERSONAL);
        assert_eq!(saved.name, "Moonships MVP");
        assert_eq!(saved.project_key.as_deref(), Some("MS-MVP"));
        assert_eq!(saved.description.as_deref(), Some("Local project"));
        assert_eq!(saved.status, "planned");
        assert!(!saved.created_at.is_empty());
        assert!(!saved.updated_at.is_empty());
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let duplicate = create_project(&mut connection, request()).unwrap_err();
        assert!(duplicate.starts_with("CONFLICT:"));
        let still_one: i64 = connection
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .unwrap();
        assert_eq!(still_one, 1);
    }

    #[test]
    fn create_project_rejects_invalid_input_without_partial_writes() {
        let mut connection = fixture();
        let mut invalid = request();
        invalid.name = "   ".to_string();
        assert!(create_project(&mut connection, invalid)
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));

        let mut invalid = request();
        invalid.status = Some("unknown".to_string());
        assert!(create_project(&mut connection, invalid)
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));

        let mut invalid = request();
        invalid.workspace_id = "missing-workspace".to_string();
        assert!(create_project(&mut connection, invalid)
            .unwrap_err()
            .starts_with("NOT_FOUND:"));

        let mut invalid = request();
        invalid.project_key = Some("K".repeat(MAX_KEY_CHARS + 1));
        assert!(create_project(&mut connection, invalid)
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn create_project_does_not_write_to_archived_workspace() {
        let mut connection = fixture();
        connection
            .execute(
                "UPDATE workspaces SET archived_at = '2026-01-01' WHERE id = ?1",
                params![PERSONAL],
            )
            .unwrap();
        let failure = create_project(&mut connection, request()).unwrap_err();
        assert!(failure.starts_with("NOT_FOUND:"));
    }
    #[test]
    fn update_project_persists_fields_and_appends_a_safe_event() {
        let mut connection = fixture();
        let created = create_project(&mut connection, request()).expect("project");
        let updated = update_project(
            &mut connection,
            UpdateProject {
                workspace_id: PERSONAL.to_string(),
                id: created.id.clone(),
                name: "  Updated project  ".to_string(),
                description: Some("secret description".to_string()),
                goal: Some("Updated goal".to_string()),
                status: "active".to_string(),
            },
        )
        .expect("update");
        assert_eq!(updated.name, "Updated project");
        assert_eq!(updated.status, "active");
        assert_eq!(updated.project_key.as_deref(), Some("MS-MVP"));
        assert_eq!(updated.goal.as_deref(), Some("Updated goal"));
        assert!(updated.archived_at.is_none());

        let (events, latest_action, latest_summary): (i64, String, String) = connection
            .query_row(
                "SELECT (SELECT COUNT(*) FROM activity_events WHERE entity_id = ?1),
                        action, summary
                 FROM activity_events WHERE entity_id = ?1
                 ORDER BY rowid DESC LIMIT 1",
                params![created.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("event");
        assert_eq!(events, 2);
        assert_eq!(latest_action, "updated");
        assert_eq!(latest_summary, "Project updated");
        assert!(!latest_summary.contains("secret"));
    }

    #[test]
    fn unchanged_project_update_keeps_timestamp_and_does_not_create_activity() {
        let mut connection = fixture();
        let original = create_project(&mut connection, request()).expect("project");
        let unchanged = update_project(
            &mut connection,
            UpdateProject {
                workspace_id: PERSONAL.to_string(),
                id: original.id.clone(),
                name: "  Moonships MVP  ".to_string(),
                description: Some(" Local project ".to_string()),
                goal: None,
                status: "planned".to_string(),
            },
        )
        .expect("no-op update");
        assert_eq!(unchanged, original);
        let events: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM activity_events WHERE entity_id = ?1",
                params![original.id],
                |row| row.get(0),
            )
            .expect("event count");
        assert_eq!(
            events, 1,
            "unchanged project must not create updated activity"
        );

        connection
            .execute(
                "UPDATE workspaces SET archived_at = '2026-01-01' WHERE id = ?1",
                params![PERSONAL],
            )
            .expect("archive workspace");
        let error = update_project(
            &mut connection,
            UpdateProject {
                workspace_id: PERSONAL.to_string(),
                id: original.id.clone(),
                name: original.name,
                description: original.description,
                goal: original.goal,
                status: original.status,
            },
        )
        .unwrap_err();
        assert!(error.starts_with("NOT_FOUND:"));
    }

    #[test]
    fn archive_project_preserves_record_and_linked_task_without_hard_delete() {
        let mut connection = fixture();
        let created = create_project(&mut connection, request()).expect("project");
        connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, project_id, title)
                 VALUES ('linked-task', ?1, ?2, 'Keep task')",
                params![PERSONAL, created.id],
            )
            .expect("linked task");
        let archived = archive_project(&mut connection, PERSONAL, &created.id).expect("archive");
        assert_eq!(archived.status, "archived");
        assert!(archived.archived_at.is_some());
        let linked_task: String = connection
            .query_row(
                "SELECT title FROM tasks WHERE id = 'linked-task'",
                [],
                |row| row.get(0),
            )
            .expect("linked task retained");
        assert_eq!(linked_task, "Keep task");
        assert!(archive_project(&mut connection, PERSONAL, &created.id).is_err());
        let invalid_update = UpdateProject {
            workspace_id: PERSONAL.to_string(),
            id: created.id,
            name: "Cannot silently edit".to_string(),
            description: None,
            goal: None,
            status: "active".to_string(),
        };
        assert!(update_project(&mut connection, invalid_update).is_err());
    }

    #[test]
    fn project_edit_and_archive_reject_wrong_scope_and_invalid_status() {
        let mut connection = fixture();
        let created = create_project(&mut connection, request()).expect("project");
        let mut request = UpdateProject {
            workspace_id: "other".to_string(),
            id: created.id.clone(),
            name: "Changed".to_string(),
            description: None,
            goal: None,
            status: "active".to_string(),
        };
        assert!(update_project(&mut connection, request.clone()).is_err());
        request.workspace_id = PERSONAL.to_string();
        request.status = "archived".to_string();
        assert!(update_project(&mut connection, request).is_err());
        assert!(archive_project(&mut connection, "other", &created.id).is_err());
        let current = read_project(&connection, PERSONAL, &created.id).expect("original");
        assert_eq!(current.name, "Moonships MVP");
        assert_eq!(current.status, "planned");
        assert!(current.archived_at.is_none());
    }

    #[test]
    fn failed_activity_write_rolls_back_project_edit_and_archive() {
        let mut connection = fixture();
        let created = create_project(&mut connection, request()).expect("project");
        connection
            .execute_batch(
                "CREATE TRIGGER reject_activity
                 BEFORE INSERT ON activity_events
                 BEGIN
                     SELECT RAISE(ABORT, 'synthetic event failure');
                 END;",
            )
            .expect("failure injector");
        assert!(update_project(
            &mut connection,
            UpdateProject {
                workspace_id: PERSONAL.to_string(),
                id: created.id.clone(),
                name: "Should roll back".to_string(),
                description: None,
                goal: None,
                status: "active".to_string(),
            },
        )
        .is_err());
        assert!(archive_project(&mut connection, PERSONAL, &created.id).is_err());
        let current = read_project(&connection, PERSONAL, &created.id).expect("read original");
        assert_eq!(current.name, "Moonships MVP");
        assert_eq!(current.status, "planned");
        assert!(current.archived_at.is_none());
        let event_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM activity_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(event_count, 1);
    }
}
