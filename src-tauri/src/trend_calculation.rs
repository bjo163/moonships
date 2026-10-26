//! Explicit, bounded comparison of attributed items across adjacent observation windows.
use crate::{
    connectors::{valid_id, MAX_CONNECTOR_ID_BYTES},
    intelligence_dedupe::{deduplicate_items, DedupeError, MAX_ITEMS_PER_DEDUPE},
    intelligence_items::{IntelligenceItem, ItemValidationError},
    intelligence_watchlists::{self, TopicWatchlist},
};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const METHOD_VERSION: &str = "window-volume-v1";
pub const MAX_TREND_WINDOW_SECONDS: i64 = 90 * 24 * 60 * 60;
pub const MAX_COVERAGE_SOURCES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservationWindow {
    pub start_at: String,
    pub end_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageLabel {
    NoSources,
    NoSuccessfulFetch,
    Partial,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowCoverageEvidence {
    /// Connector IDs selected as the scope for this observation window.
    pub selected_source_ids: Vec<String>,
    /// Selected sources that completed a fetch, including feeds that returned no matching items.
    pub successful_source_ids: Vec<String>,
    /// Selected sources whose fetch failed in this window.
    pub failed_source_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowCoverage {
    pub label: CoverageLabel,
    pub selected_source_count: usize,
    pub successful_source_count: usize,
    pub failed_source_count: usize,
    pub unattempted_source_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrendCalculationRequest {
    pub observed_window: ObservationWindow,
    pub comparison_window: ObservationWindow,
    pub observed_coverage: WindowCoverageEvidence,
    pub comparison_coverage: WindowCoverageEvidence,
    pub calculated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrendCalculationResult {
    /// False when either window is incomplete or the selected source sets differ.
    pub comparison_comparable: bool,
    pub observed_coverage: WindowCoverage,
    pub comparison_coverage: WindowCoverage,
    pub observations: Vec<WindowTrend>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowTrend {
    pub subject_key: String,
    pub category: String,
    pub observed_window: ObservationWindow,
    pub comparison_window: ObservationWindow,
    /// `None` means the window had no successful source data; a complete empty feed is `Some(0)`.
    pub count: Option<usize>,
    pub baseline_count: Option<usize>,
    /// Fractional change relative to the baseline; absent when the baseline is zero.
    pub delta_ratio: Option<f64>,
    /// Number of distinct source IDs represented in the current window's deduplicated groups.
    pub source_count: usize,
    pub observed_coverage: WindowCoverage,
    pub comparison_coverage: WindowCoverage,
    pub comparison_comparable: bool,
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
    InvalidCoverage,
    ItemSourceNotSuccessful {
        index: usize,
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
    request: TrendCalculationRequest,
) -> Result<TrendCalculationResult, TrendCalculationError> {
    if items.len() > MAX_ITEMS_PER_DEDUPE {
        return Err(TrendCalculationError::TooManyItems);
    }
    if intelligence_watchlists::validate(watchlist).is_err() {
        return Err(TrendCalculationError::InvalidWatchlist);
    }
    let observed_coverage = parse_coverage(request.observed_coverage)?;
    let comparison_coverage = parse_coverage(request.comparison_coverage)?;
    let comparison_comparable = observed_coverage.selected_ids == comparison_coverage.selected_ids
        && observed_coverage.view.label == CoverageLabel::Complete
        && comparison_coverage.view.label == CoverageLabel::Complete;
    let now =
        parse_timestamp(&request.calculated_at).ok_or(TrendCalculationError::InvalidWindow)?;
    let observed = parse_window(request.observed_window)?;
    let comparison = parse_window(request.comparison_window)?;
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
        let observed_at = DateTime::parse_from_rfc3339(&item.observed_at)
            .map_err(|_| TrendCalculationError::InvalidItem {
                index,
                reason: ItemValidationError::InvalidObservedAt,
            })?
            .with_timezone(&Utc);
        if observed_at >= observed.start && observed_at < observed.end {
            if !observed_coverage.successful_ids.contains(&item.source_id) {
                return Err(TrendCalculationError::ItemSourceNotSuccessful { index });
            }
            if !watchlist.matches(&item) {
                continue;
            }
            current_by_category
                .entry(item.category.clone())
                .or_default()
                .push(item);
        } else if observed_at >= comparison.start && observed_at < comparison.end {
            if !comparison_coverage.successful_ids.contains(&item.source_id) {
                return Err(TrendCalculationError::ItemSourceNotSuccessful { index });
            }
            if !watchlist.matches(&item) {
                continue;
            }
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
    let observations = categories
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
            let count = observed_coverage
                .view
                .label
                .has_successful_sources()
                .then_some(current_groups.len());
            let baseline_count = comparison_coverage
                .view
                .label
                .has_successful_sources()
                .then_some(baseline_groups.len());
            let delta_ratio = if comparison_comparable {
                match (count, baseline_count) {
                    (Some(count), Some(baseline_count)) if baseline_count > 0 => {
                        Some((count as f64 - baseline_count as f64) / baseline_count as f64)
                    }
                    _ => None,
                }
            } else {
                None
            };
            Ok(WindowTrend {
                subject_key: watchlist.id.clone(),
                category,
                observed_window: observed_window.clone(),
                comparison_window: comparison_window.clone(),
                count,
                baseline_count,
                delta_ratio,
                source_count: source_ids.len(),
                observed_coverage: observed_coverage.view.clone(),
                comparison_coverage: comparison_coverage.view.clone(),
                comparison_comparable,
                method_version: METHOD_VERSION.to_owned(),
                calculated_at: calculated_at.clone(),
            })
        })
        .collect::<Result<Vec<_>, TrendCalculationError>>()?;

    Ok(TrendCalculationResult {
        comparison_comparable,
        observed_coverage: observed_coverage.view,
        comparison_coverage: comparison_coverage.view,
        observations,
    })
}

#[derive(Debug)]
struct ParsedCoverage {
    view: WindowCoverage,
    selected_ids: HashSet<String>,
    successful_ids: HashSet<String>,
}

impl CoverageLabel {
    fn has_successful_sources(&self) -> bool {
        matches!(self, Self::Partial | Self::Complete)
    }
}

fn parse_coverage(
    evidence: WindowCoverageEvidence,
) -> Result<ParsedCoverage, TrendCalculationError> {
    fn parse_ids(values: Vec<String>) -> Result<HashSet<String>, TrendCalculationError> {
        if values.len() > MAX_COVERAGE_SOURCES
            || values
                .iter()
                .any(|value| value.len() > MAX_CONNECTOR_ID_BYTES || !valid_id(value))
        {
            return Err(TrendCalculationError::InvalidCoverage);
        }
        let ids: HashSet<_> = values.into_iter().collect();
        if ids.len() > MAX_COVERAGE_SOURCES {
            return Err(TrendCalculationError::InvalidCoverage);
        }
        Ok(ids)
    }

    let selected_ids = parse_ids(evidence.selected_source_ids)?;
    let successful_ids = parse_ids(evidence.successful_source_ids)?;
    let failed_ids = parse_ids(evidence.failed_source_ids)?;
    if !successful_ids.is_subset(&selected_ids)
        || !failed_ids.is_subset(&selected_ids)
        || !successful_ids.is_disjoint(&failed_ids)
    {
        return Err(TrendCalculationError::InvalidCoverage);
    }
    let selected_source_count = selected_ids.len();
    let successful_source_count = successful_ids.len();
    let failed_source_count = failed_ids.len();
    let unattempted_source_count =
        selected_source_count.saturating_sub(successful_source_count + failed_source_count);
    let label = if selected_source_count == 0 {
        CoverageLabel::NoSources
    } else if successful_source_count == 0 {
        CoverageLabel::NoSuccessfulFetch
    } else if successful_source_count < selected_source_count {
        CoverageLabel::Partial
    } else {
        CoverageLabel::Complete
    };
    Ok(ParsedCoverage {
        view: WindowCoverage {
            label,
            selected_source_count,
            successful_source_count,
            failed_source_count,
            unattempted_source_count,
        },
        selected_ids,
        successful_ids,
    })
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

    fn complete_evidence(items: &[IntelligenceItem]) -> WindowCoverageEvidence {
        let mut selected_source_ids: Vec<_> =
            items.iter().map(|item| item.source_id.clone()).collect();
        selected_source_ids.sort();
        selected_source_ids.dedup();
        WindowCoverageEvidence {
            successful_source_ids: selected_source_ids.clone(),
            selected_source_ids,
            failed_source_ids: Vec::new(),
        }
    }

    fn evidence(selected: &[&str], successful: &[&str], failed: &[&str]) -> WindowCoverageEvidence {
        WindowCoverageEvidence {
            selected_source_ids: selected.iter().map(|id| (*id).to_owned()).collect(),
            successful_source_ids: successful.iter().map(|id| (*id).to_owned()).collect(),
            failed_source_ids: failed.iter().map(|id| (*id).to_owned()).collect(),
        }
    }

    fn calculate_with_complete_coverage(
        items: &[IntelligenceItem],
        watchlist: &TopicWatchlist,
        observed_window: ObservationWindow,
        comparison_window: ObservationWindow,
        calculated_at: &str,
    ) -> Result<Vec<WindowTrend>, TrendCalculationError> {
        let coverage = complete_evidence(items);
        super::calculate_watchlist_trends(
            items,
            watchlist,
            TrendCalculationRequest {
                observed_window,
                comparison_window,
                observed_coverage: coverage.clone(),
                comparison_coverage: coverage,
                calculated_at: calculated_at.to_owned(),
            },
        )
        .map(|result| result.observations)
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
        let trends = calculate_with_complete_coverage(
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
        assert_eq!(general.count, Some(2));
        assert_eq!(general.baseline_count, Some(1));
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
            Some(0)
        );
        let archive = trends
            .iter()
            .find(|trend| trend.category == "archive")
            .unwrap();
        assert_eq!(archive.count, Some(0));
        assert_eq!(archive.baseline_count, Some(1));
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
        let trends = calculate_with_complete_coverage(
            &items,
            &watchlist(),
            observed,
            comparison,
            "2026-10-26T13:00:00Z",
        )
        .unwrap();
        assert_eq!(trends.len(), 1);
        assert_eq!(trends[0].count, Some(1));
        assert_eq!(trends[0].baseline_count, Some(0));
        assert_eq!(trends[0].delta_ratio, None);
    }

    #[test]
    fn empty_data_does_not_invent_zero_category_rows_and_paused_list_does_not_match() {
        let (observed, comparison) = windows();
        assert!(calculate_with_complete_coverage(
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
        assert!(calculate_with_complete_coverage(
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
            calculate_with_complete_coverage(
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
            calculate_with_complete_coverage(
                &[],
                &watchlist(),
                observed.clone(),
                unequal,
                "2026-10-26T13:00:00Z"
            ),
            Err(TrendCalculationError::InvalidWindow)
        );
        assert_eq!(
            calculate_with_complete_coverage(
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
        assert!(calculate_with_complete_coverage(
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
            calculate_with_complete_coverage(
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
            calculate_with_complete_coverage(
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
            calculate_with_complete_coverage(
                &too_many,
                &watchlist(),
                current,
                baseline,
                "2026-10-26T13:00:00Z"
            ),
            Err(TrendCalculationError::TooManyItems)
        );
    }

    #[test]
    fn missing_fetches_are_null_and_coverage_is_reported_without_fake_zeroes() {
        let (observed, comparison) = windows();
        for (observed_coverage, expected_label) in [
            (evidence(&[], &[], &[]), CoverageLabel::NoSources),
            (
                evidence(&["rss-one", "rss-two"], &[], &["rss-one"]),
                CoverageLabel::NoSuccessfulFetch,
            ),
        ] {
            let result = calculate_watchlist_trends(
                &[],
                &watchlist(),
                TrendCalculationRequest {
                    observed_window: observed.clone(),
                    comparison_window: comparison.clone(),
                    observed_coverage,
                    comparison_coverage: evidence(&["rss-one"], &["rss-one"], &[]),
                    calculated_at: "2026-10-26T13:00:00Z".to_owned(),
                },
            )
            .unwrap();
            assert_eq!(result.observed_coverage.label, expected_label);
            assert_eq!(result.observations.len(), 0);
            assert!(!result.comparison_comparable);
        }
    }

    #[test]
    fn partial_coverage_keeps_known_count_but_disables_comparison() {
        let (observed, comparison) = windows();
        let items = vec![item(
            "rss-one",
            "one",
            "Artemis update",
            "general",
            "2026-10-26T12:30:00Z",
        )];
        let result = calculate_watchlist_trends(
            &items,
            &watchlist(),
            TrendCalculationRequest {
                observed_window: observed,
                comparison_window: comparison,
                observed_coverage: evidence(&["rss-one", "rss-two"], &["rss-one"], &["rss-two"]),
                comparison_coverage: evidence(&["rss-one"], &["rss-one"], &[]),
                calculated_at: "2026-10-26T13:00:00Z".to_owned(),
            },
        )
        .unwrap();
        let trend = &result.observations[0];
        assert_eq!(trend.count, Some(1));
        assert_eq!(trend.observed_coverage.label, CoverageLabel::Partial);
        assert_eq!(trend.observed_coverage.unattempted_source_count, 0);
        assert_eq!(trend.delta_ratio, None);
        assert!(!trend.comparison_comparable);
    }

    #[test]
    fn complete_empty_feed_is_zero_and_source_scope_must_match_for_ratio() {
        let (observed, comparison) = windows();
        let matching_items = vec![item(
            "rss-one",
            "baseline",
            "Artemis update",
            "general",
            "2026-10-26T11:30:00Z",
        )];
        let result = calculate_watchlist_trends(
            &matching_items,
            &watchlist(),
            TrendCalculationRequest {
                observed_window: observed,
                comparison_window: comparison,
                observed_coverage: evidence(&["rss-two"], &["rss-two"], &[]),
                comparison_coverage: evidence(&["rss-one"], &["rss-one"], &[]),
                calculated_at: "2026-10-26T13:00:00Z".to_owned(),
            },
        )
        .unwrap();
        let trend = &result.observations[0];
        assert_eq!(trend.count, Some(0));
        assert_eq!(trend.baseline_count, Some(1));
        assert_eq!(trend.observed_coverage.label, CoverageLabel::Complete);
        assert_eq!(trend.delta_ratio, None);
        assert!(!result.comparison_comparable);

        let no_matches = calculate_watchlist_trends(
            &[],
            &watchlist(),
            TrendCalculationRequest {
                observed_window: windows().0,
                comparison_window: windows().1,
                observed_coverage: evidence(&["rss-one"], &["rss-one"], &[]),
                comparison_coverage: evidence(&["rss-one"], &["rss-one"], &[]),
                calculated_at: "2026-10-26T13:00:00Z".to_owned(),
            },
        )
        .unwrap();
        assert_eq!(no_matches.observed_coverage.label, CoverageLabel::Complete);
        assert!(no_matches.observations.is_empty());
    }

    #[test]
    fn rejects_invalid_coverage_and_items_from_unsuccessful_sources() {
        let (observed, comparison) = windows();
        let invalid = calculate_watchlist_trends(
            &[],
            &watchlist(),
            TrendCalculationRequest {
                observed_window: observed.clone(),
                comparison_window: comparison.clone(),
                observed_coverage: evidence(&["rss-one"], &["rss-two"], &[]),
                comparison_coverage: evidence(&[], &[], &[]),
                calculated_at: "2026-10-26T13:00:00Z".to_owned(),
            },
        );
        assert_eq!(invalid, Err(TrendCalculationError::InvalidCoverage));

        let item = item(
            "rss-one",
            "one",
            "Artemis update",
            "general",
            "2026-10-26T12:30:00Z",
        );
        let unavailable_item = calculate_watchlist_trends(
            &[item],
            &watchlist(),
            TrendCalculationRequest {
                observed_window: observed,
                comparison_window: comparison,
                observed_coverage: evidence(&["rss-one"], &[], &["rss-one"]),
                comparison_coverage: evidence(&[], &[], &[]),
                calculated_at: "2026-10-26T13:00:00Z".to_owned(),
            },
        );
        assert_eq!(
            unavailable_item,
            Err(TrendCalculationError::ItemSourceNotSuccessful { index: 0 })
        );
    }
}
