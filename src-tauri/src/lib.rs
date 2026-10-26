pub mod activity;
pub mod cleaner;
pub mod cleanup_reports;
pub mod config;
pub mod connection_profiles;
pub mod connectors;
pub mod diagnostic_task;
pub mod dns_adapter;
pub mod domain;
pub mod event_log_adapter;
pub mod hash_tools;
pub mod home_dashboard;
pub mod intelligence_dedupe;
pub mod intelligence_identity;
pub mod intelligence_items;
pub mod jobs;
pub mod labels;
pub mod local_logging;
pub mod milestones;
pub mod network_diagnostics;
pub mod ping_adapter;
pub mod process_inventory;
pub mod project_task_export;
pub mod projects;
pub mod rss_connector;
pub mod service_inventory;
pub mod subnet_calculator;
pub mod system_inventory;
pub mod task_dependencies;
pub mod task_status;
pub mod tasks;
pub mod traceroute_adapter;

#[cfg(test)]
mod repository_integration_tests;

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

const DATABASE_FILE_NAME: &str = "moonships.sqlite3";
const DATABASE_BUSY_TIMEOUT_MS: u64 = 5_000;

fn database_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(DATABASE_FILE_NAME)
}

fn open_database_in(app_data_dir: &Path) -> Result<Connection, String> {
    fs::create_dir_all(app_data_dir)
        .map_err(|error| format!("Could not create app data directory: {error}"))?;
    let path = database_path(app_data_dir);
    let connection = Connection::open(path)
        .map_err(|error| format!("Could not open Moonships database: {error}"))?;

    connection
        .busy_timeout(Duration::from_millis(DATABASE_BUSY_TIMEOUT_MS))
        .map_err(|error| format!("Could not configure SQLite busy timeout: {error}"))?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("Could not enable SQLite foreign keys: {error}"))?;

    let foreign_keys_enabled: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|error| format!("Could not verify SQLite foreign-key enforcement: {error}"))?;
    if foreign_keys_enabled != 1 {
        return Err("SQLite foreign-key enforcement is not enabled".to_string());
    }

    Ok(connection)
}

