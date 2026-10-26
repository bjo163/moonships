//! Bounded security finding shape with independent severity and likelihood dimensions.
use crate::assessment_scope::{ScopeTarget, ScopeTargetKind};
use chrono::DateTime;
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

#[cfg(test)]
mod tests {
    use super::{
        FindingLikelihood, FindingSeverity, FindingState, FindingValidationError, SecurityFinding,
        MAX_FINDING_SUMMARY_CHARS, MAX_FINDING_TITLE_CHARS,
    };
    use crate::assessment_scope::{ScopeTarget, ScopeTargetKind};
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
}
