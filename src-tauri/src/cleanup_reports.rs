//! Persisted, path-free summaries of completed cleaner executions.

use crate::cleaner::{
    CleanupApprovalRequestDto, CleanupCandidateIdentity, CleanupPreviewDto, CleanupRule,
};
use crate::cleaner::{CleanupExecutionReport, CleanupItemOutcome, MAX_CLEANUP_ITEMS_PER_EXECUTION};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{path::Path, path::PathBuf, time::SystemTime};
use uuid::Uuid;

pub const MAX_RECENT_CLEANUP_REPORTS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupReportStatus {
    Succeeded,
    Partial,
    Failed,
}

impl CleanupReportStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Partial => "partial",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CleanupReportSummaryDto {
    pub id: String,
    pub preview_id: String,
    pub task_id: Option<String>,
    pub status: CleanupReportStatus,
    pub completed_at: String,
    pub selected_count: u32,
    pub deleted_count: u32,
    pub skipped_count: u32,
    pub failed_count: u32,
    pub estimated_bytes: u64,
    pub actual_deleted_bytes: u64,
}

pub struct ApprovedCleanupExecution<'a> {
    pub workspace_id: &'a str,
    pub task_id: Option<&'a str>,
    pub approval: &'a CleanupApprovalRequestDto,
    pub preview: &'a CleanupPreviewDto,
    pub candidates: &'a [CleanupCandidateIdentity],
    pub rule: &'a CleanupRule,
    pub allowed_root: &'a Path,
    pub protected_paths: &'a [PathBuf],
    pub observed_at: SystemTime,
}

/// Native orchestration entry point: execute the exact user-approved selection, then save a
/// path-free result. Keep this behind the native approval flow; never expose it as renderer IPC.
pub fn execute_approved_cleanup_and_save(
    connection: &mut Connection,
    context: ApprovedCleanupExecution<'_>,
) -> Result<CleanupReportSummaryDto, String> {
    let execution = crate::cleaner::execute_approved_cleanup(
        context.approval,
        context.preview,
        context.candidates,
        context.rule,
        context.allowed_root,
        context.protected_paths,
        context.observed_at,
    )
    .map_err(|_| {
        "VALIDATION_FAILED: cleanup approval or candidate selection is invalid".to_string()
    })?;
    save_cleanup_report(
        connection,
        context.workspace_id,
        context.task_id,
        &execution,
    )
}

