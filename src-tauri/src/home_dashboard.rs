//! Bounded, read-only data for the local Home dashboard.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

const RECENT_PROJECT_LIMIT: i64 = 5;
const TASK_PREVIEW_LIMIT: i64 = 5;
const RECENT_ACTIVITY_LIMIT: i64 = 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DashboardCounts {
    pub open_tasks: i64,
    pub overdue_tasks: i64,
    pub due_soon_tasks: i64,
    pub blocked_tasks: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DashboardProject {
    pub id: String,
    pub name: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DashboardTask {
    pub id: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub due_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DashboardActivity {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub summary: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HomeDashboard {
    pub as_of: String,
    pub active_project_count: i64,
    pub active_task_count: i64,
    pub counts: DashboardCounts,
    pub recent_projects: Vec<DashboardProject>,
    pub overdue_tasks: Vec<DashboardTask>,
    pub upcoming_tasks: Vec<DashboardTask>,
    pub recent_activity: Vec<DashboardActivity>,
}

pub fn load(
    connection: &Connection,
    workspace_id: &str,
    as_of: DateTime<Utc>,
) -> Result<HomeDashboard, String> {
    let workspace_id = workspace_id.trim();
    if workspace_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace ID is required".to_string());
    }
    let as_of_text = as_of.to_rfc3339_opts(SecondsFormat::Millis, true);
    let due_soon_end = (as_of + Duration::days(7)).to_rfc3339_opts(SecondsFormat::Millis, true);

    let active_project_count = connection
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE workspace_id = ?1 AND archived_at IS NULL",
            params![workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| "IO_ERROR: could not count active projects".to_string())?;

    let counts = connection
        .query_row(
            "SELECT
                COUNT(*),
                COUNT(CASE WHEN status NOT IN ('done', 'cancelled') THEN 1 END),
                COUNT(CASE WHEN status NOT IN ('done', 'cancelled')
                    AND due_at IS NOT NULL AND julianday(due_at) < julianday(?2) THEN 1 END),
                COUNT(CASE WHEN status NOT IN ('done', 'cancelled')
                    AND due_at IS NOT NULL AND julianday(due_at) >= julianday(?2)
                    AND julianday(due_at) <= julianday(?3) THEN 1 END),
                COUNT(CASE WHEN status = 'blocked' THEN 1 END)
             FROM tasks WHERE workspace_id = ?1 AND archived_at IS NULL",
            params![workspace_id, as_of_text, due_soon_end],
            |row| {
                Ok(DashboardCounts {
                    open_tasks: row.get(1)?,
                    overdue_tasks: row.get(2)?,
                    due_soon_tasks: row.get(3)?,
                    blocked_tasks: row.get(4)?,
                })
            },
        )
        .map_err(|_| "IO_ERROR: could not count active tasks".to_string())?;

    let recent_projects = {
        let mut statement = connection
            .prepare(
                "SELECT id, name, status, updated_at FROM projects
                 WHERE workspace_id = ?1 AND archived_at IS NULL
                 ORDER BY updated_at DESC, id ASC LIMIT ?2",
            )
            .map_err(|_| "IO_ERROR: could not prepare recent projects".to_string())?;
        let rows = statement
            .query_map(params![workspace_id, RECENT_PROJECT_LIMIT], |row| {
                Ok(DashboardProject {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })
            .map_err(|_| "IO_ERROR: could not read recent projects".to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "IO_ERROR: could not read recent projects".to_string())?
    };

    let overdue_tasks =
        load_task_preview(connection, workspace_id, &as_of_text, &due_soon_end, true)?;
    let upcoming_tasks =
        load_task_preview(connection, workspace_id, &as_of_text, &due_soon_end, false)?;

    let recent_activity = {
        let mut statement = connection
            .prepare(
                "SELECT id, entity_type, entity_id, action, summary, occurred_at
                 FROM activity_events
                 WHERE workspace_id = ?1 AND sensitivity = 'normal'
                 ORDER BY occurred_at DESC, rowid DESC LIMIT ?2",
            )
            .map_err(|_| "IO_ERROR: could not prepare recent activity".to_string())?;
        let rows = statement
            .query_map(params![workspace_id, RECENT_ACTIVITY_LIMIT], |row| {
                Ok(DashboardActivity {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    summary: row.get(4)?,
                    occurred_at: row.get(5)?,
                })
            })
            .map_err(|_| "IO_ERROR: could not read recent activity".to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "IO_ERROR: could not read recent activity".to_string())?
    };

    Ok(HomeDashboard {
        as_of: as_of_text,
        active_project_count,
        active_task_count: connection
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE workspace_id = ?1 AND archived_at IS NULL",
                params![workspace_id],
                |row| row.get(0),
            )
            .map_err(|_| "IO_ERROR: could not count active tasks".to_string())?,
        counts,
        recent_projects,
        overdue_tasks,
        upcoming_tasks,
        recent_activity,
    })
}

fn load_task_preview(
    connection: &Connection,
    workspace_id: &str,
    as_of: &str,
    due_soon_end: &str,
    overdue: bool,
) -> Result<Vec<DashboardTask>, String> {
    const SELECT_FIELDS: &str =
        "SELECT t.id, t.project_id, p.name, t.title, t.status, t.priority, t.due_at
        FROM tasks t LEFT JOIN projects p
          ON p.workspace_id = t.workspace_id AND p.id = t.project_id AND p.archived_at IS NULL
        WHERE t.workspace_id = ?1 AND t.archived_at IS NULL
          AND t.status NOT IN ('done', 'cancelled') AND t.due_at IS NOT NULL";
    let sql = if overdue {
        format!(
            "{SELECT_FIELDS} AND julianday(t.due_at) < julianday(?2)
             ORDER BY julianday(t.due_at) ASC, t.id ASC LIMIT ?3"
        )
    } else {
        format!(
            "{SELECT_FIELDS} AND julianday(t.due_at) >= julianday(?2)
             AND julianday(t.due_at) <= julianday(?3)
             ORDER BY julianday(t.due_at) ASC, t.id ASC LIMIT ?4"
        )
    };
    let mut statement = connection
        .prepare(&sql)
        .map_err(|_| "IO_ERROR: could not prepare task dashboard preview".to_string())?;
    let mut rows = if overdue {
        statement.query(params![workspace_id, as_of, TASK_PREVIEW_LIMIT])
    } else {
        statement.query(params![
            workspace_id,
            as_of,
            due_soon_end,
            TASK_PREVIEW_LIMIT
        ])
    }
    .map_err(|_| "IO_ERROR: could not read task dashboard preview".to_string())?;
    let mut tasks = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|_| "IO_ERROR: could not read task dashboard preview".to_string())?
    {
        tasks.push(DashboardTask {
            id: row
                .get(0)
                .map_err(|_| "IO_ERROR: invalid task dashboard row".to_string())?,
            project_id: row
                .get(1)
                .map_err(|_| "IO_ERROR: invalid task dashboard row".to_string())?,
            project_name: row
                .get(2)
                .map_err(|_| "IO_ERROR: invalid task dashboard row".to_string())?,
            title: row
                .get(3)
                .map_err(|_| "IO_ERROR: invalid task dashboard row".to_string())?,
            status: row
                .get(4)
                .map_err(|_| "IO_ERROR: invalid task dashboard row".to_string())?,
            priority: row
                .get(5)
                .map_err(|_| "IO_ERROR: invalid task dashboard row".to_string())?,
            due_at: row
                .get(6)
                .map_err(|_| "IO_ERROR: invalid task dashboard row".to_string())?,
        });
    }
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERSONAL: &str = "personal";
    const OTHER: &str = "other";
    const AS_OF: &str = "2026-06-20T12:00:00.000Z";

    fn database() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory dashboard database");
        connection
            .execute_batch(
                "CREATE TABLE projects (
                    id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, name TEXT NOT NULL,
                    status TEXT NOT NULL, updated_at TEXT NOT NULL, archived_at TEXT
                );
                CREATE TABLE tasks (
                    id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, project_id TEXT,
                    title TEXT NOT NULL, status TEXT NOT NULL, priority TEXT NOT NULL,
                    due_at TEXT, archived_at TEXT
                );
                CREATE TABLE activity_events (
                    id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL,
                    entity_type TEXT NOT NULL, entity_id TEXT NOT NULL,
                    action TEXT NOT NULL, summary TEXT NOT NULL, occurred_at TEXT NOT NULL,
                    sensitivity TEXT NOT NULL
                );",
            )
            .expect("create dashboard test schema");
        connection
    }

    fn seed_task(
        connection: &Connection,
        id: &str,
        workspace: &str,
        status: &str,
        due_at: Option<&str>,
        archived_at: Option<&str>,
    ) {
        connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, project_id, title, status, priority, due_at, archived_at)
                 VALUES (?1, ?2, NULL, ?1, ?3, 'p2', ?4, ?5)",
                params![id, workspace, status, due_at, archived_at],
            )
            .expect("insert dashboard task");
    }

    #[test]
    fn dashboard_counts_persisted_work_and_omits_archived_completed_and_other_scope() {
        let connection = database();
        connection
            .execute_batch(
                "INSERT INTO projects VALUES
                    ('recent', 'personal', 'Recent project', 'active', '2026-06-20T11:00:00Z', NULL),
                    ('old', 'personal', 'Older project', 'planned', '2026-06-19T11:00:00Z', NULL),
                    ('archived', 'personal', 'Archived project', 'active', '2026-06-20T12:00:00Z', '2026-06-20T12:00:00Z'),
                    ('other-project', 'other', 'Other project', 'active', '2026-06-20T12:00:00Z', NULL);
                INSERT INTO activity_events VALUES
                    ('real-event', 'personal', 'task', 'late', 'created', 'Task created', '2026-06-20T11:00:00Z', 'normal'),
                    ('private-event', 'personal', 'task', 'late', 'updated', 'Restricted event', '2026-06-20T12:00:00Z', 'restricted'),
                    ('other-event', 'other', 'task', 'x', 'created', 'Other workspace event', '2026-06-20T12:00:00Z', 'normal');",
            )
            .expect("seed dashboard projects and activity");
        seed_task(
            &connection,
            "late",
            PERSONAL,
            "todo",
            Some("2026-06-19T11:00:00Z"),
            None,
        );
        seed_task(
            &connection,
            "soon",
            PERSONAL,
            "in_progress",
            Some("2026-06-25T11:00:00Z"),
            None,
        );
        seed_task(&connection, "blocked", PERSONAL, "blocked", None, None);
        seed_task(
            &connection,
            "done",
            PERSONAL,
            "done",
            Some("2026-06-19T11:00:00Z"),
            None,
        );
        seed_task(
            &connection,
            "archived-task",
            PERSONAL,
            "todo",
            Some("2026-06-19T11:00:00Z"),
            Some(AS_OF),
        );
        seed_task(
            &connection,
            "other-task",
            OTHER,
            "todo",
            Some("2026-06-19T11:00:00Z"),
            None,
        );

        let dashboard = load(
            &connection,
            PERSONAL,
            DateTime::parse_from_rfc3339(AS_OF)
                .expect("fixed UTC time")
                .with_timezone(&Utc),
        )
        .expect("load dashboard");

        assert_eq!(dashboard.active_project_count, 2);
        assert_eq!(dashboard.counts.open_tasks, 3);
        assert_eq!(dashboard.active_task_count, 4);
        assert_eq!(dashboard.counts.overdue_tasks, 1);
        assert_eq!(dashboard.counts.due_soon_tasks, 1);
        assert_eq!(dashboard.counts.blocked_tasks, 1);
        assert_eq!(dashboard.recent_projects[0].id, "recent");
        assert_eq!(dashboard.overdue_tasks[0].id, "late");
        assert_eq!(dashboard.upcoming_tasks[0].id, "soon");
        assert_eq!(dashboard.recent_activity.len(), 1);
        assert_eq!(dashboard.recent_activity[0].id, "real-event");
    }

    #[test]
    fn dashboard_returns_truthful_empty_data_and_requires_workspace_scope() {
        let connection = database();
        let dashboard = load(
            &connection,
            PERSONAL,
            DateTime::parse_from_rfc3339(AS_OF)
                .expect("fixed UTC time")
                .with_timezone(&Utc),
        )
        .expect("empty dashboard is valid");
        assert_eq!(dashboard.active_project_count, 0);
        assert_eq!(dashboard.counts.open_tasks, 0);
        assert_eq!(dashboard.active_task_count, 0);
        assert!(dashboard.recent_projects.is_empty());
        assert!(dashboard.recent_activity.is_empty());
        assert!(load(
            &connection,
            " ",
            DateTime::parse_from_rfc3339(AS_OF)
                .expect("fixed UTC time")
                .with_timezone(&Utc)
        )
        .is_err());
    }
}
