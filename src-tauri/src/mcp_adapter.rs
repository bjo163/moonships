//! Feature-gated MCP contract spike. This validates scoped discovery only;
//! it has no transport, process launch, network call, or tool execution path.

use rmcp::model::{ListToolsResult, Tool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const MCP_PROTOCOL_VERSION: &str = "2026-07-28";
pub const MCP_SDK_VERSION: &str = "3.7.0";
pub const MAX_MCP_RESPONSE_BYTES: usize = 1024 * 1024;
pub const MAX_MCP_TOOLS_PER_RESPONSE: usize = 64;
pub const MAX_MCP_TOOL_SCHEMA_BYTES: usize = 32 * 1024;
pub const MAX_MCP_SCOPE_TOOLS: usize = 32;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpToolScope {
    pub enabled: bool,
    pub server_id: String,
    pub allowed_tool_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopedMcpTool {
    pub name: String,
    pub input_schema: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpAdapterError {
    Disabled,
    InvalidScope,
    ServerMismatch,
    InvalidProtocolResponse,
    ResponseTooLarge,
    DuplicateToolName,
    ToolLimitExceeded,
    SchemaLimitExceeded,
    ToolNotAllowed,
}

impl McpToolScope {
    pub fn validate(&self) -> Result<(), McpAdapterError> {
        if !self.enabled {
            return Ok(());
        }
        if !valid_name(&self.server_id)
            || self.allowed_tool_names.is_empty()
            || self.allowed_tool_names.len() > MAX_MCP_SCOPE_TOOLS
        {
            return Err(McpAdapterError::InvalidScope);
        }
        let mut names = BTreeSet::new();
        if self
            .allowed_tool_names
            .iter()
            .any(|name| !valid_name(name) || !names.insert(name))
        {
            return Err(McpAdapterError::InvalidScope);
        }
        Ok(())
    }
}

/// Accepts an MCP `tools/list` result body and keeps only explicitly scoped names.
/// Tool descriptions and annotations are not forwarded because the remote server controls them.
pub fn parse_scoped_tool_list(
    scope: &McpToolScope,
    connected_server_id: &str,
    result_json: &str,
) -> Result<Vec<ScopedMcpTool>, McpAdapterError> {
    scope.validate()?;
    if !scope.enabled {
        return Err(McpAdapterError::Disabled);
    }
    if connected_server_id != scope.server_id {
        return Err(McpAdapterError::ServerMismatch);
    }
    if result_json.len() > MAX_MCP_RESPONSE_BYTES {
        return Err(McpAdapterError::ResponseTooLarge);
    }
    let response: ListToolsResult =
        serde_json::from_str(result_json).map_err(|_| McpAdapterError::InvalidProtocolResponse)?;
    if response.tools.len() > MAX_MCP_TOOLS_PER_RESPONSE {
        return Err(McpAdapterError::ToolLimitExceeded);
    }
    let allowed: BTreeSet<&str> = scope
        .allowed_tool_names
        .iter()
        .map(String::as_str)
        .collect();
    let mut seen = BTreeSet::new();
    let mut selected = Vec::new();
    for tool in response.tools {
        let name = tool.name.to_string();
        if !valid_name(&name) || !seen.insert(name.clone()) {
            return Err(McpAdapterError::DuplicateToolName);
        }
        if !allowed.contains(name.as_str()) {
            continue;
        }
        selected.push(project_tool(tool, name)?);
    }
    Ok(selected)
}

pub fn authorize_tool_call(
    scope: &McpToolScope,
    connected_server_id: &str,
    tool_name: &str,
) -> Result<(), McpAdapterError> {
    scope.validate()?;
    if !scope.enabled {
        return Err(McpAdapterError::Disabled);
    }
    if connected_server_id != scope.server_id {
        return Err(McpAdapterError::ServerMismatch);
    }
    if !scope
        .allowed_tool_names
        .iter()
        .any(|name| name == tool_name)
    {
        return Err(McpAdapterError::ToolNotAllowed);
    }
    Ok(())
}

fn project_tool(tool: Tool, name: String) -> Result<ScopedMcpTool, McpAdapterError> {
    let input_schema = Value::Object((*tool.input_schema).clone());
    let encoded =
        serde_json::to_vec(&input_schema).map_err(|_| McpAdapterError::InvalidProtocolResponse)?;
    if encoded.len() > MAX_MCP_TOOL_SCHEMA_BYTES {
        return Err(McpAdapterError::SchemaLimitExceeded);
    }
    Ok(ScopedMcpTool { name, input_schema })
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{
        authorize_tool_call, parse_scoped_tool_list, McpAdapterError, McpToolScope,
        MCP_PROTOCOL_VERSION, MCP_SDK_VERSION,
    };
    use rmcp::model::ProtocolVersion;

    fn scope(enabled: bool) -> McpToolScope {
        McpToolScope {
            enabled,
            server_id: "reviewed-server".to_owned(),
            allowed_tool_names: vec!["tasks.read".to_owned()],
        }
    }

    #[test]
    fn pinned_sdk_matches_protocol_and_parses_conformant_tools_list_result() {
        assert_eq!(MCP_PROTOCOL_VERSION, "2026-07-28");
        assert_eq!(MCP_SDK_VERSION, "3.7.0");
        assert_eq!(ProtocolVersion::V_2026_07_28.as_str(), MCP_PROTOCOL_VERSION);
        let response = r#"{"tools":[{"name":"tasks.read","description":"Untrusted remote description","inputSchema":{"type":"object","properties":{"id":{"type":"string"}}}},{"name":"system.shell","inputSchema":{"type":"object"}}]}"#;
        let tools = parse_scoped_tool_list(&scope(true), "reviewed-server", response).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "tasks.read");
        assert!(!serde_json::to_string(&tools)
            .unwrap()
            .contains("Untrusted remote description"));
    }

    #[test]
    fn disabled_wrong_server_and_unapproved_names_are_rejected() {
        let response = r#"{"tools":[{"name":"tasks.read","inputSchema":{"type":"object"}}]}"#;
        assert_eq!(
            parse_scoped_tool_list(&scope(false), "reviewed-server", response),
            Err(McpAdapterError::Disabled)
        );
        assert_eq!(
            parse_scoped_tool_list(&scope(true), "other-server", response),
            Err(McpAdapterError::ServerMismatch)
        );
        assert_eq!(
            authorize_tool_call(&scope(true), "reviewed-server", "system.shell"),
            Err(McpAdapterError::ToolNotAllowed)
        );
    }

    #[test]
    fn malformed_duplicate_oversized_and_unbounded_protocol_responses_fail_closed() {
        assert_eq!(
            parse_scoped_tool_list(&scope(true), "reviewed-server", "not json"),
            Err(McpAdapterError::InvalidProtocolResponse)
        );
        let duplicate = r#"{"tools":[{"name":"tasks.read","inputSchema":{"type":"object"}},{"name":"tasks.read","inputSchema":{"type":"object"}}]}"#;
        assert_eq!(
            parse_scoped_tool_list(&scope(true), "reviewed-server", duplicate),
            Err(McpAdapterError::DuplicateToolName)
        );
        let invalid_scope = McpToolScope {
            enabled: true,
            server_id: "reviewed-server".to_owned(),
            allowed_tool_names: vec!["tasks.read".to_owned(), "tasks.read".to_owned()],
        };
        assert_eq!(
            parse_scoped_tool_list(&invalid_scope, "reviewed-server", duplicate),
            Err(McpAdapterError::InvalidScope)
        );
        let too_large_response = " ".repeat(super::MAX_MCP_RESPONSE_BYTES + 1);
        assert_eq!(
            parse_scoped_tool_list(&scope(true), "reviewed-server", &too_large_response),
            Err(McpAdapterError::ResponseTooLarge)
        );
        let large_schema = format!(
            r#"{{"tools":[{{"name":"tasks.read","inputSchema":{{"description":"{}"}}}}]}}"#,
            "x".repeat(super::MAX_MCP_TOOL_SCHEMA_BYTES + 1)
        );
        assert_eq!(
            parse_scoped_tool_list(&scope(true), "reviewed-server", &large_schema),
            Err(McpAdapterError::SchemaLimitExceeded)
        );
    }
}
