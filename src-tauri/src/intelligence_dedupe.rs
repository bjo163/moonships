//! Bounded in-memory intelligence item grouping that retains every source record.
use crate::intelligence_identity::content_fingerprint;
use crate::intelligence_items::{IntelligenceItem, ItemValidationError};
use std::collections::{HashMap, HashSet};

pub const MAX_ITEMS_PER_DEDUPE: usize = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelligenceItemGroup {
    /// Source records stay intact and in input order. The first record is the display representative.
    pub items: Vec<IntelligenceItem>,
    /// Unique source IDs in first-seen order; never replaced by the representative's source alone.
    pub source_ids: Vec<String>,
}

impl IntelligenceItemGroup {
    pub fn representative(&self) -> Option<&IntelligenceItem> {
        self.items.first()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupeError {
    TooManyItems,
    InvalidItem {
        index: usize,
        reason: ItemValidationError,
    },
}

/// Group records that share a canonical URL or recomputed content fingerprint.
/// Content hashes supplied by an external source are not trusted as a grouping key.
pub fn deduplicate_items(
    mut items: Vec<IntelligenceItem>,
) -> Result<Vec<IntelligenceItemGroup>, DedupeError> {
    if items.len() > MAX_ITEMS_PER_DEDUPE {
        return Err(DedupeError::TooManyItems);
    }
    for (index, item) in items.iter_mut().enumerate() {
        item.validate()
            .map_err(|reason| DedupeError::InvalidItem { index, reason })?;
    }

    let count = items.len();
    let mut parents: Vec<usize> = (0..count).collect();
    let mut sizes = vec![1usize; count];
    let mut first_for_key: HashMap<String, usize> = HashMap::with_capacity(count * 2);
    for (index, item) in items.iter().enumerate() {
        let fingerprint =
            content_fingerprint(&item.title, item.summary.as_deref()).map_err(|_| {
                DedupeError::InvalidItem {
                    index,
                    reason: ItemValidationError::InvalidContentHash,
                }
            })?;
        for key in [
            format!("url:{}", item.canonical_url),
            format!("content:{fingerprint}"),
        ] {
            if let Some(previous) = first_for_key.get(&key).copied() {
                union(&mut parents, &mut sizes, index, previous);
            } else {
                first_for_key.insert(key, index);
            }
        }
    }

    let mut group_for_root = HashMap::with_capacity(count);
    let mut group_indices: Vec<Vec<usize>> = Vec::new();
    for index in 0..count {
        let root = find(&mut parents, index);
        let group_index = *group_for_root.entry(root).or_insert_with(|| {
            group_indices.push(Vec::new());
            group_indices.len() - 1
        });
        group_indices[group_index].push(index);
    }

    let mut slots: Vec<Option<IntelligenceItem>> = items.into_iter().map(Some).collect();
    let mut groups = Vec::with_capacity(group_indices.len());
    for indices in group_indices {
        let mut source_ids = Vec::new();
        let mut seen_source_ids = HashSet::new();
        let mut members = Vec::with_capacity(indices.len());
        for index in indices {
            let item = slots[index]
                .take()
                .expect("each input item belongs to exactly one group");
            if seen_source_ids.insert(item.source_id.clone()) {
                source_ids.push(item.source_id.clone());
            }
            members.push(item);
        }
        groups.push(IntelligenceItemGroup {
            items: members,
            source_ids,
        });
    }
    Ok(groups)
}

fn find(parents: &mut [usize], mut index: usize) -> usize {
    let mut root = index;
    while parents[root] != root {
        root = parents[root];
    }
    while parents[index] != index {
        let next = parents[index];
        parents[index] = root;
        index = next;
    }
    root
}

fn union(parents: &mut [usize], sizes: &mut [usize], left: usize, right: usize) {
    let mut left_root = find(parents, left);
    let mut right_root = find(parents, right);
    if left_root == right_root {
        return;
    }
    if sizes[left_root] < sizes[right_root] {
        std::mem::swap(&mut left_root, &mut right_root);
    }
    parents[right_root] = left_root;
    sizes[left_root] += sizes[right_root];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence_identity::canonicalize_item_url;
    use crate::intelligence_items::{ItemFreshness, ItemVerification};
    use uuid::Uuid;

    fn item(source_id: &str, url: &str, title: &str, summary: &str) -> IntelligenceItem {
        let identity = canonicalize_item_url(url).unwrap();
        IntelligenceItem {
            id: Uuid::new_v4().to_string(),
            source_id: source_id.to_owned(),
            source_url: format!("https://{source_id}.example/feed.xml"),
            external_id: None,
            original_url: identity.original,
            canonical_url: identity.canonical,
            title: title.to_owned(),
            summary: Some(summary.to_owned()),
            published_at: None,
            observed_at: "2026-10-26T04:35:00Z".to_owned(),
            event_time: None,
            category: "general".to_owned(),
            content_hash: None,
            freshness_state: ItemFreshness::Fresh,
            verification_state: ItemVerification::SourceAttributed,
        }
    }

    #[test]
    fn duplicate_urls_group_items_without_losing_source_or_original_url() {
        let first_url = "https://news.example/story?id=7&utm_source=one#top";
        let second_url = "https://news.example/story?utm_source=two&id=7#related";
        let groups = deduplicate_items(vec![
            item("source-one", first_url, "Story", "A short report"),
            item("source-two", second_url, "Story updated", "Updated report"),
        ])
        .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].source_ids, ["source-one", "source-two"]);
        assert_eq!(groups[0].items.len(), 2);
        assert_eq!(groups[0].representative().unwrap().source_id, "source-one");
        assert_eq!(groups[0].items[0].original_url, first_url);
        assert_eq!(groups[0].items[1].original_url, second_url);
    }

