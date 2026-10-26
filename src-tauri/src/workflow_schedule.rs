//! Explicitly opted-in, bounded workflow schedule declarations.
//! This prototype claims due occurrences only; it does not start a timer or execute tools.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MIN_INTERVAL_MINUTES: u32 = 5;
pub const MAX_INTERVAL_MINUTES: u32 = 43_200;
pub const MAX_SCHEDULES: usize = 100;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowSchedule {
    pub schedule_id: String,
    pub workflow_id: String,
    pub workflow_version: u32,
    pub interval_minutes: u32,
    pub enabled: bool,
    pub next_run_at: Option<String>,
    pub last_claimed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateWorkflowScheduleRequest {
    pub workflow_id: String,
    pub workflow_version: u32,
    pub interval_minutes: u32,
}

pub fn create(
    connection: &mut Connection,
    request: &CreateWorkflowScheduleRequest,
) -> Result<WorkflowSchedule, String> {
    validate_request(request)?;
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM workflow_schedules", [], |row| {
            row.get(0)
        })
        .map_err(|_| "IO_ERROR: could not count workflow schedules".to_owned())?;
    if count >= MAX_SCHEDULES as i64 {
        return Err("LIMIT_EXCEEDED: workflow schedule limit reached".to_owned());
    }
    let schedule = WorkflowSchedule {
        schedule_id: Uuid::new_v4().to_string(),
        workflow_id: request.workflow_id.clone(),
        workflow_version: request.workflow_version,
        interval_minutes: request.interval_minutes,
        enabled: false,
        next_run_at: None,
        last_claimed_at: None,
    };
    connection
        .execute(
            "INSERT INTO workflow_schedules
                (schedule_id, workflow_id, workflow_version, interval_minutes, enabled)
             VALUES (?1, ?2, ?3, ?4, 0)",
            params![
                schedule.schedule_id,
                schedule.workflow_id,
                schedule.workflow_version,
                schedule.interval_minutes
            ],
        )
        .map_err(|_| "IO_ERROR: could not create workflow schedule".to_owned())?;
    Ok(schedule)
}

pub fn list(connection: &Connection) -> Result<Vec<WorkflowSchedule>, String> {
    let mut statement = connection
        .prepare(
            "SELECT schedule_id, workflow_id, workflow_version, interval_minutes, enabled, next_run_at, last_claimed_at
             FROM workflow_schedules ORDER BY schedule_id LIMIT ?1",
        )
        .map_err(|_| "IO_ERROR: could not list workflow schedules".to_owned())?;
    let rows = statement
        .query_map([MAX_SCHEDULES as i64], read_schedule)
        .map_err(|_| "IO_ERROR: could not list workflow schedules".to_owned())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "IO_ERROR: could not read workflow schedules".to_owned())
}

pub fn set_enabled(
    connection: &mut Connection,
    schedule_id: &str,
    enabled: bool,
    now: DateTime<Utc>,
) -> Result<WorkflowSchedule, String> {
    let id = parse_schedule_id(schedule_id)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin workflow schedule update".to_owned())?;
    let current = transaction
        .query_row(
            "SELECT schedule_id, workflow_id, workflow_version, interval_minutes, enabled, next_run_at, last_claimed_at
             FROM workflow_schedules WHERE schedule_id=?1",
            [&id],
            read_schedule,
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read workflow schedule".to_owned())?
        .ok_or_else(|| "NOT_FOUND: workflow schedule is unavailable".to_owned())?;
    let next_run_at = if enabled {
        Some(
            (now + Duration::minutes(current.interval_minutes.into()))
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        )
    } else {
        None
    };
    transaction
        .execute(
            "UPDATE workflow_schedules SET enabled=?2, next_run_at=?3 WHERE schedule_id=?1",
            params![id, enabled, next_run_at],
        )
        .map_err(|_| "IO_ERROR: could not update workflow schedule".to_owned())?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit workflow schedule update".to_owned())?;
    Ok(WorkflowSchedule {
        enabled,
        next_run_at,
        ..current
    })
}

/// Atomically claims due occurrences and advances their next time. Callers must
/// recheck that a schedule remains enabled before any future execution begins.
pub fn claim_due(
    connection: &mut Connection,
    now: DateTime<Utc>,
) -> Result<Vec<WorkflowSchedule>, String> {
    let now_text = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin workflow schedule tick".to_owned())?;
    let due = {
        let mut statement = transaction
            .prepare(
                "SELECT schedule_id, workflow_id, workflow_version, interval_minutes, enabled, next_run_at, last_claimed_at
                 FROM workflow_schedules WHERE enabled=1 AND next_run_at<=?1
                 ORDER BY next_run_at, schedule_id LIMIT ?2",
            )
            .map_err(|_| "IO_ERROR: could not query due workflows".to_owned())?;
        let rows = statement
            .query_map(params![now_text, MAX_SCHEDULES as i64], read_schedule)
            .map_err(|_| "IO_ERROR: could not query due workflows".to_owned())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| "IO_ERROR: could not read due workflows".to_owned())?
    };
    for schedule in &due {
        let next = (now + Duration::minutes(schedule.interval_minutes.into()))
            .to_rfc3339_opts(SecondsFormat::Millis, true);
        transaction
            .execute(
                "UPDATE workflow_schedules SET last_claimed_at=?2, next_run_at=?3
                 WHERE schedule_id=?1 AND enabled=1 AND next_run_at<=?2",
                params![schedule.schedule_id, now_text, next],
            )
            .map_err(|_| "IO_ERROR: could not claim due workflow".to_owned())?;
    }
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit workflow schedule tick".to_owned())?;
    Ok(due
        .into_iter()
        .map(|mut schedule| {
            schedule.last_claimed_at = Some(now_text.clone());
            schedule.next_run_at = Some(
                (now + Duration::minutes(schedule.interval_minutes.into()))
                    .to_rfc3339_opts(SecondsFormat::Millis, true),
            );
            schedule
        })
        .collect())
}

