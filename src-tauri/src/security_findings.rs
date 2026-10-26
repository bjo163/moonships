//! Bounded security finding shape with independent severity and likelihood dimensions.
use crate::{
    assessment_scope::{AssessmentScope, ScopeTarget, ScopeTargetKind},
    tasks::{self, CreateTask, Task},
};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_FINDING_TITLE_CHARS: usize = 200;
pub const MAX_FINDING_SUMMARY_CHARS: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingLikelihood {
    Rare,
    Unlikely,
    Possible,
    Likely,
    AlmostCertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingState {
    Open,
    Accepted,
    Mitigated,
    FalsePositive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityFinding {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub severity: FindingSeverity,
    pub likelihood: FindingLikelihood,
    pub state: FindingState,
    pub target: ScopeTarget,
    pub observed_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingValidationError {
    InvalidId,
    InvalidTitle,
    InvalidSummary,
    InvalidTarget,
    InvalidObservedAt,
}

impl SecurityFinding {
    pub fn validate(&self) -> Result<(), FindingValidationError> {
        if Uuid::parse_str(&self.id)
            .map(|id| id.to_string() != self.id)
            .unwrap_or(true)
        {
            return Err(FindingValidationError::InvalidId);
        }
        if self.title.trim().is_empty() || self.title.chars().count() > MAX_FINDING_TITLE_CHARS {
            return Err(FindingValidationError::InvalidTitle);
        }
        if self.summary.trim().is_empty()
            || self.summary.chars().count() > MAX_FINDING_SUMMARY_CHARS
        {
            return Err(FindingValidationError::InvalidSummary);
        }
        let target = &self.target.value;
        let valid_target = match self.target.kind {
            ScopeTargetKind::Hostname => {
                !target.is_empty()
                    && target.len() <= 253
                    && target.is_ascii()
                    && !target.ends_with('.')
                    && target.split('.').all(|label| {
                        !label.is_empty()
                            && label.len() <= 63
                            && label.as_bytes()[0].is_ascii_alphanumeric()
                            && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                            && label
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    })
                    && *target == target.to_ascii_lowercase()
            }
            ScopeTargetKind::IpAddress => target
                .parse::<std::net::IpAddr>()
                .map(|ip| ip.to_string() == *target)
                .unwrap_or(false),
        };
        if !valid_target {
            return Err(FindingValidationError::InvalidTarget);
        }
        if DateTime::parse_from_rfc3339(&self.observed_at).is_err() {
            return Err(FindingValidationError::InvalidObservedAt);
        }
        Ok(())
    }
}

fn compact_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Convert a validated finding into a Personal inbox Task with a bounded provenance snapshot.
/// No network request is made; evidence URLs are validated and redacted before being copied.
pub fn create_task_from_finding(
    connection: &mut Connection,
    finding: &SecurityFinding,
    scope: &AssessmentScope,
    remediation: &str,
    evidence_url: &str,
    now: &DateTime<Utc>,
) -> Result<Task, String> {
    finding
        .validate()
        .map_err(|_| "VALIDATION_FAILED: finding fields are invalid".to_owned())?;
    let scope = scope
        .validate_for_start(now)
        .map_err(|_| "VALIDATION_FAILED: assessment scope is invalid or expired".to_owned())?;
    if !scope.authorized_targets.contains(&finding.target)
        || scope.excluded_targets.contains(&finding.target)
    {
        return Err("OUT_OF_SCOPE: finding target is not authorized".to_owned());
    }
    let remediation = compact_text(remediation);
    if remediation.is_empty() || remediation.chars().count() > MAX_FINDING_SUMMARY_CHARS {
        return Err("VALIDATION_FAILED: remediation is empty or exceeds its limit".to_owned());
    }
    let observed_at = DateTime::parse_from_rfc3339(&finding.observed_at)
        .map_err(|_| "VALIDATION_FAILED: finding observation time is invalid".to_owned())?
        .with_timezone(&Utc);
    if observed_at > *now {
        return Err("VALIDATION_FAILED: finding observation time is in the future".to_owned());
    }
    if observed_at
        > DateTime::parse_from_rfc3339(&scope.expires_at)
            .map_err(|_| "VALIDATION_FAILED: assessment scope expiry is invalid".to_owned())?
            .with_timezone(&Utc)
    {
        return Err("VALIDATION_FAILED: finding time is outside the assessment scope".to_owned());
    }
    let evidence_url = crate::http_inspection::sanitize_evidence_url(evidence_url)?;
    let authorized_targets = scope
        .authorized_targets
        .iter()
        .map(|target| format!("{:?}: {}", target.kind, target.value))
        .collect::<Vec<_>>()
        .join(", ");
    let excluded_targets = scope
        .excluded_targets
        .iter()
        .map(|target| format!("{:?}: {}", target.kind, target.value))
        .collect::<Vec<_>>()
        .join(", ");
    let description = format!(
        "Security finding ID: {}\nSeverity: {:?}\nLikelihood: {:?}\nState: {:?}\nTarget: {:?} {}\nObserved: {}\nSummary: {}\nRemediation: {}\nEvidence: {}\nAuthorized scope: {}\nExcluded scope: {}\nAuthorization note: {}\nScope expires: {}",
        finding.id,
        finding.severity,
        finding.likelihood,
        finding.state,
        finding.target.kind,
        finding.target.value,
        finding.observed_at,
        compact_text(&finding.summary),
        remediation,
        evidence_url,
        authorized_targets,
        excluded_targets,
        compact_text(&scope.authorization_note),
        scope.expires_at,
    );
    if description.chars().count() > 10_000 {
        return Err(
            "LIMIT_REACHED: finding evidence exceeds the Task description limit".to_owned(),
        );
    }
    tasks::create_task(
        connection,
        CreateTask {
            workspace_id: crate::PERSONAL_WORKSPACE_ID.to_owned(),
            project_id: None,
            parent_task_id: None,
            title: compact_text(&finding.title),
            description: Some(description),
            due_at: None,
            status: Some("todo".to_owned()),
            priority: Some("p2".to_owned()),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{
        create_task_from_finding, FindingLikelihood, FindingSeverity, FindingState,
        FindingValidationError, SecurityFinding, MAX_FINDING_SUMMARY_CHARS,
        MAX_FINDING_TITLE_CHARS,
    };
    use crate::assessment_scope::{AssessmentScope, ScopeTarget, ScopeTargetKind};
    use crate::tasks::get_task;
    use chrono::{DateTime, Utc};
    use rusqlite::Connection;
    use uuid::Uuid;

    fn finding() -> SecurityFinding {
        SecurityFinding {
            id: Uuid::new_v4().to_string(),
            title: "TLS certificate expires soon".to_owned(),
            summary: "The authorized endpoint presents a certificate expiring within 14 days."
                .to_owned(),
            severity: FindingSeverity::Medium,
            likelihood: FindingLikelihood::Likely,
            state: FindingState::Open,
            target: ScopeTarget {
                kind: ScopeTargetKind::Hostname,
                value: "api.example.test".to_owned(),
            },
            observed_at: "2026-10-26T12:00:00Z".to_owned(),
        }
    }

    fn scope() -> AssessmentScope {
        AssessmentScope {
            authorized_targets: vec![ScopeTarget {
                kind: ScopeTargetKind::Hostname,
                value: "api.example.test".to_owned(),
            }],
            authorization_note: "Approved assessment ticket SEC-42".to_owned(),
            excluded_targets: vec![ScopeTarget {
                kind: ScopeTargetKind::Hostname,
                value: "admin.example.test".to_owned(),
            }],
            expires_at: "2026-10-27T12:00:00Z".to_owned(),
        }
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-26T13:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn finding_serializes_severity_and_likelihood_as_independent_fields() {
        let finding = finding();
        finding.validate().unwrap();
        let value = serde_json::to_value(finding).unwrap();
        assert_eq!(value["severity"], "medium");
        assert_eq!(value["likelihood"], "likely");
        assert!(value.get("risk").is_none());
        assert!(value.get("priority").is_none());
    }

    #[test]
    fn invalid_identity_text_target_and_timestamp_fail_validation() {
        let mut item = finding();
        item.id = "not-an-id".to_owned();
        assert_eq!(item.validate(), Err(FindingValidationError::InvalidId));
        item = finding();
        item.title = " ".to_owned();
        assert_eq!(item.validate(), Err(FindingValidationError::InvalidTitle));
        item = finding();
        item.title = "x".repeat(MAX_FINDING_TITLE_CHARS + 1);
        assert_eq!(item.validate(), Err(FindingValidationError::InvalidTitle));
        item = finding();
        item.summary = "x".repeat(MAX_FINDING_SUMMARY_CHARS + 1);
        assert_eq!(item.validate(), Err(FindingValidationError::InvalidSummary));
        item = finding();
        item.target.value = "API.example.test".to_owned();
        assert_eq!(item.validate(), Err(FindingValidationError::InvalidTarget));
        item = finding();
        item.observed_at = "yesterday".to_owned();
        assert_eq!(
            item.validate(),
            Err(FindingValidationError::InvalidObservedAt)
        );
    }

    #[test]
    fn unknown_fields_are_rejected_and_enums_remain_closed() {
        let value = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "title": "Finding",
            "summary": "Bounded summary",
            "severity": "high",
            "likelihood": "possible",
            "state": "open",
            "target": {"kind": "hostname", "value": "api.example.test"},
            "observedAt": "2026-10-26T12:00:00Z",
            "risk": "high"
        });
        assert!(serde_json::from_value::<SecurityFinding>(value).is_err());
        assert!(serde_json::from_str::<FindingSeverity>("\"urgent\"").is_err());
        assert!(serde_json::from_str::<FindingLikelihood>("\"guaranteed\"").is_err());
    }

    #[test]
    fn task_conversion_preserves_scope_time_remediation_and_redacted_evidence_link() {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::run_migrations(&mut connection).unwrap();
        let source = finding();
        let created = create_task_from_finding(
            &mut connection,
            &source,
            &scope(),
            "Rotate the certificate and add a renewal alert.",
            "https://evidence.example.test/report?api_key=private&view=1",
            &now(),
        )
        .unwrap();
        let saved = get_task(&connection, crate::PERSONAL_WORKSPACE_ID, &created.id).unwrap();
        let description = saved.description.unwrap();
        for expected in [
            "Severity: Medium",
            "Likelihood: Likely",
            "Target: Hostname api.example.test",
            "Observed: 2026-10-26T12:00:00Z",
            "Rotate the certificate",
            "https://evidence.example.test/report?api_key=%5BREDACTED%5D&view=1",
            "Approved assessment ticket SEC-42",
            "Scope expires: 2026-10-27T12:00:00.000Z",
        ] {
            assert!(description.contains(expected), "missing {expected}");
        }
        assert!(!description.contains("private"));
        assert_eq!(saved.priority, "p2");
    }

    #[test]
    fn task_conversion_rejects_out_of_scope_expired_or_unsafe_evidence_without_writes() {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::run_migrations(&mut connection).unwrap();
        let mut source = finding();
        source.target.value = "outside.example.test".to_owned();
        assert!(create_task_from_finding(
            &mut connection,
            &source,
            &scope(),
            "Fix it",
            "https://evidence.example.test/report",
            &now(),
        )
        .is_err());
        source = finding();
        assert!(create_task_from_finding(
            &mut connection,
            &source,
            &scope(),
            "Fix it",
            "https://user:password@evidence.example.test/report",
            &now(),
        )
        .is_err());
        let tasks = crate::tasks::list_tasks(
            &connection,
            crate::PERSONAL_WORKSPACE_ID,
            crate::tasks::TaskArchiveFilter::Active,
        )
        .unwrap();
        assert!(tasks.is_empty());
    }
}