const CURRENT_SCHEMA_VERSION: i64 = 10;
const PERSONAL_WORKSPACE_ID: &str = "00000000-0000-4000-8000-000000000001";

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListProjectsRequestDto {}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateProjectRequestDto {
    name: String,
    description: Option<String>,
    goal: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateProjectRequestDto {
    id: String,
    name: String,
    description: Option<String>,
    goal: Option<String>,
    status: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectDto {
    id: String,
    workspace_id: String,
    name: String,
    project_key: Option<String>,
    description: Option<String>,
    goal: Option<String>,
    status: String,
    archived_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<projects::Project> for ProjectDto {
    fn from(project: projects::Project) -> Self {
        Self {
            id: project.id,
            workspace_id: project.workspace_id,
            name: project.name,
            project_key: project.project_key,
            description: project.description,
            goal: project.goal,
            status: project.status,
            archived_at: project.archived_at,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectListResponseDto {
    projects: Vec<ProjectDto>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportProjectTaskResponseDto {
    cancelled: bool,
    file_name: Option<String>,
    project_count: usize,
    task_count: usize,
    activity_event_count: usize,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectTaskImportPreviewDto {
    cancelled: bool,
    file_name: Option<String>,
    schema_version: Option<u32>,
    app_version: Option<String>,
    exported_at: Option<String>,
    project_count: usize,
    task_count: usize,
    activity_event_count: usize,
    project_names: Vec<String>,
    task_titles: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Sha256FileResultDto {
    cancelled: bool,
    file_name: Option<String>,
    bytes_hashed: u64,
    sha256: Option<String>,
}

#[tauri::command]
fn hash_sha256_text(text: String) -> Result<String, String> {
    hash_tools::sha256_text(&text).map_err(|code| match code {
        "text_too_large" => "VALIDATION_FAILED: text exceeds the 1 MiB size limit".to_string(),
        _ => "INTERNAL_ERROR: text could not be hashed".to_string(),
    })
}

#[tauri::command]
async fn hash_sha256_selected_file(app: AppHandle) -> Result<Sha256FileResultDto, String> {
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("UNSUPPORTED: file hashing is currently available on Windows".to_string())
    }

    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| "IO_ERROR: main window is unavailable".to_string())?;
            let Some(path) = rfd::FileDialog::new()
                .set_parent(&window)
                .set_title("Select a file to calculate its SHA-256 hash")
                .pick_file()
            else {
                return Ok(Sha256FileResultDto {
                    cancelled: true,
                    file_name: None,
                    bytes_hashed: 0,
                    sha256: None,
                });
            };

            let file = std::fs::File::open(&path)
                .map_err(|_| "IO_ERROR: could not read the selected file".to_string())?;
            let metadata = file
                .metadata()
                .map_err(|_| "IO_ERROR: could not inspect the selected file".to_string())?;
            if !metadata.is_file() {
                return Err("VALIDATION_FAILED: selected target is not a regular file".to_string());
            }
            if metadata.len() > hash_tools::MAX_FILE_BYTES {
                return Err("VALIDATION_FAILED: file exceeds the 2 GiB size limit".to_string());
            }
            let (sha256, bytes_hashed) =
                hash_tools::sha256_reader(file, hash_tools::MAX_FILE_BYTES).map_err(|error| {
                    if error.kind() == std::io::ErrorKind::InvalidData {
                        "VALIDATION_FAILED: file exceeds the 2 GiB size limit".to_string()
                    } else {
                        "IO_ERROR: could not read the selected file".to_string()
                    }
                })?;

            Ok(Sha256FileResultDto {
                cancelled: false,
                file_name: path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned()),
                bytes_hashed,
                sha256: Some(sha256),
            })
        })
        .await
        .map_err(|_| "IO_ERROR: file hashing did not complete".to_string())?
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateTaskRequestDto {
    title: String,
    project_id: Option<String>,
    parent_task_id: Option<String>,
    status: String,
    priority: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateTaskFromDiagnosticRequestDto {
    diagnostic_result_id: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListTasksRequestDto {}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TaskRequestDto {
    id: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateTaskRequestDto {
    id: String,
    project_id: Option<String>,
    title: String,
    description: Option<String>,
    due_at: Option<String>,
    status: String,
    priority: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TaskDto {
    id: String,
    workspace_id: String,
    project_id: Option<String>,
    parent_task_id: Option<String>,
    title: String,
    description: Option<String>,
    due_at: Option<String>,
    status: String,
    priority: String,
    created_at: String,
    updated_at: String,
    archived_at: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TaskListResponseDto {
    tasks: Vec<TaskDto>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TaskActivityDto {
    id: String,
    entity_type: String,
    action: String,
    summary: String,
    occurred_at: String,
}

impl From<activity::TaskActivity> for TaskActivityDto {
    fn from(event: activity::TaskActivity) -> Self {
        Self {
            id: event.id,
            entity_type: event.entity_type,
            action: event.action,
            summary: event.summary,
            occurred_at: event.occurred_at,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TaskActivityListResponseDto {
    events: Vec<TaskActivityDto>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectOverviewRequestDto {
    id: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectActivityDto {
    id: String,
    entity_type: String,
    action: String,
    summary: String,
    occurred_at: String,
}

impl From<activity::ProjectActivity> for ProjectActivityDto {
    fn from(event: activity::ProjectActivity) -> Self {
        Self {
            id: event.id,
            entity_type: event.entity_type,
            action: event.action,
            summary: event.summary,
            occurred_at: event.occurred_at,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectOverviewResponseDto {
    project: ProjectDto,
    active_task_count: u64,
    recent_activity: Vec<ProjectActivityDto>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateMilestoneRequestDto {
    project_id: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListMilestonesRequestDto {
    project_id: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssociateMilestoneTaskRequestDto {
    project_id: String,
    milestone_id: String,
    task_id: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateMilestoneRequestDto {
    project_id: String,
    id: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
    status: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MilestoneTaskDto {
    id: String,
    title: String,
    status: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MilestoneDto {
    id: String,
    workspace_id: String,
    project_id: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
    status: String,
    sort_order: i64,
    created_at: String,
    updated_at: String,
    tasks: Vec<MilestoneTaskDto>,
}

impl From<milestones::Milestone> for MilestoneDto {
    fn from(value: milestones::Milestone) -> Self {
        Self {
            id: value.id,
            workspace_id: value.workspace_id,
            project_id: value.project_id,
            title: value.title,
            description: value.description,
            due_date: value.due_date,
            status: value.status,
            sort_order: value.sort_order,
            created_at: value.created_at,
            updated_at: value.updated_at,
            tasks: value
                .tasks
                .into_iter()
                .map(|task| MilestoneTaskDto {
                    id: task.id,
                    title: task.title,
                    status: task.status,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MilestoneListResponseDto {
    milestones: Vec<MilestoneDto>,
}

impl From<tasks::Task> for TaskDto {
    fn from(task: tasks::Task) -> Self {
        Self {
            id: task.id,
            workspace_id: task.workspace_id,
            project_id: task.project_id,
            parent_task_id: task.parent_task_id,
            title: task.title,
            description: task.description,
            due_at: task.due_at,
            status: task.status,
            priority: task.priority,
            created_at: task.created_at,
            updated_at: task.updated_at,
            archived_at: task.archived_at,
        }
    }
}

pub(crate) fn run_migrations(connection: &mut Connection) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Could not begin schema migration transaction: {error}"))?;

    transaction
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            );",
        )
        .map_err(|error| format!("Could not create schema version table: {error}"))?;

    let current_version: i64 = transaction
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("Could not read schema version: {error}"))?;

    if current_version > CURRENT_SCHEMA_VERSION {
        return Err(format!(
            "Database schema version {current_version} is newer than this app supports ({CURRENT_SCHEMA_VERSION})"
        ));
    }

    if current_version < 1 {
        transaction
            .execute_batch(
                "CREATE TABLE app_metadata (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL
                );
                INSERT INTO schema_migrations (version) VALUES (1);",
            )
            .map_err(|error| format!("Could not apply initial schema migration: {error}"))?;
    }

    if current_version < 2 {
        transaction
            .execute_batch(
                "CREATE TABLE workspaces (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
                    description TEXT,
                    profile_type TEXT,
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    archived_at TEXT
                );",
            )
            .map_err(|error| format!("Could not create workspace table: {error}"))?;
        transaction
            .execute(
                "INSERT INTO workspaces (id, name) VALUES (?1, ?2)",
                rusqlite::params![PERSONAL_WORKSPACE_ID, "Personal"],
            )
            .map_err(|error| format!("Could not seed Personal workspace: {error}"))?;
        transaction
            .execute("INSERT INTO schema_migrations (version) VALUES (2)", [])
            .map_err(|error| format!("Could not record workspace migration: {error}"))?;
    }

    if current_version < 3 {
        transaction
            .execute_batch(
                "CREATE TABLE projects (
                    id TEXT PRIMARY KEY NOT NULL,
                    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE RESTRICT,
                    project_key TEXT,
                    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
                    description TEXT,
                    goal TEXT,
                    status TEXT NOT NULL DEFAULT 'planned'
                        CHECK (status IN ('planned', 'active', 'paused', 'completed', 'archived')),
                    color_token TEXT,
                    start_date TEXT,
                    target_date TEXT,
                    archived_at TEXT,
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    UNIQUE (workspace_id, project_key)
                );
                CREATE INDEX idx_projects_workspace_status
                    ON projects (workspace_id, status);
                INSERT INTO schema_migrations (version) VALUES (3);",
            )
            .map_err(|error| format!("Could not apply project schema migration: {error}"))?;
    }

    if current_version < 4 {
        transaction
            .execute_batch(
                "CREATE UNIQUE INDEX idx_projects_workspace_id
                    ON projects (workspace_id, id);
                CREATE TABLE tasks (
                    id TEXT PRIMARY KEY NOT NULL,
                    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE RESTRICT,
                    project_id TEXT,
                    parent_task_id TEXT,
                    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
                    description TEXT,
                    status TEXT NOT NULL DEFAULT 'backlog'
                        CHECK (status IN (
                            'backlog', 'todo', 'in_progress', 'blocked',
                            'in_review', 'done', 'cancelled'
                        )),
                    priority TEXT NOT NULL DEFAULT 'p2'
                        CHECK (priority IN ('p0', 'p1', 'p2', 'p3')),
                    due_at TEXT,
                    estimate_minutes INTEGER CHECK (
                        estimate_minutes IS NULL OR estimate_minutes >= 0
                    ),
                    completed_at TEXT,
                    archived_at TEXT,
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    FOREIGN KEY (workspace_id, project_id)
                        REFERENCES projects (workspace_id, id) ON DELETE RESTRICT,
                    FOREIGN KEY (workspace_id, parent_task_id)
                        REFERENCES tasks (workspace_id, id) ON DELETE RESTRICT,
                    CHECK (parent_task_id IS NULL OR parent_task_id != id),
                    UNIQUE (workspace_id, id)
                );
                CREATE INDEX idx_tasks_workspace_status
                    ON tasks (workspace_id, status);
                CREATE INDEX idx_tasks_workspace_project
                    ON tasks (workspace_id, project_id);
                INSERT INTO schema_migrations (version) VALUES (4);",
            )
            .map_err(|error| format!("Could not apply task schema migration: {error}"))?;
    }

    if current_version < 5 {
        transaction
            .execute_batch(
                "CREATE TABLE activity_events (
                    id TEXT PRIMARY KEY NOT NULL,
                    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE RESTRICT,
                    actor_type TEXT NOT NULL DEFAULT 'local'
                        CHECK (actor_type IN ('local', 'system', 'tool')),
                    entity_type TEXT NOT NULL
                        CHECK (entity_type IN ('project', 'task')),
                    entity_id TEXT NOT NULL CHECK (length(trim(entity_id)) > 0),
                    project_id TEXT,
                    task_id TEXT,
                    action TEXT NOT NULL CHECK (length(trim(action)) > 0),
                    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    summary TEXT NOT NULL CHECK (length(trim(summary)) > 0),
                    sensitivity TEXT NOT NULL DEFAULT 'normal'
                        CHECK (sensitivity IN ('normal', 'restricted')),
                    FOREIGN KEY (workspace_id, project_id)
                        REFERENCES projects(workspace_id, id) ON DELETE RESTRICT,
                    FOREIGN KEY (workspace_id, task_id)
                        REFERENCES tasks(workspace_id, id) ON DELETE RESTRICT,
                    CHECK (
                        (entity_type = 'project' AND project_id IS NOT NULL
                         AND task_id IS NULL AND entity_id = project_id)
                        OR
                        (entity_type = 'task' AND task_id IS NOT NULL
                         AND project_id IS NULL AND entity_id = task_id)
                    )
                );
                CREATE INDEX idx_activity_events_entity
                    ON activity_events(workspace_id, entity_type, entity_id, occurred_at);
                INSERT INTO schema_migrations (version) VALUES (5);",
            )
            .map_err(|error| format!("Could not apply activity schema migration: {error}"))?;
    }

    if current_version < 6 {
        transaction
            .execute_batch(
                "CREATE TABLE labels (
                    id TEXT PRIMARY KEY NOT NULL,
                    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE RESTRICT,
                    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 64),
                    normalized_name TEXT NOT NULL CHECK (length(trim(normalized_name)) > 0),
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    UNIQUE (workspace_id, normalized_name),
                    UNIQUE (workspace_id, id)
                );
                CREATE INDEX idx_labels_workspace_name
                    ON labels (workspace_id, normalized_name, id);
                CREATE TABLE task_labels (
                    workspace_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    label_id TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    PRIMARY KEY (workspace_id, task_id, label_id),
                    FOREIGN KEY (workspace_id, task_id)
                        REFERENCES tasks (workspace_id, id) ON DELETE CASCADE,
                    FOREIGN KEY (workspace_id, label_id)
                        REFERENCES labels (workspace_id, id) ON DELETE CASCADE
                );
                CREATE INDEX idx_task_labels_workspace_label
                    ON task_labels (workspace_id, label_id, task_id);
                INSERT INTO schema_migrations (version) VALUES (6);",
            )
            .map_err(|error| format!("Could not apply label schema migration: {error}"))?;
    }

    if current_version < 7 {
        transaction
            .execute_batch(
                "CREATE TABLE task_dependencies (
                    workspace_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    depends_on_task_id TEXT NOT NULL,
                    dependency_type TEXT NOT NULL DEFAULT 'blocked_by'
                        CHECK (dependency_type = 'blocked_by'),
                    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                    PRIMARY KEY (workspace_id, task_id, depends_on_task_id),
                    FOREIGN KEY (workspace_id, task_id)
                        REFERENCES tasks (workspace_id, id) ON DELETE CASCADE,
                    FOREIGN KEY (workspace_id, depends_on_task_id)
                        REFERENCES tasks (workspace_id, id) ON DELETE CASCADE,
                    CHECK (task_id != depends_on_task_id)
                );
                CREATE INDEX idx_task_dependencies_workspace_prerequisite
                    ON task_dependencies (workspace_id, depends_on_task_id, task_id);
                INSERT INTO schema_migrations (version) VALUES (7);",
            )
            .map_err(|error| {
                format!("Could not apply Task dependency schema migration: {error}")
            })?;
    }

    if current_version < 8 {
        transaction
            .execute_batch(
                "CREATE UNIQUE INDEX idx_tasks_workspace_project_identity
                    ON tasks (workspace_id, project_id, id);
                CREATE TABLE milestones (
                id TEXT PRIMARY KEY NOT NULL,
                workspace_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 160),
                description TEXT,
                due_date TEXT CHECK (due_date IS NULL OR due_date GLOB '????-??-??'),
                status TEXT NOT NULL DEFAULT 'planned'
                    CHECK (status IN ('planned', 'active', 'completed')),
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                UNIQUE (workspace_id, project_id, id),
                FOREIGN KEY (workspace_id, project_id)
                    REFERENCES projects (workspace_id, id) ON DELETE RESTRICT
            );
            CREATE INDEX idx_milestones_project_order
                ON milestones (workspace_id, project_id, sort_order, due_date, id);
            CREATE TABLE milestone_tasks (
                workspace_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                milestone_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                PRIMARY KEY (workspace_id, milestone_id, task_id),
                FOREIGN KEY (workspace_id, project_id, milestone_id)
                    REFERENCES milestones (workspace_id, project_id, id) ON DELETE CASCADE,
                FOREIGN KEY (workspace_id, project_id, task_id)
                    REFERENCES tasks (workspace_id, project_id, id) ON DELETE CASCADE
            );
            CREATE INDEX idx_milestone_tasks_project_task
                ON milestone_tasks (workspace_id, project_id, task_id);
            INSERT INTO schema_migrations (version) VALUES (8);",
            )
            .map_err(|error| format!("Could not apply milestone schema migration: {error}"))?;
    }

    if current_version < 9 {
        transaction
            .execute_batch(
                "CREATE TABLE cleanup_reports (
                    id TEXT PRIMARY KEY NOT NULL,
                    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE RESTRICT,
                    task_id TEXT,
                    preview_id TEXT NOT NULL,
                    status TEXT NOT NULL CHECK (status IN ('succeeded', 'partial', 'failed')),
                    completed_at TEXT NOT NULL,
                    selected_count INTEGER NOT NULL CHECK (selected_count BETWEEN 1 AND 100),
                    deleted_count INTEGER NOT NULL CHECK (deleted_count >= 0),
                    skipped_count INTEGER NOT NULL CHECK (skipped_count >= 0),
                    failed_count INTEGER NOT NULL CHECK (failed_count >= 0),
                    estimated_bytes INTEGER NOT NULL CHECK (estimated_bytes >= 0),
                    actual_deleted_bytes INTEGER NOT NULL
                        CHECK (actual_deleted_bytes >= 0 AND actual_deleted_bytes <= estimated_bytes),
                    CHECK (selected_count = deleted_count + skipped_count + failed_count),
                    FOREIGN KEY (workspace_id, task_id)
                        REFERENCES tasks (workspace_id, id) ON DELETE RESTRICT
                );
                CREATE INDEX idx_cleanup_reports_workspace_completed
                    ON cleanup_reports (workspace_id, completed_at DESC, id);
                CREATE INDEX idx_cleanup_reports_workspace_task
                    ON cleanup_reports (workspace_id, task_id, completed_at DESC);
                INSERT INTO schema_migrations (version) VALUES (9);",
            )
            .map_err(|error| format!("Could not apply cleanup report migration: {error}"))?;
    }

    if current_version < 10 {
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 160),
                status TEXT NOT NULL CHECK (status IN ('queued','running','waiting_approval','succeeded','partial','failed','cancelled','timed_out')),
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
                started_at TEXT,
                finished_at TEXT,
                stage TEXT CHECK (stage IS NULL OR length(stage) <= 160),
                progress_percent REAL CHECK (progress_percent IS NULL OR progress_percent BETWEEN 0 AND 100),
                result_summary TEXT CHECK (result_summary IS NULL OR length(result_summary) <= 500)
            );
            CREATE INDEX IF NOT EXISTS idx_jobs_created ON jobs(created_at DESC,id DESC);
            INSERT OR IGNORE INTO schema_migrations (version) VALUES (10);"
        ).map_err(|error| format!("Could not apply job migration: {error}"))?;
    }

    transaction
        .commit()
        .map_err(|error| format!("Could not commit schema migration: {error}"))
}

fn open_personal_workspace_database(app: &AppHandle) -> Result<Connection, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "IO_ERROR: could not resolve application data directory".to_string())?;
    let mut connection = open_database_in(&app_data_dir)?;
    run_migrations(&mut connection)?;
    Ok(connection)
}

#[tauri::command]
fn list_projects(
    app: AppHandle,
    request: ListProjectsRequestDto,
) -> Result<ProjectListResponseDto, String> {
    let _ = request;
    let connection = open_personal_workspace_database(&app)?;
    let projects = projects::list_projects(&connection, PERSONAL_WORKSPACE_ID)?
        .into_iter()
        .map(ProjectDto::from)
        .collect();
    Ok(ProjectListResponseDto { projects })
}

#[tauri::command]
fn create_project(app: AppHandle, request: CreateProjectRequestDto) -> Result<ProjectDto, String> {
    let mut connection = open_personal_workspace_database(&app)?;
    projects::create_project(
        &mut connection,
        projects::CreateProject {
            workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
            name: request.name,
            project_key: None,
            description: request.description,
            goal: request.goal,
            status: None,
        },
    )
    .map(ProjectDto::from)
}

#[tauri::command]
fn update_project(app: AppHandle, request: UpdateProjectRequestDto) -> Result<ProjectDto, String> {
    let mut connection = open_personal_workspace_database(&app)?;
    projects::update_project(
        &mut connection,
        projects::UpdateProject {
            workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
            id: request.id,
            name: request.name,
            description: request.description,
            goal: request.goal,
            status: request.status,
        },
    )
    .map(ProjectDto::from)
}

#[tauri::command]
async fn export_project_task_data(app: AppHandle) -> Result<ExportProjectTaskResponseDto, String> {
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("UNSUPPORTED: Project and Task export is currently available on Windows".to_string())
    }

    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let connection = open_personal_workspace_database(&app)?;
            let document =
                project_task_export::load_personal_export(&connection, env!("CARGO_PKG_VERSION"))?;
            let json = document.to_json()?;
            let project_count = document.projects.len();
            let task_count = document.tasks.len();
            let activity_event_count = document.activity_events.len();
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| "IO_ERROR: main window is unavailable".to_string())?;
            let Some(path) = rfd::FileDialog::new()
                .set_parent(&window)
                .set_title("Export Moonships Projects and Tasks")
                .set_file_name("moonships-projects-tasks.json")
                .add_filter("JSON", &["json"])
                .save_file()
            else {
                return Ok(ExportProjectTaskResponseDto {
                    cancelled: true,
                    file_name: None,
                    project_count: 0,
                    task_count: 0,
                    activity_event_count: 0,
                });
            };
            std::fs::write(&path, json)
                .map_err(|_| "IO_ERROR: could not save the selected export file".to_string())?;
            Ok(ExportProjectTaskResponseDto {
                cancelled: false,
                file_name: path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned()),
                project_count,
                task_count,
                activity_event_count,
            })
        })
        .await
        .map_err(|_| "IO_ERROR: export operation did not complete".to_string())?
    }
}

#[tauri::command]
async fn preview_project_task_import(
    app: AppHandle,
) -> Result<ProjectTaskImportPreviewDto, String> {
    #[cfg(not(windows))]
    {
        let _ = app;
        Err(
            "UNSUPPORTED: Project and Task import preview is currently available on Windows"
                .to_string(),
        )
    }

    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| "IO_ERROR: main window is unavailable".to_string())?;
            let Some(path) = rfd::FileDialog::new()
                .set_parent(&window)
                .set_title("Preview Moonships Project and Task import")
                .add_filter("Moonships JSON export", &["json"])
                .pick_file()
            else {
                return Ok(ProjectTaskImportPreviewDto {
                    cancelled: true,
                    file_name: None,
                    schema_version: None,
                    app_version: None,
                    exported_at: None,
                    project_count: 0,
                    task_count: 0,
                    activity_event_count: 0,
                    project_names: Vec::new(),
                    task_titles: Vec::new(),
                });
            };

            let file = std::fs::File::open(&path)
                .map_err(|_| "IO_ERROR: could not read the selected import file".to_string())?;
            let mut bytes = Vec::new();
            file.take((project_task_export::MAX_EXPORT_SIZE_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| "IO_ERROR: could not read the selected import file".to_string())?;
            if bytes.len() > project_task_export::MAX_EXPORT_SIZE_BYTES {
                return Err(
                    "VALIDATION_FAILED: import file exceeds the 50 MiB size limit".to_string(),
                );
            }
            let json = std::str::from_utf8(&bytes)
                .map_err(|_| "VALIDATION_FAILED: import file must be UTF-8 JSON".to_string())?;
            let document = project_task_export::ProjectTaskExport::from_json(json)?;
            Ok(ProjectTaskImportPreviewDto {
                cancelled: false,
                file_name: path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned()),
                schema_version: Some(document.schema_version),
                app_version: Some(document.app_version),
                exported_at: Some(document.exported_at),
                project_count: document.projects.len(),
                task_count: document.tasks.len(),
                activity_event_count: document.activity_events.len(),
                project_names: document
                    .projects
                    .iter()
                    .take(5)
                    .map(|project| project.name.clone())
                    .collect(),
                task_titles: document
                    .tasks
                    .iter()
                    .take(5)
                    .map(|task| task.title.clone())
                    .collect(),
            })
        })
        .await
        .map_err(|_| "IO_ERROR: import preview did not complete".to_string())?
    }
}

#[tauri::command]
fn create_task(app: AppHandle, request: CreateTaskRequestDto) -> Result<TaskDto, String> {
    let mut connection = open_personal_workspace_database(&app)?;
    tasks::create_task(
        &mut connection,
        tasks::CreateTask {
            workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
            project_id: request.project_id,
            parent_task_id: request.parent_task_id,
            title: request.title,
            description: None,
            due_at: None,
            status: Some(request.status),
            priority: Some(request.priority),
        },
    )
    .map(TaskDto::from)
}

#[tauri::command]
fn list_tasks(app: AppHandle, request: ListTasksRequestDto) -> Result<TaskListResponseDto, String> {
    let _ = request;
    let connection = open_personal_workspace_database(&app)?;
    let tasks = tasks::list_tasks(
        &connection,
        PERSONAL_WORKSPACE_ID,
        tasks::TaskArchiveFilter::Active,
    )?
    .into_iter()
    .map(TaskDto::from)
    .collect();
    Ok(TaskListResponseDto { tasks })
}

#[tauri::command]
fn get_home_dashboard(app: AppHandle) -> Result<home_dashboard::HomeDashboard, String> {
    let connection = open_personal_workspace_database(&app)?;
    home_dashboard::load(
        &connection,
        PERSONAL_WORKSPACE_ID,
        chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now()),
    )
}

#[tauri::command]
fn list_process_inventory() -> process_inventory::ProcessInventorySnapshot {
    process_inventory::read_process_inventory()
}

#[tauri::command]
fn list_service_inventory() -> service_inventory::ServiceInventorySnapshot {
    service_inventory::read_service_inventory()
}

#[tauri::command]
fn list_jobs(app: AppHandle) -> Result<Vec<jobs::JobRecord>, String> {
    let connection = open_personal_workspace_database(&app)?;
    jobs::list(&connection)
}

#[tauri::command]
fn run_network_diagnostic(
    request: diagnostic_task::DiagnosticRequest,
) -> Result<diagnostic_task::DiagnosticResultRecord, String> {
    diagnostic_task::run(request)
}

#[tauri::command]
fn create_task_from_diagnostic(
    app: AppHandle,
    request: CreateTaskFromDiagnosticRequestDto,
) -> Result<TaskDto, String> {
    let mut connection = open_personal_workspace_database(&app)?;
    diagnostic_task::create_task_from_result(
        &mut connection,
        PERSONAL_WORKSPACE_ID,
        &request.diagnostic_result_id,
    )
    .map(TaskDto::from)
}

#[tauri::command]
fn get_task(app: AppHandle, request: TaskRequestDto) -> Result<TaskDto, String> {
    let connection = open_personal_workspace_database(&app)?;
    tasks::get_task(&connection, PERSONAL_WORKSPACE_ID, &request.id).map(TaskDto::from)
}

#[tauri::command]
fn list_task_activity(
    app: AppHandle,
    request: TaskRequestDto,
) -> Result<TaskActivityListResponseDto, String> {
    let connection = open_personal_workspace_database(&app)?;
    let events = activity::list_task_activity(&connection, PERSONAL_WORKSPACE_ID, &request.id)?
        .into_iter()
        .map(TaskActivityDto::from)
        .collect();
    Ok(TaskActivityListResponseDto { events })
}

#[tauri::command]
fn update_task(app: AppHandle, request: UpdateTaskRequestDto) -> Result<TaskDto, String> {
    let mut connection = open_personal_workspace_database(&app)?;
    tasks::update_task(
        &mut connection,
        tasks::UpdateTask {
            workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
            id: request.id,
            project_id: request.project_id,
            title: request.title,
            description: request.description,
            due_at: request.due_at,
            status: request.status,
            priority: request.priority,
        },
    )
    .map(TaskDto::from)
}

#[tauri::command]
fn get_project_overview(
    app: AppHandle,
    request: ProjectOverviewRequestDto,
) -> Result<ProjectOverviewResponseDto, String> {
    let connection = open_personal_workspace_database(&app)?;
    let project = projects::get_project(&connection, PERSONAL_WORKSPACE_ID, &request.id)?;
    let active_task_count =
        tasks::count_active_project_tasks(&connection, PERSONAL_WORKSPACE_ID, &project.id)?;
    let recent_activity =
        activity::list_project_activity(&connection, PERSONAL_WORKSPACE_ID, &project.id)?
            .into_iter()
            .map(ProjectActivityDto::from)
            .collect();
    Ok(ProjectOverviewResponseDto {
        project: ProjectDto::from(project),
        active_task_count,
        recent_activity,
    })
}

#[tauri::command]
fn list_milestones(
    app: AppHandle,
    request: ListMilestonesRequestDto,
) -> Result<MilestoneListResponseDto, String> {
    let connection = open_personal_workspace_database(&app)?;
    let milestones =
        milestones::list_milestones(&connection, PERSONAL_WORKSPACE_ID, &request.project_id)?
            .into_iter()
            .map(MilestoneDto::from)
            .collect();
    Ok(MilestoneListResponseDto { milestones })
}

#[tauri::command]
fn list_cleanup_reports(
    app: AppHandle,
) -> Result<Vec<cleanup_reports::CleanupReportSummaryDto>, String> {
    let connection = open_personal_workspace_database(&app)?;
    cleanup_reports::list_recent_cleanup_reports(&connection, PERSONAL_WORKSPACE_ID)
}

#[tauri::command]
fn create_milestone(
    app: AppHandle,
    request: CreateMilestoneRequestDto,
) -> Result<MilestoneDto, String> {
    let mut connection = open_personal_workspace_database(&app)?;
    milestones::create_milestone(
        &mut connection,
        milestones::CreateMilestone {
            workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
            project_id: request.project_id,
            title: request.title,
            description: request.description,
            due_date: request.due_date,
        },
    )
    .map(MilestoneDto::from)
}

#[tauri::command]
fn associate_milestone_task(
    app: AppHandle,
    request: AssociateMilestoneTaskRequestDto,
) -> Result<MilestoneDto, String> {
    let mut connection = open_personal_workspace_database(&app)?;
    milestones::associate_task(
        &mut connection,
        PERSONAL_WORKSPACE_ID,
        &request.project_id,
        &request.milestone_id,
        &request.task_id,
    )
    .map(MilestoneDto::from)
}

#[tauri::command]
fn update_milestone(
    app: AppHandle,
    request: UpdateMilestoneRequestDto,
) -> Result<MilestoneDto, String> {
    let mut connection = open_personal_workspace_database(&app)?;
    milestones::update_milestone(
        &mut connection,
        milestones::UpdateMilestone {
            workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
            project_id: request.project_id,
            id: request.id,
            title: request.title,
            description: request.description,
            due_date: request.due_date,
            status: request.status,
        },
    )
    .map(MilestoneDto::from)
}

fn hide_to_tray_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("hide-to-tray"))
        .map_err(|error| format!("Could not resolve app config directory: {error}"))
}

