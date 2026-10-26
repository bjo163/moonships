//! Persisted job records and guarded status transitions for the Activity Center.
use crate::domain::JobStatus;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JobRecord {
    pub id: String,
    pub title: String,
    pub status: JobStatus,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub stage: Option<String>,
    pub progress_percent: Option<f64>,
    pub result_summary: Option<String>,
}

pub fn create(connection: &Connection, title: &str) -> Result<JobRecord, String> {
    let title = title.trim();
    if title.is_empty() || title.len() > 160 {
        return Err("VALIDATION_FAILED: job title must contain 1 to 160 bytes".into());
    }
    let id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO jobs (id, title, status) VALUES (?1, ?2, 'queued')",
            params![id, title],
        )
        .map_err(|_| "IO_ERROR: could not create job".to_string())?;
    get(connection, &id)
}

pub fn transition(
    connection: &Connection,
    id: &str,
    status: JobStatus,
    stage: Option<&str>,
    progress_percent: Option<f64>,
    result_summary: Option<&str>,
) -> Result<JobRecord, String> {
    let current = get(connection, id)?;
    if !allowed_transition(current.status, status) {
        return Err("VALIDATION_FAILED: job status transition is not allowed".into());
    }
    if progress_percent.is_some_and(|value| !value.is_finite() || !(0.0..=100.0).contains(&value)) {
        return Err("VALIDATION_FAILED: job progress must be between 0 and 100".into());
    }
    let stage = stage.map(str::trim).filter(|value| !value.is_empty());
    if stage.is_some_and(|value| value.len() > 160)
        || result_summary.is_some_and(|value| value.trim().len() > 500)
    {
        return Err("VALIDATION_FAILED: job text exceeds its limit".into());
    }
    let finished = matches!(
        status,
        JobStatus::Succeeded
            | JobStatus::Partial
            | JobStatus::Failed
            | JobStatus::Cancelled
            | JobStatus::TimedOut
    );
    connection.execute(
        "UPDATE jobs SET status=?2, stage=?3, progress_percent=?4, result_summary=?5,
         started_at=CASE WHEN ?2 = 'running' AND started_at IS NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE started_at END,
         finished_at=CASE WHEN ?6 THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END WHERE id=?1",
        params![id, status_name(status), stage, progress_percent, result_summary.map(str::trim), finished],
    ).map_err(|_| "IO_ERROR: could not update job".to_string())?;
    get(connection, id)
}

fn allowed_transition(from: JobStatus, to: JobStatus) -> bool {
    use JobStatus::*;
    match from {
        Queued => matches!(
            to,
            Running | WaitingApproval | Cancelled | Failed | TimedOut
        ),
        Running => matches!(
            to,
            WaitingApproval | Succeeded | Partial | Failed | Cancelled | TimedOut
        ),
        WaitingApproval => matches!(to, Running | Cancelled | Failed | TimedOut),
        Succeeded | Partial | Failed | Cancelled | TimedOut => false,
    }
}

fn status_name(status: JobStatus) -> &'static str {
    match status {
        JobStatus::Queued => "queued",
        JobStatus::Running => "running",
        JobStatus::WaitingApproval => "waiting_approval",
        JobStatus::Succeeded => "succeeded",
        JobStatus::Partial => "partial",
        JobStatus::Failed => "failed",
        JobStatus::Cancelled => "cancelled",
        JobStatus::TimedOut => "timed_out",
    }
}

fn parse_status(value: String) -> rusqlite::Result<JobStatus> {
    match value.as_str() {
        "queued" => Ok(JobStatus::Queued),
        "running" => Ok(JobStatus::Running),
        "waiting_approval" => Ok(JobStatus::WaitingApproval),
        "succeeded" => Ok(JobStatus::Succeeded),
        "partial" => Ok(JobStatus::Partial),
        "failed" => Ok(JobStatus::Failed),
        "cancelled" => Ok(JobStatus::Cancelled),
        "timed_out" => Ok(JobStatus::TimedOut),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

pub fn get(connection: &Connection, id: &str) -> Result<JobRecord, String> {
    connection.query_row(
        "SELECT id,title,status,created_at,started_at,finished_at,stage,progress_percent,result_summary FROM jobs WHERE id=?1",
        [id],
        |row| Ok(JobRecord { id:row.get(0)?, title:row.get(1)?, status:parse_status(row.get(2)?)?, created_at:row.get(3)?, started_at:row.get(4)?, finished_at:row.get(5)?, stage:row.get(6)?, progress_percent:row.get(7)?, result_summary:row.get(8)? }),
    ).map_err(|error| match error { rusqlite::Error::QueryReturnedNoRows => "NOT_FOUND: job is unavailable".into(), _ => "IO_ERROR: could not read job".into() })
}

pub fn list(connection: &Connection) -> Result<Vec<JobRecord>, String> {
    let mut statement = connection.prepare("SELECT id,title,status,created_at,started_at,finished_at,stage,progress_percent,result_summary FROM jobs ORDER BY created_at DESC,id DESC LIMIT 200")
        .map_err(|_| "IO_ERROR: could not list jobs".to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(JobRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                status: parse_status(row.get(2)?)?,
                created_at: row.get(3)?,
                started_at: row.get(4)?,
                finished_at: row.get(5)?,
                stage: row.get(6)?,
                progress_percent: row.get(7)?,
                result_summary: row.get(8)?,
            })
        })
        .map_err(|_| "IO_ERROR: could not list jobs".to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "IO_ERROR: could not list jobs".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let mut db = Connection::open_in_memory().unwrap();
        crate::run_migrations(&mut db).unwrap();
        db
    }

    #[test]
    fn job_status_transitions_record_progress_duration_and_result() {
        let db = db();
        let job = create(&db, "Index workspace").unwrap();
        assert_eq!(job.status, JobStatus::Queued);
        let running = transition(
            &db,
            &job.id,
            JobStatus::Running,
            Some("Scanning"),
            Some(40.0),
            None,
        )
        .unwrap();
        assert!(running.started_at.is_some());
        assert_eq!(running.progress_percent, Some(40.0));
        let done = transition(
            &db,
            &job.id,
            JobStatus::Succeeded,
            Some("Complete"),
            Some(100.0),
            Some("12 files indexed"),
        )
        .unwrap();
        assert!(done.finished_at.is_some());
        assert_eq!(done.result_summary.as_deref(), Some("12 files indexed"));
        assert_eq!(list(&db).unwrap(), vec![done]);
    }

    #[test]
    fn terminal_status_and_invalid_progress_cannot_be_overwritten() {
        let db = db();
        let job = create(&db, "Run check").unwrap();
        assert!(transition(&db, &job.id, JobStatus::Succeeded, None, Some(101.0), None).is_err());
        transition(&db, &job.id, JobStatus::Cancelled, None, None, None).unwrap();
        assert!(transition(&db, &job.id, JobStatus::Running, None, None, None).is_err());
    }

    #[test]
    fn approval_wait_can_resume_or_cancel() {
        let db = db();
        let job = create(&db, "Needs approval").unwrap();
        transition(&db, &job.id, JobStatus::WaitingApproval, None, None, None).unwrap();
        assert_eq!(
            transition(&db, &job.id, JobStatus::Running, None, None, None)
                .unwrap()
                .status,
            JobStatus::Running
        );
    }
}
