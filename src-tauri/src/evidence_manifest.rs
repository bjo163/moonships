//! User-selected evidence file hashes and a self-checking metadata manifest.
//! Hashes support later integrity comparison only; they do not establish custody or authenticity.
use crate::hash_tools;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const EVIDENCE_MANIFEST_FORMAT: &str = "moonships.evidence-manifest";
pub const EVIDENCE_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const MAX_EVIDENCE_ENTRIES: usize = 50;
pub const MAX_EVIDENCE_TOTAL_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_EVIDENCE_MANIFEST_BYTES: usize = 512 * 1024;
const INTEGRITY_NOTICE: &str =
    "SHA-256 integrity metadata only; does not establish chain of custody or forensic validity.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceManifestEntry {
    pub id: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAssurance {
    HashIntegrityOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceManifestPayload {
    pub manifest_id: String,
    pub format: String,
    pub schema_version: u32,
    pub created_at: String,
    pub assurance: EvidenceAssurance,
    pub integrity_notice: String,
    pub entries: Vec<EvidenceManifestEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceManifest {
    pub manifest_id: String,
    pub format: String,
    pub schema_version: u32,
    pub created_at: String,
    pub assurance: EvidenceAssurance,
    pub integrity_notice: String,
    pub entries: Vec<EvidenceManifestEntry>,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceManifestError {
    InvalidFormat,
    InvalidTimestamp,
    InvalidManifestId,
    InvalidEntryCount,
    InvalidEntryId,
    InvalidFileName,
    InvalidSize,
    InvalidDigest,
    TotalSizeExceeded,
    ManifestTooLarge,
    ManifestHashMismatch,
}

impl EvidenceManifestPayload {
    pub fn validate(&self) -> Result<(), EvidenceManifestError> {
        if self.format != EVIDENCE_MANIFEST_FORMAT
            || self.schema_version != EVIDENCE_MANIFEST_SCHEMA_VERSION
            || self.assurance != EvidenceAssurance::HashIntegrityOnly
            || self.integrity_notice != INTEGRITY_NOTICE
        {
            return Err(EvidenceManifestError::InvalidFormat);
        }
        if DateTime::parse_from_rfc3339(&self.created_at).is_err() {
            return Err(EvidenceManifestError::InvalidTimestamp);
        }
        if uuid::Uuid::parse_str(&self.manifest_id)
            .map(|id| id.to_string() != self.manifest_id)
            .unwrap_or(true)
        {
            return Err(EvidenceManifestError::InvalidManifestId);
        }
        if self.entries.is_empty() || self.entries.len() > MAX_EVIDENCE_ENTRIES {
            return Err(EvidenceManifestError::InvalidEntryCount);
        }
        let mut total_bytes = 0_u64;
        for entry in &self.entries {
            if uuid::Uuid::parse_str(&entry.id)
                .map(|id| id.to_string() != entry.id)
                .unwrap_or(true)
            {
                return Err(EvidenceManifestError::InvalidEntryId);
            }
            if entry.file_name.trim().is_empty()
                || entry.file_name.chars().count() > 255
                || entry.file_name == "."
                || entry.file_name == ".."
                || entry.file_name.contains(['/', '\\'])
                || entry.file_name.chars().any(char::is_control)
            {
                return Err(EvidenceManifestError::InvalidFileName);
            }
            if entry.size_bytes > MAX_EVIDENCE_TOTAL_BYTES {
                return Err(EvidenceManifestError::InvalidSize);
            }
            total_bytes = total_bytes
                .checked_add(entry.size_bytes)
                .ok_or(EvidenceManifestError::TotalSizeExceeded)?;
            if total_bytes > MAX_EVIDENCE_TOTAL_BYTES {
                return Err(EvidenceManifestError::TotalSizeExceeded);
            }
            if entry.sha256.len() != 64
                || !entry
                    .sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(EvidenceManifestError::InvalidDigest);
            }
        }
        Ok(())
    }
}

impl EvidenceManifest {
    pub fn create(
        created_at: String,
        entries: Vec<EvidenceManifestEntry>,
    ) -> Result<Self, EvidenceManifestError> {
        let payload = EvidenceManifestPayload {
            manifest_id: uuid::Uuid::new_v4().to_string(),
            format: EVIDENCE_MANIFEST_FORMAT.to_owned(),
            schema_version: EVIDENCE_MANIFEST_SCHEMA_VERSION,
            created_at,
            assurance: EvidenceAssurance::HashIntegrityOnly,
            integrity_notice: INTEGRITY_NOTICE.to_owned(),
            entries,
        };
        payload.validate()?;
        let manifest_sha256 = hash_payload(&payload)?;
        let manifest = Self {
            manifest_id: payload.manifest_id,
            format: payload.format,
            schema_version: payload.schema_version,
            created_at: payload.created_at,
            assurance: payload.assurance,
            integrity_notice: payload.integrity_notice,
            entries: payload.entries,
            manifest_sha256,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), EvidenceManifestError> {
        let payload = self.payload();
        payload.validate()?;
        if self.manifest_sha256.len() != 64
            || !self
                .manifest_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || hash_payload(&payload)? != self.manifest_sha256
        {
            return Err(EvidenceManifestError::ManifestHashMismatch);
        }
        let json = serde_json::to_vec(self).map_err(|_| EvidenceManifestError::ManifestTooLarge)?;
        if json.len() > MAX_EVIDENCE_MANIFEST_BYTES {
            return Err(EvidenceManifestError::ManifestTooLarge);
        }
        Ok(())
    }

    pub fn to_pretty_json(&self) -> Result<String, EvidenceManifestError> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|_| EvidenceManifestError::ManifestTooLarge)?;
        if json.len() > MAX_EVIDENCE_MANIFEST_BYTES {
            return Err(EvidenceManifestError::ManifestTooLarge);
        }
        Ok(json)
    }

    fn payload(&self) -> EvidenceManifestPayload {
        EvidenceManifestPayload {
            manifest_id: self.manifest_id.clone(),
            format: self.format.clone(),
            schema_version: self.schema_version,
            created_at: self.created_at.clone(),
            assurance: self.assurance.clone(),
            integrity_notice: self.integrity_notice.clone(),
            entries: self.entries.clone(),
        }
    }
}

fn hash_payload(payload: &EvidenceManifestPayload) -> Result<String, EvidenceManifestError> {
    let canonical =
        serde_json::to_vec(payload).map_err(|_| EvidenceManifestError::ManifestTooLarge)?;
    Ok(format!("{:x}", Sha256::digest(canonical)))
}

pub fn hash_selected_reader<R: std::io::Read>(
    reader: R,
    remaining_bytes: u64,
) -> Result<(String, u64), String> {
    hash_tools::sha256_reader(reader, remaining_bytes).map_err(|error| {
        if error.kind() == std::io::ErrorKind::InvalidData {
            "LIMIT_REACHED: selected evidence exceeds the 2 GiB aggregate limit".to_owned()
        } else {
            "IO_ERROR: selected evidence could not be read".to_owned()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        hash_selected_reader, EvidenceManifest, EvidenceManifestEntry, EvidenceManifestError,
        EVIDENCE_MANIFEST_FORMAT, MAX_EVIDENCE_ENTRIES, MAX_EVIDENCE_TOTAL_BYTES,
    };

    fn entry() -> EvidenceManifestEntry {
        EvidenceManifestEntry {
            id: "b72f34e5-57c3-4fea-a09f-27faf6ab25ae".to_owned(),
            file_name: "capture.bin".to_owned(),
            size_bytes: 3,
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
        }
    }

    fn manifest() -> EvidenceManifest {
        EvidenceManifest::create("2026-10-26T12:00:00Z".to_owned(), vec![entry()]).unwrap()
    }

    #[test]
    fn sha256_known_vectors_match_for_evidence_bytes() {
        assert_eq!(
            hash_selected_reader(std::io::Cursor::new(b"abc"), 3).unwrap(),
            (
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
                3
            )
        );
        assert_eq!(
            hash_selected_reader(std::io::Cursor::new(b""), 0).unwrap(),
            (
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_owned(),
                0
            )
        );
    }

    #[test]
    fn manifest_hash_round_trips_and_detects_entry_tampering() {
        let mut document = manifest();
        document.validate().unwrap();
        let json = document.to_pretty_json().unwrap();
        let decoded: EvidenceManifest = serde_json::from_str(&json).unwrap();
        decoded.validate().unwrap();
        document.entries[0].file_name = "changed.bin".to_owned();
        assert_eq!(
            document.validate(),
            Err(EvidenceManifestError::ManifestHashMismatch)
        );
    }

    #[test]
    fn manifest_rejects_paths_bad_hashes_and_oversized_batches() {
        let mut document = manifest();
        document.entries[0].file_name = "..\\secret.bin".to_owned();
        assert_eq!(
            document.validate(),
            Err(EvidenceManifestError::InvalidFileName)
        );
        document = manifest();
        document.entries[0].sha256 = "not-a-digest".to_owned();
        assert_eq!(
            document.validate(),
            Err(EvidenceManifestError::InvalidDigest)
        );
        let many = vec![entry(); MAX_EVIDENCE_ENTRIES + 1];
        assert_eq!(
            EvidenceManifest::create("2026-10-26T12:00:00Z".to_owned(), many),
            Err(EvidenceManifestError::InvalidEntryCount)
        );
        let mut oversized = entry();
        oversized.size_bytes = MAX_EVIDENCE_TOTAL_BYTES + 1;
        assert_eq!(
            EvidenceManifest::create("2026-10-26T12:00:00Z".to_owned(), vec![oversized]),
            Err(EvidenceManifestError::InvalidSize)
        );
    }

    #[test]
    fn manifest_contract_states_integrity_limit_explicitly() {
        let value = serde_json::to_value(manifest()).unwrap();
        assert_eq!(value["format"], EVIDENCE_MANIFEST_FORMAT);
        assert_eq!(value["assurance"], "hash_integrity_only");
        assert!(value["integrityNotice"]
            .as_str()
            .unwrap()
            .contains("does not establish chain of custody"));
    }
}
