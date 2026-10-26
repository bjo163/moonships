//! Explicit, expiring authorization scope for future security assessments.
//! This module validates declarations only; it performs no discovery or network requests.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

pub const MAX_SCOPE_TARGETS: usize = 100;
pub const MAX_AUTHORIZATION_NOTE_CHARS: usize = 2_000;
pub const MAX_SCOPE_LIFETIME_DAYS: i64 = 365;
pub const MAX_HOSTNAME_CHARS: usize = 253;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeTargetKind {
    Hostname,
    IpAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopeTarget {
    pub kind: ScopeTargetKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentScope {
    pub authorized_targets: Vec<ScopeTarget>,
    pub authorization_note: String,
    pub excluded_targets: Vec<ScopeTarget>,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedAssessmentScope {
    pub authorized_targets: Vec<ScopeTarget>,
    pub authorization_note: String,
    pub excluded_targets: Vec<ScopeTarget>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeValidationError {
    EmptyScope,
    TooManyTargets,
    EmptyAuthorizationNote,
    AuthorizationNoteTooLong,
    InvalidExpiry,
    Expired,
    ExpiryTooFar,
    InvalidHostname,
    InvalidIpAddress,
    DuplicateTarget,
}

fn normalize_target(target: &ScopeTarget) -> Result<ScopeTarget, ScopeValidationError> {
    let value = target.value.trim();
    match target.kind {
        ScopeTargetKind::Hostname => {
            if value.is_empty()
                || value.len() > MAX_HOSTNAME_CHARS
                || value.ends_with('.')
                || !value.is_ascii()
            {
                return Err(ScopeValidationError::InvalidHostname);
            }
            let normalized = value.to_ascii_lowercase();
            let valid = normalized.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && label.as_bytes()[0].is_ascii_alphanumeric()
                    && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                    && label
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            });
            if !valid {
                return Err(ScopeValidationError::InvalidHostname);
            }
            Ok(ScopeTarget {
                kind: target.kind,
                value: normalized,
            })
        }
        ScopeTargetKind::IpAddress => {
            let address = value
                .parse::<IpAddr>()
                .map_err(|_| ScopeValidationError::InvalidIpAddress)?;
            Ok(ScopeTarget {
                kind: target.kind,
                value: address.to_string(),
            })
        }
    }
}

fn normalize_targets(targets: &[ScopeTarget]) -> Result<Vec<ScopeTarget>, ScopeValidationError> {
    if targets.len() > MAX_SCOPE_TARGETS {
        return Err(ScopeValidationError::TooManyTargets);
    }
    let normalized = targets
        .iter()
        .map(normalize_target)
        .collect::<Result<Vec<_>, _>>()?;
    for (index, target) in normalized.iter().enumerate() {
        if normalized[..index].contains(target) {
            return Err(ScopeValidationError::DuplicateTarget);
        }
    }
    Ok(normalized)
}

impl AssessmentScope {
    /// Normalize and validate the declared scope immediately before any future assessment start.
    pub fn validate_for_start(
        &self,
        now: &DateTime<Utc>,
    ) -> Result<ValidatedAssessmentScope, ScopeValidationError> {
        if self.authorized_targets.is_empty() {
            return Err(ScopeValidationError::EmptyScope);
        }
        if self.authorization_note.trim().is_empty() {
            return Err(ScopeValidationError::EmptyAuthorizationNote);
        }
        if self.authorization_note.chars().count() > MAX_AUTHORIZATION_NOTE_CHARS {
            return Err(ScopeValidationError::AuthorizationNoteTooLong);
        }
        let expires_at = DateTime::parse_from_rfc3339(&self.expires_at)
            .map_err(|_| ScopeValidationError::InvalidExpiry)?
            .with_timezone(&Utc);
        if expires_at <= *now {
            return Err(ScopeValidationError::Expired);
        }
        if expires_at > *now + chrono::Duration::days(MAX_SCOPE_LIFETIME_DAYS) {
            return Err(ScopeValidationError::ExpiryTooFar);
        }
        let authorized_targets = normalize_targets(&self.authorized_targets)?;
        let excluded_targets = normalize_targets(&self.excluded_targets)?;
        Ok(ValidatedAssessmentScope {
            authorized_targets,
            authorization_note: self.authorization_note.trim().to_owned(),
            excluded_targets,
            expires_at: expires_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AssessmentScope, ScopeTarget, ScopeTargetKind, ScopeValidationError, MAX_SCOPE_TARGETS,
    };
    use chrono::{DateTime, Utc};

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn scope(targets: Vec<ScopeTarget>) -> AssessmentScope {
        AssessmentScope {
            authorized_targets: targets,
            authorization_note: "Approved internal lab assessment, ticket SEC-42".to_owned(),
            excluded_targets: vec![ScopeTarget {
                kind: ScopeTargetKind::Hostname,
                value: "admin.example.test".to_owned(),
            }],
            expires_at: "2026-10-27T05:00:00Z".to_owned(),
        }
    }

    #[test]
    fn cannot_start_with_empty_scope_or_missing_authorization_note() {
        let mut empty = scope(vec![]);
        assert_eq!(
            empty.validate_for_start(&now()),
            Err(ScopeValidationError::EmptyScope)
        );
        empty.authorized_targets.push(ScopeTarget {
            kind: ScopeTargetKind::Hostname,
            value: "example.test".to_owned(),
        });
        empty.authorization_note = "  ".to_owned();
        assert_eq!(
            empty.validate_for_start(&now()),
            Err(ScopeValidationError::EmptyAuthorizationNote)
        );
    }

    #[test]
    fn normalizes_explicit_host_and_ip_targets_and_preserves_exclusions() {
        let validated = scope(vec![
            ScopeTarget {
                kind: ScopeTargetKind::Hostname,
                value: "API.Example.Test".to_owned(),
            },
            ScopeTarget {
                kind: ScopeTargetKind::IpAddress,
                value: "2001:0db8::1".to_owned(),
            },
        ])
        .validate_for_start(&now())
        .unwrap();
        assert_eq!(validated.authorized_targets[0].value, "api.example.test");
        assert_eq!(validated.authorized_targets[1].value, "2001:db8::1");
        assert_eq!(validated.excluded_targets[0].value, "admin.example.test");
        assert_eq!(validated.expires_at, "2026-10-27T05:00:00.000Z");
    }

    #[test]
    fn rejects_url_wildcard_bad_hosts_duplicates_and_invalid_ips() {
        for value in [
            "https://example.test",
            "*.example.test",
            "host:443",
            "bad..test",
        ] {
            assert_eq!(
                scope(vec![ScopeTarget {
                    kind: ScopeTargetKind::Hostname,
                    value: value.to_owned(),
                }])
                .validate_for_start(&now()),
                Err(ScopeValidationError::InvalidHostname)
            );
        }
        assert_eq!(
            scope(vec![ScopeTarget {
                kind: ScopeTargetKind::IpAddress,
                value: "999.1.1.1".to_owned(),
            }])
            .validate_for_start(&now()),
            Err(ScopeValidationError::InvalidIpAddress)
        );
        let host = ScopeTarget {
            kind: ScopeTargetKind::Hostname,
            value: "API.example.test".to_owned(),
        };
        assert_eq!(
            scope(vec![
                host.clone(),
                ScopeTarget {
                    value: "api.example.test".to_owned(),
                    ..host
                }
            ])
            .validate_for_start(&now()),
            Err(ScopeValidationError::DuplicateTarget)
        );
    }

    #[test]
    fn expiry_and_scope_size_are_bounded() {
        let mut value = scope(vec![ScopeTarget {
            kind: ScopeTargetKind::IpAddress,
            value: "192.0.2.4".to_owned(),
        }]);
        value.expires_at = "2026-10-26T05:00:00Z".to_owned();
        assert_eq!(
            value.validate_for_start(&now()),
            Err(ScopeValidationError::Expired)
        );
        value.expires_at = "2028-10-26T05:00:00Z".to_owned();
        assert_eq!(
            value.validate_for_start(&now()),
            Err(ScopeValidationError::ExpiryTooFar)
        );
        value.expires_at = "invalid".to_owned();
        assert_eq!(
            value.validate_for_start(&now()),
            Err(ScopeValidationError::InvalidExpiry)
        );
        value.expires_at = "2026-10-27T05:00:00Z".to_owned();
        value.authorized_targets = (0..=MAX_SCOPE_TARGETS)
            .map(|index| ScopeTarget {
                kind: ScopeTargetKind::Hostname,
                value: format!("host{index}.example.test"),
            })
            .collect();
        assert_eq!(
            value.validate_for_start(&now()),
            Err(ScopeValidationError::TooManyTargets)
        );
    }

    #[test]
    fn strict_json_rejects_unreviewed_fields() {
        let result = serde_json::from_str::<AssessmentScope>(
            r#"{"authorizedTargets":[],"authorizationNote":"ok","excludedTargets":[],"expiresAt":"2026-10-27T05:00:00Z","endpoint":"https://example.test"}"#,
        );
        assert!(result.is_err());
    }
}
