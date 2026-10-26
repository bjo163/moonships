//! Provider-neutral AI boundary and non-secret configuration schema.
//! No provider implementation or network request is registered by this module.

use crate::credential_store::SecretReference;
use serde::{Deserialize, Serialize};

pub const AI_PROVIDER_CONFIG_SCHEMA_VERSION: u32 = 1;
pub const MAX_AI_MESSAGE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderKind {
    Local,
    Remote,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiProviderConfig {
    pub schema_version: u32,
    pub enabled: bool,
    pub provider: AiProviderKind,
    pub model_id: String,
    pub endpoint_url: Option<String>,
    pub credential_reference: Option<SecretReference>,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            schema_version: AI_PROVIDER_CONFIG_SCHEMA_VERSION,
            enabled: false,
            provider: AiProviderKind::Local,
            model_id: String::new(),
            endpoint_url: None,
            credential_reference: None,
        }
    }
}

impl AiProviderConfig {
    pub fn validate(&self) -> Result<(), AiProviderError> {
        if self.schema_version != AI_PROVIDER_CONFIG_SCHEMA_VERSION
            || self.model_id.len() > 128
            || (self.enabled && self.model_id.trim().is_empty())
        {
            return Err(AiProviderError::InvalidConfiguration);
        }
        match self.provider {
            AiProviderKind::Local => {
                if self.endpoint_url.is_some() || self.credential_reference.is_some() {
                    return Err(AiProviderError::InvalidConfiguration);
                }
            }
            AiProviderKind::Remote => {
                let endpoint = self
                    .endpoint_url
                    .as_deref()
                    .ok_or(AiProviderError::InvalidConfiguration)?;
                if endpoint.len() > 2048 {
                    return Err(AiProviderError::InvalidConfiguration);
                }
                let url = reqwest::Url::parse(endpoint)
                    .map_err(|_| AiProviderError::InvalidConfiguration)?;
                if url.scheme() != "https"
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
                {
                    return Err(AiProviderError::InvalidConfiguration);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiRequest {
    pub prompt: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiResponse {
    pub content: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiProviderError {
    Disabled,
    InvalidConfiguration,
    InvalidRequest,
    ProviderMismatch,
    Unavailable,
    Rejected,
    TimedOut,
    Failed,
}

/// Implementations must keep credentials inside their secure provider boundary,
/// return closed errors, and never treat model text as authority or instructions.
pub trait AiProvider {
    fn kind(&self) -> AiProviderKind;
    fn generate(&mut self, request: &AiRequest) -> Result<AiResponse, AiProviderError>;
}

pub fn request(
    config: &AiProviderConfig,
    provider: &mut impl AiProvider,
    request: &AiRequest,
) -> Result<AiResponse, AiProviderError> {
    config.validate()?;
    if !config.enabled {
        return Err(AiProviderError::Disabled);
    }
    if provider.kind() != config.provider {
        return Err(AiProviderError::ProviderMismatch);
    }
    if request.prompt.is_empty() || request.prompt.len() > MAX_AI_MESSAGE_BYTES {
        return Err(AiProviderError::InvalidRequest);
    }
    let response = provider.generate(request)?;
    if response.content.len() > MAX_AI_MESSAGE_BYTES {
        return Err(AiProviderError::Rejected);
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::{
        request, AiProvider, AiProviderConfig, AiProviderError, AiProviderKind, AiRequest,
        AiResponse, MAX_AI_MESSAGE_BYTES,
    };
    use std::cell::Cell;

    struct FakeProvider {
        calls: Cell<usize>,
        response: String,
    }

    struct FailedProvider;

    impl AiProvider for FailedProvider {
        fn kind(&self) -> AiProviderKind {
            AiProviderKind::Local
        }

        fn generate(&mut self, _request: &AiRequest) -> Result<AiResponse, AiProviderError> {
            Err(AiProviderError::Unavailable)
        }
    }

    impl AiProvider for FakeProvider {
        fn kind(&self) -> AiProviderKind {
            AiProviderKind::Local
        }

        fn generate(&mut self, request: &AiRequest) -> Result<AiResponse, AiProviderError> {
            self.calls.set(self.calls.get() + 1);
            Ok(AiResponse {
                content: format!("{}: {}", self.response, request.prompt),
            })
        }
    }

    fn enabled_local_config() -> AiProviderConfig {
        AiProviderConfig {
            enabled: true,
            provider: AiProviderKind::Local,
            model_id: "fixture-model".to_owned(),
            ..AiProviderConfig::default()
        }
    }

    #[test]
    fn fake_provider_exercises_core_without_network() {
        let mut provider = FakeProvider {
            calls: Cell::new(0),
            response: "fixture".to_owned(),
        };
        let response = request(
            &enabled_local_config(),
            &mut provider,
            &AiRequest {
                prompt: "hello".to_owned(),
            },
        )
        .unwrap();
        assert_eq!(response.content, "fixture: hello");
        assert_eq!(provider.calls.get(), 1);
    }

    #[test]
    fn provider_failure_is_returned_without_fallback() {
        let mut provider = FailedProvider;
        let result = request(
            &enabled_local_config(),
            &mut provider,
            &AiRequest {
                prompt: "summarize".to_owned(),
            },
        );
        assert_eq!(result, Err(AiProviderError::Unavailable));
    }

    #[test]
    fn disabled_or_mismatched_provider_never_generates() {
        let mut provider = FakeProvider {
            calls: Cell::new(0),
            response: "unused".to_owned(),
        };
        assert_eq!(
            request(
                &AiProviderConfig::default(),
                &mut provider,
                &AiRequest {
                    prompt: "hello".to_owned()
                }
            ),
            Err(AiProviderError::Disabled)
        );
        let mut mismatch = enabled_local_config();
        mismatch.provider = AiProviderKind::Remote;
        mismatch.endpoint_url = Some("https://provider.example/v1".to_owned());
        assert_eq!(
            request(
                &mismatch,
                &mut provider,
                &AiRequest {
                    prompt: "hello".to_owned()
                }
            ),
            Err(AiProviderError::ProviderMismatch)
        );
        assert_eq!(provider.calls.get(), 0);
    }

    #[test]
    fn config_rejects_plaintext_secret_fields_and_unsafe_endpoints() {
        assert!(serde_json::from_str::<AiProviderConfig>(
            r#"{"schemaVersion":1,"enabled":true,"provider":"remote","modelId":"m","endpointUrl":"https://provider.example","apiKey":"secret"}"#
        )
        .is_err());

        let mut remote = enabled_local_config();
        remote.provider = AiProviderKind::Remote;
        remote.endpoint_url = Some("http://provider.example".to_owned());
        assert_eq!(
            remote.validate(),
            Err(AiProviderError::InvalidConfiguration)
        );
        remote.endpoint_url = Some("https://user:pass@provider.example".to_owned());
        assert_eq!(
            remote.validate(),
            Err(AiProviderError::InvalidConfiguration)
        );
        remote.endpoint_url = Some("https://provider.example/path?token=secret".to_owned());
        assert_eq!(
            remote.validate(),
            Err(AiProviderError::InvalidConfiguration)
        );
    }

    #[test]
    fn message_sizes_are_bounded_and_oversized_provider_response_is_rejected() {
        let mut provider = FakeProvider {
            calls: Cell::new(0),
            response: "x".repeat(MAX_AI_MESSAGE_BYTES),
        };
        assert_eq!(
            request(
                &enabled_local_config(),
                &mut provider,
                &AiRequest {
                    prompt: "prompt".to_owned()
                }
            ),
            Err(AiProviderError::Rejected)
        );
        assert_eq!(
            request(
                &enabled_local_config(),
                &mut provider,
                &AiRequest {
                    prompt: "x".repeat(MAX_AI_MESSAGE_BYTES + 1)
                }
            ),
            Err(AiProviderError::InvalidRequest)
        );
    }
}
