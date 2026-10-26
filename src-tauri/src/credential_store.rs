//! Secret storage boundary. Callers pass plaintext only to the provider operation;
//! references, errors and diagnostic summaries never contain secret values.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SecretReference(Uuid);

impl SecretReference {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(value: &str) -> Result<Self, CredentialStoreError> {
        Uuid::parse_str(value)
            .map(Self)
            .map_err(|_| CredentialStoreError::InvalidReference)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for SecretReference {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialStoreError {
    InvalidReference,
    NotFound,
    Unavailable,
    Rejected,
}

impl fmt::Display for CredentialStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidReference => "credential reference is invalid",
            Self::NotFound => "credential reference was not found",
            Self::Unavailable => "secure credential provider is unavailable",
            Self::Rejected => "secure credential provider rejected the operation",
        })
    }
}

impl std::error::Error for CredentialStoreError {}

/// Implementations keep secret values inside the provider boundary. Returned errors
/// are closed codes and must not include input values or provider response text.
pub trait CredentialStore {
    fn store(&self, secret: &[u8]) -> Result<SecretReference, CredentialStoreError>;
    fn retrieve(&self, reference: SecretReference) -> Result<Vec<u8>, CredentialStoreError>;
    fn delete(&self, reference: SecretReference) -> Result<(), CredentialStoreError>;
}

/// Safe operation metadata suitable for logs; never accepts a secret value.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialOperation {
    Store,
    Retrieve,
    Delete,
}

pub fn safe_operation_summary(operation: CredentialOperation) -> serde_json::Value {
    serde_json::json!({ "category": "credential_store", "operation": operation })
}

#[cfg(test)]
mod tests {
    use super::{
        safe_operation_summary, CredentialOperation, CredentialStore, CredentialStoreError,
        SecretReference,
    };
    use std::{cell::RefCell, collections::HashMap};

    #[derive(Default)]
    struct MockCredentialStore {
        secrets: RefCell<HashMap<SecretReference, Vec<u8>>>,
    }

    impl CredentialStore for MockCredentialStore {
        fn store(&self, secret: &[u8]) -> Result<SecretReference, CredentialStoreError> {
            if secret.is_empty() || secret.len() > 16 * 1024 {
                return Err(CredentialStoreError::Rejected);
            }
            let reference = SecretReference::new();
            self.secrets.borrow_mut().insert(reference, secret.to_vec());
            Ok(reference)
        }

        fn retrieve(&self, reference: SecretReference) -> Result<Vec<u8>, CredentialStoreError> {
            self.secrets
                .borrow()
                .get(&reference)
                .cloned()
                .ok_or(CredentialStoreError::NotFound)
        }

        fn delete(&self, reference: SecretReference) -> Result<(), CredentialStoreError> {
            self.secrets
                .borrow_mut()
                .remove(&reference)
                .map(|_| ())
                .ok_or(CredentialStoreError::NotFound)
        }
    }

    #[test]
    fn mock_provider_stores_retrieves_and_deletes_without_expanding_the_reference_contract() {
        let store = MockCredentialStore::default();
        let secret = b"mock-secret-sentinel";
        let reference = store.store(secret).unwrap();

        assert_eq!(store.retrieve(reference).unwrap(), secret);
        store.delete(reference).unwrap();
        assert_eq!(
            store.retrieve(reference),
            Err(CredentialStoreError::NotFound)
        );
        assert_eq!(store.delete(reference), Err(CredentialStoreError::NotFound));
    }

    #[test]
    fn references_and_safe_summaries_never_contain_secret_values() {
        let store = MockCredentialStore::default();
        let sentinel = "mock-secret-sentinel";
        let reference = store.store(sentinel.as_bytes()).unwrap();
        let serialized_reference = serde_json::to_string(&reference).unwrap();
        let summary = safe_operation_summary(CredentialOperation::Store).to_string();

        assert!(SecretReference::parse(serialized_reference.trim_matches('"')).is_ok());
        assert!(!serialized_reference.contains(sentinel));
        assert!(!summary.contains(sentinel));
        assert!(!summary.contains(&reference.as_uuid().to_string()));
        assert_eq!(
            CredentialStoreError::Rejected.to_string(),
            "secure credential provider rejected the operation"
        );
        assert!(!CredentialStoreError::Rejected
            .to_string()
            .contains(sentinel));
        assert!(SecretReference::parse(sentinel).is_err());
    }

    #[test]
    fn serialization_rejects_non_uuid_reference_and_mock_rejects_empty_or_oversized_secrets() {
        let store = MockCredentialStore::default();
        assert!(serde_json::from_str::<SecretReference>("\"not-a-uuid\"").is_err());
        assert_eq!(store.store(b""), Err(CredentialStoreError::Rejected));
        assert_eq!(
            store.store(&vec![b'x'; 16 * 1024 + 1]),
            Err(CredentialStoreError::Rejected)
        );
    }
}
