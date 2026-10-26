//! Typed metadata contract for future external adapters. This module does not load or execute them.
use serde::{Deserialize, Serialize};

pub const MAX_ADAPTERS: usize = 128;
pub const MAX_ADAPTER_NAME_CHARS: usize = 120;
pub const MAX_ADAPTER_DESCRIPTION_CHARS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterKind {
    HttpInspection,
    OpenApi,
    Observability,
    SecurityAssessment,
    BrowserAutomation,
    SecurityLab,
}

/// Closed capability labels are metadata only; they grant no permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterCapability {
    NetworkRead,
    LocalFileRead,
    AuthorizedTargetAssessment,
    IsolatedBrowser,
    IsolatedLab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterAvailability {
    Available,
    Degraded,
    Unavailable,
    Disabled,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: AdapterKind,
    pub description: String,
    pub capabilities: Vec<AdapterCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterHealth {
    pub adapter_id: String,
    pub availability: AdapterAvailability,
    pub checked_at: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestError {
    InvalidId,
    InvalidName,
    InvalidVersion,
    InvalidDescription,
    InvalidCapabilities,
}

impl AdapterManifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.id.is_empty()
            || self.id.len() > 64
            || !self.id.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
            })
        {
            return Err(ManifestError::InvalidId);
        }
        if self.name.trim().is_empty() || self.name.chars().count() > MAX_ADAPTER_NAME_CHARS {
            return Err(ManifestError::InvalidName);
        }
        let version_parts = self.version.split('.').collect::<Vec<_>>();
        if version_parts.len() != 3
            || version_parts.iter().any(|part| {
                part.is_empty()
                    || (part.len() > 1 && part.starts_with('0'))
                    || !part.bytes().all(|byte| byte.is_ascii_digit())
            })
        {
            return Err(ManifestError::InvalidVersion);
        }
        if self.description.trim().is_empty()
            || self.description.chars().count() > MAX_ADAPTER_DESCRIPTION_CHARS
        {
            return Err(ManifestError::InvalidDescription);
        }
        if self.capabilities.len() > 16 {
            return Err(ManifestError::InvalidCapabilities);
        }
        let mut unique = self.capabilities.clone();
        unique.sort_by_key(|capability| *capability as u8);
        unique.dedup();
        if unique.len() != self.capabilities.len() {
            return Err(ManifestError::InvalidCapabilities);
        }
        Ok(())
    }
}

impl AdapterHealth {
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.adapter_id.is_empty()
            || self.adapter_id.len() > 64
            || !self.adapter_id.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
            })
        {
            return Err(ManifestError::InvalidId);
        }
        if self
            .detail
            .as_ref()
            .is_some_and(|detail| detail.chars().count() > 500)
        {
            return Err(ManifestError::InvalidDescription);
        }
        if let Some(checked_at) = &self.checked_at {
            if chrono::DateTime::parse_from_rfc3339(checked_at).is_err() {
                return Err(ManifestError::InvalidDescription);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> AdapterManifest {
        AdapterManifest {
            id: "example.observability".to_owned(),
            name: "Example Observability Adapter".to_owned(),
            version: "1.2.3".to_owned(),
            kind: AdapterKind::Observability,
            description: "Reports configured metrics with source and freshness.".to_owned(),
            capabilities: vec![AdapterCapability::NetworkRead],
        }
    }

    #[test]
    fn accepts_bounded_manifest_and_serializes_stable_metadata() {
        let manifest = manifest();
        manifest.validate().unwrap();
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("example.observability"));
        assert!(json.contains("network_read"));
        assert!(serde_json::from_str::<AdapterManifest>(&json).is_ok());
    }

    #[test]
    fn rejects_invalid_identity_version_duplicate_permissions_and_unbounded_text() {
        let mut bad = manifest();
        bad.id = "../execute".to_owned();
        assert_eq!(bad.validate(), Err(ManifestError::InvalidId));
        let mut bad = manifest();
        bad.version = "1.02.3".to_owned();
        assert_eq!(bad.validate(), Err(ManifestError::InvalidVersion));
        let mut bad = manifest();
        bad.capabilities = vec![AdapterCapability::NetworkRead; 2];
        assert_eq!(bad.validate(), Err(ManifestError::InvalidCapabilities));
        let mut bad = manifest();
        bad.description = "x".repeat(MAX_ADAPTER_DESCRIPTION_CHARS + 1);
        assert_eq!(bad.validate(), Err(ManifestError::InvalidDescription));
    }

    #[test]
    fn health_distinguishes_disabled_and_unavailable_with_bounded_diagnostics() {
        let disabled = AdapterHealth {
            adapter_id: "example.observability".to_owned(),
            availability: AdapterAvailability::Disabled,
            checked_at: None,
            detail: Some("Not configured".to_owned()),
        };
        disabled.validate().unwrap();
        let mut invalid = disabled;
        invalid.checked_at = Some("later".to_owned());
        assert_eq!(invalid.validate(), Err(ManifestError::InvalidDescription));
        invalid.checked_at = None;
        invalid.detail = Some("x".repeat(501));
        assert_eq!(invalid.validate(), Err(ManifestError::InvalidDescription));
    }
}
