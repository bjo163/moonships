//! One explicit, bounded HTTPS GET inspection. Response bodies are returned to the UI only in memory.
use crate::rss_connector::{self, RssFetchError};
use crate::tasks::{self, CreateTask, Task};
use reqwest::{blocking::Client, redirect::Policy, Url};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
use uuid::Uuid;

pub const MAX_URL_CHARS: usize = 2_048;
pub const MAX_RESPONSE_BYTES: usize = 256 * 1_024;
pub const MAX_HEADERS: usize = 64;
pub const MAX_HISTORY: usize = 200;
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MIN_REQUEST_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InspectRequest {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SafeHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InspectionHistory {
    pub id: String,
    pub requested_url: String,
    pub observed_at: String,
    pub status: Option<u16>,
    pub request_headers: Vec<SafeHeader>,
    pub response_headers: Vec<SafeHeader>,
    pub response_size_bytes: u32,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InspectionResult {
    pub history: InspectionHistory,
    pub body_text: String,
    pub body_is_utf8: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectionError {
    InvalidUrl,
    RateLimited,
    ResolverBusy,
    ResolutionFailed,
    UnsafeAddress,
    TimedOut,
    NetworkFailure,
    HttpStatus,
    ResponseTooLarge,
}

#[derive(Default)]
struct RequestGate {
    last_started: Option<Instant>,
    active: bool,
}
static REQUEST_GATE: OnceLock<Mutex<RequestGate>> = OnceLock::new();

fn claim_request() -> Result<(), InspectionError> {
    let gate = REQUEST_GATE.get_or_init(|| Mutex::new(RequestGate::default()));
    let mut gate = gate.lock().map_err(|_| InspectionError::RateLimited)?;
    let now = Instant::now();
    if gate.active
        || gate
            .last_started
            .is_some_and(|last| now.saturating_duration_since(last) < MIN_REQUEST_INTERVAL)
    {
        return Err(InspectionError::RateLimited);
    }
    gate.last_started = Some(now);
    gate.active = true;
    Ok(())
}

struct RequestLease;
impl Drop for RequestLease {
    fn drop(&mut self) {
        if let Some(gate) = REQUEST_GATE.get() {
            if let Ok(mut gate) = gate.lock() {
                gate.active = false;
            }
        }
    }
}

pub fn inspect(request: InspectRequest) -> Result<InspectionResult, InspectionError> {
    if request.url.chars().count() > MAX_URL_CHARS {
        return Err(InspectionError::InvalidUrl);
    }
    let mut url = Url::parse(&request.url).map_err(|_| InspectionError::InvalidUrl)?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(InspectionError::InvalidUrl);
    }
    let host = url
        .host_str()
        .ok_or(InspectionError::InvalidUrl)?
        .to_owned();
    let port = url
        .port_or_known_default()
        .ok_or(InspectionError::InvalidUrl)?;
    if port != 443 {
        return Err(InspectionError::InvalidUrl);
    }
    let safe_url = redact_url(&mut url);
    claim_request()?;
    let _lease = RequestLease;
    let started = Instant::now();
    let addresses = rss_connector::resolve_public_addresses_for_port(&host, port, REQUEST_TIMEOUT)
        .map_err(map_fetch_error)?;
    let remaining = REQUEST_TIMEOUT.saturating_sub(started.elapsed());
    if remaining.is_zero() {
        return Err(InspectionError::TimedOut);
    }
    let client = Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(Policy::none())
        .timeout(remaining)
        .resolve_to_addrs(&host, &addresses)
        .build()
        .map_err(|_| InspectionError::NetworkFailure)?;
    let response = client
        .get(&request.url)
        .header(
            reqwest::header::USER_AGENT,
            "Moonships Workbench/0.1.0 (https://github.com/bjo163/moonships)",
        )
        .header(reqwest::header::ACCEPT, "*/*")
        .send()
        .map_err(|error| {
            if error.is_timeout() {
                InspectionError::TimedOut
            } else {
                InspectionError::NetworkFailure
            }
        })?;
    let status = response.status().as_u16();
    let response_headers = redact_headers(
        response
            .headers()
            .iter()
            .map(|(name, value)| (name.as_str(), value.to_str().unwrap_or("[non-text header]"))),
    );
    let mut body = Vec::new();
    response
        .take((MAX_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::TimedOut {
                InspectionError::TimedOut
            } else {
                InspectionError::NetworkFailure
            }
        })?;
    if body.len() > MAX_RESPONSE_BYTES {
        return Err(InspectionError::ResponseTooLarge);
    }
    let body_is_utf8 = std::str::from_utf8(&body).is_ok();
    let body_text = String::from_utf8_lossy(&body).into_owned();
    Ok(InspectionResult {
        history: InspectionHistory {
            id: Uuid::new_v4().to_string(),
            requested_url: safe_url,
            observed_at: chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now())
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            status: Some(status),
            request_headers: vec![
                SafeHeader {
                    name: "accept".to_owned(),
                    value: "*/*".to_owned(),
                },
                SafeHeader {
                    name: "user-agent".to_owned(),
                    value: "Moonships Workbench/0.1.0".to_owned(),
                },
            ],
            response_headers,
            response_size_bytes: body.len() as u32,
            error_code: None,
        },
        body_text,
        body_is_utf8,
    })
}

pub fn persist_history(connection: &Connection, history: &InspectionHistory) -> Result<(), String> {
    let mut sanitized = history.clone();
    let mut url = Url::parse(&sanitized.requested_url)
        .map_err(|_| "DATA_ERROR: inspection history URL is invalid".to_owned())?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return Err("DATA_ERROR: inspection history URL is invalid".to_owned());
    }
    sanitized.requested_url = redact_url(&mut url);
    sanitized.request_headers = redact_headers(
        sanitized
            .request_headers
            .iter()
            .map(|header| (header.name.as_str(), header.value.as_str())),
    );
    sanitized.response_headers = redact_headers(
        sanitized
            .response_headers
            .iter()
            .map(|header| (header.name.as_str(), header.value.as_str())),
    );
    let json = serde_json::to_string(&sanitized)
        .map_err(|_| "DATA_ERROR: could not encode HTTP history".to_owned())?;
    if json.len() > 256 * 1024 {
        return Err("LIMIT_REACHED: HTTP history record is too large".to_owned());
    }
    connection
        .execute(
            "INSERT INTO http_inspection_history(id,observed_at,history_json) VALUES(?1,?2,?3)",
            params![sanitized.id, sanitized.observed_at, json],
        )
        .map_err(|_| "DB_ERROR: could not save HTTP history".to_owned())?;
    connection.execute("DELETE FROM http_inspection_history WHERE id NOT IN (SELECT id FROM http_inspection_history ORDER BY observed_at DESC,id DESC LIMIT ?1)", [MAX_HISTORY as i64])
        .map_err(|_| "DB_ERROR: could not bound HTTP history".to_owned())?;
    Ok(())
}