/// Persists summary values only. Candidate IDs, locators and paths are never stored.
pub fn save_cleanup_report(
    connection: &mut Connection,
    workspace_id: &str,
    task_id: Option<&str>,
    report: &CleanupExecutionReport,
) -> Result<CleanupReportSummaryDto, String> {
    let workspace_id = workspace_id.trim();
    if workspace_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace ID is required".to_string());
    }
    if report.preview_id.is_nil() || report.items.is_empty() {
        return Err("VALIDATION_FAILED: cleanup report is empty or invalid".to_string());
    }
    if report.items.len() > MAX_CLEANUP_ITEMS_PER_EXECUTION {
        return Err("VALIDATION_FAILED: cleanup report exceeds the item limit".to_string());
    }

    let mut estimated_bytes = 0_u64;
    let mut actual_deleted_bytes = 0_u64;
    let mut deleted_count = 0_u32;
    let mut skipped_count = 0_u32;
    let mut failed_count = 0_u32;
    let mut candidate_ids = Vec::with_capacity(report.items.len());
    for item in &report.items {
        if candidate_ids.contains(&item.candidate_id) {
            return Err("VALIDATION_FAILED: cleanup report contains duplicate items".to_string());
        }
        candidate_ids.push(item.candidate_id);
        estimated_bytes = estimated_bytes
            .checked_add(item.estimated_bytes)
            .ok_or_else(|| "VALIDATION_FAILED: cleanup report byte total overflowed".to_string())?;
        match item.outcome {
            CleanupItemOutcome::Deleted { actual_bytes } => {
                if actual_bytes > item.estimated_bytes {
                    return Err(
                        "VALIDATION_FAILED: actual bytes exceed the item estimate".to_string()
                    );
                }
                actual_deleted_bytes = actual_deleted_bytes
                    .checked_add(actual_bytes)
                    .ok_or_else(|| "VALIDATION_FAILED: actual byte total overflowed".to_string())?;
                deleted_count += 1;
            }
            CleanupItemOutcome::Skipped(_) => skipped_count += 1,
            CleanupItemOutcome::Failed(_) => failed_count += 1,
        }
    }
    if estimated_bytes != report.estimated_bytes
        || actual_deleted_bytes != report.actual_deleted_bytes
        || actual_deleted_bytes > estimated_bytes
    {
        return Err("VALIDATION_FAILED: cleanup report totals are inconsistent".to_string());
    }

    let task_id = match task_id {
        None => None,
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                return Err("VALIDATION_FAILED: linked Task ID is invalid".to_string());
            }
            Some(
                Uuid::parse_str(value)
                    .map_err(|_| "VALIDATION_FAILED: linked Task ID is invalid".to_string())?
                    .to_string(),
            )
        }
    };
    if let Some(task_id) = task_id.as_deref() {
        let task_exists: Option<i64> = connection
            .query_row(
                "SELECT 1 FROM tasks WHERE workspace_id = ?1 AND id = ?2",
                params![workspace_id, task_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "IO_ERROR: could not validate linked Task".to_string())?;
        if task_exists.is_none() {
            return Err("NOT_FOUND: linked Task is unavailable in this workspace".to_string());
        }
    }

    let status = if failed_count == report.items.len() as u32 {
        CleanupReportStatus::Failed
    } else if skipped_count > 0 || failed_count > 0 {
        CleanupReportStatus::Partial
    } else {
        CleanupReportStatus::Succeeded
    };
    let now = chrono::DateTime::<chrono::Utc>::from(SystemTime::now()).to_rfc3339();
    let summary = CleanupReportSummaryDto {
        id: Uuid::new_v4().to_string(),
        preview_id: report.preview_id.to_string(),
        task_id,
        status,
        completed_at: now,
        selected_count: report.items.len() as u32,
        deleted_count,
        skipped_count,
        failed_count,
        estimated_bytes,
        actual_deleted_bytes,
    };
    let transaction = connection
        .transaction()
        .map_err(|_| "IO_ERROR: could not begin cleanup report transaction".to_string())?;
    transaction
        .execute(
            "INSERT INTO cleanup_reports (
                id, workspace_id, task_id, preview_id, status, completed_at,
                selected_count, deleted_count, skipped_count, failed_count,
                estimated_bytes, actual_deleted_bytes
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                summary.id,
                workspace_id,
                summary.task_id,
                summary.preview_id,
                summary.status.as_str(),
                summary.completed_at,
                i64::from(summary.selected_count),
                i64::from(summary.deleted_count),
                i64::from(summary.skipped_count),
                i64::from(summary.failed_count),
                i64::try_from(summary.estimated_bytes).map_err(|_| {
                    "VALIDATION_FAILED: estimated bytes exceed storage limits".to_string()
                })?,
                i64::try_from(summary.actual_deleted_bytes).map_err(|_| {
                    "VALIDATION_FAILED: actual bytes exceed storage limits".to_string()
                })?,
            ],
        )
        .map_err(|_| "IO_ERROR: could not save cleanup report".to_string())?;
    transaction
        .commit()
        .map_err(|_| "IO_ERROR: could not commit cleanup report".to_string())?;
    Ok(summary)
}

/// Loads bounded, recent report summaries in the native workspace scope.
pub fn list_recent_cleanup_reports(
    connection: &Connection,
    workspace_id: &str,
) -> Result<Vec<CleanupReportSummaryDto>, String> {
    let workspace_id = workspace_id.trim();
    if workspace_id.is_empty() {
        return Err("VALIDATION_FAILED: workspace ID is required".to_string());
    }
    let mut statement = connection
        .prepare(
            "SELECT id, preview_id, task_id, status, completed_at,
                    selected_count, deleted_count, skipped_count, failed_count,
                    estimated_bytes, actual_deleted_bytes
             FROM cleanup_reports
             WHERE workspace_id = ?1
             ORDER BY completed_at DESC, rowid DESC
             LIMIT ?2",
        )
        .map_err(|_| "IO_ERROR: could not prepare cleanup report history".to_string())?;
    let rows = statement
        .query_map(
            params![workspace_id, MAX_RECENT_CLEANUP_REPORTS as i64],
            cleanup_report_from_row,
        )
        .map_err(|_| "IO_ERROR: could not list cleanup reports".to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "IO_ERROR: could not read cleanup report history".to_string())
}

