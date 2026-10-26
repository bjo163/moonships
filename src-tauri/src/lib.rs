pub mod config;
pub mod domain;
pub mod local_logging;
pub mod projects;

use std::{
    fs,
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

const CURRENT_SCHEMA_VERSION: i64 = 4;
const PERSONAL_WORKSPACE_ID: &str = "00000000-0000-4000-8000-000000000001";

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

    transaction
        .commit()
        .map_err(|error| format!("Could not commit schema migration: {error}"))
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
            set_app_config
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
        CURRENT_SCHEMA_VERSION, DATABASE_BUSY_TIMEOUT_MS, DATABASE_FILE_NAME,
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
                "DROP TABLE projects;
                 DELETE FROM schema_migrations WHERE version = 3;
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
                "DROP TABLE tasks;
                 DROP INDEX idx_projects_workspace_id;
                 DELETE FROM schema_migrations WHERE version = 4;
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
                "DROP TABLE tasks;
                 DROP INDEX idx_projects_workspace_id;
                 DELETE FROM schema_migrations WHERE version = 4;
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