pub fn list_history(connection: &Connection) -> Result<Vec<InspectionHistory>, String> {
    let mut statement = connection.prepare("SELECT history_json FROM http_inspection_history ORDER BY observed_at DESC,id DESC LIMIT ?1")
        .map_err(|_| "DB_ERROR: could not read HTTP history".to_owned())?;
    let rows = statement
        .query_map([MAX_HISTORY as i64 + 1], |row| row.get::<_, String>(0))
        .map_err(|_| "DB_ERROR: could not read HTTP history".to_owned())?;
    let mut history = Vec::new();
    for row in rows {
        history.push(
            serde_json::from_str(
                &row.map_err(|_| "DATA_ERROR: saved HTTP history is invalid".to_owned())?,
            )
            .map_err(|_| "DATA_ERROR: saved HTTP history is invalid".to_owned())?,
        );
    }
    if history.len() > MAX_HISTORY {
        return Err("DATA_ERROR: saved HTTP history exceeds its limit".to_owned());
    }
    Ok(history)
}

/// Create a local Task from persisted, already-redacted inspection metadata.
/// The result body is never read from storage or accepted from the renderer.
pub fn create_task_from_history(
    connection: &mut Connection,
    history_id: &str,
) -> Result<Task, String> {
    let id = Uuid::parse_str(history_id)
        .map_err(|_| "VALIDATION_FAILED: inspection result ID is invalid".to_owned())?
        .to_string();
    let history = connection
        .query_row(
            "SELECT history_json FROM http_inspection_history WHERE id = ?1",
            [&id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "NOT_FOUND: saved inspection result is unavailable".to_owned())?;
    if history.len() > 256 * 1024 {
        return Err("LIMIT_REACHED: inspection metadata exceeds its limit".to_owned());
    }
    let mut history: InspectionHistory = serde_json::from_str(&history)
        .map_err(|_| "DATA_ERROR: saved inspection metadata is invalid".to_owned())?;
    if history.id != id
        || history.response_size_bytes as usize > MAX_RESPONSE_BYTES
        || history.observed_at.chars().count() > 64
        || chrono::DateTime::parse_from_rfc3339(&history.observed_at).is_err()
    {
        return Err("DATA_ERROR: saved inspection metadata is invalid".to_owned());
    }
    let mut url = Url::parse(&history.requested_url)
        .map_err(|_| "DATA_ERROR: saved inspection metadata is invalid".to_owned())?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return Err("DATA_ERROR: saved inspection metadata is invalid".to_owned());
    }
    history.requested_url = redact_url(&mut url);
    let description = format!(
        "HTTP inspection result\nResult ID: {}\nURL: {}\nStatus: {}\nObserved: {}\nResponse size: {} bytes",
        history.id,
        history.requested_url,
            history.status.map_or_else(|| "Failed".to_owned(), |status| format!("HTTP {status}")),
        history.observed_at,
        history.response_size_bytes,
    );
    if description.len() > 10_000 {
        return Err(
            "LIMIT_REACHED: inspection metadata exceeds the Task description limit".to_owned(),
        );
    }
    tasks::create_task(
        connection,
        CreateTask {
            workspace_id: crate::PERSONAL_WORKSPACE_ID.to_owned(),
            project_id: None,
            parent_task_id: None,
            title: "Follow up HTTP inspection".to_owned(),
            description: Some(description),
            due_at: None,
            status: Some("todo".to_owned()),
            priority: Some("p2".to_owned()),
        },
    )
}

