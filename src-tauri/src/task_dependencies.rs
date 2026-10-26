//! Workspace-scoped directed Task dependencies.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskDependency {
    pub workspace_id: String,
    /// The Task whose completion is blocked.
    pub task_id: String,
    /// The prerequisite Task that must be completed first.
    pub depends_on_task_id: String,
    /// The canonical stored relation; the inverse "blocks" label is derived.
    pub dependency_type: String,
    pub created_at: String,
}

const CANONICAL_DEPENDENCY_TYPE: &str = "blocked_by";

fn ensure_workspace_is_active(connection: &Connection, workspace_id: &str) -> Result<(), String> {
    let workspace_exists: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM workspaces WHERE id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read dependency workspace".to_string())?;
    if workspace_exists.is_none() {
        return Err("NOT_FOUND: dependency workspace is unavailable".to_string());
    }
    Ok(())
}

pub fn add_dependency(
    connection: &Connection,
    workspace_id: &str,
    task_id: &str,
    depends_on_task_id: &str,
) -> Result<TaskDependency, String> {
    let workspace_id = workspace_id.trim();
    let task_id = task_id.trim();
    let depends_on_task_id = depends_on_task_id.trim();
    if workspace_id.is_empty() || task_id.is_empty() || depends_on_task_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace and Task IDs are required".to_string());
    }
    if task_id == depends_on_task_id {
        return Err("VALIDATION_FAILED: a Task cannot depend on itself".to_string());
    }
    ensure_workspace_is_active(connection, workspace_id)?;

    connection
        .execute(
            "INSERT INTO task_dependencies
                (workspace_id, task_id, depends_on_task_id, dependency_type)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (workspace_id, task_id, depends_on_task_id) DO NOTHING",
            params![
                workspace_id,
                task_id,
                depends_on_task_id,
                CANONICAL_DEPENDENCY_TYPE
            ],
        )
        .map_err(|_| {
            "NOT_FOUND: Task dependency must reference two Tasks in the same workspace".to_string()
        })?;
    connection
        .query_row(
            "SELECT workspace_id, task_id, depends_on_task_id, dependency_type, created_at
             FROM task_dependencies
             WHERE workspace_id = ?1 AND task_id = ?2 AND depends_on_task_id = ?3",
            params![workspace_id, task_id, depends_on_task_id],
            dependency_from_row,
        )
        .map_err(|_| "IO_ERROR: could not read Task dependency".to_string())
}

fn dependency_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskDependency> {
    Ok(TaskDependency {
        workspace_id: row.get(0)?,
        task_id: row.get(1)?,
        depends_on_task_id: row.get(2)?,
        dependency_type: row.get(3)?,
        created_at: row.get(4)?,
    })
}

pub fn list_dependencies_for_task(
    connection: &Connection,
    workspace_id: &str,
    task_id: &str,
) -> Result<Vec<TaskDependency>, String> {
    let workspace_id = workspace_id.trim();
    let task_id = task_id.trim();
    if workspace_id.is_empty() || task_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace and Task IDs are required".to_string());
    }
    ensure_workspace_is_active(connection, workspace_id)?;
    let task_exists: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM tasks WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, task_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read Task for dependencies".to_string())?;
    if task_exists.is_none() {
        return Err("NOT_FOUND: Task is unavailable in this workspace".to_string());
    }
    let mut statement = connection
        .prepare(
            "SELECT workspace_id, task_id, depends_on_task_id, dependency_type, created_at
             FROM task_dependencies
             WHERE workspace_id = ?1 AND task_id = ?2
             ORDER BY depends_on_task_id",
        )
        .map_err(|_| "IO_ERROR: could not prepare Task dependency list".to_string())?;
    let rows = statement
        .query_map(params![workspace_id, task_id], dependency_from_row)
        .map_err(|_| "IO_ERROR: could not query Task dependencies".to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "IO_ERROR: could not read Task dependencies".to_string())
}

pub fn remove_dependency(
    connection: &Connection,
    workspace_id: &str,
    task_id: &str,
    depends_on_task_id: &str,
) -> Result<(), String> {
    let workspace_id = workspace_id.trim();
    let task_id = task_id.trim();
    let depends_on_task_id = depends_on_task_id.trim();
    if workspace_id.is_empty() || task_id.is_empty() || depends_on_task_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace and Task IDs are required".to_string());
    }
    connection
        .execute(
            "DELETE FROM task_dependencies
             WHERE workspace_id = ?1 AND task_id = ?2 AND depends_on_task_id = ?3",
            params![workspace_id, task_id, depends_on_task_id],
        )
        .map_err(|_| "IO_ERROR: could not remove Task dependency".to_string())?;
    Ok(())
}
