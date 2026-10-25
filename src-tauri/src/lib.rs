use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager};

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

#[tauri::command]
fn get_hide_to_tray(app: AppHandle) -> Result<bool, String> {
    let path = hide_to_tray_path(&app)?;
    Ok(hide_to_tray_enabled(&path))
}

#[tauri::command]
fn set_hide_to_tray(app: AppHandle, enabled: bool) -> Result<(), String> {
    let path = hide_to_tray_path(&app)?;
    let directory = path
        .parent()
        .ok_or_else(|| "App config directory has no parent".to_string())?;
    fs::create_dir_all(directory)
        .map_err(|error| format!("Could not create app config directory: {error}"))?;
    fs::write(path, if enabled { "true" } else { "false" })
        .map_err(|error| format!("Could not save hide-to-tray preference: {error}"))
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
    use super::hide_to_tray_enabled;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

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