pub fn redact_headers<'a>(headers: impl Iterator<Item = (&'a str, &'a str)>) -> Vec<SafeHeader> {
    headers
        .take(MAX_HEADERS)
        .map(|(name, value)| SafeHeader {
            name: name.chars().take(128).collect(),
            value: if is_sensitive_header(name) {
                "[REDACTED]".to_owned()
            } else {
                value.chars().take(2_048).collect()
            },
        })
        .collect()
}

fn is_sensitive_header(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    [
        "authorization",
        "proxy-authorization",
        "cookie",
        "set-cookie",
        "token",
        "api-key",
        "apikey",
        "secret",
        "password",
    ]
    .iter()
    .any(|sensitive| name.contains(sensitive))
}

fn redact_url(url: &mut Url) -> String {
    const SENSITIVE: &[&str] = &[
        "token",
        "access_token",
        "api_key",
        "apikey",
        "key",
        "secret",
        "password",
        "auth",
        "authorization",
        "signature",
        "sig",
    ];
    let pairs = url
        .query_pairs()
        .map(|(key, value)| {
            if SENSITIVE
                .iter()
                .any(|sensitive| key.eq_ignore_ascii_case(sensitive))
            {
                (key.into_owned(), "[REDACTED]".to_owned())
            } else {
                (key.into_owned(), value.into_owned())
            }
        })
        .collect::<Vec<_>>();
    if url.query().is_some() {
        url.query_pairs_mut().clear().extend_pairs(pairs);
    }
    url.to_string()
}

/// Validate an evidence reference and redact common sensitive query values before saving it.
pub fn sanitize_evidence_url(value: &str) -> Result<String, String> {
    if value.chars().count() > MAX_URL_CHARS {
        return Err("VALIDATION_FAILED: evidence URL is too long".to_owned());
    }
    let mut url =
        Url::parse(value).map_err(|_| "VALIDATION_FAILED: evidence URL is invalid".to_owned())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("VALIDATION_FAILED: evidence URL is unsafe".to_owned());
    }
    Ok(redact_url(&mut url))
}

