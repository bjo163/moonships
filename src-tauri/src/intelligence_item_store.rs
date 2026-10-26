//! Bounded persistence and reads for manually fetched, source-attributed intelligence items.
use crate::{
    connectors::ConnectorDefinition,
    intelligence_items::IntelligenceItem,
    rss_connector::{self, RssFetchError},
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub const MAX_STORED_ITEMS: usize = 10_000;
pub const MAX_ITEMS_PER_PAGE: usize = 200;
pub const MAX_ITEM_JSON_BYTES: usize = 16_384;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListIntelligenceItems {
    pub source_id: Option<String>,
    pub category: Option<String>,
    pub search: Option<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStoreError {
    InvalidFilter,
}

pub fn fetch_and_store(
    connection: &mut Connection,
    source: &mut ConnectorDefinition,
) -> Result<usize, RssFetchError> {
    let items = rss_connector::fetch_rss(source)?;
    let transaction = connection
        .transaction()
        .map_err(|_| RssFetchError::InvalidItem)?;
    let stored_source: Option<String> = transaction
        .query_row(
            "SELECT connector_json FROM intelligence_connectors WHERE id = ?1",
            [&source.id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| RssFetchError::InvalidItem)?;
    let Some(stored_source) = stored_source else {
        return Err(RssFetchError::Disabled);
    };
    let stored_source: ConnectorDefinition =
        serde_json::from_str(&stored_source).map_err(|_| RssFetchError::InvalidItem)?;
    if !stored_source.is_enabled()
        || stored_source.source_url != source.source_url
        || stored_source.display_name != source.display_name
    {
        return Err(RssFetchError::Disabled);
    }
    for item in &items {
        persist(&transaction, item).map_err(|_| RssFetchError::InvalidItem)?;
    }
    transaction
        .execute(
            "DELETE FROM intelligence_items WHERE id IN (
                SELECT id FROM intelligence_items ORDER BY observed_at DESC, id DESC LIMIT -1 OFFSET ?1
            )",
            [MAX_STORED_ITEMS as i64],
        )
        .map_err(|_| RssFetchError::InvalidItem)?;
    transaction
        .commit()
        .map_err(|_| RssFetchError::InvalidItem)?;
    Ok(items.len())
}

pub fn list(
    connection: &Connection,
    request: ListIntelligenceItems,
) -> Result<Vec<IntelligenceItem>, ItemStoreError> {
    validate_filter(
        request.source_id.as_deref(),
        request.category.as_deref(),
        request.search.as_deref(),
    )?;
    if request.limit == 0 || request.limit > MAX_ITEMS_PER_PAGE {
        return Err(ItemStoreError::InvalidFilter);
    }
    let source = request.source_id.unwrap_or_default();
    let category = request.category.unwrap_or_default();
    let search = request.search.unwrap_or_default();
    let mut statement = connection
        .prepare(
            "SELECT item_json FROM intelligence_items
            WHERE (?1 = '' OR source_id = ?1) AND (?2 = '' OR category = ?2)
            ORDER BY observed_at DESC, id DESC LIMIT ?3",
        )
        .map_err(|_| ItemStoreError::InvalidFilter)?;
    let records = statement
        .query_map(params![source, category, request.limit as i64], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| ItemStoreError::InvalidFilter)?;
    records
        .map(|record| {
            let json = record.map_err(|_| ItemStoreError::InvalidFilter)?;
            let mut item: IntelligenceItem =
                serde_json::from_str(&json).map_err(|_| ItemStoreError::InvalidFilter)?;
            item.validate().map_err(|_| ItemStoreError::InvalidFilter)?;
            Ok(item)
        })
        .filter(|result| {
            result.as_ref().is_ok_and(|item| {
                search.is_empty()
                    || item.title.to_lowercase().contains(&search.to_lowercase())
                    || item.summary.as_deref().is_some_and(|summary| {
                        summary.to_lowercase().contains(&search.to_lowercase())
                    })
            })
        })
        .collect()
}

pub fn list_all_for_source_export(
    connection: &Connection,
    source_id: &str,
) -> Result<Vec<IntelligenceItem>, ItemStoreError> {
    validate_filter(Some(source_id), None, None)?;
    let mut statement = connection
        .prepare("SELECT item_json FROM intelligence_items WHERE source_id = ?1 ORDER BY observed_at DESC, id DESC LIMIT ?2")
        .map_err(|_| ItemStoreError::InvalidFilter)?;
    let records = statement
        .query_map(params![source_id, MAX_STORED_ITEMS as i64 + 1], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| ItemStoreError::InvalidFilter)?;
    let mut items = Vec::new();
    for record in records {
        let mut item: IntelligenceItem =
            serde_json::from_str(&record.map_err(|_| ItemStoreError::InvalidFilter)?)
                .map_err(|_| ItemStoreError::InvalidFilter)?;
        item.validate().map_err(|_| ItemStoreError::InvalidFilter)?;
        items.push(item);
    }
    if items.len() > MAX_STORED_ITEMS {
        return Err(ItemStoreError::InvalidFilter);
    }
    Ok(items)
}

pub fn get(connection: &Connection, id: &str) -> Result<Option<IntelligenceItem>, ItemStoreError> {
    if id.len() != 36
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
    {
        return Err(ItemStoreError::InvalidFilter);
    }
    let json: Option<String> = connection
        .query_row(
            "SELECT item_json FROM intelligence_items WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| ItemStoreError::InvalidFilter)?;
    json.map(|json| {
        let mut item: IntelligenceItem =
            serde_json::from_str(&json).map_err(|_| ItemStoreError::InvalidFilter)?;
        item.validate().map_err(|_| ItemStoreError::InvalidFilter)?;
        Ok(item)
    })
    .transpose()
}

fn validate_filter(
    source: Option<&str>,
    category: Option<&str>,
    search: Option<&str>,
) -> Result<(), ItemStoreError> {
    if source.is_some_and(|value| value.len() > 64 || !crate::connectors::valid_id(value))
        || category.is_some_and(|value| {
            value.is_empty()
                || value.len() > 64
                || !value.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'_' | b'-')
                })
        })
        || search
            .is_some_and(|value| value.chars().count() > 120 || value.chars().any(char::is_control))
    {
        return Err(ItemStoreError::InvalidFilter);
    }
    Ok(())
}

