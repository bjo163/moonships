//! Exact, inert AI action previews and approval bindings.

use crate::{ai_registry_discovery::AiToolRecommendation, domain::ToolRegistry};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const MAX_PREVIEW_ARGUMENT_BYTES: usize = 64 * 1024;
const MAX_TARGET_BYTES: usize = 256;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiActionPreview {
    pub preview_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub api_version: u32,
    pub target: Option<String>,
    pub arguments: Value,
    pub binding_hash: String,
}

/// Approval fields are private and this type is not deserializable, so serialized
/// renderer/model data cannot manufacture an approved action object.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiActionApproval {
    preview_id: String,
    binding_hash: String,
    approved_at: String,
}

impl AiActionApproval {
    pub fn approved_at(&self) -> &str {
        &self.approved_at
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiActionPreviewError {
    ToolUnavailable,
    InvalidTarget,
    InvalidPreview,
    ConfirmationMismatch,
    ApprovalMismatch,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BindingPayload<'a> {
    schema_version: u32,
    tool_id: &'a str,
    tool_version: &'a str,
    api_version: u32,
    target: &'a Option<String>,
    arguments: &'a Value,
}

pub fn create_preview(
    recommendation: &AiToolRecommendation,
    registry: &ToolRegistry,
    target: Option<String>,
) -> Result<AiActionPreview, AiActionPreviewError> {
    let registered = registry
        .get(&recommendation.tool.id)
        .filter(|tool| tool.enabled)
        .filter(|tool| {
            tool.version == recommendation.tool.version
                && tool.api_version == recommendation.tool.api_version
                && tool.risk_class == recommendation.tool.risk_class
        })
        .ok_or(AiActionPreviewError::ToolUnavailable)?;
    if target.as_ref().is_some_and(|value| {
        value.is_empty() || value.len() > MAX_TARGET_BYTES || value.chars().any(char::is_control)
    }) {
        return Err(AiActionPreviewError::InvalidTarget);
    }
    let arguments = recommendation.arguments.clone();
    if !arguments.is_object()
        || serde_json::to_vec(&arguments)
            .map(|serialized| serialized.len() > MAX_PREVIEW_ARGUMENT_BYTES)
            .unwrap_or(true)
    {
        return Err(AiActionPreviewError::InvalidPreview);
    }
    let mut preview = AiActionPreview {
        preview_id: Uuid::new_v4().to_string(),
        tool_id: registered.id.clone(),
        tool_version: registered.version.clone(),
        api_version: registered.api_version,
        target,
        arguments,
        binding_hash: String::new(),
    };
    preview.binding_hash = compute_binding_hash(&preview)?;
    Ok(preview)
}

pub fn approve_preview(
    preview: &AiActionPreview,
    displayed_preview_id: &str,
    approved_at: DateTime<Utc>,
) -> Result<AiActionApproval, AiActionPreviewError> {
    validate_preview(preview)?;
    if preview.preview_id != displayed_preview_id {
        return Err(AiActionPreviewError::ConfirmationMismatch);
    }
    Ok(AiActionApproval {
        preview_id: preview.preview_id.clone(),
        binding_hash: preview.binding_hash.clone(),
        approved_at: approved_at.to_rfc3339_opts(SecondsFormat::Millis, true),
    })
}

pub fn verify_approval(
    approval: &AiActionApproval,
    preview: &AiActionPreview,
) -> Result<(), AiActionPreviewError> {
    validate_preview(preview)?;
    if approval.preview_id != preview.preview_id || approval.binding_hash != preview.binding_hash {
        return Err(AiActionPreviewError::ApprovalMismatch);
    }
    Ok(())
}

fn validate_preview(preview: &AiActionPreview) -> Result<(), AiActionPreviewError> {
    if Uuid::parse_str(&preview.preview_id)
        .map(|id| id.to_string() != preview.preview_id)
        .unwrap_or(true)
        || preview.tool_id.is_empty()
        || preview.tool_id.len() > 128
        || preview.tool_version.is_empty()
        || preview.tool_version.len() > 64
        || preview.api_version == 0
        || preview.target.as_ref().is_some_and(|value| {
            value.is_empty()
                || value.len() > MAX_TARGET_BYTES
                || value.chars().any(char::is_control)
        })
        || !preview.arguments.is_object()
        || serde_json::to_vec(&preview.arguments)
            .map(|serialized| serialized.len() > MAX_PREVIEW_ARGUMENT_BYTES)
            .unwrap_or(true)
        || compute_binding_hash(preview)? != preview.binding_hash
    {
        return Err(AiActionPreviewError::InvalidPreview);
    }
    Ok(())
}

fn compute_binding_hash(preview: &AiActionPreview) -> Result<String, AiActionPreviewError> {
    let payload = BindingPayload {
        schema_version: 1,
        tool_id: &preview.tool_id,
        tool_version: &preview.tool_version,
        api_version: preview.api_version,
        target: &preview.target,
        arguments: &preview.arguments,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|_| AiActionPreviewError::InvalidPreview)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::{approve_preview, create_preview, verify_approval, AiActionPreviewError};
    use crate::{
        ai_registry_discovery::AiToolRecommendation,
        domain::{RiskClass, ToolDefinition, ToolRegistry},
    };
    use chrono::{DateTime, Utc};
    use serde_json::{json, Value};

    fn fixture() -> (ToolRegistry, AiToolRecommendation) {
        let mut registry = ToolRegistry::default();
        registry
            .register(ToolDefinition {
                id: "tasks.create".to_owned(),
                version: "1.0.0".to_owned(),
                api_version: 2,
                input_schema: json!({"type":"object"}),
                output_schema: Value::Null,
                risk_class: RiskClass::StateChanging,
                idempotency: Default::default(),
                retry_policy: Default::default(),
                permissions: vec!["tasks.write".to_owned()],
                enabled: true,
            })
            .unwrap();
        let recommendation = AiToolRecommendation {
            tool: crate::ai_registry_discovery::DiscoveredTool {
                id: "tasks.create".to_owned(),
                version: "1.0.0".to_owned(),
                api_version: 2,
                risk_class: RiskClass::StateChanging,
                input_schema: json!({"type":"object"}),
            },
            arguments: json!({"title":"A draft"}),
        };
        (registry, recommendation)
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-26T12:00:00Z")
            .unwrap()
            .to_utc()
    }

    #[test]
    fn approval_binds_exact_preview_identity_and_can_be_verified() {
        let (registry, recommendation) = fixture();
        let preview =
            create_preview(&recommendation, &registry, Some("project:alpha".to_owned())).unwrap();
        let approval = approve_preview(&preview, &preview.preview_id, now()).unwrap();
        assert_eq!(verify_approval(&approval, &preview), Ok(()));
        assert!(approval.approved_at().starts_with("2026-10-26T12:00:00"));
    }

    #[test]
    fn changed_arguments_tool_version_target_or_preview_id_invalidate_approval() {
        let (registry, recommendation) = fixture();
        let preview =
            create_preview(&recommendation, &registry, Some("project:alpha".to_owned())).unwrap();
        let approval = approve_preview(&preview, &preview.preview_id, now()).unwrap();
        for mutate in 0..4 {
            let mut changed = preview.clone();
            match mutate {
                0 => changed.arguments["title"] = json!("Different"),
                1 => changed.tool_version = "2.0.0".to_owned(),
                2 => changed.target = Some("project:beta".to_owned()),
                _ => changed.preview_id = uuid::Uuid::new_v4().to_string(),
            }
            assert!(verify_approval(&approval, &changed).is_err());
        }
    }

    #[test]
    fn confirmation_requires_the_exact_displayed_preview_and_registered_version() {
        let (registry, recommendation) = fixture();
        let preview = create_preview(&recommendation, &registry, None).unwrap();
        assert_eq!(
            approve_preview(&preview, "different-preview", now()),
            Err(AiActionPreviewError::ConfirmationMismatch)
        );
        let mut stale = recommendation;
        stale.tool.version = "9.9.9".to_owned();
        assert_eq!(
            create_preview(&stale, &registry, None),
            Err(AiActionPreviewError::ToolUnavailable)
        );
    }
}
