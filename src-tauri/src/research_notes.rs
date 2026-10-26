//! Small local research notes linked to validated intelligence-item references.
use crate::{
    intelligence_item_store,
    intelligence_items::{IntelligenceItem, ItemVerification},
};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_NOTES: usize = 2_000;
pub const MAX_TITLE_CHARS: usize = 160;
pub const MAX_BODY_CHARS: usize = 8_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub item_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchReference {
    pub note_id: String,
    pub item_id: String,
    pub source_id: String,
    pub source_url: String,
    pub original_url: String,
    pub published_at: Option<String>,
    pub observed_at: String,
    pub verification_state: String,
    pub item_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchExport {
    pub schema_version: u32,
    pub exported_at: String,
    pub note: ResearchNote,
    pub reference: ResearchReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchError {
    InvalidInput,
    ItemNotFound,
    NoteNotFound,
}

pub fn save_for_item(
    connection: &mut Connection,
    item_id: &str,
) -> Result<ResearchNote, ResearchError> {
    let item = intelligence_item_store::get(connection, item_id)
        .map_err(|_| ResearchError::InvalidInput)?
        .ok_or(ResearchError::ItemNotFound)?;
    let title = item.title.chars().take(MAX_TITLE_CHARS).collect::<String>();
    let transaction = connection
        .transaction()
        .map_err(|_| ResearchError::InvalidInput)?;
    let count: i64 = transaction
        .query_row("SELECT COUNT(*) FROM research_notes", [], |row| row.get(0))
        .map_err(|_| ResearchError::InvalidInput)?;
    let note = if count as usize >= MAX_NOTES {
        let existing_id: Option<String> = transaction
            .query_row(
                "SELECT note_id FROM research_note_references WHERE item_id = ?1",
                [&item.id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| ResearchError::InvalidInput)?;
        if let Some(existing_id) = existing_id {
            load(&transaction, &existing_id)?.ok_or(ResearchError::NoteNotFound)?
        } else {
            return Err(ResearchError::InvalidInput);
        }
    } else if let Some(existing_id) = transaction
        .query_row(
            "SELECT note_id FROM research_note_references WHERE item_id = ?1",
            [&item.id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|_| ResearchError::InvalidInput)?
    {
        load(&transaction, &existing_id)?.ok_or(ResearchError::NoteNotFound)?
    } else {
        let now = now_timestamp();
        let note = ResearchNote {
            id: Uuid::new_v4().to_string(),
            title,
            body: String::new(),
            item_id: item.id.clone(),
            created_at: now.clone(),
            updated_at: now,
        };
        persist(&transaction, &note, &item)?;
        note
    };
    transaction
        .commit()
        .map_err(|_| ResearchError::InvalidInput)?;
    Ok(note)
}

pub fn list(connection: &Connection) -> Result<Vec<ResearchNote>, ResearchError> {
    let mut statement = connection.prepare("SELECT id, title, body, item_id, created_at, updated_at FROM research_notes ORDER BY updated_at DESC, id DESC LIMIT ?1").map_err(|_| ResearchError::InvalidInput)?;
    let rows = statement
        .query_map([MAX_NOTES as i64 + 1], |row| {
            Ok(ResearchNote {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                item_id: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|_| ResearchError::InvalidInput)?;
    let values = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ResearchError::InvalidInput)?;
    if values.len() > MAX_NOTES {
        return Err(ResearchError::InvalidInput);
    }
    Ok(values)
}

pub fn export(connection: &Connection, note_id: &str) -> Result<ResearchExport, ResearchError> {
    let note = load(connection, note_id)?.ok_or(ResearchError::NoteNotFound)?;
    let reference = connection.query_row("SELECT r.note_id, r.item_id, r.source_id, r.source_url, r.original_url, r.published_at, r.observed_at, r.verification_state, EXISTS(SELECT 1 FROM intelligence_items i WHERE i.id=r.item_id) FROM research_note_references r WHERE r.note_id = ?1", [&note.id], |row| Ok(ResearchReference { note_id: row.get(0)?, item_id: row.get(1)?, source_id: row.get(2)?, source_url: row.get(3)?, original_url: row.get(4)?, published_at: row.get(5)?, observed_at: row.get(6)?, verification_state: row.get(7)?, item_available: row.get(8)? })).map_err(|_| ResearchError::InvalidInput)?;
    Ok(ResearchExport {
        schema_version: 1,
        exported_at: now_timestamp(),
        note,
        reference,
    })
}

pub fn get(
    connection: &Connection,
    note_id: &str,
) -> Result<Option<ResearchExport>, ResearchError> {
    let Some(note) = load(connection, note_id)? else {
        return Ok(None);
    };
    export(connection, &note.id).map(Some)
}

fn persist(
    transaction: &Transaction<'_>,
    note: &ResearchNote,
    item: &IntelligenceItem,
) -> Result<(), ResearchError> {
    transaction.execute("INSERT INTO research_notes (id,title,body,item_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6)", params![note.id,note.title,note.body,note.item_id,note.created_at,note.updated_at]).map_err(|_| ResearchError::InvalidInput)?;
    transaction.execute("INSERT INTO research_note_references (note_id,item_id,source_id,source_url,original_url,published_at,observed_at,verification_state) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", params![note.id,item.id,item.source_id,item.source_url,item.original_url,item.published_at,item.observed_at,verification_state_name(item.verification_state)]).map_err(|_| ResearchError::InvalidInput)?;
    Ok(())
}

fn verification_state_name(state: ItemVerification) -> &'static str {
    match state {
        ItemVerification::Unverified => "unverified",
        ItemVerification::SourceAttributed => "source_attributed",
        ItemVerification::IndependentlyVerified => "independently_verified",
    }
}

fn load(connection: &Connection, id: &str) -> Result<Option<ResearchNote>, ResearchError> {
    connection
        .query_row(
            "SELECT id,title,body,item_id,created_at,updated_at FROM research_notes WHERE id=?1",
            [id],
            |row| {
                Ok(ResearchNote {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    body: row.get(2)?,
                    item_id: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|_| ResearchError::InvalidInput)
}

fn now_timestamp() -> String {
    chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now())
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        intelligence_items::{ItemFreshness, ItemVerification},
        run_migrations,
    };

    fn item() -> IntelligenceItem {
        IntelligenceItem {
            id: "00000000-0000-4000-8000-000000000042".to_owned(),
            source_id: "rss-test".to_owned(),
            source_url: "https://example.test/feed".to_owned(),
            external_id: None,
            original_url: "https://example.test/item".to_owned(),
            canonical_url: "https://example.test/item".to_owned(),
            title: "Attributed event".to_owned(),
            summary: None,
            published_at: Some("2026-10-26T12:00:00Z".to_owned()),
            observed_at: "2026-10-26T12:01:00Z".to_owned(),
            event_time: None,
            category: "general".to_owned(),
            content_hash: None,
            freshness_state: ItemFreshness::Fresh,
            verification_state: ItemVerification::SourceAttributed,
        }
    }

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        run_migrations(&mut connection).unwrap();
        let item = item();
        let mut validated = item.clone();
        validated.validate().unwrap();
        let json = serde_json::to_string(&validated).unwrap();
        connection.execute("INSERT INTO intelligence_items(id,source_id,category,observed_at,item_json) VALUES(?1,?2,?3,?4,?5)",params![item.id,item.source_id,item.category,validated.observed_at,json]).unwrap();
        connection
    }

    #[test]
    fn save_is_idempotent_and_export_keeps_source_provenance_without_item_body_copy() {
        let mut connection = database();
        let note = save_for_item(&mut connection, "00000000-0000-4000-8000-000000000042").unwrap();
        assert_eq!(
            save_for_item(&mut connection, &note.item_id).unwrap().id,
            note.id
        );
        let document = export(&connection, &note.id).unwrap();
        assert_eq!(document.schema_version, 1);
        assert_eq!(document.reference.source_id, "rss-test");
        assert_eq!(document.reference.verification_state, "source_attributed");
        assert_eq!(document.reference.original_url, "https://example.test/item");
        assert_eq!(
            document.reference.published_at.as_deref(),
            Some("2026-10-26T12:00:00.000Z")
        );
        assert!(serde_json::to_string(&document)
            .unwrap()
            .contains("sourceId"));
        connection
            .execute(
                "DELETE FROM intelligence_items WHERE id = ?1",
                [&note.item_id],
            )
            .unwrap();
        let retained = export(&connection, &note.id).unwrap();
        assert!(!retained.reference.item_available);
        assert_eq!(retained.reference.original_url, "https://example.test/item");
        assert!(save_for_item(&mut connection, "00000000-0000-4000-8000-000000000043").is_err());
    }
}
