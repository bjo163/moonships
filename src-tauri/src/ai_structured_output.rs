//! Strict parser for untrusted AI-proposed tool requests. Parsing never executes a tool.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const AI_TOOL_REQUEST_SCHEMA_VERSION: u32 = 1;
pub const MAX_AI_TOOL_REQUEST_BYTES: usize = 64 * 1024;
const MAX_VALUE_DEPTH: usize = 16;
const MAX_VALUE_NODES: usize = 4096;
const MAX_ARGUMENTS: usize = 128;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiToolRequest {
    pub schema_version: u32,
    pub tool_id: String,
    pub arguments: BTreeMap<String, Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiStructuredOutputError {
    TooLarge,
    InvalidJson,
    UnsupportedSchemaVersion,
    InvalidToolId,
    InvalidArguments,
}

pub fn parse_ai_tool_request(input: &str) -> Result<AiToolRequest, AiStructuredOutputError> {
    if input.len() > MAX_AI_TOOL_REQUEST_BYTES {
        return Err(AiStructuredOutputError::TooLarge);
    }
    let request: AiToolRequest =
        serde_json::from_str(input).map_err(|_| AiStructuredOutputError::InvalidJson)?;
    if request.schema_version != AI_TOOL_REQUEST_SCHEMA_VERSION {
        return Err(AiStructuredOutputError::UnsupportedSchemaVersion);
    }
    if !valid_identifier(&request.tool_id) {
        return Err(AiStructuredOutputError::InvalidToolId);
    }
    if request.arguments.len() > MAX_ARGUMENTS
        || request.arguments.keys().any(|key| !valid_identifier(key))
    {
        return Err(AiStructuredOutputError::InvalidArguments);
    }
    let mut node_count = 0;
    for value in request.arguments.values() {
        validate_value(value, 0, &mut node_count)?;
    }
    Ok(request)
}

fn validate_value(
    value: &Value,
    depth: usize,
    node_count: &mut usize,
) -> Result<(), AiStructuredOutputError> {
    *node_count += 1;
    if depth > MAX_VALUE_DEPTH || *node_count > MAX_VALUE_NODES {
        return Err(AiStructuredOutputError::InvalidArguments);
    }
    match value {
        Value::String(text) if text.len() > MAX_AI_TOOL_REQUEST_BYTES => {
            Err(AiStructuredOutputError::InvalidArguments)
        }
        Value::Array(values) => {
            if values.len() > MAX_ARGUMENTS {
                return Err(AiStructuredOutputError::InvalidArguments);
            }
            for item in values {
                validate_value(item, depth + 1, node_count)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            if values.len() > MAX_ARGUMENTS || values.keys().any(|key| !valid_identifier(key)) {
                return Err(AiStructuredOutputError::InvalidArguments);
            }
            for item in values.values() {
                validate_value(item, depth + 1, node_count)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{parse_ai_tool_request, AiStructuredOutputError, MAX_AI_TOOL_REQUEST_BYTES};
    use serde_json::json;
    use std::cell::Cell;

    #[test]
    fn valid_bounded_request_parses_without_executing() {
        let request = parse_ai_tool_request(
            r#"{"schemaVersion":1,"toolId":"tasks.create","arguments":{"title":"Fixture task","urgent":false}}"#,
        )
        .unwrap();
        assert_eq!(request.tool_id, "tasks.create");
        assert_eq!(request.arguments["urgent"], json!(false));
    }

    #[test]
    fn malformed_or_unknown_output_never_reaches_a_tool_callback() {
        let tool_calls = Cell::new(0);
        let malformed =
            r#"{"schemaVersion":1,"toolId":"../../shell","arguments":{},"command":"run"}"#;
        if let Ok(request) = parse_ai_tool_request(malformed) {
            let _ = request;
            tool_calls.set(tool_calls.get() + 1);
        }
        assert_eq!(tool_calls.get(), 0);
        assert_eq!(
            parse_ai_tool_request(malformed),
            Err(AiStructuredOutputError::InvalidJson)
        );
    }

    #[test]
    fn rejects_oversized_unsupported_and_deep_values() {
        assert_eq!(
            parse_ai_tool_request(&"x".repeat(MAX_AI_TOOL_REQUEST_BYTES + 1)),
            Err(AiStructuredOutputError::TooLarge)
        );
        assert_eq!(
            parse_ai_tool_request(r#"{"schemaVersion":2,"toolId":"tools.read","arguments":{}}"#),
            Err(AiStructuredOutputError::UnsupportedSchemaVersion)
        );
        let nested = format!("{}null{}", "[".repeat(18), "]".repeat(18));
        let input = format!(
            "{{\"schemaVersion\":1,\"toolId\":\"tools.read\",\"arguments\":{{\"x\":{nested}}}}}"
        );
        assert_eq!(
            parse_ai_tool_request(&input),
            Err(AiStructuredOutputError::InvalidArguments)
        );
    }
}
