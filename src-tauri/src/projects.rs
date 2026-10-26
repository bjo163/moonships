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

pub fn create_project(connection: &mut Connection, request: CreateProject) -> Result<Project, String> {
    let workspace_id = request.workspace_id.trim();
    let name = request.name.trim();
    if workspace_id.is_empty() || name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err("VALIDATION_FAILED: valid workspace and project name required".to_string());
    }

    let status = request.status.as_deref().unwrap_or("planned");
    if !matches!(status, "planned" | "active" | "paused" | "completed" | "archived") {
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

    let created = transaction
        .query_row(
            "SELECT id, workspace_id, name, project_key, description,
                    goal, status, created_at, updated_at
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
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .map_err(|_| "IO_ERROR: could not read created project".to_string())?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit project".to_string())?;
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERSONAL: &str = "00000000-0000-4000-8000-000000000001";

    fn fixture() -> Connection {
        let mut connection = Connection::open_in_memory().expect("disposable memory database");
        connection.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
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
            .execute("UPDATE workspaces SET archived_at = '2026-01-01' WHERE id = ?1", params![PERSONAL])
            .unwrap();
        let failure = create_project(&mut connection, request()).unwrap_err();
        assert!(failure.starts_with("NOT_FOUND:"));
    }
}