fn persist(connection: &Connection, item: &IntelligenceItem) -> Result<(), ()> {
    let mut validated = item.clone();
    validated.validate().map_err(|_| ())?;
    let json = serde_json::to_string(&validated).map_err(|_| ())?;
    if json.len() > MAX_ITEM_JSON_BYTES {
        return Err(());
    }
    connection.execute(
        "INSERT INTO intelligence_items (id, source_id, category, observed_at, item_json)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET source_id=excluded.source_id, category=excluded.category, observed_at=excluded.observed_at, item_json=excluded.item_json",
        params![validated.id, validated.source_id, validated.category, validated.observed_at, json],
    ).map_err(|_| ())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connectors::{ConnectorDefinition, TermsReview},
        intelligence_items::{ItemFreshness, ItemVerification},
        run_migrations,
    };

    fn item(title: &str, category: &str, observed_at: &str) -> IntelligenceItem {
        let url = format!(
            "https://example.test/{}",
            title.to_lowercase().replace(' ', "-")
        );
        IntelligenceItem {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: "rss-test".to_owned(),
            source_url: "https://example.test/feed.xml".to_owned(),
            external_id: None,
            original_url: url.clone(),
            canonical_url: url,
            title: title.to_owned(),
            summary: None,
            published_at: None,
            observed_at: observed_at.to_owned(),
            event_time: None,
            category: category.to_owned(),
            content_hash: None,
            freshness_state: ItemFreshness::Fresh,
            verification_state: ItemVerification::SourceAttributed,
        }
    }

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        connection
    }

    #[test]
    fn persists_valid_items_and_reads_bounded_filtered_rows_and_details() {
        let connection = database();
        persist(
            &connection,
            &item("Artemis launch", "general", "2026-10-26T12:00:00Z"),
        )
        .unwrap();
        persist(
            &connection,
            &item("Weather notice", "advisory", "2026-10-26T12:01:00Z"),
        )
        .unwrap();
        let results = list(
            &connection,
            ListIntelligenceItems {
                source_id: None,
                category: Some("general".to_owned()),
                search: Some("ARTEMIS".to_owned()),
                limit: 20,
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Artemis launch");
        assert_eq!(
            get(&connection, &results[0].id).unwrap(),
            Some(results[0].clone())
        );
        let exported = list_all_for_source_export(&connection, "rss-test").unwrap();
        assert_eq!(exported.len(), 2);
        assert!(exported.iter().all(|row| row.source_id == "rss-test"));
        assert_eq!(
            list(
                &connection,
                ListIntelligenceItems {
                    source_id: None,
                    category: None,
                    search: None,
                    limit: 201
                }
            ),
            Err(ItemStoreError::InvalidFilter)
        );
    }

    #[test]
    fn rejects_invalid_item_and_filter_without_partial_storage() {
        let connection = database();
        let mut bad = item("Invalid", "bad category", "2026-10-26T12:00:00Z");
        assert!(persist(&connection, &bad).is_err());
        bad.category = "general".to_owned();
        bad.original_url = "javascript:alert(1)".to_owned();
        assert!(persist(&connection, &bad).is_err());
        assert_eq!(get(&connection, &bad.id).unwrap(), None);
        assert_eq!(
            get(&connection, "not-an-id"),
            Err(ItemStoreError::InvalidFilter)
        );
    }

    #[test]
    fn explicit_fetch_persists_complete_items_and_rejects_disabled_sources_without_network() {
        let mut connection = database();
        let mut source = ConnectorDefinition::user_configured_rss(
            "rss-local-fixture".to_owned(),
            "Local fixture".to_owned(),
            "https://example.test/feed.xml".to_owned(),
            None,
            "Local test fixture".to_owned(),
            TermsReview::Reviewed,
            60,
        );
        let disabled = fetch_and_store(&mut connection, &mut source);
        assert_eq!(disabled, Err(RssFetchError::Disabled));

        source.enable().unwrap();
        let source_json = serde_json::to_string(&source).unwrap();
        connection
            .execute(
                "INSERT INTO intelligence_connectors(id,connector_json) VALUES(?1,?2)",
                params![source.id, source_json],
            )
            .unwrap();
        assert_eq!(
            fetch_and_store(&mut connection, &mut source),
            Err(RssFetchError::ResolutionFailed)
        );
        assert!(list(
            &connection,
            ListIntelligenceItems {
                source_id: None,
                category: None,
                search: None,
                limit: 10
            }
        )
        .unwrap()
        .is_empty());
    }
}
