//! Declarative, fail-closed contract for externally configured information sources.
use crate::network_diagnostics::{
    validate_network_diagnostic_request, NetworkDiagnosticKind, NetworkDiagnosticRequest,
    DEFAULT_TIMEOUT_MS,
};
use serde::{de, Deserialize, Deserializer, Serialize};

pub const MIN_POLL_INTERVAL_SECONDS: u64 = 60;
pub const MAX_POLL_INTERVAL_SECONDS: u64 = 30 * 24 * 60 * 60;
pub const MIN_REQUEST_TIMEOUT_MS: u32 = 100;
pub const MAX_REQUEST_TIMEOUT_MS: u32 = 30_000;
pub const MAX_RESPONSE_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_ITEMS_PER_RUN: u16 = 500;
pub const MAX_CONNECTOR_ID_BYTES: usize = 64;
pub const MAX_CONNECTOR_NAME_CHARS: usize = 120;
pub const MAX_TERMS_NOTE_CHARS: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind {
    Rss,
    DocumentedApi,
    AuthorizedExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessMethod {
    PublicRss,
    DocumentedApi,
    UserAuthorizedExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TermsReview {
    Unreviewed,
    Reviewed,
    Restricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorHealth {
    NotConfigured,
    Ready,
    Healthy,
    Stale,
    RateLimited,
    Blocked,
    Unavailable,
    Error,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorDefinition {
    pub id: String,
    pub kind: ConnectorKind,
    pub display_name: String,
    pub source_url: String,
    pub access_method: AccessMethod,
    pub terms_review: TermsReview,
    pub terms_url: Option<String>,
    pub terms_notes: String,
    pub minimum_poll_interval_seconds: u64,
    pub request_timeout_ms: u32,
    pub maximum_response_bytes: u64,
    pub maximum_items_per_run: u16,
    pub freshness_threshold_seconds: u64,
    enabled: bool,
    pub health: ConnectorHealth,
    pub last_attempt_at: Option<String>,
    pub last_success_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConnectorDefinitionInput {
    id: String,
    kind: ConnectorKind,
    display_name: String,
    source_url: String,
    access_method: AccessMethod,
    terms_review: TermsReview,
    terms_url: Option<String>,
    terms_notes: String,
    minimum_poll_interval_seconds: u64,
    request_timeout_ms: u32,
    maximum_response_bytes: u64,
    maximum_items_per_run: u16,
    freshness_threshold_seconds: u64,
    enabled: bool,
    health: ConnectorHealth,
    last_attempt_at: Option<String>,
    last_success_at: Option<String>,
}

impl<'de> Deserialize<'de> for ConnectorDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = ConnectorDefinitionInput::deserialize(deserializer)?;
        let definition = Self {
            id: input.id,
            kind: input.kind,
            display_name: input.display_name,
            source_url: input.source_url,
            access_method: input.access_method,
            terms_review: input.terms_review,
            terms_url: input.terms_url,
            terms_notes: input.terms_notes,
            minimum_poll_interval_seconds: input.minimum_poll_interval_seconds,
            request_timeout_ms: input.request_timeout_ms,
            maximum_response_bytes: input.maximum_response_bytes,
            maximum_items_per_run: input.maximum_items_per_run,
            freshness_threshold_seconds: input.freshness_threshold_seconds,
            enabled: input.enabled,
            health: input.health,
            last_attempt_at: input.last_attempt_at,
            last_success_at: input.last_success_at,
        };
        if definition.enabled {
            definition.validate().map_err(|error| {
                de::Error::custom(format!("connector cannot be enabled: {error:?}"))
            })?;
        }
        Ok(definition)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorValidationError {
    InvalidId,
    InvalidName,
    InvalidSourceUrl,
    InvalidTerms,
    MethodMismatch,
    InvalidPollInterval,
    InvalidTimeout,
    InvalidResponseLimit,
    InvalidItemLimit,
    InvalidFreshness,
    TermsNotReviewed,
    TermsRestricted,
    HealthMismatch,
}

impl ConnectorDefinition {
    pub fn user_configured_rss(
        id: String,
        display_name: String,
        source_url: String,
        terms_url: Option<String>,
        terms_notes: String,
        terms_review: TermsReview,
        minimum_poll_interval_seconds: u64,
    ) -> Self {
        Self {
            id,
            kind: ConnectorKind::Rss,
            display_name,
            source_url,
            access_method: AccessMethod::PublicRss,
            terms_review,
            terms_url,
            terms_notes,
            minimum_poll_interval_seconds,
            request_timeout_ms: 5_000,
            maximum_response_bytes: 512_000,
            maximum_items_per_run: 100,
            freshness_threshold_seconds: 3_600.max(minimum_poll_interval_seconds),
            enabled: false,
            health: ConnectorHealth::NotConfigured,
            last_attempt_at: None,
            last_success_at: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.health = ConnectorHealth::Disabled;
    }

    pub fn validate(&self) -> Result<(), ConnectorValidationError> {
        if !valid_id(&self.id) {
            return Err(ConnectorValidationError::InvalidId);
        }
        if self.display_name.trim().is_empty()
            || self.display_name.chars().count() > MAX_CONNECTOR_NAME_CHARS
        {
            return Err(ConnectorValidationError::InvalidName);
        }
        if !matches!(
            (self.kind, self.access_method),
            (ConnectorKind::Rss, AccessMethod::PublicRss)
                | (ConnectorKind::DocumentedApi, AccessMethod::DocumentedApi)
                | (
                    ConnectorKind::AuthorizedExport,
                    AccessMethod::UserAuthorizedExport
                )
        ) {
            return Err(ConnectorValidationError::MethodMismatch);
        }
        if !valid_https_url(&self.source_url)
            || self
                .terms_url
                .as_deref()
                .is_some_and(|url| !valid_https_url(url))
        {
            return Err(ConnectorValidationError::InvalidSourceUrl);
        }
        if self.terms_notes.trim().is_empty()
            || self.terms_notes.chars().count() > MAX_TERMS_NOTE_CHARS
        {
            return Err(ConnectorValidationError::InvalidTerms);
        }
        if !(MIN_POLL_INTERVAL_SECONDS..=MAX_POLL_INTERVAL_SECONDS)
            .contains(&self.minimum_poll_interval_seconds)
        {
            return Err(ConnectorValidationError::InvalidPollInterval);
        }
        if !(MIN_REQUEST_TIMEOUT_MS..=MAX_REQUEST_TIMEOUT_MS).contains(&self.request_timeout_ms) {
            return Err(ConnectorValidationError::InvalidTimeout);
        }
        if self.maximum_response_bytes == 0 || self.maximum_response_bytes > MAX_RESPONSE_BYTES {
            return Err(ConnectorValidationError::InvalidResponseLimit);
        }
        if self.maximum_items_per_run == 0 || self.maximum_items_per_run > MAX_ITEMS_PER_RUN {
            return Err(ConnectorValidationError::InvalidItemLimit);
        }
        if self.freshness_threshold_seconds < self.minimum_poll_interval_seconds
            || self.freshness_threshold_seconds > MAX_POLL_INTERVAL_SECONDS
        {
            return Err(ConnectorValidationError::InvalidFreshness);
        }
        if self.enabled && self.terms_review != TermsReview::Reviewed {
            return Err(if self.terms_review == TermsReview::Restricted {
                ConnectorValidationError::TermsRestricted
            } else {
                ConnectorValidationError::TermsNotReviewed
            });
        }
        if self.enabled
            && matches!(
                self.health,
                ConnectorHealth::Disabled
                    | ConnectorHealth::Blocked
                    | ConnectorHealth::NotConfigured
            )
        {
            return Err(ConnectorValidationError::HealthMismatch);
        }
        Ok(())
    }

    pub fn enable(&mut self) -> Result<(), ConnectorValidationError> {
        if self.health == ConnectorHealth::Blocked {
            return Err(ConnectorValidationError::HealthMismatch);
        }
        let mut candidate = self.clone();
        candidate.enabled = true;
        if matches!(
            candidate.health,
            ConnectorHealth::NotConfigured | ConnectorHealth::Disabled
        ) {
            candidate.health = ConnectorHealth::Ready;
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }
}

pub(crate) fn valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= MAX_CONNECTOR_ID_BYTES
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && (bytes[bytes.len() - 1].is_ascii_lowercase() || bytes[bytes.len() - 1].is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

pub(crate) fn valid_https_url(value: &str) -> bool {
    if value.len() > 2_048 || value.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return false;
    }
    let Some(rest) = value.strip_prefix("https://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.is_empty() || authority.contains('@') || authority.contains(':') {
        return false;
    }
    validate_network_diagnostic_request(NetworkDiagnosticRequest {
        kind: NetworkDiagnosticKind::DnsLookup,
        target_host: authority.to_string(),
        port: None,
        timeout_ms: DEFAULT_TIMEOUT_MS,
    })
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    fn connector() -> ConnectorDefinition {
        ConnectorDefinition {
            id: "example.status-feed".to_string(),
            kind: ConnectorKind::Rss,
            display_name: "Example status feed".to_string(),
            source_url: "https://status.example/feed.xml".to_string(),
            access_method: AccessMethod::PublicRss,
            terms_review: TermsReview::Reviewed,
            terms_url: Some("https://status.example/terms".to_string()),
            terms_notes: "Public RSS feed; respect the source's published cadence.".to_string(),
            minimum_poll_interval_seconds: 900,
            request_timeout_ms: 5_000,
            maximum_response_bytes: 512_000,
            maximum_items_per_run: 100,
            freshness_threshold_seconds: 3_600,
            enabled: false,
            health: ConnectorHealth::NotConfigured,
            last_attempt_at: None,
            last_success_at: None,
        }
    }

    #[test]
    fn reviewed_source_with_explicit_limits_can_be_enabled() {
        let mut definition = connector();
        definition.enable().unwrap();
        assert!(definition.is_enabled());
        assert_eq!(definition.health, ConnectorHealth::Ready);
        assert_eq!(
            from_str::<ConnectorDefinition>(&to_string(&definition).unwrap()).unwrap(),
            definition
        );
    }

    #[test]
    fn enable_fails_closed_without_a_declared_id_or_poll_limits() {
        let mut missing_id = connector();
        missing_id.id.clear();
        assert_eq!(
            missing_id.enable(),
            Err(ConnectorValidationError::InvalidId)
        );
        assert!(!missing_id.is_enabled());

        let mut missing_poll = connector();
        missing_poll.minimum_poll_interval_seconds = 0;
        assert_eq!(
            missing_poll.enable(),
            Err(ConnectorValidationError::InvalidPollInterval)
        );
        assert!(!missing_poll.is_enabled());
    }

    #[test]
    fn deserialization_cannot_bypass_enabled_connector_validation() {
        let mut invalid = connector();
        invalid.id.clear();
        invalid.enabled = true;
        invalid.health = ConnectorHealth::Ready;
        let serialized = to_string(&invalid).unwrap();
        assert!(from_str::<ConnectorDefinition>(&serialized).is_err());

        let mut missing_limit = connector();
        missing_limit.enabled = true;
        missing_limit.health = ConnectorHealth::Ready;
        missing_limit.minimum_poll_interval_seconds = 0;
        assert!(from_str::<ConnectorDefinition>(&to_string(&missing_limit).unwrap()).is_err());
    }

    #[test]
    fn unreviewed_or_restricted_terms_and_unbounded_limits_cannot_be_enabled() {
        let mut unreviewed = connector();
        unreviewed.terms_review = TermsReview::Unreviewed;
        assert_eq!(
            unreviewed.enable(),
            Err(ConnectorValidationError::TermsNotReviewed)
        );
        let mut restricted = connector();
        restricted.terms_review = TermsReview::Restricted;
        assert_eq!(
            restricted.enable(),
            Err(ConnectorValidationError::TermsRestricted)
        );
        let mut oversized = connector();
        oversized.maximum_response_bytes = MAX_RESPONSE_BYTES + 1;
        assert_eq!(
            oversized.enable(),
            Err(ConnectorValidationError::InvalidResponseLimit)
        );
    }

    #[test]
    fn rejects_non_https_urls_and_embedded_credentials_without_fetching() {
        let mut definition = connector();
        definition.source_url = "http://status.example/feed.xml".to_string();
        assert_eq!(
            definition.validate(),
            Err(ConnectorValidationError::InvalidSourceUrl)
        );
        definition.source_url = "https://user:pass@status.example/feed.xml".to_string();
        assert_eq!(
            definition.validate(),
            Err(ConnectorValidationError::InvalidSourceUrl)
        );
        assert_eq!(definition.health, ConnectorHealth::NotConfigured);
    }

    #[test]
    fn source_kind_must_match_its_declared_access_method() {
        let mut definition = connector();
        definition.access_method = AccessMethod::DocumentedApi;
        assert_eq!(
            definition.validate(),
            Err(ConnectorValidationError::MethodMismatch)
        );
    }

    #[test]
    fn explicitly_blocked_source_cannot_be_reenabled_by_resetting_health() {
        let mut definition = connector();
        definition.health = ConnectorHealth::Blocked;
        assert_eq!(
            definition.enable(),
            Err(ConnectorValidationError::HealthMismatch)
        );
        assert_eq!(definition.health, ConnectorHealth::Blocked);
        assert!(!definition.is_enabled());
    }
}
