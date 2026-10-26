//! Local persistence for user-configured intelligence connector declarations.
use crate::connectors::{ConnectorDefinition, ConnectorHealth, TermsReview};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_CONNECTOR_SOURCES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveConnectorSettings {
    pub id: Option<String>,
    pub display_name: String,
    pub source_url: String,
    pub terms_url: Option<String>,
    pub terms_notes: String,
    pub terms_review: TermsReview,
    pub minimum_poll_interval_seconds: u64,
}

pub fn list_sources(connection: &Connection) -> Result<Vec<ConnectorDefinition>, String> {
    let mut statement = connection
        .prepare("SELECT id, connector_json FROM intelligence_connectors ORDER BY id LIMIT ?1")
        .map_err(|error| format!("DB_ERROR: could not prepare source list: {error}"))?;
    let records = statement
        .query_map([MAX_CONNECTOR_SOURCES as i64 + 1], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("DB_ERROR: could not list source settings: {error}"))?;
    let mut sources = Vec::new();
    for record in records {
        let (id, json) =
            record.map_err(|error| format!("DB_ERROR: could not read source setting: {error}"))?;
        let source: ConnectorDefinition = serde_json::from_str(&json)
            .map_err(|_| "DATA_ERROR: saved source settings are invalid".to_owned())?;
        if source.id != id || source.validate().is_err() {
            return Err("DATA_ERROR: saved source settings failed validation".to_owned());
        }
        sources.push(source);
    }
    if sources.len() > MAX_CONNECTOR_SOURCES {
        return Err("DATA_ERROR: saved source count exceeds the supported limit".to_owned());
    }
    Ok(sources)
}

pub fn save_source(
    connection: &mut Connection,
    settings: SaveConnectorSettings,
) -> Result<ConnectorDefinition, String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("DB_ERROR: could not begin source update: {error}"))?;
    let mut source = match settings.id.as_deref() {
        Some(id) => load_source(&transaction, id)?
            .ok_or_else(|| "NOT_FOUND: source does not exist".to_owned())?,
        None => {
            let count: i64 = transaction
                .query_row("SELECT COUNT(*) FROM intelligence_connectors", [], |row| {
                    row.get(0)
                })
                .map_err(|error| format!("DB_ERROR: could not count sources: {error}"))?;
            if count >= MAX_CONNECTOR_SOURCES as i64 {
                return Err("LIMIT_REACHED: maximum source count reached".to_owned());
            }
            ConnectorDefinition::user_configured_rss(
                format!("rss-{}", Uuid::new_v4().simple()),
                settings.display_name.clone(),
                settings.source_url.clone(),
                settings.terms_url.clone(),
                settings.terms_notes.clone(),
                settings.terms_review,
                settings.minimum_poll_interval_seconds,
            )
        }
    };

    let source_url_changed = source.source_url != settings.source_url;
    source.display_name = settings.display_name;
    source.source_url = settings.source_url;
    source.terms_url = settings.terms_url;
    source.terms_notes = settings.terms_notes;
    source.terms_review = settings.terms_review;
    source.minimum_poll_interval_seconds = settings.minimum_poll_interval_seconds;
    source.freshness_threshold_seconds = 3_600.max(source.minimum_poll_interval_seconds);
    if source_url_changed {
        source.disable();
        source.health = ConnectorHealth::NotConfigured;
        source.last_attempt_at = None;
        source.last_success_at = None;
    } else if source.is_enabled() && source.terms_review != TermsReview::Reviewed {
        source.disable();
    }
    source
        .validate()
        .map_err(|error| format!("VALIDATION_ERROR: connector settings are invalid ({error:?})"))?;
    persist_source(&transaction, &source)?;
    transaction
        .commit()
        .map_err(|error| format!("DB_ERROR: could not save source settings: {error}"))?;
    Ok(source)
}

pub fn set_enabled(
    connection: &mut Connection,
    id: &str,
    enabled: bool,
) -> Result<ConnectorDefinition, String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("DB_ERROR: could not begin source update: {error}"))?;
    let mut source = load_source(&transaction, id)?
        .ok_or_else(|| "NOT_FOUND: source does not exist".to_owned())?;
    if enabled {
        source
            .enable()
            .map_err(|error| format!("VALIDATION_ERROR: source cannot be enabled ({error:?})"))?;
    } else {
        source.disable();
    }
    source
        .validate()
        .map_err(|error| format!("VALIDATION_ERROR: source settings are invalid ({error:?})"))?;
    persist_source(&transaction, &source)?;
    transaction
        .commit()
        .map_err(|error| format!("DB_ERROR: could not update source state: {error}"))?;
    Ok(source)
}