fn cleanup_report_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CleanupReportSummaryDto> {
    let status = match row.get::<_, String>(3)?.as_str() {
        "succeeded" => CleanupReportStatus::Succeeded,
        "partial" => CleanupReportStatus::Partial,
        "failed" => CleanupReportStatus::Failed,
        _ => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "unknown cleanup report status",
                )),
            ));
        }
    };
    let selected_count = row.get::<_, i64>(5)?;
    let deleted_count = row.get::<_, i64>(6)?;
    let skipped_count = row.get::<_, i64>(7)?;
    let failed_count = row.get::<_, i64>(8)?;
    let estimated_bytes = row.get::<_, i64>(9)?;
    let actual_deleted_bytes = row.get::<_, i64>(10)?;
    Ok(CleanupReportSummaryDto {
        id: row.get(0)?,
        preview_id: row.get(1)?,
        task_id: row.get(2)?,
        status,
        completed_at: row.get(4)?,
        selected_count: u32::try_from(selected_count)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, selected_count))?,
        deleted_count: u32::try_from(deleted_count)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(6, deleted_count))?,
        skipped_count: u32::try_from(skipped_count)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(7, skipped_count))?,
        failed_count: u32::try_from(failed_count)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(8, failed_count))?,
        estimated_bytes: u64::try_from(estimated_bytes)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(9, estimated_bytes))?,
        actual_deleted_bytes: u64::try_from(actual_deleted_bytes)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(10, actual_deleted_bytes))?,
    })
}

#[cfg(test)]
mod tests {
    use super::{list_recent_cleanup_reports, save_cleanup_report, CleanupReportStatus};
    use crate::cleaner::{
        CleanupApprovalRequestDto, CleanupCandidateIdentity, CleanupExecutionReport,
        CleanupItemOutcome, CleanupItemResult, CleanupPreviewDisposition, CleanupPreviewDto,
        CleanupPreviewItemDto, CleanupRevalidationError,
    };
    use rusqlite::{params, Connection};
    use std::{
        fs,
        time::{Duration, SystemTime},
    };
    use uuid::Uuid;