fn hide_to_tray_enabled(path: &std::path::Path) -> bool {
    fs::read_to_string(path)
        .map(|value| value.trim() == "true")
        .unwrap_or(false)
}

// Native SQLite is the source of truth for versioned config. A legacy hide-to-tray
// preference is imported only while no versioned config record has been written.
fn read_app_config(app: &AppHandle) -> Result<domain::AppConfig, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve app data directory: {error}"))?;
    let mut connection = open_database_in(&app_data_dir)?;
    run_migrations(&mut connection)?;
    let legacy = hide_to_tray_enabled(&hide_to_tray_path(app)?);
    config::config_or_default(&connection, legacy)
}

fn write_app_config(app: &AppHandle, request: &domain::AppConfig) -> Result<(), String> {
    request.validate()?;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve app data directory: {error}"))?;
    let mut connection = open_database_in(&app_data_dir)?;
    run_migrations(&mut connection)?;
    config::save_config(&connection, request)
}

#[tauri::command]
fn get_app_config(app: AppHandle) -> Result<domain::AppConfig, String> {
    read_app_config(&app)
}

#[tauri::command]
fn set_app_config(app: AppHandle, request: domain::AppConfig) -> Result<domain::AppConfig, String> {
    write_app_config(&app, &request)?;
    Ok(request)
}

