//! Explicit, bounded comparison of attributed items across adjacent observation windows.
use crate::{
    intelligence_dedupe::{deduplicate_items, DedupeError, MAX_ITEMS_PER_DEDUPE},
    intelligence_items::{IntelligenceItem, ItemValidationError},
    intelligence_watchlists::{self, TopicWatchlist},
};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const METHOD_VERSION: &str = "window-volume-v1";
pub const MAX_TREND_WINDOW_SECONDS: i64 = 90 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservationWindow {
    pub start_at: String,
    pub end_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowTrend {
    pub subject_key: String,
    pub category: String,
    pub observed_window: ObservationWindow,
    pub comparison_window: ObservationWindow,
    pub count: usize,
    pub baseline_count: usize,
    /// Fractional change relative to the baseline; absent when the baseline is zero.
    pub delta_ratio: Option<f64>,
    /// Number of distinct source IDs represented in the current window's deduplicated groups.
    pub source_count: usize,
    pub method_version: String,
    pub calculated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendCalculationError {
    TooManyItems,
    InvalidWatchlist,
    InvalidWindow,
    InvalidItem {
        index: usize,
        reason: ItemValidationError,
    },
}

#[derive(Clone)]
struct ParsedWindow {
    output: ObservationWindow,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

/// Calculates unique-item volume per normalized category for one saved watchlist.
/// Windows are half-open `[start, end)`, equal in length and contiguous; the comparison
/// window immediately precedes the observed window. `observed_at` is used for placement.
pub fn calculate_watchlist_trends(
    items: &[IntelligenceItem],
    watchlist: &TopicWatchlist,
    observed_window: ObservationWindow,
    comparison_window: ObservationWindow,
    calculated_at: &str,
) -> Result<Vec<WindowTrend>, TrendCalculationError> {
    if items.len() > MAX_ITEMS_PER_DEDUPE {
        return Err(TrendCalculationError::TooManyItems);
    }
    if intelligence_watchlists::validate(watchlist).is_err() {
        return Err(TrendCalculationError::InvalidWatchlist);
    }
    let now = parse_timestamp(calculated_at).ok_or(TrendCalculationError::InvalidWindow)?;
    let observed = parse_window(observed_window)?;
    let comparison = parse_window(comparison_window)?;
    let observed_duration = observed.end.signed_duration_since(observed.start);
    let comparison_duration = comparison.end.signed_duration_since(comparison.start);
    if observed_duration <= Duration::zero()
        || observed_duration > Duration::seconds(MAX_TREND_WINDOW_SECONDS)
        || comparison_duration != observed_duration
        || comparison.end != observed.start
        || observed.end > now
    {
        return Err(TrendCalculationError::InvalidWindow);
    }

    let mut current_by_category: BTreeMap<String, Vec<IntelligenceItem>> = BTreeMap::new();
    let mut baseline_by_category: BTreeMap<String, Vec<IntelligenceItem>> = BTreeMap::new();
    for (index, original) in items.iter().enumerate() {
        let mut item = original.clone();
        item.validate()
            .map_err(|reason| TrendCalculationError::InvalidItem { index, reason })?;
        if !watchlist.matches(&item) {
            continue;
        }
        let observed_at = DateTime::parse_from_rfc3339(&item.observed_at)
            .map_err(|_| TrendCalculationError::InvalidItem {
                index,
                reason: ItemValidationError::InvalidObservedAt,
            })?
            .with_timezone(&Utc);
        if observed_at >= observed.start && observed_at < observed.end {
            current_by_category
                .entry(item.category.clone())
                .or_default()
                .push(item);
        } else if observed_at >= comparison.start && observed_at < comparison.end {
            baseline_by_category
                .entry(item.category.clone())
                .or_default()
                .push(item);
        }
    }

    let categories: HashSet<String> = current_by_category
        .keys()
        .chain(baseline_by_category.keys())
        .cloned()
        .collect();
    let mut categories: Vec<_> = categories.into_iter().collect();
    categories.sort();

    let observed_window = observed.output;
    let comparison_window = comparison.output;
    let calculated_at = normalize_timestamp(now);
    categories
        .into_iter()
        .map(|category| {
            let current_items = current_by_category.remove(&category).unwrap_or_default();
            let baseline_items = baseline_by_category.remove(&category).unwrap_or_default();
            let current_groups = group(current_items)?;
            let baseline_groups = group(baseline_items)?;
            let mut source_ids = HashSet::new();
            for group in &current_groups {
                source_ids.extend(group.source_ids.iter().cloned());
            }
            let count = current_groups.len();
            let baseline_count = baseline_groups.len();
            let delta_ratio = (baseline_count > 0)
                .then(|| (count as f64 - baseline_count as f64) / baseline_count as f64);
            Ok(WindowTrend {
                subject_key: watchlist.id.clone(),
                category,
                observed_window: observed_window.clone(),
                comparison_window: comparison_window.clone(),
                count,
                baseline_count,
                delta_ratio,
                source_count: source_ids.len(),
                method_version: METHOD_VERSION.to_owned(),
                calculated_at: calculated_at.clone(),
            })
        })
        .collect()
}

fn group(
    items: Vec<IntelligenceItem>,
) -> Result<Vec<crate::intelligence_dedupe::IntelligenceItemGroup>, TrendCalculationError> {
    deduplicate_items(items).map_err(|error| match error {
        DedupeError::TooManyItems => TrendCalculationError::TooManyItems,
        DedupeError::InvalidItem { index, reason } => {
            TrendCalculationError::InvalidItem { index, reason }
        }
    })
}

fn parse_window(window: ObservationWindow) -> Result<ParsedWindow, TrendCalculationError> {
    let start = parse_timestamp(&window.start_at).ok_or(TrendCalculationError::InvalidWindow)?;
    let end = parse_timestamp(&window.end_at).ok_or(TrendCalculationError::InvalidWindow)?;
    Ok(ParsedWindow {
        output: ObservationWindow {
            start_at: normalize_timestamp(start),
            end_at: normalize_timestamp(end),
        },
        start,
        end,
    })
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|parsed| parsed.with_timezone(&Utc))
}

fn normalize_timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence_items::{ItemFreshness, ItemVerification};
    use uuid::Uuid;

    fn watchlist() -> TopicWatchlist {
        TopicWatchlist {
            id: "watch-artemis".to_owned(),
            name: "Artemis".to_owned(),
            keywords: vec!["Artemis".to_owned()],
            categories: Vec::new(),
            paused: false,
        }
    }

    fn item(
        source: &str,
        path: &str,
        title: &str,
        category: &str,
        observed_at: &str,
    ) -> IntelligenceItem {
        let url = format!("https://example.test/{path}");
        IntelligenceItem {
            id: Uuid::new_v4().to_string(),
            source_id: source.to_owned(),
            source_url: format!("https://{source}.example/feed.xml"),
            external_id: None,
            original_url: url.clone(),
            canonical_url: url,
            title: title.to_owned(),
            summary: None,
            published_at: Some("2027-01-01T00:00:00Z".to_owned()),
            observed_at: observed_at.to_owned(),
            event_time: None,
            category: category.to_owned(),
            content_hash: None,
            freshness_state: ItemFreshness::Unknown,
            verification_state: ItemVerification::SourceAttributed,
        }
    }

    fn windows() -> (ObservationWindow, ObservationWindow) {
        (
            ObservationWindow {
                start_at: "2026-10-26T12:00:00Z".to_owned(),
                end_at: "2026-10-26T13:00:00Z".to_owned(),
            },
            ObservationWindow {
                start_at: "2026-10-26T11:00:00Z".to_owned(),
                end_at: "2026-10-26T12:00:00Z".to_owned(),
            },
        )
    }

    #[test]
    fn counts_deduplicated_watchlist_matches_per_category_and_declares_both_windows() {
        let (observed, comparison) = windows();
        let items = vec![
            item(
                "rss-one",
                "one",
                "Artemis launch update",
                "general",
                "2026-10-26T12:05:00Z",
            ),
            item(
                "rss-two",
                "mirror",
                "Artemis launch update",
                "general",
                "2026-10-26T12:10:00Z",
            ),
            item(
                "rss-three",
                "three",
                "Artemis vehicle update",
                "general",
                "2026-10-26T12:59:59Z",
            ),
            item(
                "rss-four",
                "four",
                "A different report",
                "general",
                "2026-10-26T12:30:00Z",
            ),
            item(
                "rss-five",
                "five",
                "Artemis launch update",
                "general",
                "2026-10-26T11:30:00Z",
            ),
            item(
                "rss-seven",
                "older",
                "Artemis archived advisory",
                "archive",
                "2026-10-26T11:45:00Z",
            ),
            item(
                "rss-six",
                "six",
                "Artemis advisory",
                "advisory",
                "2026-10-26T12:30:00Z",
            ),
        ];
        let trends = calculate_watchlist_trends(
            &items,
            &watchlist(),
            observed,
            comparison,
            "2026-10-26T13:00:00Z",
        )
        .unwrap();
        let general = trends
            .iter()
            .find(|trend| trend.category == "general")
            .unwrap();
        assert_eq!(general.subject_key, "watch-artemis");
        assert_eq!(general.count, 2);
        assert_eq!(general.baseline_count, 1);
        assert_eq!(general.delta_ratio, Some(1.0));
        assert_eq!(general.source_count, 3);
        assert_eq!(general.method_version, METHOD_VERSION);
        assert_eq!(general.observed_window.start_at, "2026-10-26T12:00:00.000Z");
        assert_eq!(general.comparison_window.end_at, "2026-10-26T12:00:00.000Z");
        let serialized = serde_json::to_value(general).unwrap();
        assert_eq!(serialized["baselineCount"], 1);
        assert_eq!(
            serialized["comparisonWindow"]["startAt"],
            "2026-10-26T11:00:00.000Z"
        );
        assert_eq!(serialized["methodVersion"], METHOD_VERSION);
        assert_eq!(
            trends
                .iter()
                .find(|trend| trend.category == "advisory")
                .unwrap()
                .baseline_count,
            0
        );
        let archive = trends
            .iter()
            .find(|trend| trend.category == "archive")
            .unwrap();
        assert_eq!(archive.count, 0);
        assert_eq!(archive.baseline_count, 1);
        assert_eq!(archive.delta_ratio, Some(-1.0));
    }

    #[test]
    fn half_open_boundaries_use_observation_time_and_zero_baseline_has_no_ratio() {
        let (observed, comparison) = windows();
        let items = vec![
            item(
                "rss-one",
                "start",
                "Artemis at start",
                "general",
                "2026-10-26T12:00:00Z",
            ),
            item(
                "rss-two",
                "end",
                "Artemis at end",
                "general",
                "2026-10-26T13:00:00Z",
            ),
        ];
        let trends = calculate_watchlist_trends(
            &items,
            &watchlist(),
            observed,
            comparison,
            "2026-10-26T13:00:00Z",
        )
        .unwrap();
        assert_eq!(trends.len(), 1);
        assert_eq!(trends[0].count, 1);
        assert_eq!(trends[0].baseline_count, 0);
        assert_eq!(trends[0].delta_ratio, None);
    }

    #[test]
    fn empty_data_does_not_invent_zero_category_rows_and_paused_list_does_not_match() {
        let (observed, comparison) = windows();
        assert!(calculate_watchlist_trends(
            &[],
            &watchlist(),
            observed.clone(),
            comparison.clone(),
            "2026-10-26T13:00:00Z",
        )
        .unwrap()
        .is_empty());
        let mut paused = watchlist();
        paused.paused = true;
        assert!(calculate_watchlist_trends(
            &[item(
                "rss-one",
                "one",
                "Artemis update",
                "general",
                "2026-10-26T12:30:00Z"
            )],
            &paused,
            observed,
            comparison,
            "2026-10-26T13:00:00Z",
        )
        .unwrap()
        .is_empty());
    }

    #[test]
    fn rejects_invalid_or_noncomparable_windows_and_oversized_batches() {
        let (observed, comparison) = windows();
        let mut gap = comparison.clone();
        gap.end_at = "2026-10-26T11:59:00Z".to_owned();
        assert_eq!(
            calculate_watchlist_trends(
                &[],
                &watchlist(),
                observed.clone(),
                gap,
                "2026-10-26T13:00:00Z"
            ),
            Err(TrendCalculationError::InvalidWindow)
        );
        let mut unequal = comparison.clone();
        unequal.start_at = "2026-10-26T10:00:00Z".to_owned();
        assert_eq!(
            calculate_watchlist_trends(
                &[],
                &watchlist(),
                observed.clone(),
                unequal,
                "2026-10-26T13:00:00Z"
            ),
            Err(TrendCalculationError::InvalidWindow)
        );
        assert_eq!(
            calculate_watchlist_trends(
                &[],
                &watchlist(),
                observed.clone(),
                comparison.clone(),
                "2026-10-26T12:59:59Z"
            ),
            Err(TrendCalculationError::InvalidWindow)
        );
        let future = item(
            "rss-one",
            "future",
            "Artemis update",
            "general",
            "2026-10-26T13:30:00Z",
        );
        let (current, baseline) = windows();
        assert!(calculate_watchlist_trends(
            &[future],
            &watchlist(),
            current,
            baseline,
            "2026-10-26T13:00:00Z"
        )
        .unwrap()
        .is_empty());
        let mut invalid_item = item(
            "rss-one",
            "invalid",
            "Artemis update",
            "general",
            "2026-10-26T12:30:00Z",
        );
        invalid_item.observed_at = "unknown".to_owned();
        let (current, baseline) = windows();
        assert!(matches!(
            calculate_watchlist_trends(
                &[invalid_item],
                &watchlist(),
                current,
                baseline,
                "2026-10-26T13:00:00Z"
            ),
            Err(TrendCalculationError::InvalidItem {
                reason: ItemValidationError::InvalidObservedAt,
                ..
            })
        ));
        let (current, baseline) = windows();
        let mut long_window = current.clone();
        long_window.start_at = "2026-07-27T13:00:00Z".to_owned();
        let mut long_comparison = baseline.clone();
        long_comparison.start_at = "2026-04-27T13:00:00Z".to_owned();
        long_comparison.end_at = long_window.start_at.clone();
        assert_eq!(
            calculate_watchlist_trends(
                &[],
                &watchlist(),
                long_window,
                long_comparison,
                "2026-10-26T13:00:00Z"
            ),
            Err(TrendCalculationError::InvalidWindow)
        );
        let (current, baseline) = windows();
        let too_many = vec![
            item(
                "rss-one",
                "item",
                "Artemis update",
                "general",
                "2026-10-26T12:30:00Z"
            );
            MAX_ITEMS_PER_DEDUPE + 1
        ];
        assert_eq!(
            calculate_watchlist_trends(
                &too_many,
                &watchlist(),
                current,
                baseline,
                "2026-10-26T13:00:00Z"
            ),
            Err(TrendCalculationError::TooManyItems)
        );
    }
}