    const WORKSPACE_ID: &str = "00000000-0000-4000-8000-000000000001";
    const TASK_ID: &str = "00000000-0000-4000-8000-000000000099";

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().expect("open in-memory test database");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("enable fixture foreign keys");
        crate::run_migrations(&mut connection).expect("apply disposable test migrations");
        connection
            .execute(
                "INSERT INTO tasks (id, workspace_id, title) VALUES (?1, ?2, 'Report link fixture')",
                params![TASK_ID, WORKSPACE_ID],
            )
            .expect("create disposable Task link");
        connection
    }

    fn mixed_report() -> CleanupExecutionReport {
        CleanupExecutionReport {
            preview_id: Uuid::new_v4(),
            estimated_bytes: 600,
            actual_deleted_bytes: 80,
            items: vec![
                CleanupItemResult {
                    candidate_id: Uuid::new_v4(),
                    estimated_bytes: 100,
                    outcome: CleanupItemOutcome::Deleted { actual_bytes: 80 },
                },
                CleanupItemResult {
                    candidate_id: Uuid::new_v4(),
                    estimated_bytes: 200,
                    outcome: CleanupItemOutcome::Skipped(CleanupRevalidationError::Changed),
                },
                CleanupItemResult {
                    candidate_id: Uuid::new_v4(),
                    estimated_bytes: 300,
                    outcome: CleanupItemOutcome::Failed(
                        crate::cleaner::CleanupDeletionError::PermissionDenied,
                    ),
                },
            ],
        }
    }

    #[test]
    fn cleanup_report_persists_redacted_summary_and_optional_task_link() {
        let mut connection = database();
        let report = mixed_report();
        let saved = save_cleanup_report(&mut connection, WORKSPACE_ID, Some(TASK_ID), &report)
            .expect("persist path-free report summary");

        assert_eq!(saved.task_id.as_deref(), Some(TASK_ID));
        assert_eq!(saved.status, CleanupReportStatus::Partial);
        assert_eq!(saved.selected_count, 3);
        assert_eq!(saved.deleted_count, 1);
        assert_eq!(saved.skipped_count, 1);
        assert_eq!(saved.failed_count, 1);
        assert_eq!(saved.estimated_bytes, 600);
        assert_eq!(saved.actual_deleted_bytes, 80);
        let history = list_recent_cleanup_reports(&connection, WORKSPACE_ID)
            .expect("inspect saved report history");
        assert_eq!(history, vec![saved.clone()]);
        let serialized = serde_json::to_string(&saved).expect("serialize report summary");
        assert!(serialized.contains("actualDeletedBytes"));
        assert!(!serialized.contains("candidateId"));
        assert!(!serialized.contains("relativePath"));
    }

    #[test]
    fn cleanup_report_rejects_inconsistent_totals_without_partial_writes() {
        let mut connection = database();
        let mut report = mixed_report();
        report.actual_deleted_bytes = 81;
        assert!(
            save_cleanup_report(&mut connection, WORKSPACE_ID, None, &report)
                .unwrap_err()
                .starts_with("VALIDATION_FAILED:")
        );
        assert!(list_recent_cleanup_reports(&connection, WORKSPACE_ID)
            .expect("read empty report history")
            .is_empty());
        assert!(save_cleanup_report(
            &mut connection,
            WORKSPACE_ID,
            Some("other-workspace-task"),
            &mixed_report()
        )
        .unwrap_err()
        .starts_with("VALIDATION_FAILED:"));
    }

    #[cfg(windows)]
    #[test]
    fn approved_cleanup_and_report_wrapper_saves_only_the_completed_summary() {
        let mut connection = database();
        let fixture = std::env::temp_dir().join(format!("moonships-report-{}", Uuid::new_v4()));
        let root = fixture.join("temporary");
        fs::create_dir_all(&root).expect("create disposable report fixture");
        let target = root.join("stale.bin");
        fs::write(&target, b"fixture bytes").expect("write disposable cleanup target");
        fs::OpenOptions::new()
            .write(true)
            .open(&target)
            .expect("open disposable cleanup target")
            .set_times(
                fs::FileTimes::new()
                    .set_modified(SystemTime::now() - Duration::from_secs(8 * 24 * 60 * 60)),
            )
            .expect("age disposable cleanup target");
        let rule = crate::cleaner::safe_cleanup_rules()[0].clone();
        let candidate = CleanupCandidateIdentity::capture(&root, rule.allowed_root, "stale.bin")
            .expect("capture disposable cleanup candidate");
        let preview = CleanupPreviewDto {
            preview_id: Uuid::new_v4(),
            generated_at: chrono::DateTime::<chrono::Utc>::from(SystemTime::now()).to_rfc3339(),
            items: vec![CleanupPreviewItemDto::from_candidate(
                &candidate,
                rule.category,
                CleanupPreviewDisposition::Eligible,
                None,
            )
            .expect("create candidate preview")],
        };
        let approval = CleanupApprovalRequestDto::from_selection(&preview, &[candidate.id])
            .expect("approve disposable candidate");

        let saved = super::execute_approved_cleanup_and_save(
            &mut connection,
            super::ApprovedCleanupExecution {
                workspace_id: WORKSPACE_ID,
                task_id: Some(TASK_ID),
                approval: &approval,
                preview: &preview,
                candidates: &[candidate],
                rule: &rule,
                allowed_root: &root,
                protected_paths: &[],
                observed_at: SystemTime::now(),
            },
        )
        .expect("execute and persist approved disposable cleanup");

        assert_eq!(saved.status, CleanupReportStatus::Succeeded);
        assert_eq!(saved.selected_count, 1);
        assert_eq!(saved.deleted_count, 1);
        assert_eq!(saved.estimated_bytes, b"fixture bytes".len() as u64);
        assert_eq!(saved.actual_deleted_bytes, b"fixture bytes".len() as u64);
        assert!(!target.exists());
        assert_eq!(
            list_recent_cleanup_reports(&connection, WORKSPACE_ID).unwrap(),
            vec![saved],
        );
        fs::remove_dir_all(fixture).expect("remove disposable report fixture");
    }
}