/// Stable IPC payload for reading and updating the hide-to-tray preference.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HideToTrayPreferenceDto {
    enabled: bool,
}

#[tauri::command]
fn get_hide_to_tray(app: AppHandle) -> Result<HideToTrayPreferenceDto, String> {
    Ok(HideToTrayPreferenceDto {
        enabled: read_app_config(&app)?.hide_to_tray,
    })
}

#[tauri::command]
fn set_hide_to_tray(
    app: AppHandle,
    request: HideToTrayPreferenceDto,
) -> Result<HideToTrayPreferenceDto, String> {
    let mut config = read_app_config(&app)?;
    config.hide_to_tray = request.enabled;
    write_app_config(&app, &config)?;
    Ok(request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        tray::TrayIconBuilder,
        Emitter, Manager, WindowEvent,
    };

    tauri::Builder::default()
        // A second launch focuses the existing window instead of creating a duplicate instance.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            get_hide_to_tray,
            set_hide_to_tray,
            get_app_config,
            set_app_config,
            list_projects,
            create_project,
            update_project,
            get_home_dashboard,
            list_process_inventory,
            list_service_inventory,
            list_jobs,
            run_network_diagnostic,
            create_task_from_diagnostic,
            export_project_task_data,
            preview_project_task_import,
            create_task,
            list_tasks,
            get_task,
            list_task_activity,
            update_task,
            get_project_overview,
            list_milestones,
            create_milestone,
            associate_milestone_task,
            update_milestone,
            list_cleanup_reports,
            hash_sha256_text,
            hash_sha256_selected_file
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let startup_event = local_logging::LogEvent::new(
                local_logging::LogLevel::Info,
                local_logging::LogCategory::AppLifecycle,
                "app_starting",
                std::collections::BTreeMap::new(),
            );
            if let Err(error) = local_logging::write_event(
                &app_data_dir.join("logs").join("moonships.log"),
                &startup_event,
            ) {
                eprintln!("Moonships local logging initialization failed: {error}");
            }
            // Open/create the database only under Tauri's per-user application data directory.
            let mut database = open_database_in(&app_data_dir).map_err(std::io::Error::other)?;
            run_migrations(&mut database).map_err(std::io::Error::other)?;
            let _database = database;

            let open = MenuItem::with_id(app, "open", "Open Moonships", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Moonships", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &settings, &hide, &separator, &quit])?;

            let mut tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Moonships Workbench")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                        let _ = app.emit("tray-settings", ());
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    // There are no background worker services registered yet; when one is
                    // introduced, its shutdown hook must run before this explicit exit.
                    "quit" => app.exit(0),
                    _ => {}
                });

            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }

            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if read_app_config(window.app_handle())
                    .map(|config| config.hide_to_tray)
                    .unwrap_or(false)
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Moonships");
}

