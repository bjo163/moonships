use std::{
    fs,
    path::{Path, PathBuf},
};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

const DATABASE_FILE_NAME: &str = "moonships.sqlite3";

fn database_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(DATABASE_FILE_NAME)
}

fn open_database_in(app_data_dir: &Path) -> Result<Connection, String> {
    fs::create_dir_all(app_data_dir)
        .map_err(|error| format!("Could not create app data directory: {error}"))?;
    let path = database_path(app_data_dir);
    Connection::open(path).map_err(|error| format!("Could not open Moonships database: {error}"))
}

const CURRENT_SCHEMA_VERSION: i64 = 1;

fn run_migrations(connection: &mut Connection) -> Result<(), String> {
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
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |row| row.get(0))
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

/// Stable IPC payload for reading and updating the hide-to-tray preference.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HideToTrayPreferenceDto {
    enabled: bool,
}

#[tauri::command]
fn get_hide_to_tray(app: AppHandle) -> Result<HideToTrayPreferenceDto, String> {
    let path = hide_to_tray_path(&app)?;
    Ok(HideToTrayPreferenceDto {
        enabled: hide_to_tray_enabled(&path),
    })
}

#[tauri::command]
fn set_hide_to_tray(
    app: AppHandle,
    request: HideToTrayPreferenceDto,
) -> Result<HideToTrayPreferenceDto, String> {
    let path = hide_to_tray_path(&app)?;
    let directory = path
        .parent()
        .ok_or_else(|| "App config directory has no parent".to_string())?;
    fs::create_dir_all(directory)
        .map_err(|error| format!("Could not create app config directory: {error}"))?;
    fs::write(path, if request.enabled { "true" } else { "false" })
        .map_err(|error| format!("Could not save hide-to-tray preference: {error}"))?;
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
        .invoke_handler(tauri::generate_handler![get_hide_to_tray, set_hide_to_tray])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            // Open/create the database only under Tauri's per-user application data directory.
            let mut database = open_database_in(&app_data_dir)
                .map_err(std::io::Error::other)?;
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
                let Ok(path) = hide_to_tray_path(window.app_handle()) else {
                    return;
                };
                if hide_to_tray_enabled(&path) {
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
    use super::{database_path, hide_to_tray_enabled, open_database_in, DATABASE_FILE_NAME};
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
