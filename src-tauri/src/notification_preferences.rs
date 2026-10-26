//! Fail-closed, app-local settings for desktop notifications.
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

const PREFERENCES_KEY: &str = "app.notifications.v1";
const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationPreferences {
    pub schema_version: u32,
    pub enabled: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetNotificationPreferencesRequest {
    pub enabled: bool,
}

pub fn load(connection: &Connection) -> Result<NotificationPreferences, String> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            [PREFERENCES_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "NOTIFICATION_PREFERENCES_UNAVAILABLE".to_owned())?;
    let Some(value) = value else {
        return Ok(NotificationPreferences::default());
    };
    let preferences: NotificationPreferences =
        serde_json::from_str(&value).map_err(|_| "NOTIFICATION_PREFERENCES_INVALID".to_owned())?;
    if preferences.schema_version != CURRENT_SCHEMA_VERSION {
        return Err("NOTIFICATION_PREFERENCES_INVALID".to_owned());
    }
    Ok(preferences)
}

pub fn save(
    connection: &Connection,
    request: SetNotificationPreferencesRequest,
) -> Result<NotificationPreferences, String> {
    let preferences = NotificationPreferences {
        schema_version: CURRENT_SCHEMA_VERSION,
        enabled: request.enabled,
    };
    let value = serde_json::to_string(&preferences)
        .map_err(|_| "NOTIFICATION_PREFERENCES_INVALID".to_owned())?;
    connection
        .execute(
            "INSERT INTO app_metadata (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![PREFERENCES_KEY, value],
        )
        .map_err(|_| "NOTIFICATION_PREFERENCES_UNAVAILABLE".to_owned())?;
    Ok(preferences)
}

pub fn should_notify_after_manual_fetch(enabled: bool, returned_item_count: usize) -> bool {
    enabled && returned_item_count > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn database() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute(
                "CREATE TABLE app_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)",
                [],
            )
            .unwrap();
        connection
    }

    #[test]
    fn notifications_default_off_and_persist_only_after_explicit_enable() {
        let connection = database();
        assert!(!load(&connection).unwrap().enabled);
        let enabled = save(
            &connection,
            SetNotificationPreferencesRequest { enabled: true },
        )
        .unwrap();
        assert!(enabled.enabled);
        assert!(load(&connection).unwrap().enabled);
        assert!(!should_notify_after_manual_fetch(false, 4));
        assert!(!should_notify_after_manual_fetch(true, 0));
        assert!(should_notify_after_manual_fetch(true, 4));
        save(
            &connection,
            SetNotificationPreferencesRequest { enabled: false },
        )
        .unwrap();
        assert!(!load(&connection).unwrap().enabled);
    }

    #[test]
    fn malformed_unknown_and_future_preferences_fail_closed() {
        let connection = database();
        for value in [
            json!({"schemaVersion":1,"enabled":"yes"}),
            json!({"schemaVersion":1,"enabled":true,"backgroundFetch":true}),
            json!({"schemaVersion":2,"enabled":true}),
            json!("invalid"),
        ] {
            connection
                .execute(
                    "INSERT INTO app_metadata (key,value) VALUES (?1,?2)
                     ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    params![PREFERENCES_KEY, value.to_string()],
                )
                .unwrap();
            assert!(load(&connection).is_err());
        }
    }

    #[test]
    fn preference_request_rejects_extra_fields() {
        assert!(serde_json::from_str::<SetNotificationPreferencesRequest>(
            r#"{"enabled":true,"sourceId":"untrusted"}"#
        )
        .is_err());
    }
}
