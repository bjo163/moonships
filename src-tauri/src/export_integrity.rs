//! Versioned integrity metadata for already-produced backup/export bytes.
//! This detects accidental corruption when the manifest is trusted separately;
//! it does not authenticate a manifest or encrypt its payload.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const INTEGRITY_MANIFEST_FORMAT: &str = "moonships.export-integrity";
pub const INTEGRITY_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const MAX_ARTIFACT_BYTES: usize = 2 * 1024 * 1024 * 1024;
const MAX_ARTIFACT_FORMAT_BYTES: usize = 128;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportIntegrityManifest {
    pub format: String,
    pub schema_version: u32,
    pub artifact_format: String,
    pub artifact_version: u32,
    pub payload_size_bytes: u64,
    pub payload_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityError {
    UnsupportedManifest,
    InvalidArtifactFormat,
    InvalidArtifactVersion,
    PayloadTooLarge,
    InvalidDigest,
    SizeMismatch,
    HashMismatch,
}

impl ExportIntegrityManifest {
    pub fn create(
        artifact_format: &str,
        artifact_version: u32,
        payload: &[u8],
    ) -> Result<Self, IntegrityError> {
        if payload.len() > MAX_ARTIFACT_BYTES {
            return Err(IntegrityError::PayloadTooLarge);
        }
        let manifest = Self {
            format: INTEGRITY_MANIFEST_FORMAT.to_owned(),
            schema_version: INTEGRITY_MANIFEST_SCHEMA_VERSION,
            artifact_format: artifact_format.to_owned(),
            artifact_version,
            payload_size_bytes: payload.len() as u64,
            payload_sha256: sha256(payload),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), IntegrityError> {
        if self.format != INTEGRITY_MANIFEST_FORMAT
            || self.schema_version != INTEGRITY_MANIFEST_SCHEMA_VERSION
        {
            return Err(IntegrityError::UnsupportedManifest);
        }
        if self.artifact_format.is_empty()
            || self.artifact_format.len() > MAX_ARTIFACT_FORMAT_BYTES
            || !self
                .artifact_format
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        {
            return Err(IntegrityError::InvalidArtifactFormat);
        }
        if self.artifact_version == 0 {
            return Err(IntegrityError::InvalidArtifactVersion);
        }
        if self.payload_size_bytes > MAX_ARTIFACT_BYTES as u64 {
            return Err(IntegrityError::PayloadTooLarge);
        }
        if self.payload_sha256.len() != 64
            || !self
                .payload_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(IntegrityError::InvalidDigest);
        }
        Ok(())
    }

    pub fn verify(&self, payload: &[u8]) -> Result<(), IntegrityError> {
        self.validate()?;
        if payload.len() > MAX_ARTIFACT_BYTES {
            return Err(IntegrityError::PayloadTooLarge);
        }
        if self.payload_size_bytes != payload.len() as u64 {
            return Err(IntegrityError::SizeMismatch);
        }
        if self.payload_sha256 != sha256(payload) {
            return Err(IntegrityError::HashMismatch);
        }
        Ok(())
    }
}

fn sha256(payload: &[u8]) -> String {
    format!("{:x}", Sha256::digest(payload))
}

#[cfg(test)]
mod tests {
    use super::{ExportIntegrityManifest, IntegrityError, INTEGRITY_MANIFEST_FORMAT};

    #[test]
    fn versioned_manifest_records_size_digest_and_verifies_original_fixture() {
        let fixture = br#"{"format":"moonships.backup","schemaVersion":3,"records":[]}"#;
        let manifest = ExportIntegrityManifest::create("moonships.backup", 3, fixture).unwrap();

        assert_eq!(manifest.format, INTEGRITY_MANIFEST_FORMAT);
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.artifact_version, 3);
        assert_eq!(manifest.payload_size_bytes, fixture.len() as u64);
        assert_eq!(manifest.payload_sha256.len(), 64);
        assert!(manifest.verify(fixture).is_ok());
    }

    #[test]
    fn changed_or_truncated_fixture_fails_integrity_verification() {
        let fixture = b"moonships export fixture";
        let manifest = ExportIntegrityManifest::create("moonships.export", 1, fixture).unwrap();
        let mut corrupted = fixture.to_vec();
        corrupted[0] ^= 1;

        assert_eq!(
            manifest.verify(&corrupted),
            Err(IntegrityError::HashMismatch)
        );
        assert_eq!(
            manifest.verify(&fixture[..fixture.len() - 1]),
            Err(IntegrityError::SizeMismatch)
        );
    }

    #[test]
    fn malformed_manifest_metadata_is_rejected() {
        let fixture = b"fixture";
        let mut manifest = ExportIntegrityManifest::create("moonships.export", 1, fixture).unwrap();
        manifest.payload_sha256 = "not-a-digest".to_owned();
        assert_eq!(manifest.validate(), Err(IntegrityError::InvalidDigest));

        let mut manifest = ExportIntegrityManifest::create("moonships.export", 1, fixture).unwrap();
        manifest.schema_version += 1;
        assert_eq!(
            manifest.validate(),
            Err(IntegrityError::UnsupportedManifest)
        );

        assert!(serde_json::from_str::<ExportIntegrityManifest>(
            r#"{"format":"moonships.export-integrity","schemaVersion":1,"artifactFormat":"moonships.export","artifactVersion":1,"payloadSizeBytes":1,"payloadSha256":"0000000000000000000000000000000000000000000000000000000000000000","unknown":true}"#
        )
        .is_err());
    }
}
