//! Display-safe freshness state derived from source health and last successful fetch time.
use crate::connectors::{ConnectorDefinition, ConnectorHealth, MAX_POLL_INTERVAL_SECONDS};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorFreshness {
    Fresh,
    Stale,
    Unknown,
    Blocked,
    RateLimited,
    Unavailable,
    Error,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectorFreshnessView {
    pub status: ConnectorFreshness,
    pub last_success_at: Option<String>,
    /// Only `Fresh` sources may be presented as current/live by a UI.
    pub is_live: bool,
}

impl ConnectorFreshnessView {
    pub fn evaluate(connector: &ConnectorDefinition, now: &DateTime<Utc>) -> Self {
        let parsed_success = connector
            .last_success_at
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc));
        let last_success_at = parsed_success
            .as_ref()
            .map(|value| value.to_rfc3339_opts(SecondsFormat::Millis, true));

        let status = if !connector.is_enabled() {
            ConnectorFreshness::Disabled
        } else {
            match connector.health {
                ConnectorHealth::Blocked => ConnectorFreshness::Blocked,
                ConnectorHealth::RateLimited => ConnectorFreshness::RateLimited,
                ConnectorHealth::Unavailable => ConnectorFreshness::Unavailable,
                ConnectorHealth::Error => ConnectorFreshness::Error,
                ConnectorHealth::Stale => ConnectorFreshness::Stale,
                ConnectorHealth::Disabled => ConnectorFreshness::Disabled,
                ConnectorHealth::NotConfigured => ConnectorFreshness::Unknown,
                _ => match parsed_success {
                    None => {
                        if connector.last_success_at.is_some() {
                            ConnectorFreshness::Error
                        } else {
                            ConnectorFreshness::Unknown
                        }
                    }
                    Some(success_at) => {
                        let age = now.signed_duration_since(success_at);
                        if age.num_milliseconds() < 0 {
                            ConnectorFreshness::Unknown
                        } else if connector.freshness_threshold_seconds > MAX_POLL_INTERVAL_SECONDS
                        {
                            ConnectorFreshness::Error
                        } else if age
                            > chrono::Duration::seconds(
                                connector.freshness_threshold_seconds as i64,
                            )
                        {
                            ConnectorFreshness::Stale
                        } else {
                            ConnectorFreshness::Fresh
                        }
                    }
                },
            }
        };
        Self {
            status,
            last_success_at,
            is_live: status == ConnectorFreshness::Fresh,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn connector(
        enabled: bool,
        health: ConnectorHealth,
        last_success_at: Option<&str>,
    ) -> ConnectorDefinition {
        let value = serde_json::json!({
            "id": "example.status-feed",
            "kind": "rss",
            "displayName": "Example status feed",
            "sourceUrl": "https://status.example/feed.xml",
            "accessMethod": "public_rss",
            "termsReview": "reviewed",
            "termsUrl": "https://status.example/terms",
            "termsNotes": "Public feed; follow published limits.",
            "minimumPollIntervalSeconds": 900,
            "requestTimeoutMs": 5000,
            "maximumResponseBytes": 512000,
            "maximumItemsPerRun": 100,
            "freshnessThresholdSeconds": 3600,
            "enabled": enabled,
            "health": "ready",
            "lastAttemptAt": null,
            "lastSuccessAt": last_success_at
        });
        let mut definition: ConnectorDefinition = serde_json::from_value(value).unwrap();
        definition.health = health;
        definition.last_success_at = last_success_at.map(str::to_owned);
        definition
    }

    #[test]
    fn freshness_boundary_and_last_success_timestamp_are_explicit() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let recent = connector(true, ConnectorHealth::Healthy, Some("2026-10-26T04:00:00Z"));
        let fresh = ConnectorFreshnessView::evaluate(&recent, &now);
        assert_eq!(fresh.status, ConnectorFreshness::Fresh);
        assert!(fresh.is_live);
        assert_eq!(
            fresh.last_success_at.as_deref(),
            Some("2026-10-26T04:00:00.000Z")
        );

        let stale = connector(true, ConnectorHealth::Healthy, Some("2026-10-26T03:59:59Z"));
        let stale = ConnectorFreshnessView::evaluate(&stale, &now);
        assert_eq!(stale.status, ConnectorFreshness::Stale);
        assert!(!stale.is_live);
    }

    #[test]
    fn absent_invalid_or_future_success_times_never_claim_live_data() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        for source in [
            connector(true, ConnectorHealth::Ready, None),
            connector(true, ConnectorHealth::Healthy, Some("not-a-time")),
            connector(true, ConnectorHealth::Healthy, Some("2026-10-26T05:00:01Z")),
        ] {
            let view = ConnectorFreshnessView::evaluate(&source, &now);
            assert_ne!(view.status, ConnectorFreshness::Fresh);
            assert!(!view.is_live);
        }
    }

    #[test]
    fn blocked_rate_limited_error_unavailable_and_disabled_states_override_recent_success() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        for (enabled, health, expected) in [
            (true, ConnectorHealth::Blocked, ConnectorFreshness::Blocked),
            (
                true,
                ConnectorHealth::RateLimited,
                ConnectorFreshness::RateLimited,
            ),
            (true, ConnectorHealth::Error, ConnectorFreshness::Error),
            (
                true,
                ConnectorHealth::Unavailable,
                ConnectorFreshness::Unavailable,
            ),
            (
                false,
                ConnectorHealth::Disabled,
                ConnectorFreshness::Disabled,
            ),
        ] {
            let source = connector(enabled, health, Some("2026-10-26T04:59:00Z"));
            let view = ConnectorFreshnessView::evaluate(&source, &now);
            assert_eq!(view.status, expected);
            assert!(!view.is_live);
        }
    }

    #[test]
    fn dto_uses_stable_snake_case_status_and_camel_case_fields() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T05:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let view = ConnectorFreshnessView::evaluate(
            &connector(true, ConnectorHealth::Healthy, Some("2026-10-26T04:59:00Z")),
            &now,
        );
        let json = serde_json::to_value(view).unwrap();
        assert_eq!(json["status"], "fresh");
        assert_eq!(json["isLive"], true);
        assert!(json.get("last_success_at").is_none());
    }
}
