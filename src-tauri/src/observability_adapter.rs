//! Data-only observability source metadata and freshness contract.
//! No provider endpoint, credential, query, transport or polling behavior is defined here.
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

pub const MAX_PROVIDER_ID_CHARS: usize = 64;
pub const MAX_PROVIDER_NAME_CHARS: usize = 120;
pub const MAX_FRESHNESS_THRESHOLD_SECONDS: u32 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilitySignalKind {
    Logs,
    Metrics,
    Traces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilityProviderStatus {
    Ready,
    Degraded,
    Unavailable,
    Disabled,
    Unsupported,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilityFreshness {
    Fresh,
    Stale,
    Unknown,
    Degraded,
    Unavailable,
    Disabled,
    Unsupported,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservabilityProviderManifest {
    pub id: String,
    pub display_name: String,
    pub signal_kinds: Vec<ObservabilitySignalKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservabilityProviderHealth {
    pub provider_id: String,
    pub status: ObservabilityProviderStatus,
    pub last_success_at: Option<String>,
    pub freshness_threshold_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservabilityProviderView {
    pub provider: ObservabilityProviderManifest,
    pub status: ObservabilityProviderStatus,
    pub freshness: ObservabilityFreshness,
    pub last_success_at: Option<String>,
    pub freshness_threshold_seconds: u32,
    /// True only when provider status is ready and the saved success time is within threshold.
    pub is_current: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservabilityValidationError {
    InvalidId,
    InvalidDisplayName,
    MissingSignalKind,
    DuplicateSignalKind,
    InvalidProviderId,
    InvalidFreshnessThreshold,
}

impl ObservabilityProviderManifest {
    pub fn validate(&self) -> Result<(), ObservabilityValidationError> {
        let id = self.id.trim();
        if id.is_empty()
            || self.id.chars().count() > MAX_PROVIDER_ID_CHARS
            || !id.chars().all(|ch| {
                ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '-' | '_')
            })
        {
            return Err(ObservabilityValidationError::InvalidId);
        }
        let name = self.display_name.trim();
        if name.is_empty() || name.chars().count() > MAX_PROVIDER_NAME_CHARS {
            return Err(ObservabilityValidationError::InvalidDisplayName);
        }
        if self.signal_kinds.is_empty() {
            return Err(ObservabilityValidationError::MissingSignalKind);
        }
        for (index, kind) in self.signal_kinds.iter().enumerate() {
            if self.signal_kinds[..index].contains(kind) {
                return Err(ObservabilityValidationError::DuplicateSignalKind);
            }
        }
        Ok(())
    }
}

impl ObservabilityProviderHealth {
    pub fn evaluate(
        &self,
        provider: &ObservabilityProviderManifest,
        now: &DateTime<Utc>,
    ) -> Result<ObservabilityProviderView, ObservabilityValidationError> {
        provider.validate()?;
        if self.provider_id != provider.id {
            return Err(ObservabilityValidationError::InvalidProviderId);
        }
        if !(1..=MAX_FRESHNESS_THRESHOLD_SECONDS).contains(&self.freshness_threshold_seconds) {
            return Err(ObservabilityValidationError::InvalidFreshnessThreshold);
        }
        let parsed = self
            .last_success_at
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc));
        let last_success_at = parsed
            .as_ref()
            .map(|time| time.to_rfc3339_opts(SecondsFormat::Millis, true));
        let freshness = match self.status {
            ObservabilityProviderStatus::Degraded => ObservabilityFreshness::Degraded,
            ObservabilityProviderStatus::Unavailable => ObservabilityFreshness::Unavailable,
            ObservabilityProviderStatus::Disabled => ObservabilityFreshness::Disabled,
            ObservabilityProviderStatus::Unsupported => ObservabilityFreshness::Unsupported,
            ObservabilityProviderStatus::Error => ObservabilityFreshness::Error,
            ObservabilityProviderStatus::Ready => match parsed {
                None => ObservabilityFreshness::Unknown,
                Some(last_success) => {
                    let age = now.signed_duration_since(last_success);
                    if age.num_milliseconds() < 0 {
                        ObservabilityFreshness::Unknown
                    } else if age
                        > chrono::Duration::seconds(self.freshness_threshold_seconds as i64)
                    {
                        ObservabilityFreshness::Stale
                    } else {
                        ObservabilityFreshness::Fresh
                    }
                }
            },
        };
        Ok(ObservabilityProviderView {
            provider: provider.clone(),
            status: self.status,
            freshness,
            last_success_at,
            freshness_threshold_seconds: self.freshness_threshold_seconds,
            is_current: freshness == ObservabilityFreshness::Fresh,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider() -> ObservabilityProviderManifest {
        ObservabilityProviderManifest {
            id: "local.fixture".to_owned(),
            display_name: "Local fixture".to_owned(),
            signal_kinds: vec![
                ObservabilitySignalKind::Logs,
                ObservabilitySignalKind::Metrics,
            ],
        }
    }

    fn health(
        status: ObservabilityProviderStatus,
        last_success_at: Option<&str>,
    ) -> ObservabilityProviderHealth {
        ObservabilityProviderHealth {
            provider_id: "local.fixture".to_owned(),
            status,
            last_success_at: last_success_at.map(str::to_owned),
            freshness_threshold_seconds: 3600,
        }
    }

    #[test]
    fn fresh_stale_missing_invalid_and_future_times_are_distinguished() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .to_utc();
        for (timestamp, expected) in [
            (Some("2026-10-26T04:30:00Z"), ObservabilityFreshness::Fresh),
            (Some("2026-10-26T03:59:59Z"), ObservabilityFreshness::Stale),
            (None, ObservabilityFreshness::Unknown),
            (Some("invalid"), ObservabilityFreshness::Unknown),
            (
                Some("2026-10-26T05:00:01Z"),
                ObservabilityFreshness::Unknown,
            ),
        ] {
            let view = health(ObservabilityProviderStatus::Ready, timestamp)
                .evaluate(&provider(), &now)
                .unwrap();
            assert_eq!(view.freshness, expected);
            assert_eq!(view.is_current, expected == ObservabilityFreshness::Fresh);
        }
    }

    #[test]
    fn non_ready_provider_states_override_a_recent_success() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .to_utc();
        for (status, expected) in [
            (
                ObservabilityProviderStatus::Degraded,
                ObservabilityFreshness::Degraded,
            ),
            (
                ObservabilityProviderStatus::Unavailable,
                ObservabilityFreshness::Unavailable,
            ),
            (
                ObservabilityProviderStatus::Disabled,
                ObservabilityFreshness::Disabled,
            ),
            (
                ObservabilityProviderStatus::Unsupported,
                ObservabilityFreshness::Unsupported,
            ),
            (
                ObservabilityProviderStatus::Error,
                ObservabilityFreshness::Error,
            ),
        ] {
            let view = health(status, Some("2026-10-26T04:59:00Z"))
                .evaluate(&provider(), &now)
                .unwrap();
            assert_eq!(view.freshness, expected);
            assert!(!view.is_current);
        }
    }

    #[test]
    fn manifest_and_health_reject_unbounded_or_inconsistent_metadata() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .to_utc();
        let mut manifest = provider();
        manifest.id = "../provider".to_owned();
        assert_eq!(
            manifest.validate(),
            Err(ObservabilityValidationError::InvalidId)
        );
        manifest = provider();
        manifest.signal_kinds.push(ObservabilitySignalKind::Logs);
        assert_eq!(
            manifest.validate(),
            Err(ObservabilityValidationError::DuplicateSignalKind)
        );
        let mut state = health(ObservabilityProviderStatus::Ready, None);
        state.provider_id = "other".to_owned();
        assert_eq!(
            state.evaluate(&provider(), &now),
            Err(ObservabilityValidationError::InvalidProviderId)
        );
        state = health(ObservabilityProviderStatus::Ready, None);
        state.freshness_threshold_seconds = 0;
        assert_eq!(
            state.evaluate(&provider(), &now),
            Err(ObservabilityValidationError::InvalidFreshnessThreshold)
        );
    }

    #[test]
    fn contract_json_has_status_and_freshness_without_endpoints_or_credentials() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .to_utc();
        let view = health(
            ObservabilityProviderStatus::Ready,
            Some("2026-10-26T04:59:00Z"),
        )
        .evaluate(&provider(), &now)
        .unwrap();
        let json = serde_json::to_value(view).unwrap();
        assert_eq!(json["status"], "ready");
        assert_eq!(json["freshness"], "fresh");
        assert_eq!(json["isCurrent"], true);
        assert!(json.get("endpoint").is_none());
        assert!(json.get("credential").is_none());
    }
}
