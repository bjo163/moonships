//! Read-only AI discovery over registered enabled tools. Recommendations stay inert.

use crate::{
    ai_provider::{self, AiProvider, AiProviderConfig, AiProviderError, AiRequest},
    ai_structured_output::{parse_ai_tool_request, AiStructuredOutputError},
    domain::{RiskClass, ToolDefinition, ToolRegistry},
};
use serde::Serialize;
use serde_json::Value;

const MAX_DISCOVERABLE_TOOLS: usize = 64;
const MAX_CATALOG_BYTES: usize = 512 * 1024;
const MAX_DISCOVERY_QUESTION_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredTool {
    pub id: String,
    pub version: String,
    pub api_version: u32,
    pub risk_class: RiskClass,
    pub input_schema: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AiToolRecommendation {
    pub tool: DiscoveredTool,
    pub arguments: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiDiscoveryError {
    Provider(AiProviderError),
    InvalidQuestion,
    CatalogTooLarge,
    InvalidModelOutput,
    UnknownOrUnavailableTool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryPrompt<'a> {
    instruction: &'static str,
    question: &'a str,
    available_tools: &'a [DiscoveredTool],
}

pub fn discover_tool(
    config: &AiProviderConfig,
    provider: &mut impl AiProvider,
    registry: &ToolRegistry,
    question: &str,
) -> Result<AiToolRecommendation, AiDiscoveryError> {
    if question.trim().is_empty() || question.len() > MAX_DISCOVERY_QUESTION_BYTES {
        return Err(AiDiscoveryError::InvalidQuestion);
    }
    let catalog = tool_catalog(registry)?;
    let catalog_json =
        serde_json::to_vec(&catalog).map_err(|_| AiDiscoveryError::CatalogTooLarge)?;
    if catalog_json.len() > MAX_CATALOG_BYTES {
        return Err(AiDiscoveryError::CatalogTooLarge);
    }
    let prompt = serde_json::to_string(&DiscoveryPrompt {
        instruction: "Read this registry catalog and propose one listed tool only. Return only the version-1 JSON tool request. Do not execute tools or infer unlisted IDs.",
        question,
        available_tools: &catalog,
    })
    .map_err(|_| AiDiscoveryError::CatalogTooLarge)?;
    let response = ai_provider::request(config, provider, &AiRequest { prompt })
        .map_err(AiDiscoveryError::Provider)?;
    let proposed = parse_ai_tool_request(&response.content)
        .map_err(|_: AiStructuredOutputError| AiDiscoveryError::InvalidModelOutput)?;
    let definition = registry
        .get(&proposed.tool_id)
        .filter(|tool| tool.enabled)
        .ok_or(AiDiscoveryError::UnknownOrUnavailableTool)?;
    let tool = discovery_entry(definition);
    if !catalog
        .iter()
        .any(|entry| entry.id == tool.id && entry.version == tool.version)
    {
        return Err(AiDiscoveryError::UnknownOrUnavailableTool);
    }
    Ok(AiToolRecommendation {
        tool,
        arguments: Value::Object(proposed.arguments.into_iter().collect()),
    })
}

pub fn tool_catalog(registry: &ToolRegistry) -> Result<Vec<DiscoveredTool>, AiDiscoveryError> {
    let tools: Vec<_> = registry.enabled_tools();
    if tools.len() > MAX_DISCOVERABLE_TOOLS {
        return Err(AiDiscoveryError::CatalogTooLarge);
    }
    let catalog: Vec<_> = tools.into_iter().map(discovery_entry).collect();
    let encoded = serde_json::to_vec(&catalog).map_err(|_| AiDiscoveryError::CatalogTooLarge)?;
    if encoded.len() > MAX_CATALOG_BYTES {
        return Err(AiDiscoveryError::CatalogTooLarge);
    }
    Ok(catalog)
}

fn discovery_entry(definition: &ToolDefinition) -> DiscoveredTool {
    DiscoveredTool {
        id: definition.id.clone(),
        version: definition.version.clone(),
        api_version: definition.api_version,
        risk_class: definition.risk_class,
        input_schema: definition.input_schema.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{discover_tool, tool_catalog, AiDiscoveryError};
    use crate::{
        ai_provider::{
            AiProvider, AiProviderConfig, AiProviderError, AiProviderKind, AiRequest, AiResponse,
        },
        domain::{RiskClass, ToolDefinition, ToolRegistry},
    };
    use serde_json::{json, Value};

    struct FakeProvider {
        response: String,
        prompt: Option<String>,
    }

    impl AiProvider for FakeProvider {
        fn kind(&self) -> AiProviderKind {
            AiProviderKind::Local
        }

        fn generate(&mut self, request: &AiRequest) -> Result<AiResponse, AiProviderError> {
            self.prompt = Some(request.prompt.clone());
            Ok(AiResponse {
                content: self.response.clone(),
            })
        }
    }

    fn registry() -> ToolRegistry {
        let mut registry = ToolRegistry::default();
        registry
            .register(ToolDefinition {
                id: "tasks.create".to_owned(),
                version: "1.0.0".to_owned(),
                api_version: 1,
                input_schema: json!({"type":"object","properties":{"title":{"type":"string"}}}),
                output_schema: json!({"type":"object"}),
                risk_class: RiskClass::StateChanging,
                idempotency: Default::default(),
                retry_policy: Default::default(),
                permissions: vec!["tasks.write".to_owned()],
                enabled: true,
            })
            .unwrap();
        registry
            .register(ToolDefinition {
                id: "tasks.disabled".to_owned(),
                version: "1.0.0".to_owned(),
                api_version: 1,
                input_schema: json!({"type":"object"}),
                output_schema: Value::Null,
                risk_class: RiskClass::ReadOnly,
                idempotency: Default::default(),
                retry_policy: Default::default(),
                permissions: Vec::new(),
                enabled: false,
            })
            .unwrap();
        registry
    }

    fn config() -> AiProviderConfig {
        AiProviderConfig {
            enabled: true,
            model_id: "fixture".to_owned(),
            ..AiProviderConfig::default()
        }
    }

    #[test]
    fn model_receives_only_enabled_registered_catalog_and_known_recommendation_is_inert() {
        let registry = registry();
        let mut provider = FakeProvider {
            response:
                r#"{"schemaVersion":1,"toolId":"tasks.create","arguments":{"title":"Draft"}}"#
                    .to_owned(),
            prompt: None,
        };
        let result = discover_tool(&config(), &mut provider, &registry, "Create a task").unwrap();
        assert_eq!(result.tool.id, "tasks.create");
        assert_eq!(result.arguments["title"], "Draft");
        let prompt = provider.prompt.unwrap();
        assert!(prompt.contains("tasks.create"));
        assert!(!prompt.contains("tasks.disabled"));
        assert!(!prompt.contains("tasks.write"));
    }

    #[test]
    fn invented_or_disabled_tool_ids_are_rejected_without_execution() {
        for tool_id in ["invented.command", "tasks.disabled"] {
            let mut provider = FakeProvider {
                response: format!(r#"{{"schemaVersion":1,"toolId":"{tool_id}","arguments":{{}}}}"#),
                prompt: None,
            };
            assert_eq!(
                discover_tool(&config(), &mut provider, &registry(), "Do something"),
                Err(AiDiscoveryError::UnknownOrUnavailableTool)
            );
        }
    }

    #[test]
    fn catalog_is_stable_read_only_and_rejects_invalid_questions() {
        let registry = registry();
        let catalog = tool_catalog(&registry).unwrap();
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0].id, "tasks.create");
        let mut provider = FakeProvider {
            response: String::new(),
            prompt: None,
        };
        assert_eq!(
            discover_tool(&config(), &mut provider, &registry, " "),
            Err(AiDiscoveryError::InvalidQuestion)
        );
        assert!(provider.prompt.is_none());
    }
}
