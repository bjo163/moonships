//! Local Project milestones and explicit Task associations.

use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_TITLE_CHARS: usize = 160;
const MAX_DESCRIPTION_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateMilestone {
    pub workspace_id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateMilestone {
    pub workspace_id: String,
    pub project_id: String,
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Milestone {
    pub id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub status: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tasks: Vec<MilestoneTask>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MilestoneTask {
    pub id: String,
    pub title: String,
    pub status: String,
}

fn validate_due_date(value: Option<String>) -> Result<Option<String>, String> {
    value
        .map(|value| {
            let value = value.trim();
            if value.is_empty() {
                return Ok(None);
            }
            let parsed = NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|_| "VALIDATION_FAILED: dueDate must be YYYY-MM-DD".to_string())?;
            if parsed.format("%Y-%m-%d").to_string() != value {
                return Err("VALIDATION_FAILED: dueDate must be YYYY-MM-DD".to_string());
            }
            Ok(Some(value.to_string()))
        })
        .transpose()
        .map(Option::flatten)
}

fn read_milestone(
    connection: &Connection,
    workspace_id: &str,
    id: &str,
) -> Result<Milestone, String> {
    let mut milestone = connection.query_row(
        "SELECT id, workspace_id, project_id, title, description, due_date, status, sort_order, created_at, updated_at
         FROM milestones WHERE workspace_id = ?1 AND id = ?2",
        params![workspace_id, id],
        |row| Ok(Milestone {
            id: row.get(0)?, workspace_id: row.get(1)?, project_id: row.get(2)?,
            title: row.get(3)?, description: row.get(4)?, due_date: row.get(5)?,
            status: row.get(6)?, sort_order: row.get(7)?, created_at: row.get(8)?,
            updated_at: row.get(9)?, tasks: Vec::new(),
        }),
    ).optional().map_err(|_| "IO_ERROR: could not read milestone".to_string())?
        .ok_or_else(|| "NOT_FOUND: milestone is unavailable in this project".to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT t.id, t.title, t.status FROM milestone_tasks mt
         JOIN tasks t ON t.workspace_id = mt.workspace_id AND t.id = mt.task_id
         WHERE mt.workspace_id = ?1 AND mt.milestone_id = ?2 AND t.archived_at IS NULL
         ORDER BY t.created_at, t.id",
        )
        .map_err(|_| "IO_ERROR: could not prepare milestone task list".to_string())?;
    milestone.tasks = statement
        .query_map(params![workspace_id, id], |row| {
            Ok(MilestoneTask {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
            })
        })
        .map_err(|_| "IO_ERROR: could not list milestone tasks".to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "IO_ERROR: could not read milestone task list".to_string())?;
    Ok(milestone)
}

pub fn list_milestones(
    connection: &Connection,
    workspace_id: &str,
    project_id: &str,
) -> Result<Vec<Milestone>, String> {
    let workspace_id = workspace_id.trim();
    let project_id = project_id.trim();
    if workspace_id.is_empty() || project_id.is_empty() {
        return Err("VALIDATION_FAILED: project and workspace IDs are required".to_string());
    }
    let exists: Option<i64> = connection.query_row(
        "SELECT 1 FROM projects WHERE id = ?1 AND workspace_id = ?2 AND archived_at IS NULL AND status != 'archived'",
        params![project_id, workspace_id], |row| row.get(0),
    ).optional().map_err(|_| "IO_ERROR: could not read milestone project".to_string())?;
    if exists.is_none() {
        return Err("NOT_FOUND: project is unavailable".to_string());
    }
    let mut statement = connection.prepare("SELECT id FROM milestones WHERE workspace_id = ?1 AND project_id = ?2 ORDER BY sort_order, due_date IS NULL, due_date, id")
        .map_err(|_| "IO_ERROR: could not prepare milestone list".to_string())?;
    let ids = statement
        .query_map(params![workspace_id, project_id], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| "IO_ERROR: could not list milestones".to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "IO_ERROR: could not read milestone IDs".to_string())?;
    ids.into_iter()
        .map(|id| read_milestone(connection, workspace_id, &id))
        .collect()
}

pub fn create_milestone(
    connection: &mut Connection,
    request: CreateMilestone,
) -> Result<Milestone, String> {
    let workspace_id = request.workspace_id.trim();
    let project_id = request.project_id.trim();
    let title = request.title.trim();
    if workspace_id.is_empty()
        || project_id.is_empty()
        || title.is_empty()
        || title.chars().count() > MAX_TITLE_CHARS
    {
        return Err("VALIDATION_FAILED: valid project and milestone title required".to_string());
    }
    let description = request
        .description
        .map(|value| {
            let value = value.trim();
            if value.chars().count() > MAX_DESCRIPTION_CHARS {
                Err("VALIDATION_FAILED: milestone description is too long".to_string())
            } else {
                Ok((!value.is_empty()).then(|| value.to_string()))
            }
        })
        .transpose()?
        .flatten();
    let due_date = validate_due_date(request.due_date)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin milestone create".to_string())?;
    let project_exists: Option<i64> = transaction.query_row(
        "SELECT 1 FROM projects WHERE id = ?1 AND workspace_id = ?2 AND archived_at IS NULL AND status != 'archived'",
        params![project_id, workspace_id], |row| row.get(0),
    ).optional().map_err(|_| "IO_ERROR: could not read milestone project".to_string())?;
    if project_exists.is_none() {
        return Err("NOT_FOUND: project is unavailable".to_string());
    }
    let sort_order: i64 = transaction.query_row("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM milestones WHERE workspace_id = ?1 AND project_id = ?2", params![workspace_id, project_id], |row| row.get(0)).map_err(|_| "IO_ERROR: could not order milestone".to_string())?;
    let id = Uuid::new_v4().to_string();
    transaction.execute("INSERT INTO milestones (id, workspace_id, project_id, title, description, due_date, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![id, workspace_id, project_id, title, description, due_date, sort_order]).map_err(|_| "IO_ERROR: could not create milestone".to_string())?;
    let created = read_milestone(&transaction, workspace_id, &id)?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit milestone".to_string())?;
    Ok(created)
}

pub fn associate_task(
    connection: &mut Connection,
    workspace_id: &str,
    project_id: &str,
    milestone_id: &str,
    task_id: &str,
) -> Result<Milestone, String> {
    let workspace_id = workspace_id.trim();
    let project_id = project_id.trim();
    let milestone_id = milestone_id.trim();
    let task_id = task_id.trim();
    if [workspace_id, project_id, milestone_id, task_id]
        .iter()
        .any(|value| value.is_empty())
    {
        return Err(
            "VALIDATION_FAILED: project, milestone, task and workspace IDs are required"
                .to_string(),
        );
    }
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin milestone association".to_string())?;
    let milestone_project: Option<String> = transaction
        .query_row(
            "SELECT project_id FROM milestones WHERE id = ?1 AND workspace_id = ?2",
            params![milestone_id, workspace_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read milestone".to_string())?;
    if milestone_project.as_deref() != Some(project_id) {
        return Err("NOT_FOUND: milestone is unavailable in this project".to_string());
    }
    let valid_task: Option<i64> = transaction.query_row("SELECT 1 FROM tasks WHERE id = ?1 AND workspace_id = ?2 AND project_id = ?3 AND archived_at IS NULL", params![task_id, workspace_id, project_id], |row| row.get(0)).optional().map_err(|_| "IO_ERROR: could not read milestone task".to_string())?;
    if valid_task.is_none() {
        return Err("CONFLICT: only active tasks in this project can be associated".to_string());
    }
    transaction.execute("INSERT OR IGNORE INTO milestone_tasks (workspace_id, project_id, milestone_id, task_id) VALUES (?1, ?2, ?3, ?4)", params![workspace_id, project_id, milestone_id, task_id]).map_err(|_| "IO_ERROR: could not associate milestone task".to_string())?;
    let result = read_milestone(&transaction, workspace_id, milestone_id)?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit milestone association".to_string())?;
    Ok(result)
}

pub fn update_milestone(
    connection: &mut Connection,
    request: UpdateMilestone,
) -> Result<Milestone, String> {
    let workspace_id = request.workspace_id.trim();
    let project_id = request.project_id.trim();
    let id = request.id.trim();
    let title = request.title.trim();
    if [workspace_id, project_id, id, title]
        .iter()
        .any(|value| value.is_empty())
        || title.chars().count() > MAX_TITLE_CHARS
    {
        return Err("VALIDATION_FAILED: valid project and milestone title required".to_string());
    }
    if !matches!(request.status.as_str(), "planned" | "active" | "completed") {
        return Err("VALIDATION_FAILED: unsupported milestone status".to_string());
    }
    let description = request
        .description
        .map(|value| {
            let value = value.trim();
            if value.chars().count() > MAX_DESCRIPTION_CHARS {
                Err("VALIDATION_FAILED: milestone description is too long".to_string())
            } else {
                Ok((!value.is_empty()).then(|| value.to_string()))
            }
        })
        .transpose()?
        .flatten();
    let due_date = validate_due_date(request.due_date)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin milestone update".to_string())?;
    let exists: Option<i64> = transaction.query_row(
        "SELECT 1 FROM milestones m JOIN projects p ON p.workspace_id=m.workspace_id AND p.id=m.project_id
         WHERE m.id=?1 AND m.workspace_id=?2 AND m.project_id=?3 AND p.archived_at IS NULL AND p.status!='archived'",
        params![id, workspace_id, project_id], |row| row.get(0),
    ).optional().map_err(|_| "IO_ERROR: could not read milestone".to_string())?;
    if exists.is_none() {
        return Err("NOT_FOUND: milestone is unavailable".to_string());
    }
    transaction.execute(
        "UPDATE milestones SET title=?1, description=?2, due_date=?3, status=?4,
         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?5 AND workspace_id=?6 AND project_id=?7",
        params![title, description, due_date, request.status, id, workspace_id, project_id],
    ).map_err(|_| "IO_ERROR: could not update milestone".to_string())?;
    let saved = read_milestone(&transaction, workspace_id, id)?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit milestone update".to_string())?;
    Ok(saved)
}

#[cfg(test)]
mod tests {
    use super::*;
    const PERSONAL: &str = "00000000-0000-4000-8000-000000000001";
    fn fixture() -> Connection {
        let mut db = Connection::open_in_memory().unwrap();
        db.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::run_migrations(&mut db).unwrap();
        db.execute("INSERT INTO projects (id, workspace_id, name) VALUES ('p1', ?1, 'Project one'), ('p2', ?1, 'Project two')", params![PERSONAL]).unwrap();
        db
    }
    fn request() -> CreateMilestone {
        CreateMilestone {
            workspace_id: PERSONAL.into(),
            project_id: "p1".into(),
            title: "Release 1".into(),
            description: None,
            due_date: Some("2026-12-31".into()),
        }
    }
    #[test]
    fn create_and_list_milestones_validates_dates_and_keeps_status_metadata_only() {
        let mut db = fixture();
        let m = create_milestone(&mut db, request()).unwrap();
        assert_eq!(m.status, "planned");
        assert_eq!(m.due_date.as_deref(), Some("2026-12-31"));
        assert_eq!(list_milestones(&db, PERSONAL, "p1").unwrap(), vec![m]);
        let mut bad = request();
        bad.due_date = Some("2026-02-30".into());
        assert!(create_milestone(&mut db, bad)
            .unwrap_err()
            .starts_with("VALIDATION_FAILED:"));
    }
    #[test]
    fn associations_are_project_scoped_idempotent_and_do_not_change_task_status() {
        let mut db = fixture();
        let task = crate::tasks::create_task(
            &mut db,
            crate::tasks::CreateTask {
                workspace_id: PERSONAL.into(),
                project_id: Some("p1".into()),
                parent_task_id: None,
                title: "Task".into(),
                description: None,
                due_at: None,
                status: Some("in_progress".into()),
                priority: Some("p2".into()),
            },
        )
        .unwrap();
        let m = create_milestone(&mut db, request()).unwrap();
        let associated = associate_task(&mut db, PERSONAL, "p1", &m.id, &task.id).unwrap();
        assert_eq!(associated.tasks.len(), 1);
        assert_eq!(associated.tasks[0].status, "in_progress");
        assert_eq!(
            crate::tasks::get_task(&db, PERSONAL, &task.id)
                .unwrap()
                .status,
            "in_progress"
        );
        assert_eq!(
            associate_task(&mut db, PERSONAL, "p1", &m.id, &task.id)
                .unwrap()
                .tasks
                .len(),
            1
        );
        assert!(associate_task(&mut db, PERSONAL, "p2", &m.id, &task.id)
            .unwrap_err()
            .starts_with("NOT_FOUND:"));
        let changed = update_milestone(
            &mut db,
            UpdateMilestone {
                workspace_id: PERSONAL.into(),
                project_id: "p1".into(),
                id: m.id,
                title: "Release 1".into(),
                description: None,
                due_date: None,
                status: "completed".into(),
            },
        )
        .unwrap();
        assert_eq!(changed.status, "completed");
        assert_eq!(
            crate::tasks::get_task(&db, PERSONAL, &task.id)
                .unwrap()
                .status,
            "in_progress"
        );
    }
}
