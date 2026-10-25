use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const REDACTED: &str = "[REDACTED]";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogCategory {
    AppLifecycle,
    Database,
    Security,
    ToolExecution,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub timestamp_unix_ms: u128,
    pub level: LogLevel,
    pub category: LogCategory,
    pub event: String,
    pub fields: BTreeMap<String, Value>,
}

impl LogEvent {
    pub fn new(
        level: LogLevel,
        category: LogCategory,
        event: impl Into<String>,
        fields: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            timestamp_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            level,
            category,
            event: event.into(),
            fields,
        }
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized: String = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();

    [
        "password",
        "passwd",
        "secret",
        "token",
        "apikey",
        "authorization",
        "cookie",
        "credential",
        "privatekey",
        "accesskey",
        "clientsecret",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut redacted = Map::new();
            for (key, nested) in object {
                redacted.insert(
                    key.clone(),
                    if is_sensitive_key(key) {
                        Value::String(REDACTED.to_string())
                    } else {
                        redact_value(nested)
                    },
                );
            }
            Value::Object(redacted)
        }
        Value::Array(values) => Value::Array(values.iter().map(redact_value).collect()),
        other => other.clone(),
    }
}

pub fn render_event(event: &LogEvent) -> Result<String, String> {
    let mut safe_event = event.clone();
    safe_event.fields = event
        .fields
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                if is_sensitive_key(key) {
                    Value::String(REDACTED.to_string())
                } else {
                    redact_value(value)
                },
            )
        })
        .collect();

    serde_json::to_string(&safe_event)
        .map_err(|error| format!("Could not serialize structured log event: {error}"))
}

pub fn write_event(path: &Path, event: &LogEvent) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Log file path must have a parent directory".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create local log directory: {error}"))?;
    let line = render_event(event)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("Could not open local log file: {error}"))?;
    writeln!(file, "{line}").map_err(|error| format!("Could not append local log event: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn structured_logs_redact_secret_fields_recursively() {
        let mut fields = BTreeMap::new();
        fields.insert("apiToken".to_string(), json!("top-secret-token-value"));
        fields.insert("password".to_string(), json!("top-secret-password-value"));
        fields.insert(
            "request".to_string(),
            json!({
                "safeLabel": "visible",
                "authorizationHeader": "Bearer private-value",
                "nested": { "client_secret": "private-client-secret" }
            }),
        );
        let event = LogEvent::new(
            LogLevel::Info,
            LogCategory::Security,
            "credential_check",
            fields,
        );

        let line = render_event(&event).expect("event should serialize");
        assert!(!line.contains("top-secret-token-value"));
        assert!(!line.contains("top-secret-password-value"));
        assert!(!line.contains("Bearer private-value"));
        assert!(!line.contains("private-client-secret"));
        assert!(line.contains("[REDACTED]"));
        assert!(line.contains("visible"));
    }

    #[test]
    fn local_log_writer_appends_json_lines_to_app_owned_path() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("moonships-log-{unique}"));
        let path = directory.join("logs").join("moonships.log");
        let event = LogEvent::new(
            LogLevel::Info,
            LogCategory::AppLifecycle,
            "app_started",
            BTreeMap::new(),
        );

        write_event(&path, &event).expect("log event should be written");
        let contents = fs::read_to_string(&path).expect("log file should be readable");
        let parsed: Value = serde_json::from_str(contents.trim()).expect("line should be JSON");
        assert_eq!(parsed["event"], "app_started");
        assert_eq!(parsed["category"], "app_lifecycle");
        let _ = fs::remove_dir_all(directory);
    }
}
