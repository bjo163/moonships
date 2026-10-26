//! Bounded normalized intelligence item schema. Items remain untrusted source data.
use crate::connectors::{valid_https_url, valid_id};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_ITEM_TITLE_CHARS: usize = 500;
pub const MAX_ITEM_SUMMARY_CHARS: usize = 4_000;
pub const MAX_ITEM_CATEGORY_CHARS: usize = 64;
pub const MAX_EXTERNAL_ID_CHARS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemFreshness {
    Fresh,
    Stale,
    Unknown,
    Blocked,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemVerification {
    Unverified,
    SourceAttributed,
    IndependentlyVerified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IntelligenceItem {
    pub id: String,
    pub source_id: String,
    pub source_url: String,
    pub external_id: Option<String>,
    pub canonical_url: String,
    pub title: String,
    pub summary: Option<String>,
    pub published_at: Option<String>,
    pub observed_at: String,
    pub event_time: Option<String>,
    pub category: String,
    pub content_hash: Option<String>,
    pub freshness_state: ItemFreshness,
    pub verification_state: ItemVerification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemValidationError {
    InvalidId,
    InvalidSourceId,
    InvalidSourceUrl,
    InvalidCanonicalUrl,
    InvalidExternalId,
    InvalidTitle,
    InvalidSummary,
    InvalidObservedAt,
    InvalidPublishedAt,
    InvalidEventTime,
    InvalidCategory,
    InvalidContentHash,
}

fn normalize_timestamp(value: &str) -> Result<String, ()> {
    let parsed = DateTime::parse_from_rfc3339(value).map_err(|_| ())?;
    Ok(parsed
        .with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Millis, true))
}

impl IntelligenceItem {
    /// Validate untrusted connector output and normalize only the timestamp representation.
    /// Missing publication or event times remain missing; they are never inferred.
    pub fn validate(&mut self) -> Result<(), ItemValidationError> {
        if Uuid::parse_str(&self.id)
            .map(|id| id.to_string() != self.id)
            .unwrap_or(true)
        {
            return Err(ItemValidationError::InvalidId);
        }
        if !valid_id(&self.source_id) {
            return Err(ItemValidationError::InvalidSourceId);
        }
        if !valid_https_url(&self.source_url) {
            return Err(ItemValidationError::InvalidSourceUrl);
        }
        if !valid_https_url(&self.canonical_url) {
            return Err(ItemValidationError::InvalidCanonicalUrl);
        }
        if self.external_id.as_ref().is_some_and(|value| {
            value.trim().is_empty() || value.chars().count() > MAX_EXTERNAL_ID_CHARS
        }) {
            return Err(ItemValidationError::InvalidExternalId);
        }
        if self.title.trim().is_empty() || self.title.chars().count() > MAX_ITEM_TITLE_CHARS {
            return Err(ItemValidationError::InvalidTitle);
        }
        if self
            .summary
            .as_ref()
            .is_some_and(|value| value.chars().count() > MAX_ITEM_SUMMARY_CHARS)
        {
            return Err(ItemValidationError::InvalidSummary);
        }
        self.observed_at = normalize_timestamp(&self.observed_at)
            .map_err(|_| ItemValidationError::InvalidObservedAt)?;
        self.published_at = self
            .published_at
            .as_deref()
            .map(normalize_timestamp)
            .transpose()
            .map_err(|_| ItemValidationError::InvalidPublishedAt)?;
        self.event_time = self
            .event_time
            .as_deref()
            .map(normalize_timestamp)
            .transpose()
            .map_err(|_| ItemValidationError::InvalidEventTime)?;
        if self.category.trim().is_empty()
            || self.category.chars().count() > MAX_ITEM_CATEGORY_CHARS
            || !self.category.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            })
        {
            return Err(ItemValidationError::InvalidCategory);
        }
        if self.content_hash.as_ref().is_some_and(|hash| {
            hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        }) {
            return Err(ItemValidationError::InvalidContentHash);
        }
        if let Some(hash) = self.content_hash.as_mut() {
            hash.make_ascii_lowercase();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    fn item() -> IntelligenceItem {
        IntelligenceItem {
            id: Uuid::new_v4().to_string(),
            source_id: "example.status-feed".to_string(),
            source_url: "https://status.example/feed.xml".to_string(),
            external_id: Some("entry-42".to_string()),
            canonical_url: "https://status.example/incidents/42".to_string(),
            title: "Service degraded".to_string(),
            summary: Some("Regional request failures".to_string()),
            published_at: Some("2026-10-26T11:30:00+07:00".to_string()),
            observed_at: "2026-10-26T04:35:00Z".to_string(),
            event_time: Some("2026-10-26T04:20:00Z".to_string()),
            category: "operations".to_string(),
            content_hash: Some("A".repeat(64)),
            freshness_state: ItemFreshness::Fresh,
            verification_state: ItemVerification::SourceAttributed,
        }
    }

    #[test]
    fn published_observed_and_event_times_remain_separate_and_normalize_to_utc() {
        let mut item = item();
        item.validate().unwrap();
        assert_eq!(
            item.published_at.as_deref(),
            Some("2026-10-26T04:30:00.000Z")
        );
        assert_eq!(item.observed_at, "2026-10-26T04:35:00.000Z");
        assert_eq!(item.event_time.as_deref(), Some("2026-10-26T04:20:00.000Z"));
        assert_eq!(item.content_hash.as_deref(), Some("a".repeat(64).as_str()));
        assert_eq!(
            from_str::<IntelligenceItem>(&to_string(&item).unwrap()).unwrap(),
            item
        );
    }

    #[test]
    fn missing_published_and_event_times_are_not_fabricated() {
        let mut item = item();
        item.published_at = None;
        item.event_time = None;
        item.validate().unwrap();
        assert_eq!(item.published_at, None);
        assert_eq!(item.event_time, None);
        assert_eq!(item.observed_at, "2026-10-26T04:35:00.000Z");
    }

    #[test]
    fn rejects_bad_source_urls_timestamps_and_unbounded_fields() {
        let mut bad_url = item();
        bad_url.source_url = "https://user:secret@status.example/feed.xml".to_string();
        assert_eq!(
            bad_url.validate(),
            Err(ItemValidationError::InvalidSourceUrl)
        );
        let mut bad_time = item();
        bad_time.observed_at = "yesterday".to_string();
        assert_eq!(
            bad_time.validate(),
            Err(ItemValidationError::InvalidObservedAt)
        );
        let mut oversized = item();
        oversized.summary = Some("x".repeat(MAX_ITEM_SUMMARY_CHARS + 1));
        assert_eq!(
            oversized.validate(),
            Err(ItemValidationError::InvalidSummary)
        );
    }
}