fn load_source(connection: &Connection, id: &str) -> Result<Option<ConnectorDefinition>, String> {
    let json: Option<String> = connection
        .query_row(
            "SELECT connector_json FROM intelligence_connectors WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("DB_ERROR: could not read source: {error}"))?;
    json.map(|json| {
        let source: ConnectorDefinition = serde_json::from_str(&json)
            .map_err(|_| "DATA_ERROR: saved source settings are invalid".to_owned())?;
        source
            .validate()
            .map_err(|_| "DATA_ERROR: saved source settings failed validation".to_owned())?;
        Ok(source)
    })
    .transpose()
}

fn persist_source(
    transaction: &Transaction<'_>,
    source: &ConnectorDefinition,
) -> Result<(), String> {
    let json = serde_json::to_string(source)
        .map_err(|_| "DATA_ERROR: could not encode source settings".to_owned())?;
    transaction
        .execute(
            "INSERT INTO intelligence_connectors (id, connector_json) VALUES (?1, ?2)
             ON CONFLICT(id) DO UPDATE SET connector_json = excluded.connector_json,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            params![source.id, json],
        )
        .map_err(|error| format!("DB_ERROR: could not save source settings: {error}"))?;
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

    fn settings() -> SaveConnectorSettings {
        SaveConnectorSettings {
            id: None,
            display_name: "JPL CNEOS news".to_owned(),
            source_url: "https://cneos.jpl.nasa.gov/feed/news.xml".to_owned(),
            terms_url: Some("https://cneos.jpl.nasa.gov/feed/".to_owned()),
            terms_notes: "Review source terms, respect rate limits, and attribute JPL/NASA."
                .to_owned(),
            terms_review: TermsReview::Unreviewed,
            minimum_poll_interval_seconds: 900,
        }
    }

    #[test]
    fn source_settings_round_trip_and_cadence_update_preserves_disabled_state() {
        let mut connection = database();
        let source = save_source(&mut connection, settings()).unwrap();
        assert!(!source.is_enabled());
        assert_eq!(source.health, ConnectorHealth::NotConfigured);
        assert_eq!(source.minimum_poll_interval_seconds, 900);
        assert_eq!(list_sources(&connection).unwrap(), vec![source.clone()]);

        let mut changed = settings();
        changed.id = Some(source.id.clone());
        changed.minimum_poll_interval_seconds = 1_800;
        let updated = save_source(&mut connection, changed).unwrap();
        assert!(!updated.is_enabled());
        assert_eq!(updated.minimum_poll_interval_seconds, 1_800);
        assert_eq!(updated.freshness_threshold_seconds, 3_600);

        let mut slower = settings();
        slower.id = Some(source.id.clone());
        slower.minimum_poll_interval_seconds = 86_400;
        let slower = save_source(&mut connection, slower).unwrap();
        assert_eq!(slower.freshness_threshold_seconds, 86_400);
        let mut faster = settings();
        faster.id = Some(source.id);
        faster.minimum_poll_interval_seconds = 900;
        let faster = save_source(&mut connection, faster).unwrap();
        assert_eq!(faster.freshness_threshold_seconds, 3_600);
    }

    #[test]
    fn source_settings_survive_database_reopen() {
        let path = std::env::temp_dir().join(format!(
            "moonships-intelligence-source-{}.sqlite3",
            Uuid::new_v4()
        ));
        {
            let mut connection = Connection::open(&path).unwrap();
            run_migrations(&mut connection).unwrap();
            let source = save_source(&mut connection, settings()).unwrap();
            assert_eq!(source.display_name, "JPL CNEOS news");
        }
        {
            let connection = Connection::open(&path).unwrap();
            let sources = list_sources(&connection).unwrap();
            assert_eq!(sources.len(), 1);
            assert_eq!(
                sources[0].source_url,
                "https://cneos.jpl.nasa.gov/feed/news.xml"
            );
            assert!(!sources[0].is_enabled());
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn enable_requires_terms_review_and_disabled_state_persists() {
        let mut connection = database();
        let source = save_source(&mut connection, settings()).unwrap();
        assert!(set_enabled(&mut connection, &source.id, true).is_err());

        let mut reviewed = settings();
        reviewed.id = Some(source.id.clone());
        reviewed.terms_review = TermsReview::Reviewed;
        let reviewed = save_source(&mut connection, reviewed).unwrap();
        let enabled = set_enabled(&mut connection, &reviewed.id, true).unwrap();
        assert!(enabled.is_enabled());
        assert_eq!(enabled.health, ConnectorHealth::Ready);
        let disabled = set_enabled(&mut connection, &enabled.id, false).unwrap();
        assert!(!disabled.is_enabled());
        assert_eq!(disabled.health, ConnectorHealth::Disabled);
        assert_eq!(list_sources(&connection).unwrap(), vec![disabled]);
    }

    #[test]
    fn invalid_updates_and_url_changes_fail_closed_without_enabling() {
        let mut connection = database();
        let source = save_source(&mut connection, settings()).unwrap();
        let mut invalid = settings();
        invalid.id = Some(source.id.clone());
        invalid.source_url = "file:///etc/passwd".to_owned();
        assert!(save_source(&mut connection, invalid).is_err());
        assert_eq!(list_sources(&connection).unwrap(), vec![source.clone()]);

        let mut changed = settings();
        changed.id = Some(source.id.clone());
        changed.terms_review = TermsReview::Reviewed;
        changed.source_url = "https://other.example/feed.xml".to_owned();
        let updated = save_source(&mut connection, changed).unwrap();
        assert!(!updated.is_enabled());
        assert_eq!(updated.health, ConnectorHealth::NotConfigured);
        assert_eq!(updated.last_success_at, None);
    }
}