    #[test]
    fn matching_content_groups_different_urls_and_computed_hashes_only() {
        let groups = deduplicate_items(vec![
            item(
                "source-one",
                "https://one.example/story",
                "Same story",
                "Shared text",
            ),
            item(
                "source-two",
                "https://two.example/story",
                "Same story",
                "Shared text",
            ),
        ])
        .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].source_ids, ["source-one", "source-two"]);

        let mut unrelated_one = item("source-one", "https://one.example/a", "Article A", "Text A");
        let mut unrelated_two = item("source-two", "https://two.example/b", "Article B", "Text B");
        unrelated_one.content_hash = Some("a".repeat(64));
        unrelated_two.content_hash = Some("a".repeat(64));
        let groups = deduplicate_items(vec![unrelated_one, unrelated_two]).unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn grouping_is_transitive_and_keeps_all_records_in_input_order() {
        let first = item(
            "source-a",
            "https://story.example/a",
            "Title A",
            "Summary A",
        );
        let mut second = item(
            "source-b",
            "https://story.example/a",
            "Title B",
            "Summary B",
        );
        let third = item(
            "source-c",
            "https://elsewhere.example/b",
            "Title B",
            "Summary B",
        );
        second.content_hash = Some("b".repeat(64));
        let groups = deduplicate_items(vec![first, second, third]).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].items.len(), 3);
        assert_eq!(groups[0].source_ids, ["source-a", "source-b", "source-c"]);
    }

    #[test]
    fn invalid_items_and_oversized_batches_fail_without_partial_groups() {
        let mut invalid = item("source-a", "https://news.example/a", "Title", "Summary");
        invalid.canonical_url = "https://wrong.example/".to_owned();
        assert!(matches!(
            deduplicate_items(vec![invalid]),
            Err(DedupeError::InvalidItem { index: 0, .. })
        ));
        assert_eq!(deduplicate_items(Vec::new()).unwrap(), Vec::new());
        let sample = item("source-a", "https://news.example/a", "Title", "Summary");
        assert_eq!(
            deduplicate_items(vec![sample; MAX_ITEMS_PER_DEDUPE + 1]),
            Err(DedupeError::TooManyItems)
        );
    }
}