fn validate_request(request: &CreateWorkflowScheduleRequest) -> Result<(), String> {
    if !valid_identifier(&request.workflow_id)
        || request.workflow_version == 0
        || !(MIN_INTERVAL_MINUTES..=MAX_INTERVAL_MINUTES).contains(&request.interval_minutes)
    {
        return Err("VALIDATION_FAILED: workflow schedule is invalid".to_owned());
    }
    Ok(())
}

fn parse_schedule_id(schedule_id: &str) -> Result<String, String> {
    Uuid::parse_str(schedule_id)
        .map(|id| id.to_string())
        .map_err(|_| "VALIDATION_FAILED: workflow schedule ID is invalid".to_owned())
}

fn read_schedule(row: &rusqlite::Row<'_>) -> rusqlite::Result<WorkflowSchedule> {
    Ok(WorkflowSchedule {
        schedule_id: row.get(0)?,
        workflow_id: row.get(1)?,
        workflow_version: row.get(2)?,
        interval_minutes: row.get(3)?,
        enabled: row.get(4)?,
        next_run_at: row.get(5)?,
        last_claimed_at: row.get(6)?,
    })
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{claim_due, create, list, set_enabled, CreateWorkflowScheduleRequest};
    use chrono::{DateTime, Utc};
    use rusqlite::Connection;

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::run_migrations(&mut connection).unwrap();
        connection
    }

    fn time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-26T12:00:00Z")
            .unwrap()
            .to_utc()
    }

    fn request() -> CreateWorkflowScheduleRequest {
        CreateWorkflowScheduleRequest {
            workflow_id: "fixture.safe".to_owned(),
            workflow_version: 2,
            interval_minutes: 5,
        }
    }

    #[test]
    fn schedule_is_opt_in_and_pausing_prevents_future_due_occurrences() {
        let mut connection = database();
        let schedule = create(&mut connection, &request()).unwrap();
        assert!(!schedule.enabled);
        assert!(schedule.next_run_at.is_none());
        assert!(
            claim_due(&mut connection, time() + chrono::Duration::days(1))
                .unwrap()
                .is_empty()
        );

        let enabled = set_enabled(&mut connection, &schedule.schedule_id, true, time()).unwrap();
        assert_eq!(
            enabled.next_run_at.as_deref(),
            Some("2026-10-26T12:05:00.000Z")
        );
        let due = claim_due(&mut connection, time() + chrono::Duration::minutes(5)).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].schedule_id, schedule.schedule_id);
        assert!(
            claim_due(&mut connection, time() + chrono::Duration::minutes(5))
                .unwrap()
                .is_empty()
        );

        set_enabled(
            &mut connection,
            &schedule.schedule_id,
            false,
            time() + chrono::Duration::minutes(10),
        )
        .unwrap();
        assert!(
            claim_due(&mut connection, time() + chrono::Duration::days(2))
                .unwrap()
                .is_empty()
        );
        assert!(!list(&connection).unwrap()[0].enabled);
    }

    #[test]
    fn invalid_cadence_and_schedule_ids_are_rejected() {
        let mut connection = database();
        let mut invalid = request();
        invalid.interval_minutes = 4;
        assert!(create(&mut connection, &invalid).is_err());
        assert!(set_enabled(&mut connection, "../bad", true, time()).is_err());
    }
}
