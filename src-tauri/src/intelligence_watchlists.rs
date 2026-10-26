//! Bounded local topic filters for future attributed IntelligenceItem consumers.
use crate::{connectors::valid_id, intelligence_items::IntelligenceItem};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_WATCHLISTS: usize = 128;
pub const MAX_FILTERS_PER_KIND: usize = 20;
pub const MAX_WATCHLIST_NAME_CHARS: usize = 120;
pub const MAX_KEYWORD_CHARS: usize = 100;
pub const MAX_CATEGORY_CHARS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TopicWatchlist {
    pub id: String,
    pub name: String,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub paused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveTopicWatchlist {
    pub id: Option<String>,
    pub name: String,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
}

impl TopicWatchlist {
    pub fn matches(&self, item: &IntelligenceItem) -> bool {
        if self.paused {
            return false;
        }
        let keyword_match = self.keywords.is_empty()
            || self.keywords.iter().any(|keyword| {
                let keyword = keyword.to_lowercase();
                item.title.to_lowercase().contains(&keyword)
                    || item
                        .summary
                        .as_deref()
                        .is_some_and(|summary| summary.to_lowercase().contains(&keyword))
            });
        let category_match = self.categories.is_empty()
            || self
                .categories
                .iter()
                .any(|category| category == &item.category);
        keyword_match && category_match
    }
}

pub fn list(connection: &Connection) -> Result<Vec<TopicWatchlist>, String> {
    let mut statement = connection
        .prepare("SELECT id, definition_json FROM intelligence_watchlists ORDER BY id LIMIT ?1")
        .map_err(|error| format!("DB_ERROR: could not prepare watchlist list: {error}"))?;
    let records = statement
        .query_map([MAX_WATCHLISTS as i64 + 1], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("DB_ERROR: could not list watchlists: {error}"))?;
    let mut watchlists = Vec::new();
    for record in records {
        let (id, json) =
            record.map_err(|error| format!("DB_ERROR: could not read watchlist: {error}"))?;
        let watchlist: TopicWatchlist = serde_json::from_str(&json)
            .map_err(|_| "DATA_ERROR: saved watchlist is invalid".to_owned())?;
        if watchlist.id != id || validate(&watchlist).is_err() {
            return Err("DATA_ERROR: saved watchlist failed validation".to_owned());
        }
        watchlists.push(watchlist);
    }
    if watchlists.len() > MAX_WATCHLISTS {
        return Err("DATA_ERROR: saved watchlist count exceeds the supported limit".to_owned());
    }
    Ok(watchlists)
}

pub fn save(
    connection: &mut Connection,
    request: SaveTopicWatchlist,
) -> Result<TopicWatchlist, String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("DB_ERROR: could not begin watchlist update: {error}"))?;
    let mut watchlist = match request.id.as_deref() {
        Some(id) => load(&transaction, id)?
            .ok_or_else(|| "NOT_FOUND: watchlist does not exist".to_owned())?,
        None => {
            let count: i64 = transaction
                .query_row("SELECT COUNT(*) FROM intelligence_watchlists", [], |row| {
                    row.get(0)
                })
                .map_err(|error| format!("DB_ERROR: could not count watchlists: {error}"))?;
            if count >= MAX_WATCHLISTS as i64 {
                return Err("LIMIT_REACHED: maximum watchlist count reached".to_owned());
            }
            TopicWatchlist {
                id: format!("watch-{}", Uuid::new_v4().simple()),
                name: String::new(),
                keywords: Vec::new(),
                categories: Vec::new(),
                paused: false,
            }
        }
    };
    watchlist.name = request.name.trim().to_owned();
    watchlist.keywords = normalize_keywords(request.keywords)?;
    watchlist.categories = normalize_categories(request.categories)?;
    validate(&watchlist)
        .map_err(|_| "VALIDATION_ERROR: watchlist requires valid filters and a name".to_owned())?;
    persist(&transaction, &watchlist)?;
    transaction
        .commit()
        .map_err(|error| format!("DB_ERROR: could not save watchlist: {error}"))?;
    Ok(watchlist)
}

