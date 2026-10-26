//! Bounded network result handoff to a redacted, local Task.
use crate::{
    dns_adapter::{self, DnsLookupError},
    network_diagnostics::{
        NetworkDiagnosticKind, NetworkDiagnosticRequest, NetworkDiagnosticResult,
    },
    tasks::{self, CreateTask, Task},
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};
use uuid::Uuid;

const MAX_CACHED_RESULTS: usize = 100;
static RESULTS: OnceLock<Mutex<VecDeque<DiagnosticResultRecord>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    DnsLookup,
    TcpConnectivity,
}

impl From<DiagnosticKind> for NetworkDiagnosticKind {
    fn from(kind: DiagnosticKind) -> Self {
        match kind {
            DiagnosticKind::DnsLookup => Self::DnsLookup,
            DiagnosticKind::TcpConnectivity => Self::TcpConnectivity,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticRequest {
    pub kind: DiagnosticKind,
    pub target_host: String,
    pub target_port: Option<u16>,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticResultRecord {
    pub id: String,
    pub kind: DiagnosticKind,
    pub target_host: String,
    pub target_port: Option<u16>,
    pub observed_at_epoch_ms: i64,
    pub duration_ms: u32,
    pub source: String,
    pub outcome: String,
    pub error_code: Option<String>,
    pub resolved_addresses: Vec<String>,
}

fn store_result(result: &NetworkDiagnosticResult) -> DiagnosticResultRecord {
    let record = DiagnosticResultRecord {
        id: Uuid::new_v4().to_string(),
        kind: match result.kind {
            NetworkDiagnosticKind::DnsLookup => DiagnosticKind::DnsLookup,
            NetworkDiagnosticKind::TcpConnectivity => DiagnosticKind::TcpConnectivity,
        },
        target_host: result.target_host.clone(),
        target_port: result.target_port,
        observed_at_epoch_ms: result.observed_at_epoch_ms,
        duration_ms: result.duration_ms,
        source: format!("{:?}", result.source),
        outcome: format!("{:?}", result.outcome),
        error_code: result.error_code.map(|value| format!("{value:?}")),
        resolved_addresses: result
            .resolved_addresses
            .iter()
            .map(ToString::to_string)
            .collect(),
    };
    if let Ok(mut results) = RESULTS.get_or_init(|| Mutex::new(VecDeque::new())).lock() {
        results.push_back(record.clone());
        while results.len() > MAX_CACHED_RESULTS {
            results.pop_front();
        }
    }
    record
}

pub fn run(request: DiagnosticRequest) -> Result<DiagnosticResultRecord, String> {
    let request = NetworkDiagnosticRequest {
        kind: request.kind.into(),
        target_host: request.target_host,
        port: request.target_port,
        timeout_ms: request.timeout_ms,
    };
    let result = match request.kind {
        NetworkDiagnosticKind::DnsLookup => dns_adapter::lookup_dns(request),
        NetworkDiagnosticKind::TcpConnectivity => dns_adapter::test_tcp_connectivity(request),
    }
    .map_err(|error| match error {
        DnsLookupError::InvalidRequest(_) => {
            "VALIDATION_FAILED: diagnostic request is invalid".to_string()
        }
        DnsLookupError::WrongOperation => {
            "VALIDATION_FAILED: diagnostic operation is invalid".to_string()
        }
        DnsLookupError::RateLimited => "RATE_LIMITED: diagnostic request limit reached".to_string(),
        DnsLookupError::ResolverUnavailable => {
            "UNAVAILABLE: operating system resolver is unavailable".to_string()
        }
        DnsLookupError::ResolverError => {
            "UNAVAILABLE: operating system resolver failed".to_string()
        }
        DnsLookupError::TimedOut => "TIMED_OUT: diagnostic exceeded its deadline".to_string(),
    })?;
    Ok(store_result(&result))
}

pub fn create_task_from_result(
    connection: &mut Connection,
    workspace_id: &str,
    result_id: &str,
) -> Result<Task, String> {
    let id = Uuid::parse_str(result_id)
        .map_err(|_| "VALIDATION_FAILED: diagnostic result is invalid".to_string())?
        .to_string();
    let result = RESULTS
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .map_err(|_| "UNAVAILABLE: diagnostic result cache is unavailable".to_string())?
        .iter()
        .find(|record| record.id == id)
        .cloned()
        .ok_or_else(|| "NOT_FOUND: diagnostic result expired; run the check again".to_string())?;
    let observed_at = chrono::DateTime::from_timestamp_millis(result.observed_at_epoch_ms)
        .ok_or_else(|| "VALIDATION_FAILED: diagnostic result time is invalid".to_string())?
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let operation = match result.kind {
        DiagnosticKind::DnsLookup => "DNS lookup",
        DiagnosticKind::TcpConnectivity => "TCP connectivity test",
    };
    let target = result.target_port.map_or_else(
        || result.target_host.clone(),
        |port| format!("{}:{port}", result.target_host),
    );
    // Deliberately omit resolved addresses and arbitrary provider/message content.
    let description = format!("Diagnostic: {operation}\nTarget: {target}\nResult: {}{}\nObserved: {observed_at}\nDuration: {} ms\nSource: {}",
        result.outcome,
        result.error_code.as_deref().map(|code| format!(" ({code})")).unwrap_or_default(),
        result.duration_ms,
        result.source,
    );
    tasks::create_task(
        connection,
        CreateTask {
            workspace_id: workspace_id.to_string(),
            project_id: None,
            parent_task_id: None,
            title: "Follow up network diagnostic".to_string(),
            description: Some(description),
            due_at: None,
            status: Some("todo".to_string()),
            priority: Some("p2".to_string()),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network_diagnostics::{NetworkDiagnosticOutcome, NetworkDiagnosticSource};
    use std::net::IpAddr;

    #[test]
    fn diagnostic_task_links_target_result_and_time_without_address_payloads() {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::run_migrations(&mut connection).unwrap();
        let result = NetworkDiagnosticResult {
            kind: NetworkDiagnosticKind::DnsLookup,
            target_host: "internal.example".into(),
            target_port: None,
            observed_at_epoch_ms: 1_800_000_000_000,
            duration_ms: 23,
            source: NetworkDiagnosticSource::SystemResolver,
            outcome: NetworkDiagnosticOutcome::Succeeded,
            error_code: None,
            resolved_addresses: vec!["192.0.2.10".parse::<IpAddr>().unwrap()],
        };
        let record = store_result(&result);
        let task =
            create_task_from_result(&mut connection, crate::PERSONAL_WORKSPACE_ID, &record.id)
                .unwrap();
        let description = task.description.unwrap();
        assert!(description.contains("internal.example"));
        assert!(description.contains("Succeeded"));
        assert!(description.contains("Observed: 2027-01-15T08:00:00.000Z"));
        assert!(!description.contains("192.0.2.10"));
    }

    #[test]
    fn expired_result_handle_cannot_be_used_to_create_a_task() {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::run_migrations(&mut connection).unwrap();
        assert!(create_task_from_result(
            &mut connection,
            crate::PERSONAL_WORKSPACE_ID,
            &Uuid::new_v4().to_string()
        )
        .is_err());
    }
}