fn map_fetch_error(error: RssFetchError) -> InspectionError {
    match error {
        RssFetchError::RateLimited | RssFetchError::BackoffActive => InspectionError::RateLimited,
        RssFetchError::ResolverBusy => InspectionError::ResolverBusy,
        RssFetchError::ResolutionFailed => InspectionError::ResolutionFailed,
        RssFetchError::UnsafeAddress => InspectionError::UnsafeAddress,
        RssFetchError::RequestTimedOut => InspectionError::TimedOut,
        _ => InspectionError::NetworkFailure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_migrations;

    #[test]
    fn redacts_auth_and_cookie_headers_but_keeps_safe_metadata() {
        let headers = vec![
            ("authorization", "Bearer secret-token"),
            ("cookie", "sid=secret"),
            ("set-cookie", "sid=secret; HttpOnly"),
            ("content-type", "application/json"),
        ];
        let result = redact_headers(headers.into_iter());
        assert_eq!(result[0].value, "[REDACTED]");
        assert_eq!(result[1].value, "[REDACTED]");
        assert_eq!(result[2].value, "[REDACTED]");
        assert_eq!(result[3].value, "application/json");
        assert!(!serde_json::to_string(&result).unwrap().contains("secret"));
    }

    #[test]
    fn request_rejects_non_https_credentials_and_fragments_before_network() {
        for url in [
            "http://example.com/",
            "https://user:pass@example.com/",
            "https://example.com/#fragment",
            "https://example.com:8443/",
        ] {
            assert_eq!(
                inspect(InspectRequest {
                    url: url.to_owned()
                }),
                Err(InspectionError::InvalidUrl)
            );
        }
    }

    #[test]
    fn url_history_redacts_sensitive_query_parameters() {
        let mut url =
            Url::parse("https://example.com/status?region=west&api_key=secret&token=abc").unwrap();
        let safe = redact_url(&mut url);
        assert!(safe.contains("region=west"));
        assert!(!safe.contains("secret"));
        assert!(!safe.contains("abc"));
        assert!(safe.contains("%5BREDACTED%5D"));
    }

    #[test]
    fn persisted_history_contains_redacted_metadata_and_never_a_body_field() {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        let history = InspectionHistory {
            id: Uuid::new_v4().to_string(),
            requested_url: "https://example.test/?token=%5BREDACTED%5D".to_owned(),
            observed_at: "2026-10-26T12:00:00Z".to_owned(),
            status: Some(200),
            request_headers: vec![SafeHeader {
                name: "Authorization".to_owned(),
                value: "Bearer sensitive".to_owned(),
            }],
            response_headers: vec![SafeHeader {
                name: "Set-Cookie".to_owned(),
                value: "sid=sensitive".to_owned(),
            }],
            response_size_bytes: 25,
            error_code: None,
        };
        persist_history(&connection, &history).unwrap();
        let saved = list_history(&connection).unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].request_headers[0].value, "[REDACTED]");
        assert_eq!(saved[0].response_headers[0].value, "[REDACTED]");
        let json: String = connection
            .query_row(
                "SELECT history_json FROM http_inspection_history",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!json.contains("sensitive"));
        assert!(!json.contains("bodyText"));
    }

    #[test]
    fn follow_up_task_uses_only_saved_redacted_bounded_metadata() {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        let history = InspectionHistory {
            id: Uuid::new_v4().to_string(),
            requested_url: "https://example.test/status?token=%5BREDACTED%5D".to_owned(),
            observed_at: "2026-10-26T12:00:00Z".to_owned(),
            status: Some(200),
            request_headers: vec![],
            response_headers: vec![SafeHeader {
                name: "set-cookie".to_owned(),
                value: "[REDACTED]".to_owned(),
            }],
            response_size_bytes: 12,
            error_code: None,
        };
        persist_history(&connection, &history).unwrap();
        connection
            .execute(
                "UPDATE http_inspection_history SET history_json = replace(history_json, '%5BREDACTED%5D', 'secret') WHERE id = ?1",
                [&history.id],
            )
            .unwrap();
        let task = create_task_from_history(&mut connection, &history.id).unwrap();
        let description = task.description.unwrap();
        assert!(description.contains(&history.id));
        assert!(description.contains("token=%5BREDACTED%5D"));
        assert!(!description.contains("secret"));
        assert!(description.contains("Response size: 12 bytes"));
        assert!(!description.contains("REDACTED]"));
        assert!(!description.contains("body"));
    }

    #[test]
    fn follow_up_task_rejects_unknown_or_invalid_result_handles() {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        assert!(create_task_from_history(&mut connection, "not-a-uuid").is_err());
        assert!(create_task_from_history(&mut connection, &Uuid::new_v4().to_string()).is_err());
    }
}