#[cfg(test)]
mod tests {
    use super::{
        database_path, hide_to_tray_enabled, open_database_in, run_migrations,
        AssociateMilestoneTaskRequestDto, CreateMilestoneRequestDto, CreateProjectRequestDto,
        CreateTaskRequestDto, ListMilestonesRequestDto, ListProjectsRequestDto,
        ListTasksRequestDto, MilestoneDto, MilestoneListResponseDto, ProjectActivityDto,
        ProjectDto, ProjectListResponseDto, ProjectOverviewRequestDto, ProjectOverviewResponseDto,
        TaskActivityDto, TaskActivityListResponseDto, TaskDto, TaskListResponseDto, TaskRequestDto,
        UpdateTaskRequestDto, CURRENT_SCHEMA_VERSION, DATABASE_BUSY_TIMEOUT_MS, DATABASE_FILE_NAME,
        PERSONAL_WORKSPACE_ID,
    };
    use std::{
        fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "moonships-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn project_ipc_dtos_use_camel_case_and_reject_unknown_request_fields() {
        let request: CreateProjectRequestDto =
            serde_json::from_str(r#"{"name":"Moonships","description":null,"goal":null}"#)
                .expect("valid create DTO");
        assert_eq!(request.name, "Moonships");
        assert!(serde_json::from_str::<CreateProjectRequestDto>(
            r#"{"name":"Moonships","workspaceId":"other","description":null,"goal":null}"#
        )
        .is_err());
        assert!(serde_json::from_str::<ListProjectsRequestDto>(r#"{"path":"C:/data"}"#).is_err());

        let response = ProjectListResponseDto {
            projects: vec![ProjectDto {
                id: "project-id".to_string(),
                workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
                name: "Moonships".to_string(),
                project_key: None,
                description: None,
                goal: None,
                status: "planned".to_string(),
                archived_at: None,
                created_at: "2026-10-26T00:00:00Z".to_string(),
                updated_at: "2026-10-26T00:00:00Z".to_string(),
            }],
        };
        let json = serde_json::to_value(response).expect("serialize project list");
        assert_eq!(json["projects"][0]["workspaceId"], PERSONAL_WORKSPACE_ID);
        assert_eq!(json["projects"][0]["createdAt"], "2026-10-26T00:00:00Z");
        assert!(json["projects"][0].get("workspace_id").is_none());
    }

    #[test]
    fn task_create_ipc_is_scoped_and_serializes_canonical_fields() {
        let request: CreateTaskRequestDto = serde_json::from_str(
            r#"{"title":"Write acceptance criteria","projectId":null,"parentTaskId":"parent-id","status":"todo","priority":"p2"}"#,
        )
        .expect("valid task create DTO");
        assert_eq!(request.project_id, None);
        assert_eq!(request.parent_task_id.as_deref(), Some("parent-id"));
        assert_eq!(request.status, "todo");
        assert!(serde_json::from_str::<CreateTaskRequestDto>(
            r#"{"title":"Task","projectId":null,"status":"todo","priority":"p2","workspaceId":"other"}"#
        )
        .is_err());

        let task = TaskDto {
            id: "task-id".to_string(),
            workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
            project_id: None,
            parent_task_id: None,
            title: request.title,
            description: None,
            due_at: None,
            status: request.status,
            priority: request.priority,
            created_at: "2026-10-26T00:00:00Z".to_string(),
            updated_at: "2026-10-26T00:00:00Z".to_string(),
            archived_at: None,
        };
        let json = serde_json::to_value(task).expect("serialize task response");
        assert_eq!(json["workspaceId"], PERSONAL_WORKSPACE_ID);
        assert_eq!(json["projectId"], serde_json::Value::Null);
        assert_eq!(json["createdAt"], "2026-10-26T00:00:00Z");
        assert!(json.get("workspace_id").is_none());
    }

    #[test]
    fn task_list_ipc_rejects_scope_arguments_and_returns_camel_case_records() {
        assert!(serde_json::from_str::<ListTasksRequestDto>(r#"{"workspaceId":"other"}"#).is_err());
        assert!(serde_json::from_str::<ListTasksRequestDto>(r#"{"archived":true}"#).is_err());

        let response = TaskListResponseDto {
            tasks: vec![TaskDto {
                id: "task-id".to_string(),
                workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
                project_id: Some("project-id".to_string()),
                parent_task_id: None,
                title: "Persisted task".to_string(),
                description: None,
                due_at: None,
                status: "todo".to_string(),
                priority: "p2".to_string(),
                created_at: "2026-10-26T00:00:00Z".to_string(),
                updated_at: "2026-10-26T00:00:00Z".to_string(),
                archived_at: None,
            }],
        };
        let json = serde_json::to_value(response).expect("serialize task list");
        assert_eq!(json["tasks"][0]["workspaceId"], PERSONAL_WORKSPACE_ID);
        assert_eq!(json["tasks"][0]["projectId"], "project-id");
        assert_eq!(json["tasks"][0]["createdAt"], "2026-10-26T00:00:00Z");
        assert!(json["tasks"][0].get("workspace_id").is_none());
    }

    #[test]
    fn task_detail_and_update_ipc_reject_scope_and_arbitrary_fields() {
        let request: TaskRequestDto =
            serde_json::from_str(r#"{"id":"task-id"}"#).expect("task ID request");
        assert_eq!(request.id, "task-id");
        assert!(serde_json::from_str::<TaskRequestDto>(
            r#"{"id":"task-id","workspaceId":"other"}"#
        )
        .is_err());

        let update: UpdateTaskRequestDto = serde_json::from_str(
            r#"{"id":"task-id","projectId":null,"title":"Updated","description":"Details","dueAt":"2026-10-27T12:30:00Z","status":"todo","priority":"p1"}"#,
        )
        .expect("typed task update");
        assert_eq!(update.due_at.as_deref(), Some("2026-10-27T12:30:00Z"));
        assert!(serde_json::from_str::<UpdateTaskRequestDto>(
            r#"{"id":"task-id","projectId":null,"title":"Task","description":null,"dueAt":null,"status":"todo","priority":"p2","path":"C:/data"}"#
        )
        .is_err());
    }

    #[test]
    fn task_activity_ipc_is_narrow_and_serializes_trusted_event_fields() {
        let request: TaskRequestDto =
            serde_json::from_str(r#"{"id":"task-id"}"#).expect("task activity request");
        assert_eq!(request.id, "task-id");
        assert!(serde_json::from_str::<TaskRequestDto>(
            r#"{"id":"task-id","workspaceId":"other"}"#
        )
        .is_err());

        let response = TaskActivityListResponseDto {
            events: vec![TaskActivityDto {
                id: "event-id".to_string(),
                entity_type: "task".to_string(),
                action: "status_changed".to_string(),
                summary: "Task status changed".to_string(),
                occurred_at: "2026-10-26T01:00:00Z".to_string(),
            }],
        };
        let json = serde_json::to_value(response).expect("serialize task activity");
        assert_eq!(json["events"][0]["entityType"], "task");
        assert_eq!(json["events"][0]["summary"], "Task status changed");
        assert_eq!(json["events"][0]["occurredAt"], "2026-10-26T01:00:00Z");
        assert!(json["events"][0].get("occurred_at").is_none());
    }

    #[test]
    fn project_overview_ipc_accepts_only_project_id_and_serializes_real_summary_fields() {
        let request: ProjectOverviewRequestDto =
            serde_json::from_str(r#"{"id":"project-id"}"#).expect("project ID request");
        assert_eq!(request.id, "project-id");
        assert!(serde_json::from_str::<ProjectOverviewRequestDto>(
            r#"{"id":"project-id","workspaceId":"other","path":"C:/data"}"#
        )
        .is_err());

        let response = ProjectOverviewResponseDto {
            project: ProjectDto {
                id: "project-id".to_string(),
                workspace_id: PERSONAL_WORKSPACE_ID.to_string(),
                name: "Moonships".to_string(),
                project_key: None,
                description: None,
                goal: Some("Ship the local workflow".to_string()),
                status: "active".to_string(),
                archived_at: None,
                created_at: "2026-10-26T00:00:00Z".to_string(),
                updated_at: "2026-10-26T00:00:00Z".to_string(),
            },
            active_task_count: 2,
            recent_activity: vec![ProjectActivityDto {
                id: "event-id".to_string(),
                entity_type: "task".to_string(),
                action: "created".to_string(),
                summary: "Task created".to_string(),
                occurred_at: "2026-10-26T00:00:00Z".to_string(),
            }],
        };
        let json = serde_json::to_value(response).expect("serialize overview");
        assert_eq!(json["project"]["goal"], "Ship the local workflow");
        assert_eq!(json["activeTaskCount"], 2);
        assert_eq!(json["recentActivity"][0]["entityType"], "task");
        assert_eq!(
            json["recentActivity"][0]["occurredAt"],
            "2026-10-26T00:00:00Z"
        );
        assert!(json.get("active_task_count").is_none());
    }

    #[test]
    fn database_uses_named_file_inside_the_supplied_app_data_directory() {
        let app_data_dir = Path::new("user-data").join("Moonships");
        assert_eq!(
            database_path(&app_data_dir),
            app_data_dir.join(DATABASE_FILE_NAME)
        );
        assert_eq!(DATABASE_FILE_NAME, "moonships.sqlite3");
    }

    #[test]
    fn opening_database_creates_directory_and_file_under_given_path() {
        let app_data_dir = unique_temp_dir("sqlite");
        let expected_database = database_path(&app_data_dir);

        let connection = open_database_in(&app_data_dir).expect("database should open");
        connection
            .execute("CREATE TABLE smoke_test (id INTEGER PRIMARY KEY)", [])
            .expect("connection should execute SQL");
        drop(connection);

        assert!(app_data_dir.is_dir());
        assert!(expected_database.is_file());
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn every_opened_connection_enables_foreign_keys_and_sets_busy_timeout() {
        let app_data_dir = unique_temp_dir("sqlite-pragmas");
        let connection = open_database_in(&app_data_dir).expect("database should open");

        let foreign_keys_enabled: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign-key pragma should be readable");
        let busy_timeout_ms: i64 = connection
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("busy timeout pragma should be readable");

        assert_eq!(foreign_keys_enabled, 1);
        assert_eq!(busy_timeout_ms, DATABASE_BUSY_TIMEOUT_MS as i64);
        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn foreign_key_violation_is_rejected_on_opened_connection() {
        let app_data_dir = unique_temp_dir("sqlite-foreign-key");
        let connection = open_database_in(&app_data_dir).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE parent_records (id INTEGER PRIMARY KEY);
                 CREATE TABLE child_records (
                    id INTEGER PRIMARY KEY,
                    parent_id INTEGER NOT NULL REFERENCES parent_records(id)
                 );",
            )
            .expect("foreign-key fixture should be created");

        let result = connection.execute(
            "INSERT INTO child_records (id, parent_id) VALUES (?1, ?2)",
            rusqlite::params![1, 999],
        );
        assert!(result.is_err(), "invalid parent reference must be rejected");
        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn fresh_database_migration_persists_schema_version_and_is_idempotent() {
        let app_data_dir = unique_temp_dir("migration");
        let mut connection = open_database_in(&app_data_dir).expect("database should open");
        run_migrations(&mut connection).expect("fresh migration should succeed");

        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version should be recorded");
        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        run_migrations(&mut connection).expect("migration should be safe to rerun");
        let version_after_rerun: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version should remain available");
        assert_eq!(version_after_rerun, CURRENT_SCHEMA_VERSION);

        let (id, name, workspace_count): (String, String, i64) = connection
            .query_row(
                "SELECT id, name, (SELECT COUNT(*) FROM workspaces) FROM workspaces",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("exactly one default workspace should exist");
        assert_eq!(id, PERSONAL_WORKSPACE_ID);
        assert_eq!(name, "Personal");
        assert_eq!(workspace_count, 1);
        assert!(
            connection
                .execute(
                    "INSERT INTO workspaces (id, name) VALUES ('empty-name', '   ')",
                    [],
                )
                .is_err(),
            "empty workspace names must fail validation"
        );
        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn upgrading_v8_adds_cleanup_reports_without_changing_existing_tasks() {
        let app_data_dir = unique_temp_dir("cleanup-reports-v8-upgrade");
        let mut connection = open_database_in(&app_data_dir).expect("open disposable database");
        run_migrations(&mut connection).expect("apply fresh migrations");
        connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, title) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    "00000000-0000-4000-8000-000000000199",
                    PERSONAL_WORKSPACE_ID,
                    "Preserved before v9"
                ],
            )
            .expect("insert task before upgrade fixture");
        connection
            .execute_batch(
                "DROP TABLE jobs;
                 DROP TABLE cleanup_reports;
                 DELETE FROM schema_migrations WHERE version >= 9;",
            )
            .expect("restore version 8 schema fixture");

        run_migrations(&mut connection).expect("apply cleanup report and job migrations");

        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("read migrated schema version");
        let task_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE id = ?1",
                ["00000000-0000-4000-8000-000000000199"],
                |row| row.get(0),
            )
            .expect("verify pre-upgrade task remains");
        let report_table_exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'cleanup_reports'",
                [],
                |row| row.get(0),
            )
            .expect("check cleanup report table");
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
        assert_eq!(task_count, 1);
        assert_eq!(report_table_exists, 1);
        let jobs_table_exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'jobs'",
                [],
                |row| row.get(0),
            )
            .expect("check jobs table");
        assert_eq!(jobs_table_exists, 1);

        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn upgrading_v1_preserves_user_metadata_and_seeds_only_one_personal_workspace() {
        let app_data_dir = unique_temp_dir("migration-v1-upgrade");
        let mut connection = open_database_in(&app_data_dir).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                );
                INSERT INTO schema_migrations (version) VALUES (1);
                CREATE TABLE app_metadata (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL
                );
                INSERT INTO app_metadata (key, value) VALUES ('existing', 'preserve-me');",
            )
            .expect("v1 fixture should be created");

        run_migrations(&mut connection).expect("v1 to v2 migration should succeed");
        run_migrations(&mut connection).expect("v2 migration should be idempotent");

        let existing: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'existing'",
                [],
                |row| row.get(0),
            )
            .expect("legacy metadata must remain");
        assert_eq!(existing, "preserve-me");

        let workspace_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
            .expect("workspace count should be readable");
        assert_eq!(workspace_count, 1);
        let current_version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("workspace schema version should exist");
        assert_eq!(current_version, CURRENT_SCHEMA_VERSION);

        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn project_migration_enforces_workspace_scope_name_and_canonical_status() {
        let app_data_dir = unique_temp_dir("projects");
        let mut connection = open_database_in(&app_data_dir).expect("database should open");
        run_migrations(&mut connection).expect("project migration should succeed");

        connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name) VALUES (?1, ?2, ?3)",
                rusqlite::params!["project-1", PERSONAL_WORKSPACE_ID, "Moonships"],
            )
            .expect("minimal project should be created with defaults");

        let (status, created_at, updated_at): (String, String, String) = connection
            .query_row(
                "SELECT status, created_at, updated_at FROM projects WHERE id = 'project-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("project defaults should persist");
        assert_eq!(status, "planned");
        assert!(!created_at.is_empty());
        assert!(!updated_at.is_empty());

        assert!(connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name) VALUES (?1, ?2, ?3)",
                rusqlite::params!["invalid-name", PERSONAL_WORKSPACE_ID, "   "],
            )
            .is_err());
        assert!(connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name) VALUES (?1, ?2, ?3)",
                rusqlite::params!["invalid-workspace", "missing-workspace", "Example"],
            )
            .is_err());
        assert!(connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name, status) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    "invalid-status",
                    PERSONAL_WORKSPACE_ID,
                    "Example",
                    "unknown"
                ],
            )
            .is_err());

        connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name, project_key)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["unique-project", PERSONAL_WORKSPACE_ID, "Unique", "MOON"],
            )
            .expect("unique key should save");
        assert!(connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name, project_key)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    "duplicate-project",
                    PERSONAL_WORKSPACE_ID,
                    "Duplicate",
                    "MOON"
                ],
            )
            .is_err());

        run_migrations(&mut connection).expect("project migration should not duplicate data");
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .expect("project count should be readable");
        assert_eq!(count, 2);
        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn upgrading_v2_preserves_existing_workspaces_and_app_settings() {
        let app_data_dir = unique_temp_dir("projects-upgrade");
        let mut connection = open_database_in(&app_data_dir).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                );
                INSERT INTO schema_migrations (version) VALUES (1);
                INSERT INTO schema_migrations (version) VALUES (2);
                CREATE TABLE app_metadata (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL
                );
                INSERT INTO app_metadata (key, value) VALUES ('existing', 'keep-existing');
                CREATE TABLE workspaces (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    profile_type TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    archived_at TEXT
                );
                INSERT INTO workspaces (id, name, created_at, updated_at)
                VALUES ('my-existing-workspace', 'Saved Workspace', '2026-01-01', '2026-01-01');",
            )
            .expect("v2 fixture should be created");

        run_migrations(&mut connection).expect("v2 to v3 migration should succeed");
        run_migrations(&mut connection).expect("repeated migration should be safe");
        let saved: (String, String) = connection
            .query_row("SELECT id, name FROM workspaces", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("existing workspace should remain");
        assert_eq!(saved.0, "my-existing-workspace");
        assert_eq!(saved.1, "Saved Workspace");
        let metadata: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'existing'",
                [],
                |row| row.get(0),
            )
            .expect("settings must remain");
        assert_eq!(metadata, "keep-existing");
        let project_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .expect("new projects table must exist");
        assert_eq!(project_count, 0);

        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn project_migration_failure_rolls_back_all_schema_changes() {
        let app_data_dir = unique_temp_dir("projects-rollback");
        let mut connection = open_database_in(&app_data_dir).expect("database should open");
        run_migrations(&mut connection).expect("baseline migration");

        // Reconstruct v2 in a disposable fixture, then deliberately reserve the
        // index name so schema v3 fails *after* CREATE TABLE projects succeeds.
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE task_labels;
                 DROP TABLE labels;
                 DROP TABLE IF EXISTS activity_events;
                 DROP TABLE tasks;
                 DROP TABLE projects;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 3;
                 INSERT INTO app_metadata (key, value)
                     VALUES ('rollback-marker', 'preserved');
                 CREATE TABLE unrelated_fixture (id TEXT PRIMARY KEY);
                 CREATE INDEX idx_projects_workspace_status
                     ON unrelated_fixture (id);",
            )
            .expect("v2 failure fixture");

        assert!(run_migrations(&mut connection).is_err());
        let projects_table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'projects'",
                [],
                |row| row.get(0),
            )
            .expect("schema inspection");
        assert_eq!(
            projects_table_count, 0,
            "partial CREATE TABLE must roll back"
        );

        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version");
        assert_eq!(version, 2, "failed migration must not record version 3");

        let preserved: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'rollback-marker'",
                [],
                |row| row.get(0),
            )
            .expect("existing metadata");
        assert_eq!(preserved, "preserved");
        let workspace_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
            .expect("existing workspaces");
        assert_eq!(workspace_count, 1);

        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn task_migration_supports_inbox_defaults_and_workspace_isolation() {
        let app_data_dir = unique_temp_dir("tasks");
        let mut connection = open_database_in(&app_data_dir).expect("database should open");
        run_migrations(&mut connection).expect("task migration");

        connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, title) VALUES (?1, ?2, ?3)",
                rusqlite::params!["inbox-1", PERSONAL_WORKSPACE_ID, "Write acceptance tests"],
            )
            .expect("title-only personal inbox task");
        let (project_id, status, priority, created_at, updated_at): (
            Option<String>,
            String,
            String,
            String,
            String,
        ) = connection
            .query_row(
                "SELECT project_id, status, priority, created_at, updated_at
                 FROM tasks WHERE id = 'inbox-1'",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .expect("inbox defaults");
        assert_eq!(project_id, None);
        assert_eq!(status, "backlog");
        assert_eq!(priority, "p2");
        assert!(!created_at.is_empty());
        assert!(!updated_at.is_empty());

        connection
            .execute(
                "INSERT INTO workspaces (id, name) VALUES ('other-workspace', 'Other')",
                [],
            )
            .expect("other workspace fixture");
        connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name)
                 VALUES ('project-a', ?1, 'Project A')",
                rusqlite::params![PERSONAL_WORKSPACE_ID],
            )
            .expect("same workspace project");
        connection
            .execute(
                "INSERT INTO projects (id, workspace_id, name)
                 VALUES ('project-b', 'other-workspace', 'Project B')",
                [],
            )
            .expect("other workspace project");
        connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, project_id, title)
                 VALUES ('project-task', ?1, 'project-a', 'Scoped task')",
                rusqlite::params![PERSONAL_WORKSPACE_ID],
            )
            .expect("valid project association");

        for (id, title, status, priority) in [
            ("blank-title", "   ", "backlog", "p2"),
            ("invalid-status", "Valid", "unknown", "p2"),
            ("invalid-priority", "Valid", "backlog", "critical"),
        ] {
            assert!(
                connection
                    .execute(
                        "INSERT INTO tasks (id, workspace_id, title, status, priority)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![id, PERSONAL_WORKSPACE_ID, title, status, priority],
                    )
                    .is_err(),
                "invalid task {id} must be rejected"
            );
        }
        for (id, workspace_id, project_id) in [
            ("bad-workspace", "missing-workspace", None),
            (
                "bad-project",
                PERSONAL_WORKSPACE_ID,
                Some("missing-project"),
            ),
            ("cross-workspace", PERSONAL_WORKSPACE_ID, Some("project-b")),
        ] {
            assert!(
                connection
                    .execute(
                        "INSERT INTO tasks (id, workspace_id, project_id, title)
                         VALUES (?1, ?2, ?3, 'Invalid relationship')",
                        rusqlite::params![id, workspace_id, project_id],
                    )
                    .is_err(),
                "invalid relationship {id} must be rejected"
            );
        }
        assert!(connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, title, estimate_minutes)
                 VALUES ('bad-estimate', ?1, 'Invalid', -1)",
                rusqlite::params![PERSONAL_WORKSPACE_ID],
            )
            .is_err());
        assert!(connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, parent_task_id, title)
                 VALUES ('self-parent', ?1, 'self-parent', 'Invalid')",
                rusqlite::params![PERSONAL_WORKSPACE_ID],
            )
            .is_err());

        connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, title)
                 VALUES ('foreign-parent', 'other-workspace', 'Parent elsewhere')",
                [],
            )
            .expect("other workspace parent");
        assert!(connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, parent_task_id, title)
                 VALUES ('cross-parent', ?1, 'foreign-parent', 'Not permitted')",
                rusqlite::params![PERSONAL_WORKSPACE_ID],
            )
            .is_err());
        connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, parent_task_id, title)
                 VALUES ('valid-child', ?1, 'inbox-1', 'Child in same workspace')",
                rusqlite::params![PERSONAL_WORKSPACE_ID],
            )
            .expect("valid parent reference");

        run_migrations(&mut connection).expect("idempotent task migration");
        drop(connection);
        let mut reopened = open_database_in(&app_data_dir).expect("reopen persisted database");
        run_migrations(&mut reopened).expect("reopen migration");
        let task_count: i64 = reopened
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .expect("persisted task count");
        assert_eq!(task_count, 4);
        drop(reopened);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn upgrading_v3_preserves_projects_workspaces_and_metadata() {
        let app_data_dir = unique_temp_dir("tasks-v3-upgrade");
        let mut connection = open_database_in(&app_data_dir).expect("database");
        run_migrations(&mut connection).expect("create schema fixture");
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE task_labels;
                 DROP TABLE labels;
                 DROP TABLE IF EXISTS activity_events;
                 DROP TABLE tasks;
                 DROP INDEX idx_projects_workspace_id;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 4;
                 INSERT INTO app_metadata (key, value) VALUES ('saved', 'keep-me');
                 INSERT INTO projects (id, workspace_id, name)
                 VALUES ('existing-project', '00000000-0000-4000-8000-000000000001', 'Keep Project');",
            )
            .expect("v3 fixture with existing records");
        run_migrations(&mut connection).expect("v3 to v4 upgrade");
        run_migrations(&mut connection).expect("idempotent upgrade");

        let existing: (String, String) = connection
            .query_row(
                "SELECT p.name, m.value
                 FROM projects p CROSS JOIN app_metadata m
                 WHERE p.id = 'existing-project' AND m.key = 'saved'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("preexisting data survives");
        assert_eq!(existing.0, "Keep Project");
        assert_eq!(existing.1, "keep-me");
        let workspaces: i64 = connection
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
            .unwrap();
        assert_eq!(workspaces, 1);
        let schema: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(schema, CURRENT_SCHEMA_VERSION);

        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn failed_task_migration_rolls_back_partial_schema_without_losing_data() {
        let app_data_dir = unique_temp_dir("tasks-rollback");
        let mut connection = open_database_in(&app_data_dir).expect("database");
        run_migrations(&mut connection).expect("create fixture");
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE task_labels;
                 DROP TABLE labels;
                 DROP TABLE IF EXISTS activity_events;
                 DROP TABLE tasks;
                 DROP INDEX idx_projects_workspace_id;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 4;
                 INSERT INTO app_metadata (key, value) VALUES ('saved', 'preserve-me');
                 CREATE TABLE unrelated_fixture (id TEXT);
                 CREATE INDEX idx_tasks_workspace_status ON unrelated_fixture (id);",
            )
            .expect("v3 collision fixture");

        assert!(run_migrations(&mut connection).is_err());
        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'tasks'",
                [],
                |row| row.get(0),
            )
            .expect("schema table check");
        assert_eq!(table_count, 0);
        let index_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'index' AND name = 'idx_projects_workspace_id'",
                [],
                |row| row.get(0),
            )
            .expect("schema index check");
        assert_eq!(index_count, 0);
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version");
        assert_eq!(version, 3);
        let saved: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'saved'",
                [],
                |row| row.get(0),
            )
            .expect("original metadata survives");
        assert_eq!(saved, "preserve-me");

        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn upgrading_v4_to_activity_schema_preserves_existing_projects_and_tasks() {
        let dir = unique_temp_dir("activity-upgrade");
        let mut connection = open_database_in(&dir).expect("disposable connection");
        run_migrations(&mut connection).expect("fresh baseline");
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE task_labels;
                 DROP TABLE labels;
                 DROP TABLE IF EXISTS activity_events;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 5;
                 INSERT INTO projects (id, workspace_id, name)
                 VALUES ('saved-project', '00000000-0000-4000-8000-000000000001', 'Preserve');
                 INSERT INTO tasks (id, workspace_id, project_id, title)
                 VALUES ('saved-task', '00000000-0000-4000-8000-000000000001',
                         'saved-project', 'Preserve task');
                 INSERT INTO app_metadata (key, value) VALUES ('saved-key', 'keep-me');",
            )
            .expect("v4 user-data fixture");

        run_migrations(&mut connection).expect("v4 to v5");
        run_migrations(&mut connection).expect("idempotent v5");
        let saved: (String, String, String) = connection
            .query_row(
                "SELECT p.name, t.title, m.value
                 FROM projects p JOIN tasks t ON t.project_id = p.id
                 JOIN app_metadata m ON m.key = 'saved-key'
                 WHERE p.id = 'saved-project'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("existing rows preserved");
        assert_eq!(
            saved,
            (
                "Preserve".to_string(),
                "Preserve task".to_string(),
                "keep-me".to_string()
            )
        );
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM activity_events", [], |row| row.get(0))
            .expect("activity available");
        assert_eq!(count, 0, "upgrade must not manufacture historical events");
        drop(connection);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_activity_migration_is_atomic_and_preserves_existing_data() {
        let dir = unique_temp_dir("activity-rollback");
        let mut connection = open_database_in(&dir).expect("disposable connection");
        run_migrations(&mut connection).expect("baseline");
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE task_labels;
                 DROP TABLE labels;
                 DROP TABLE IF EXISTS activity_events;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 5;
                 INSERT INTO app_metadata (key, value) VALUES ('keep', 'original');
                 CREATE TABLE unrelated_fixture (id TEXT);
                 CREATE INDEX idx_activity_events_entity ON unrelated_fixture(id);",
            )
            .expect("reserved index collision fixture");

        assert!(run_migrations(&mut connection).is_err());
        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'activity_events'",
                [],
                |row| row.get(0),
            )
            .expect("schema inspection");
        assert_eq!(table_count, 0, "failed CREATE TABLE must roll back");
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("version");
        assert_eq!(version, 4);
        let saved: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'keep'",
                [],
                |row| row.get(0),
            )
            .expect("saved metadata");
        assert_eq!(saved, "original");
        drop(connection);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn upgrading_v5_adds_labels_and_dependencies_without_changing_existing_data() {
        let dir = unique_temp_dir("labels-upgrade");
        let mut connection = open_database_in(&dir).expect("disposable connection");
        run_migrations(&mut connection).expect("create current baseline");
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE task_labels;
                 DROP TABLE labels;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 6;
                 INSERT INTO projects (id, workspace_id, name)
                 VALUES ('label-project', '00000000-0000-4000-8000-000000000001', 'Preserve');
                 INSERT INTO tasks (id, workspace_id, project_id, title)
                 VALUES ('label-task', '00000000-0000-4000-8000-000000000001',
                         'label-project', 'Preserve task');
                 INSERT INTO app_metadata (key, value) VALUES ('label-upgrade', 'keep-me');",
            )
            .expect("v5 fixture with existing records");

        run_migrations(&mut connection).expect("v5 to v6");
        run_migrations(&mut connection).expect("idempotent v6");
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version");
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
        let preserved: (String, String, String) = connection
            .query_row(
                "SELECT p.name, t.title, m.value
                 FROM projects p JOIN tasks t ON t.project_id = p.id
                 JOIN app_metadata m ON m.key = 'label-upgrade'
                 WHERE p.id = 'label-project'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("existing records survive label migration");
        assert_eq!(
            preserved,
            (
                "Preserve".to_string(),
                "Preserve task".to_string(),
                "keep-me".to_string()
            )
        );
        drop(connection);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_label_migration_rolls_back_schema_and_version() {
        let dir = unique_temp_dir("labels-rollback");
        let mut connection = open_database_in(&dir).expect("disposable connection");
        run_migrations(&mut connection).expect("create current baseline");
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE task_labels;
                 DROP TABLE labels;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 6;
                 INSERT INTO app_metadata (key, value) VALUES ('keep-label-data', 'original');
                 CREATE TABLE unrelated_label_fixture (id TEXT);
                 CREATE INDEX idx_labels_workspace_name
                    ON unrelated_label_fixture (id);",
            )
            .expect("reserved index collision fixture");

        assert!(run_migrations(&mut connection).is_err());
        let label_tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name IN ('labels', 'task_labels')",
                [],
                |row| row.get(0),
            )
            .expect("schema inspection");
        assert_eq!(label_tables, 0, "failed migration tables must roll back");
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version");
        assert_eq!(version, 5, "failed migration must not record version 6");
        let saved: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'keep-label-data'",
                [],
                |row| row.get(0),
            )
            .expect("existing metadata preserved");
        assert_eq!(saved, "original");
        drop(connection);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn upgrading_v6_adds_task_dependencies_without_changing_existing_labels() {
        let dir = unique_temp_dir("dependencies-upgrade");
        let mut connection = open_database_in(&dir).expect("disposable connection");
        run_migrations(&mut connection).expect("create current baseline");
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 7;
                 INSERT INTO tasks (id, workspace_id, title)
                 VALUES ('blocked-task', '00000000-0000-4000-8000-000000000001', 'Blocked');
                 INSERT INTO tasks (id, workspace_id, title)
                 VALUES ('prerequisite-task', '00000000-0000-4000-8000-000000000001', 'Prerequisite');
                 INSERT INTO labels (id, workspace_id, name, normalized_name)
                 VALUES ('saved-label', '00000000-0000-4000-8000-000000000001', 'Keep', 'keep');
                 INSERT INTO task_labels (workspace_id, task_id, label_id)
                 VALUES ('00000000-0000-4000-8000-000000000001', 'blocked-task', 'saved-label');",
            )
            .expect("v6 fixture with existing labeled task");

        run_migrations(&mut connection).expect("v6 to v7");
        run_migrations(&mut connection).expect("idempotent v7");
        let dependency_exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'task_dependencies'",
                [],
                |row| row.get(0),
            )
            .expect("dependency schema exists");
        assert_eq!(dependency_exists, 1);
        let label_link_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM task_labels", [], |row| row.get(0))
            .expect("existing Task-label links survive");
        assert_eq!(label_link_count, 1);
        drop(connection);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_task_dependency_migration_rolls_back_schema_and_version() {
        let dir = unique_temp_dir("dependencies-rollback");
        let mut connection = open_database_in(&dir).expect("disposable connection");
        run_migrations(&mut connection).expect("create current baseline");
        connection
            .execute_batch(
                "DROP TABLE task_dependencies;
                 DROP TABLE IF EXISTS cleanup_reports; DROP TABLE IF EXISTS milestone_tasks; DROP TABLE IF EXISTS milestones; DROP INDEX IF EXISTS idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version >= 7;
                 INSERT INTO app_metadata (key, value) VALUES ('keep-dependency-data', 'original');
                 CREATE TABLE unrelated_dependency_fixture (id TEXT);
                 CREATE INDEX idx_task_dependencies_workspace_prerequisite
                    ON unrelated_dependency_fixture (id);",
            )
            .expect("reserved index collision fixture");

        assert!(run_migrations(&mut connection).is_err());
        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'task_dependencies'",
                [],
                |row| row.get(0),
            )
            .expect("schema inspection");
        assert_eq!(table_count, 0, "failed migration table must roll back");
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version");
        assert_eq!(version, 6, "failed migration must not record version 7");
        let saved: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'keep-dependency-data'",
                [],
                |row| row.get(0),
            )
            .expect("existing metadata preserved");
        assert_eq!(saved, "original");
        drop(connection);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn milestone_ipc_dtos_are_camel_case_and_reject_renderer_scope() {
        let create: CreateMilestoneRequestDto = serde_json::from_str(
            r#"{"projectId":"project-id","title":"Release","description":null,"dueDate":"2026-12-31"}"#,
        ).expect("typed milestone create");
        assert_eq!(create.due_date.as_deref(), Some("2026-12-31"));
        assert!(serde_json::from_str::<CreateMilestoneRequestDto>(
            r#"{"projectId":"project-id","title":"Release","description":null,"dueDate":null,"workspaceId":"other"}"#,
        ).is_err());
        assert!(serde_json::from_str::<ListMilestonesRequestDto>(
            r#"{"projectId":"project-id","path":"C:/data"}"#,
        )
        .is_err());
        assert!(serde_json::from_str::<AssociateMilestoneTaskRequestDto>(
            r#"{"projectId":"p1","milestoneId":"m1","taskId":"t1","workspaceId":"other"}"#,
        )
        .is_err());
        let response = MilestoneListResponseDto {
            milestones: vec![MilestoneDto {
                id: "m1".into(),
                workspace_id: PERSONAL_WORKSPACE_ID.into(),
                project_id: "p1".into(),
                title: "Release".into(),
                description: None,
                due_date: create.due_date,
                status: "planned".into(),
                sort_order: 0,
                created_at: "2026-10-26T00:00:00Z".into(),
                updated_at: "2026-10-26T00:00:00Z".into(),
                tasks: vec![],
            }],
        };
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["milestones"][0]["projectId"], "p1");
        assert_eq!(json["milestones"][0]["dueDate"], "2026-12-31");
        assert!(json["milestones"][0].get("workspace_id").is_none());
    }

    #[test]
    fn migration_v8_preserves_existing_tasks_and_rejects_cross_project_links() {
        let dir = unique_temp_dir("milestones-upgrade");
        let mut connection = open_database_in(&dir).unwrap();
        run_migrations(&mut connection).unwrap();
        connection.execute_batch(
            "DROP TABLE cleanup_reports; DROP TABLE milestone_tasks; DROP TABLE milestones;
             DROP INDEX idx_tasks_workspace_project_identity;
             DELETE FROM schema_migrations WHERE version >= 8;
             INSERT INTO projects (id, workspace_id, name) VALUES ('old-project', '00000000-0000-4000-8000-000000000001', 'Old project');
             INSERT INTO tasks (id, workspace_id, project_id, title) VALUES ('old-task', '00000000-0000-4000-8000-000000000001', 'old-project', 'Keep task');",
        ).unwrap();
        run_migrations(&mut connection).expect("upgrade v7 to v8");
        run_migrations(&mut connection).expect("idempotent v8");
        let title: String = connection
            .query_row("SELECT title FROM tasks WHERE id='old-task'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(title, "Keep task");
        let milestone = crate::milestones::create_milestone(
            &mut connection,
            crate::milestones::CreateMilestone {
                workspace_id: PERSONAL_WORKSPACE_ID.into(),
                project_id: "old-project".into(),
                title: "Release".into(),
                description: None,
                due_date: None,
            },
        )
        .unwrap();
        let error = connection.execute("INSERT INTO milestone_tasks (workspace_id, project_id, milestone_id, task_id) VALUES (?1, 'wrong-project', ?2, 'old-task')", rusqlite::params![PERSONAL_WORKSPACE_ID, milestone.id]).unwrap_err();
        assert!(
            matches!(error, rusqlite::Error::SqliteFailure(ref code, _) if code.code == rusqlite::ErrorCode::ConstraintViolation)
        );
        drop(connection);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_milestone_migration_rolls_back_schema_and_version() {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        connection.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&mut connection).unwrap();
        connection.execute_batch(
            "DROP TABLE cleanup_reports; DROP TABLE milestone_tasks; DROP TABLE milestones;
             DROP INDEX idx_tasks_workspace_project_identity; DELETE FROM schema_migrations WHERE version>=8;
             CREATE TABLE unrelated_milestone_fixture (id TEXT);
             CREATE INDEX idx_milestones_project_order ON unrelated_milestone_fixture(id);",
        ).unwrap();
        assert!(run_migrations(&mut connection).is_err());
        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='milestones'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(table_count, 0);
        assert_eq!(version, 7);
    }

    #[test]
    fn migration_rejects_a_schema_version_newer_than_supported() {
        let app_data_dir = unique_temp_dir("future-schema");
        let mut connection = open_database_in(&app_data_dir).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                );
                INSERT INTO schema_migrations (version) VALUES (999);",
            )
            .expect("future schema fixture should be created");

        let error = run_migrations(&mut connection).expect_err("future schema must be rejected");
        assert!(error.contains("newer than this app supports"));
        drop(connection);
        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn hide_to_tray_preference_is_fail_closed_when_missing_or_invalid() {
        let missing = std::env::temp_dir().join(format!(
            "moonships-hide-to-tray-missing-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(!hide_to_tray_enabled(&missing));
    }

    #[test]
    fn hide_to_tray_preference_accepts_only_true() {
        let path = std::env::temp_dir().join(format!(
            "moonships-hide-to-tray-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, "true").unwrap();
        assert!(hide_to_tray_enabled(&path));
        fs::write(&path, "false").unwrap();
        assert!(!hide_to_tray_enabled(&path));
        fs::write(&path, "unexpected").unwrap();
        assert!(!hide_to_tray_enabled(&path));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn desktop_crate_is_configured_for_moonships() {
        assert_eq!(env!("CARGO_PKG_NAME"), "moonships");
    }
}
