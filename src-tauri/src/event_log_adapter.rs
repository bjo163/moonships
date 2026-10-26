use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_EVENT_LOG_RECORDS: usize = 100;
pub const DEFAULT_EVENT_LOG_RECORDS: usize = 30;
pub const MIN_EVENT_LOG_TIMEOUT_MS: u32 = 100;
pub const DEFAULT_EVENT_LOG_TIMEOUT_MS: u32 = 2_000;
pub const MAX_EVENT_LOG_TIMEOUT_MS: u32 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventLogChannel {
    Application,
    System,
    Security,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventLogRequest {
    pub channel: EventLogChannel,
    pub max_records: usize,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLogValidationError {
    InvalidRecordLimit,
    InvalidTimeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventLogSnapshotStatus {
    Available,
    Partial,
    Empty,
    PermissionDenied,
    TimedOut,
    Unsupported,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventLogErrorCode {
    AccessDenied,
    TimedOut,
    ProviderUnavailable,
    QueryFailed,
    ResultLimitReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventLogLevel {
    Critical,
    Error,
    Warning,
    Information,
    Verbose,
    Unknown,
}

/// A privacy-minimized summary; raw event messages, account names and payloads are excluded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLogSummary {
    pub event_id: u32,
    pub provider_name: String,
    pub level: EventLogLevel,
    pub time_created_epoch_ms: Option<i64>,
    pub record_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLogSnapshot {
    pub channel: EventLogChannel,
    pub status: EventLogSnapshotStatus,
    pub observed_at_epoch_ms: i64,
    pub events: Vec<EventLogSummary>,
    pub error_code: Option<EventLogErrorCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLogReadError {
    PermissionDenied,
    TimedOut,
    Unsupported,
    Unavailable,
    QueryFailed,
}

/// Future OS readers must honor its result/deadline bounds and return summary fields only.
pub trait EventLogReader {
    fn read_recent(
        &self,
        channel: EventLogChannel,
        max_records: usize,
        timeout_ms: u32,
    ) -> Result<Vec<EventLogSummary>, EventLogReadError>;
}

/// Validates a fixed-channel, bounded read and normalizes provider states for the UI seam.
pub fn read_event_log_snapshot(
    request: EventLogRequest,
    reader: &impl EventLogReader,
) -> Result<EventLogSnapshot, EventLogValidationError> {
    if !(1..=MAX_EVENT_LOG_RECORDS).contains(&request.max_records) {
        return Err(EventLogValidationError::InvalidRecordLimit);
    }
    if !(MIN_EVENT_LOG_TIMEOUT_MS..=MAX_EVENT_LOG_TIMEOUT_MS).contains(&request.timeout_ms) {
        return Err(EventLogValidationError::InvalidTimeout);
    }

    let (status, events, error_code) =
        match reader.read_recent(request.channel, request.max_records, request.timeout_ms) {
            Ok(events) if events.is_empty() => (EventLogSnapshotStatus::Empty, events, None),
            Ok(mut events) if events.len() > request.max_records => {
                events.truncate(request.max_records);
                (
                    EventLogSnapshotStatus::Partial,
                    events,
                    Some(EventLogErrorCode::ResultLimitReached),
                )
            }
            Ok(events) => (EventLogSnapshotStatus::Available, events, None),
            Err(EventLogReadError::PermissionDenied) => (
                EventLogSnapshotStatus::PermissionDenied,
                Vec::new(),
                Some(EventLogErrorCode::AccessDenied),
            ),
            Err(EventLogReadError::TimedOut) => (
                EventLogSnapshotStatus::TimedOut,
                Vec::new(),
                Some(EventLogErrorCode::TimedOut),
            ),
            Err(EventLogReadError::Unsupported) => {
                (EventLogSnapshotStatus::Unsupported, Vec::new(), None)
            }
            Err(EventLogReadError::Unavailable) => (
                EventLogSnapshotStatus::Unavailable,
                Vec::new(),
                Some(EventLogErrorCode::ProviderUnavailable),
            ),
            Err(EventLogReadError::QueryFailed) => (
                EventLogSnapshotStatus::Unavailable,
                Vec::new(),
                Some(EventLogErrorCode::QueryFailed),
            ),
        };

    Ok(EventLogSnapshot {
        channel: request.channel,
        status,
        observed_at_epoch_ms: unix_epoch_milliseconds(),
        events,
        error_code,
    })
}

fn unix_epoch_milliseconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        read_event_log_snapshot, EventLogChannel, EventLogErrorCode, EventLogLevel,
        EventLogReadError, EventLogReader, EventLogRequest, EventLogSnapshotStatus,
        EventLogSummary, EventLogValidationError, DEFAULT_EVENT_LOG_RECORDS,
        DEFAULT_EVENT_LOG_TIMEOUT_MS, MAX_EVENT_LOG_RECORDS,
    };

    struct FixtureReader(Result<Vec<EventLogSummary>, EventLogReadError>);

    impl EventLogReader for FixtureReader {
        fn read_recent(
            &self,
            _channel: EventLogChannel,
            _max_records: usize,
            _timeout_ms: u32,
        ) -> Result<Vec<EventLogSummary>, EventLogReadError> {
            self.0.clone()
        }
    }

    fn request(max_records: usize) -> EventLogRequest {
        EventLogRequest {
            channel: EventLogChannel::Application,
            max_records,
            timeout_ms: DEFAULT_EVENT_LOG_TIMEOUT_MS,
        }
    }

    #[test]
    fn distinguishes_unavailable_permission_denied_and_empty_log() {
        let empty = read_event_log_snapshot(
            request(DEFAULT_EVENT_LOG_RECORDS),
            &FixtureReader(Ok(vec![])),
        )
        .unwrap();
        assert_eq!(empty.status, EventLogSnapshotStatus::Empty);
        assert_eq!(empty.error_code, None);

        let denied = read_event_log_snapshot(
            request(DEFAULT_EVENT_LOG_RECORDS),
            &FixtureReader(Err(EventLogReadError::PermissionDenied)),
        )
        .unwrap();
        assert_eq!(denied.status, EventLogSnapshotStatus::PermissionDenied);
        assert_eq!(denied.error_code, Some(EventLogErrorCode::AccessDenied));

        let unavailable = read_event_log_snapshot(
            request(DEFAULT_EVENT_LOG_RECORDS),
            &FixtureReader(Err(EventLogReadError::Unavailable)),
        )
        .unwrap();
        assert_eq!(unavailable.status, EventLogSnapshotStatus::Unavailable);
        assert_eq!(
            unavailable.error_code,
            Some(EventLogErrorCode::ProviderUnavailable)
        );

        let timed_out = read_event_log_snapshot(
            request(DEFAULT_EVENT_LOG_RECORDS),
            &FixtureReader(Err(EventLogReadError::TimedOut)),
        )
        .unwrap();
        assert_eq!(timed_out.status, EventLogSnapshotStatus::TimedOut);
        assert_eq!(timed_out.error_code, Some(EventLogErrorCode::TimedOut));
    }

    #[test]
    fn request_limit_is_strict_and_provider_overflow_is_truncated() {
        assert_eq!(
            read_event_log_snapshot(request(0), &FixtureReader(Ok(vec![]))),
            Err(EventLogValidationError::InvalidRecordLimit)
        );
        assert_eq!(
            read_event_log_snapshot(
                request(MAX_EVENT_LOG_RECORDS + 1),
                &FixtureReader(Ok(vec![]))
            ),
            Err(EventLogValidationError::InvalidRecordLimit)
        );
        let mut invalid_timeout = request(DEFAULT_EVENT_LOG_RECORDS);
        invalid_timeout.timeout_ms = 0;
        assert_eq!(
            read_event_log_snapshot(invalid_timeout, &FixtureReader(Ok(vec![]))),
            Err(EventLogValidationError::InvalidTimeout)
        );

        let event = EventLogSummary {
            event_id: 7,
            provider_name: "Fixture".to_string(),
            level: EventLogLevel::Information,
            time_created_epoch_ms: Some(1_800_000_000_000),
            record_id: Some(7),
        };
        let snapshot =
            read_event_log_snapshot(request(1), &FixtureReader(Ok(vec![event.clone(), event])))
                .unwrap();
        assert_eq!(snapshot.status, EventLogSnapshotStatus::Partial);
        assert_eq!(
            snapshot.error_code,
            Some(EventLogErrorCode::ResultLimitReached)
        );
        assert_eq!(snapshot.events.len(), 1);
    }

    #[test]
    fn summary_serialization_excludes_raw_message_payload() {
        let event = EventLogSummary {
            event_id: 1,
            provider_name: "Fixture".to_string(),
            level: EventLogLevel::Warning,
            time_created_epoch_ms: None,
            record_id: None,
        };
        let value = serde_json::to_value(event).unwrap();
        assert!(value.get("message").is_none());
        assert!(value.get("payload").is_none());
        assert!(value.get("providerName").is_some());
    }
}