pub fn set_paused(
    connection: &mut Connection,
    id: &str,
    paused: bool,
) -> Result<TopicWatchlist, String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("DB_ERROR: could not begin watchlist update: {error}"))?;
    let mut watchlist =
        load(&transaction, id)?.ok_or_else(|| "NOT_FOUND: watchlist does not exist".to_owned())?;
    watchlist.paused = paused;
    persist(&transaction, &watchlist)?;
    transaction
        .commit()
        .map_err(|error| format!("DB_ERROR: could not update watchlist state: {error}"))?;
    Ok(watchlist)
}

pub fn delete(connection: &mut Connection, id: &str) -> Result<bool, String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("DB_ERROR: could not begin watchlist delete: {error}"))?;
    let changed = transaction
        .execute("DELETE FROM intelligence_watchlists WHERE id = ?1", [id])
        .map_err(|error| format!("DB_ERROR: could not delete watchlist: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("DB_ERROR: could not complete watchlist delete: {error}"))?;
    Ok(changed == 1)
}

fn load(connection: &Connection, id: &str) -> Result<Option<TopicWatchlist>, String> {
    let json: Option<String> = connection
        .query_row(
            "SELECT definition_json FROM intelligence_watchlists WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("DB_ERROR: could not read watchlist: {error}"))?;
    json.map(|json| {
        let watchlist: TopicWatchlist = serde_json::from_str(&json)
            .map_err(|_| "DATA_ERROR: saved watchlist is invalid".to_owned())?;
        validate(&watchlist)
            .map_err(|_| "DATA_ERROR: saved watchlist failed validation".to_owned())?;
        Ok(watchlist)
    })
    .transpose()
}

fn persist(transaction: &Transaction<'_>, watchlist: &TopicWatchlist) -> Result<(), String> {
    let json = serde_json::to_string(watchlist)
        .map_err(|_| "DATA_ERROR: could not encode watchlist".to_owned())?;
    transaction
        .execute(
            "INSERT INTO intelligence_watchlists (id, definition_json) VALUES (?1, ?2)
             ON CONFLICT(id) DO UPDATE SET definition_json = excluded.definition_json,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            params![watchlist.id, json],
        )
        .map_err(|error| format!("DB_ERROR: could not save watchlist: {error}"))?;
    Ok(())
}

fn normalize_keywords(values: Vec<String>) -> Result<Vec<String>, String> {
    if values.len() > MAX_FILTERS_PER_KIND {
        return Err("VALIDATION_ERROR: too many keywords".to_owned());
    }
    let mut keywords = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty()
            || value.chars().count() > MAX_KEYWORD_CHARS
            || value.chars().any(char::is_control)
        {
            return Err(
                "VALIDATION_ERROR: keyword is empty, too long, or contains controls".to_owned(),
            );
        }
        if !keywords
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(value))
        {
            keywords.push(value.to_owned());
        }
    }
    Ok(keywords)
}

fn normalize_categories(values: Vec<String>) -> Result<Vec<String>, String> {
    if values.len() > MAX_FILTERS_PER_KIND {
        return Err("VALIDATION_ERROR: too many categories".to_owned());
    }
    let mut categories = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty()
            || value.len() > MAX_CATEGORY_CHARS
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            })
        {
            return Err(
                "VALIDATION_ERROR: category must use the normalized category format".to_owned(),
            );
        }
        if !categories.iter().any(|existing| existing == value) {
            categories.push(value.to_owned());
        }
    }
    Ok(categories)
}

