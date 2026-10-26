//! Workspace-scoped labels and Task-label relationships backed by SQLite.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_LABEL_CHARS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Label {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub normalized_name: String,
    pub created_at: String,
    pub updated_at: String,
}

fn normalize_name(name: &str) -> Result<(String, String), String> {
    let display_name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if display_name.is_empty() || display_name.chars().count() > MAX_LABEL_CHARS {
        return Err("VALIDATION_FAILED: label name must contain 1 to 64 characters".to_string());
    }
    let normalized_name = display_name.to_lowercase();
    Ok((display_name, normalized_name))
}

/// Create a label. Within a workspace, names are unique after Unicode lowercase
/// conversion and collapsing/ trimming whitespace. The first display spelling is kept.
pub fn create_label(
    connection: &mut Connection,
    workspace_id: &str,
    name: &str,
) -> Result<Label, String> {
    let workspace_id = workspace_id.trim();
    if workspace_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace ID is required".to_string());
    }
    let (name, normalized_name) = normalize_name(name)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin label transaction".to_string())?;
    let workspace_exists: Option<i64> = transaction
        .query_row(
            "SELECT 1 FROM workspaces WHERE id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read label workspace".to_string())?;
    if workspace_exists.is_none() {
        return Err("NOT_FOUND: label workspace does not exist or is archived".to_string());
    }

    let id = Uuid::new_v4().to_string();
    transaction
        .execute(
            "INSERT INTO labels (id, workspace_id, name, normalized_name)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, workspace_id, name, normalized_name],
        )
        .map_err(|error| {
            if matches!(
                error,
                rusqlite::Error::SqliteFailure(ref reason, _)
                    if reason.code == rusqlite::ErrorCode::ConstraintViolation
            ) {
                "CONFLICT: label name already exists in this workspace".to_string()
            } else {
                "IO_ERROR: could not create label".to_string()
            }
        })?;
    let label = transaction
        .query_row(
            "SELECT id, workspace_id, name, normalized_name, created_at, updated_at
             FROM labels WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, id],
            label_from_row,
        )
        .map_err(|_| "IO_ERROR: could not read created label".to_string())?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit label".to_string())?;
    Ok(label)
}

fn label_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Label> {
    Ok(Label {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        name: row.get(2)?,
        normalized_name: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn ensure_workspace_exists(connection: &Connection, workspace_id: &str) -> Result<(), String> {
    let exists: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM workspaces WHERE id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read label workspace".to_string())?;
    if exists.is_none() {
        return Err("NOT_FOUND: label workspace does not exist or is archived".to_string());
    }
    Ok(())
}

pub fn list_labels(connection: &Connection, workspace_id: &str) -> Result<Vec<Label>, String> {
    let workspace_id = workspace_id.trim();
    if workspace_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace ID is required".to_string());
    }
    ensure_workspace_exists(connection, workspace_id)?;
    let mut statement = connection
        .prepare(
            "SELECT id, workspace_id, name, normalized_name, created_at, updated_at
             FROM labels WHERE workspace_id = ?1 ORDER BY normalized_name, id",
        )
        .map_err(|_| "IO_ERROR: could not prepare label list".to_string())?;
    let rows = statement
        .query_map(params![workspace_id], label_from_row)
        .map_err(|_| "IO_ERROR: could not query labels".to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "IO_ERROR: could not read labels".to_string())
}

/// Associate a label and task in the same workspace. Repeating the same
/// association is idempotent.
pub fn add_label_to_task(
    connection: &Connection,
    workspace_id: &str,
    task_id: &str,
    label_id: &str,
) -> Result<(), String> {
    let workspace_id = workspace_id.trim();
    let task_id = task_id.trim();
    let label_id = label_id.trim();
    if workspace_id.is_empty() || task_id.is_empty() || label_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace, task and label IDs are required".to_string());
    }
    connection
        .execute(
            "INSERT INTO task_labels (workspace_id, task_id, label_id)
             VALUES (?1, ?2, ?3) ON CONFLICT (workspace_id, task_id, label_id) DO NOTHING",
            params![workspace_id, task_id, label_id],
        )
        .map_err(|_| "NOT_FOUND: task or label is unavailable in this workspace".to_string())?;
    Ok(())
}

pub fn remove_label_from_task(
    connection: &Connection,
    workspace_id: &str,
    task_id: &str,
    label_id: &str,
) -> Result<(), String> {
    let workspace_id = workspace_id.trim();
    let task_id = task_id.trim();
    let label_id = label_id.trim();
    if workspace_id.is_empty() || task_id.is_empty() || label_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace, task and label IDs are required".to_string());
    }
    connection
        .execute(
            "DELETE FROM task_labels WHERE workspace_id = ?1 AND task_id = ?2 AND label_id = ?3",
            params![workspace_id, task_id, label_id],
        )
        .map_err(|_| "IO_ERROR: could not remove Task label".to_string())?;
    Ok(())
}

pub fn list_task_labels(
    connection: &Connection,
    workspace_id: &str,
    task_id: &str,
) -> Result<Vec<Label>, String> {
    let workspace_id = workspace_id.trim();
    let task_id = task_id.trim();
    if workspace_id.is_empty() || task_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace and task IDs are required".to_string());
    }
    ensure_workspace_exists(connection, workspace_id)?;
    let task_exists: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM tasks WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, task_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read Task for labels".to_string())?;
    if task_exists.is_none() {
        return Err("NOT_FOUND: Task is unavailable in this workspace".to_string());
    }
    let mut statement = connection
        .prepare(
            "SELECT l.id, l.workspace_id, l.name, l.normalized_name, l.created_at, l.updated_at
             FROM labels l JOIN task_labels tl
               ON tl.workspace_id = l.workspace_id AND tl.label_id = l.id
             WHERE tl.workspace_id = ?1 AND tl.task_id = ?2
             ORDER BY l.normalized_name, l.id",
        )
        .map_err(|_| "IO_ERROR: could not prepare Task label list".to_string())?;
    let rows = statement
        .query_map(params![workspace_id, task_id], label_from_row)
        .map_err(|_| "IO_ERROR: could not query Task labels".to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "IO_ERROR: could not read Task labels".to_string())
}
