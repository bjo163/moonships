//! Strict, versioned, non-secret app configuration stored in the existing SQLite metadata table.
//! Never silently replace an invalid or future-version record with defaults.

use crate::domain::{AppConfig, ConfigLogLevel, ThemeMode, WorkspaceProfile};
use rusqlite::{params, Connection, OptionalExtension};

const APP_CONFIG_KEY: &str = "app.config.v1";

pub fn load_config(connection: &Connection) -> Result<Option<AppConfig>, String> {
    let raw: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            params![APP_CONFIG_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not read local app configuration: {error}"))?;

    raw.map(|content| {
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|error| format!("Invalid persisted app configuration: {error}"))?;
        let version = value
            .get("schemaVersion")
            .and_then(serde_json::Value::as_u64);
        let config = match version {
            Some(1) => {
                let old: LegacyAppConfig = serde_json::from_value(value)
                    .map_err(|error| format!("Invalid persisted app configuration: {error}"))?;
                if old.schema_version != 1 {
                    return Err(
                        "Invalid persisted app configuration: unsupported legacy version"
                            .to_owned(),
                    );
                }
                let migrated = AppConfig {
                    schema_version: crate::domain::CURRENT_CONFIG_SCHEMA_VERSION,
                    theme: old.theme,
                    profile: old.profile,
                    log_level: old.log_level,
                    hide_to_tray: old.hide_to_tray,
                    ai_enabled: false,
                };
                save_config(connection, &migrated)?;
                migrated
            }
            Some(_) => serde_json::from_value::<AppConfig>(value)
                .map_err(|error| format!("Invalid persisted app configuration: {error}"))?,
            None => {
                return Err("Invalid persisted app configuration: missing schemaVersion".to_owned())
            }
        };
        config.validate()?;
        Ok(config)
    })
    .transpose()
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyAppConfig {
    schema_version: u32,
    theme: ThemeMode,
    profile: WorkspaceProfile,
    log_level: ConfigLogLevel,
    hide_to_tray: bool,
}

pub fn config_or_default(
    connection: &Connection,
    legacy_hide_to_tray: bool,
) -> Result<AppConfig, String> {
    Ok(load_config(connection)?.unwrap_or_else(|| AppConfig {
        hide_to_tray: legacy_hide_to_tray,
        ..AppConfig::default()
    }))
}

pub fn save_config(connection: &Connection, config: &AppConfig) -> Result<(), String> {
    config.validate()?;
    let serialized = serde_json::to_string(config)
        .map_err(|error| format!("Could not encode app configuration: {error}"))?;
    connection
        .execute(
            "INSERT INTO app_metadata (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![APP_CONFIG_KEY, serialized],
        )
        .map_err(|error| format!("Could not save local app configuration: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ConfigLogLevel, ThemeMode, WorkspaceProfile, CURRENT_CONFIG_SCHEMA_VERSION,
    };
    use serde_json::json;

    fn fixture() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory database");
        connection
            .execute(
                "CREATE TABLE app_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)",
                [],
            )
            .expect("metadata table");
        connection
    }

    #[test]
    fn absent_record_returns_versioned_defaults_without_creating_a_record() {
        let connection = fixture();
        let config = config_or_default(&connection, false).expect("defaults");
        assert_eq!(config.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
        assert_eq!(config.theme, ThemeMode::Dark);
        assert_eq!(config.profile, WorkspaceProfile::Developer);
        assert_eq!(config.log_level, ConfigLogLevel::Info);
        assert!(!config.hide_to_tray);
        assert!(!config.ai_enabled);
        assert!(load_config(&connection).expect("query").is_none());
    }

    #[test]
    fn legacy_hide_to_tray_is_only_used_when_no_versioned_record_exists() {
        let connection = fixture();
        assert!(config_or_default(&connection, true).unwrap().hide_to_tray);

        save_config(&connection, &AppConfig::default()).expect("save");
        assert!(!config_or_default(&connection, true).unwrap().hide_to_tray);
    }

    #[test]
    fn valid_config_round_trips_through_sqlite_with_explicit_schema_version() {
        let connection = fixture();
        let config = AppConfig {
            theme: ThemeMode::Light,
            profile: WorkspaceProfile::NetworkNoc,
            log_level: ConfigLogLevel::Warn,
            hide_to_tray: true,
            ai_enabled: true,
            ..AppConfig::default()
        };
        save_config(&connection, &config).expect("save");
        assert_eq!(load_config(&connection).unwrap(), Some(config.clone()));
        let persisted: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = ?1",
                params![APP_CONFIG_KEY],
                |row| row.get(0),
            )
            .expect("persisted JSON");
        let json: serde_json::Value = serde_json::from_str(&persisted).expect("valid JSON");
        assert_eq!(json["schemaVersion"], 2);
        assert_eq!(json["profile"], "network_noc");
        assert_eq!(json["hideToTray"], true);
        assert_eq!(json["aiEnabled"], true);
    }

    #[test]
    fn invalid_updates_fail_before_write_and_preserve_previous_settings() {
        let connection = fixture();
        let original = AppConfig::default();
        save_config(&connection, &original).expect("seed");
        let invalid = AppConfig {
            schema_version: 999,
            ..original.clone()
        };
        assert!(save_config(&connection, &invalid).is_err());
        assert_eq!(load_config(&connection).unwrap(), Some(original));
    }

    #[test]
    fn persisted_malformed_unknown_or_future_settings_fail_closed() {
        let connection = fixture();
        let valid = serde_json::to_value(AppConfig::default()).unwrap();
        for value in [
            json!({"schemaVersion": 2, "theme": "neon", "profile": "developer", "logLevel": "info", "hideToTray": false, "aiEnabled": false}),
            json!({"schemaVersion": 2, "theme": "dark", "profile": "developer", "logLevel": "info", "hideToTray": "false", "aiEnabled": false}),
            json!({"schemaVersion": 2, "theme": "dark", "profile": "developer", "logLevel": "info", "hideToTray": false, "aiEnabled": false, "apiToken": "secret"}),
            json!({"schemaVersion": 3, "theme": "dark", "profile": "developer", "logLevel": "info", "hideToTray": false, "aiEnabled": false}),
            json!({"schemaVersion": 2, "theme": "dark", "logLevel": "info", "hideToTray": false, "aiEnabled": false}),
            json!({"schemaVersion": 2, "theme": "dark", "profile": "developer", "logLevel": "info", "hideToTray": null, "aiEnabled": false}),
            json!("not an object"),
        ] {
            connection
                .execute(
                    "INSERT INTO app_metadata (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![APP_CONFIG_KEY, value.to_string()],
                )
                .unwrap();
            assert!(
                config_or_default(&connection, true).is_err(),
                "corrupt config must not silently fall back: {value}"
            );
        }

        connection
            .execute(
                "UPDATE app_metadata SET value = ?1 WHERE key = ?2",
                params![valid.to_string(), APP_CONFIG_KEY],
            )
            .unwrap();
        assert!(load_config(&connection).unwrap().is_some());
    }

    #[test]
    fn version_one_preferences_migrate_and_keep_ai_disabled() {
        let connection = fixture();
        connection.execute(
            "INSERT INTO app_metadata (key, value) VALUES (?1, ?2)",
            params![APP_CONFIG_KEY, r#"{"schemaVersion":1,"theme":"light","profile":"network_noc","logLevel":"warn","hideToTray":true}"#],
        ).unwrap();
        let config = load_config(&connection).unwrap().unwrap();
        assert_eq!(config.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
        assert_eq!(config.theme, ThemeMode::Light);
        assert_eq!(config.profile, WorkspaceProfile::NetworkNoc);
        assert!(config.hide_to_tray);
        assert!(!config.ai_enabled);
        assert_eq!(load_config(&connection).unwrap(), Some(config));
    }
}
