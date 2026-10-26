//! Bounded local workflow run history. Step output values are deliberately not
//! persisted; successful output is represented only by a fixed redaction marker.

use crate::workflow_runner::{WorkflowRunResult, WorkflowRunStatus, WorkflowStepOutcome};
use chrono::DateTime;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_RETAINED_WORKFLOW_RUNS: usize = 500;
pub const MAX_WORKFLOW_RUN_LIST: usize = 200;
const REDACTED_OUTPUT_SUMMARY: &str = "[REDACTED]";
const MAX_RUN_DURATION_MS: u64 = 24 * 60 * 60 * 1000;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowRunSummary {
    pub run_id: String,
    pub workflow_id: String,
    pub workflow_version: u32,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub correlation_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowStepHistory {
    pub ordinal: u32,
    pub step_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub status: String,
    pub duration_ms: u64,
    pub output_summary: Option<String>,
    pub error_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowRunDetail {
    pub run: WorkflowRunSummary,
    pub steps: Vec<WorkflowStepHistory>,
}

pub fn record(connection: &mut Connection, result: &WorkflowRunResult) -> Result<(), String> {
    validate_result(result)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin workflow history write".to_owned())?;
    transaction
        .execute(
            "INSERT INTO workflow_runs
                (run_id, workflow_id, workflow_version, status, started_at, finished_at, duration_ms, correlation_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                result.run_id,
                result.workflow_id,
                result.workflow_version,
                run_status_name(result.status),
                result.started_at,
                result.finished_at,
                result.duration_ms,
                result.correlation_id
            ],
        )
        .map_err(|_| "IO_ERROR: could not save workflow run history".to_owned())?;

    for (ordinal, step) in result.steps.iter().enumerate() {
        let (status, output_summary, error_code) = match &step.outcome {
            WorkflowStepOutcome::Succeeded { .. } => {
                ("succeeded", Some(REDACTED_OUTPUT_SUMMARY), None)
            }
            WorkflowStepOutcome::Failed { error } => {
                ("failed", None, Some(tool_error_code(*error)))
            }
            WorkflowStepOutcome::Skipped { .. } => ("skipped", None, Some("SKIPPED")),
        };
        transaction
            .execute(
                "INSERT INTO workflow_run_steps
                    (run_id, ordinal, step_id, tool_id, tool_version, status, duration_ms, output_summary, error_code)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    result.run_id,
                    ordinal as u32,
                    step.step_id,
                    step.tool_id,
                    step.tool_version,
                    status,
                    step.duration_ms,
                    output_summary,
                    error_code
                ],
            )
            .map_err(|_| "IO_ERROR: could not save workflow step history".to_owned())?;
    }

    transaction
        .execute(
            "DELETE FROM workflow_runs WHERE run_id IN (
                SELECT run_id FROM workflow_runs ORDER BY finished_at DESC, run_id DESC LIMIT -1 OFFSET ?1
             )",
            [MAX_RETAINED_WORKFLOW_RUNS as i64],
        )
        .map_err(|_| "IO_ERROR: could not trim workflow run history".to_owned())?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit workflow run history".to_owned())
}

pub fn list(connection: &Connection) -> Result<Vec<WorkflowRunSummary>, String> {
    let mut statement = connection
        .prepare(
            "SELECT run_id, workflow_id, workflow_version, status, started_at, finished_at, duration_ms, correlation_id
             FROM workflow_runs ORDER BY finished_at DESC, run_id DESC LIMIT ?1",
        )
        .map_err(|_| "IO_ERROR: could not list workflow run history".to_owned())?;
    let rows = statement
        .query_map([MAX_WORKFLOW_RUN_LIST as i64], read_summary)
        .map_err(|_| "IO_ERROR: could not list workflow run history".to_owned())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "IO_ERROR: could not read workflow run history".to_owned())
}

pub fn get(connection: &Connection, run_id: &str) -> Result<WorkflowRunDetail, String> {
    let parsed_id = Uuid::parse_str(run_id)
        .map_err(|_| "VALIDATION_FAILED: workflow run ID is invalid".to_owned())?
        .to_string();
    let run = connection
        .query_row(
            "SELECT run_id, workflow_id, workflow_version, status, started_at, finished_at, duration_ms, correlation_id
             FROM workflow_runs WHERE run_id=?1",
            [&parsed_id],
            read_summary,
        )
        .optional()
        .map_err(|_| "IO_ERROR: could not read workflow run".to_owned())?
        .ok_or_else(|| "NOT_FOUND: workflow run is unavailable".to_owned())?;
    let mut statement = connection
        .prepare(
            "SELECT ordinal, step_id, tool_id, tool_version, status, duration_ms, output_summary, error_code
             FROM workflow_run_steps WHERE run_id=?1 ORDER BY ordinal ASC",
        )
        .map_err(|_| "IO_ERROR: could not read workflow step history".to_owned())?;
    let rows = statement
        .query_map([&parsed_id], |row| {
            Ok(WorkflowStepHistory {
                ordinal: row.get(0)?,
                step_id: row.get(1)?,
                tool_id: row.get(2)?,
                tool_version: row.get(3)?,
                status: row.get(4)?,
                duration_ms: row.get(5)?,
                output_summary: row.get(6)?,
                error_code: row.get(7)?,
            })
        })
        .map_err(|_| "IO_ERROR: could not read workflow step history".to_owned())?;
    let steps = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "IO_ERROR: could not read workflow step history".to_owned())?;
    Ok(WorkflowRunDetail { run, steps })
}