fn validate(watchlist: &TopicWatchlist) -> Result<(), ()> {
    if !valid_id(&watchlist.id)
        || watchlist.name.trim().is_empty()
        || watchlist.name.trim() != watchlist.name
        || watchlist.name.chars().count() > MAX_WATCHLIST_NAME_CHARS
        || watchlist.name.chars().any(char::is_control)
        || watchlist.keywords.len() > MAX_FILTERS_PER_KIND
        || watchlist.categories.len() > MAX_FILTERS_PER_KIND
        || (watchlist.keywords.is_empty() && watchlist.categories.is_empty())
    {
        return Err(());
    }
    if watchlist.keywords.iter().any(|keyword| {
        keyword.trim().is_empty()
            || keyword.trim() != keyword
            || keyword.chars().count() > MAX_KEYWORD_CHARS
            || keyword.chars().any(char::is_control)
    }) || watchlist.categories.iter().any(|category| {
        category.is_empty()
            || category.len() > MAX_CATEGORY_CHARS
            || !category.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            })
    }) {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_migrations;

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        connection
    }

    fn request() -> SaveTopicWatchlist {
        SaveTopicWatchlist {
            id: None,
            name: "  Launch Watch  ".to_owned(),
            keywords: vec![
                "  Artemis  ".to_owned(),
                "NASA".to_owned(),
                "artemis".to_owned(),
            ],
            categories: vec!["general".to_owned(), "advisory".to_owned()],
        }
    }

    fn item(category: &str, title: &str, summary: Option<&str>) -> IntelligenceItem {
        let original_url = "https://example.test/story".to_owned();
        IntelligenceItem {
            id: Uuid::new_v4().to_string(),
            source_id: "rss-test".to_owned(),
            source_url: "https://example.test/feed.xml".to_owned(),
            external_id: None,
            canonical_url: original_url.clone(),
            original_url,
            title: title.to_owned(),
            summary: summary.map(str::to_owned),
            published_at: None,
            observed_at: "2026-10-26T00:00:00Z".to_owned(),
            event_time: None,
            category: category.to_owned(),
            content_hash: None,
            freshness_state: crate::intelligence_items::ItemFreshness::Unknown,
            verification_state: crate::intelligence_items::ItemVerification::SourceAttributed,
        }
    }

    #[test]
    fn saves_filters_and_matches_case_insensitive_keywords_and_exact_categories() {
        let mut connection = database();
        let watchlist = save(&mut connection, request()).unwrap();
        assert_eq!(watchlist.name, "Launch Watch");
        assert_eq!(watchlist.keywords, ["Artemis", "NASA"]);
        assert!(watchlist.matches(&item("general", "NASA prepares Artemis", None)));
        assert!(watchlist.matches(&item("advisory", "Status", Some("Artemis launch"))));
        assert!(!watchlist.matches(&item("other", "NASA prepares Artemis", None)));
        assert!(!watchlist.matches(&item("general", "A different story", None)));
        assert_eq!(list(&connection).unwrap(), [watchlist]);
    }

    #[test]
    fn paused_watchlist_never_matches_and_pause_state_persists() {
        let mut connection = database();
        let watchlist = save(&mut connection, request()).unwrap();
        let paused = set_paused(&mut connection, &watchlist.id, true).unwrap();
        assert!(paused.paused);
        assert!(!paused.matches(&item("general", "NASA prepares Artemis", None)));
        assert_eq!(list(&connection).unwrap(), [paused]);
        let active = set_paused(&mut connection, &watchlist.id, false).unwrap();
        assert!(!active.paused);
    }

    #[test]
    fn saved_watchlist_survives_database_reopen() {
        let path = std::env::temp_dir().join(format!(
            "moonships-topic-watchlist-{}.sqlite3",
            Uuid::new_v4()
        ));
        let saved = {
            let mut connection = Connection::open(&path).unwrap();
            run_migrations(&mut connection).unwrap();
            save(&mut connection, request()).unwrap()
        };
        {
            let connection = Connection::open(&path).unwrap();
            assert_eq!(list(&connection).unwrap(), [saved]);
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_filters_do_not_overwrite_and_delete_removes_only_the_watchlist() {
        let mut connection = database();
        let saved = save(&mut connection, request()).unwrap();
        let mut invalid = request();
        invalid.id = Some(saved.id.clone());
        invalid.categories = vec!["Not a category".to_owned()];
        assert!(save(&mut connection, invalid).is_err());
        assert_eq!(list(&connection).unwrap(), std::slice::from_ref(&saved));
        assert!(delete(&mut connection, &saved.id).unwrap());
        assert!(list(&connection).unwrap().is_empty());
        assert!(!delete(&mut connection, &saved.id).unwrap());
    }

    #[test]
    fn filters_require_at_least_one_bounded_criterion() {
        let mut connection = database();
        let mut empty = request();
        empty.keywords.clear();
        empty.categories.clear();
        assert!(save(&mut connection, empty).is_err());
        let mut too_many = request();
        too_many.keywords = (0..=MAX_FILTERS_PER_KIND)
            .map(|index| format!("term-{index}"))
            .collect();
        assert!(save(&mut connection, too_many).is_err());
    }
}