fn validate_result(result: &WorkflowRunResult) -> Result<(), String> {
    for value in [&result.run_id, &result.correlation_id] {
        if Uuid::parse_str(value)
            .map(|id| id.to_string() != *value)
            .unwrap_or(true)
        {
            return Err("VALIDATION_FAILED: workflow run identity is invalid".to_owned());
        }
    }
    if !valid_identifier(&result.workflow_id)
        || result.workflow_version == 0
        || result.steps.is_empty()
        || result.steps.len() > 128
        || result.duration_ms > MAX_RUN_DURATION_MS
        || DateTime::parse_from_rfc3339(&result.started_at).is_err()
        || DateTime::parse_from_rfc3339(&result.finished_at).is_err()
    {
        return Err("VALIDATION_FAILED: workflow run metadata is invalid".to_owned());
    }
    let mut seen = std::collections::BTreeSet::new();
    for step in &result.steps {
        if !valid_identifier(&step.step_id)
            || !valid_identifier(&step.tool_id)
            || !valid_tool_version(&step.tool_version)
            || step.duration_ms > MAX_RUN_DURATION_MS
            || !seen.insert(step.step_id.as_str())
        {
            return Err("VALIDATION_FAILED: workflow step history is invalid".to_owned());
        }
    }
    Ok(())
}

fn read_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<WorkflowRunSummary> {
    let status: String = row.get(3)?;
    if !matches!(status.as_str(), "succeeded" | "failed") {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(WorkflowRunSummary {
        run_id: row.get(0)?,
        workflow_id: row.get(1)?,
        workflow_version: row.get(2)?,
        status,
        started_at: row.get(4)?,
        finished_at: row.get(5)?,
        duration_ms: row.get(6)?,
        correlation_id: row.get(7)?,
    })
}

fn run_status_name(status: WorkflowRunStatus) -> &'static str {
    match status {
        WorkflowRunStatus::Succeeded => "succeeded",
        WorkflowRunStatus::Failed => "failed",
    }
}

fn tool_error_code(error: crate::workflow_runner::WorkflowToolError) -> &'static str {
    use crate::workflow_runner::WorkflowToolError;
    match error {
        WorkflowToolError::Failed => "TOOL_FAILED",
        WorkflowToolError::Unavailable => "TOOL_UNAVAILABLE",
        WorkflowToolError::Rejected => "TOOL_REJECTED",
        WorkflowToolError::TimedOut => "TOOL_TIMED_OUT",
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

fn valid_tool_version(value: &str) -> bool {
    let mut parts = value.split('.');
    let valid_part = |part: Option<&str>| {
        part.is_some_and(|part| {
            !part.is_empty()
                && part.len() <= 9
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == "0" || !part.starts_with('0'))
        })
    };
    valid_part(parts.next())
        && valid_part(parts.next())
        && valid_part(parts.next())
        && parts.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::{get, list, record, WorkflowRunStatus};
    use crate::{
        run_migrations,
        workflow_runner::{WorkflowRunResult, WorkflowStepOutcome, WorkflowStepResult},
        workflow_schema::WorkflowValue,
    };
    use rusqlite::Connection;
    use uuid::Uuid;

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        connection
    }

    fn result() -> WorkflowRunResult {
        WorkflowRunResult {
            run_id: Uuid::from_u128(1).to_string(),
            correlation_id: Uuid::from_u128(2).to_string(),
            workflow_id: "fixture.safe".to_owned(),
            workflow_version: 2,
            status: WorkflowRunStatus::Succeeded,
            started_at: "2026-10-26T00:00:00Z".to_owned(),
            finished_at: "2026-10-26T00:00:01Z".to_owned(),
            duration_ms: 1_000,
            steps: vec![WorkflowStepResult {
                step_id: "step.one".to_owned(),
                tool_id: "fixture.read".to_owned(),
                tool_version: "1.0.0".to_owned(),
                duration_ms: 750,
                outcome: WorkflowStepOutcome::Succeeded {
                    output: WorkflowValue::Object(
                        [(
                            "apiToken".to_owned(),
                            WorkflowValue::String("synthetic-secret-sentinel".to_owned()),
                        )]
                        .into_iter()
                        .collect(),
                    ),
                },
            }],
        }
    }

    #[test]
    fn persisted_history_shows_status_duration_correlation_and_redacts_step_output() {
        let mut connection = database();
        let result = result();
        record(&mut connection, &result).unwrap();

        let summaries = list(&connection).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].status, "succeeded");
        assert_eq!(summaries[0].duration_ms, 1_000);
        assert_eq!(summaries[0].correlation_id, Uuid::from_u128(2).to_string());
        let detail = get(&connection, &result.run_id).unwrap();
        assert_eq!(detail.steps[0].duration_ms, 750);
        assert_eq!(
            detail.steps[0].output_summary.as_deref(),
            Some("[REDACTED]")
        );
        assert!(!serde_json::to_string(&detail)
            .unwrap()
            .contains("synthetic-secret-sentinel"));
    }

    #[test]
    fn invalid_run_metadata_and_lookup_ids_are_rejected() {
        let mut connection = database();
        let mut malformed = result();
        malformed.correlation_id = "not-a-uuid".to_owned();
        assert!(record(&mut connection, &malformed).is_err());

        let result = result();
        record(&mut connection, &result).unwrap();
        assert!(get(&connection, "../outside").is_err());
    }
}
