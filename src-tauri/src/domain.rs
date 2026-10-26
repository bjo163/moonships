use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    WaitingApproval,
    Succeeded,
    Partial,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskClass {
    ReadOnly,
    LowRisk,
    StateChanging,
    Destructive,
    Privileged,
    Remote,
}

impl RiskClass {
    fn requires_approval(self) -> bool {
        matches!(
            self,
            Self::StateChanging | Self::Destructive | Self::Privileged | Self::Remote
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencyClass {
    Idempotent,
    NonIdempotent,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "mode",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum RetryPolicy {
    #[default]
    Never,
    TransientFailures {
        max_retries: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolDefinition {
    pub id: String,
    pub version: String,
    pub api_version: u32,
    pub input_schema: Value,
    pub output_schema: Value,
    pub risk_class: RiskClass,
    #[serde(default)]
    pub idempotency: IdempotencyClass,
    #[serde(default)]
    pub retry_policy: RetryPolicy,
    pub permissions: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolResult {
    pub status: ToolOutcome,
    pub summary: String,
    pub output: Option<Value>,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutcome {
    Succeeded,
    Partial,
    Failed,
    Cancelled,
}

#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, ToolDefinition>,
}

impl ToolRegistry {
    pub fn register(&mut self, definition: ToolDefinition) -> Result<(), String> {
        if definition.id.trim().is_empty() {
            return Err("Tool ID must not be empty".to_string());
        }
        if definition.version.trim().is_empty() || definition.api_version == 0 {
            return Err("Tool and API versions must be explicit".to_string());
        }
        definition.validate_retry_policy()?;
        if self.tools.contains_key(&definition.id) {
            return Err(format!("Tool ID already registered: {}", definition.id));
        }
        self.tools.insert(definition.id.clone(), definition);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&ToolDefinition> {
        self.tools.get(id)
    }

    /// Returns registered enabled tools in stable ID order for read-only discovery.
    pub fn enabled_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.values().filter(|tool| tool.enabled).collect()
    }
}

impl ToolDefinition {
    pub fn validate_retry_policy(&self) -> Result<(), String> {
        let RetryPolicy::TransientFailures { max_retries } = self.retry_policy else {
            return Ok(());
        };
        if !(1..=5).contains(&max_retries) {
            return Err("Retry count must be between 1 and 5".to_string());
        }
        if self.idempotency != IdempotencyClass::Idempotent {
            return Err("Automatic retry requires an explicitly idempotent tool".to_string());
        }
        if matches!(
            self.risk_class,
            RiskClass::Destructive | RiskClass::Privileged | RiskClass::Remote
        ) {
            return Err(
                "Automatic retry is disabled for high-impact tool risk classes".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRequest {
    pub tool_id: String,
    pub required_capabilities: Vec<String>,
    pub risk_class: RiskClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    Allowed,
    Denied { reason: String },
    ApprovalRequired { reason: String },
}

pub fn evaluate_policy(
    request: &PermissionRequest,
    granted_capabilities: &BTreeSet<String>,
    approval_granted: bool,
) -> PolicyDecision {
    let missing: Vec<&str> = request
        .required_capabilities
        .iter()
        .map(String::as_str)
        .filter(|capability| !granted_capabilities.contains(*capability))
        .collect();

    if !missing.is_empty() {
        return PolicyDecision::Denied {
            reason: format!("Missing required capabilities: {}", missing.join(", ")),
        };
    }

    if request.risk_class.requires_approval() && !approval_granted {
        return PolicyDecision::ApprovalRequired {
            reason: format!("Tool '{}' requires explicit approval", request.tool_id),
        };
    }

    PolicyDecision::Allowed
}

pub const CURRENT_CONFIG_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceProfile {
    Developer,
    NetworkNoc,
    Sysadmin,
    SecurityResearch,
    IntelligenceResearch,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigLogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppConfig {
    pub schema_version: u32,
    pub theme: ThemeMode,
    pub profile: WorkspaceProfile,
    pub log_level: ConfigLogLevel,
    pub hide_to_tray: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_CONFIG_SCHEMA_VERSION,
            theme: ThemeMode::Dark,
            profile: WorkspaceProfile::Developer,
            log_level: ConfigLogLevel::Info,
            hide_to_tray: false,
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != CURRENT_CONFIG_SCHEMA_VERSION {
            return Err(format!(
                "Unsupported config schema version {}; expected {}",
                self.schema_version, CURRENT_CONFIG_SCHEMA_VERSION
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn job_status_serialization_names_are_stable() {
        let cases = [
            (JobStatus::Queued, "\"queued\""),
            (JobStatus::Running, "\"running\""),
            (JobStatus::WaitingApproval, "\"waiting_approval\""),
            (JobStatus::Succeeded, "\"succeeded\""),
            (JobStatus::Partial, "\"partial\""),
            (JobStatus::Failed, "\"failed\""),
            (JobStatus::Cancelled, "\"cancelled\""),
            (JobStatus::TimedOut, "\"timed_out\""),
        ];
        for (status, expected) in cases {
            assert_eq!(serde_json::to_string(&status).unwrap(), expected);
        }
    }

    #[test]
    fn sample_tool_can_be_registered_and_duplicate_ids_are_rejected() {
        let definition = ToolDefinition {
            id: "workspace.inspect".to_string(),
            version: "1.0.0".to_string(),
            api_version: 1,
            input_schema: json!({"type": "object"}),
            output_schema: json!({"type": "object"}),
            risk_class: RiskClass::ReadOnly,
            idempotency: IdempotencyClass::Unknown,
            retry_policy: RetryPolicy::Never,
            permissions: vec!["workspace.read".to_string()],
            enabled: true,
        };
        let mut registry = ToolRegistry::default();
        registry.register(definition.clone()).unwrap();
        assert_eq!(registry.get("workspace.inspect"), Some(&definition));
        assert!(registry.register(definition).is_err());
    }

    #[test]
    fn tool_retry_defaults_are_never_even_for_destructive_operations() {
        let mut value = json!({
            "id": "storage.delete",
            "version": "1.0.0",
            "apiVersion": 1,
            "inputSchema": {"type": "object"},
            "outputSchema": {"type": "object"},
            "riskClass": "destructive",
            "permissions": ["filesystem.delete"],
            "enabled": true
        });
        let definition: ToolDefinition = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(definition.idempotency, IdempotencyClass::Unknown);
        assert_eq!(definition.retry_policy, RetryPolicy::Never);
        definition.validate_retry_policy().unwrap();

        value["retryPolicy"] = json!({"mode":"transient_failures","maxRetries":1});
        let definition: ToolDefinition = serde_json::from_value(value).unwrap();
        assert!(definition.validate_retry_policy().is_err());
    }

    #[test]
    fn explicit_retries_require_idempotency_bounded_attempts_and_safe_risk() {
        let mut definition = ToolDefinition {
            id: "workspace.refresh".to_string(),
            version: "1.0.0".to_string(),
            api_version: 1,
            input_schema: json!({"type":"object"}),
            output_schema: json!({"type":"object"}),
            risk_class: RiskClass::StateChanging,
            idempotency: IdempotencyClass::Idempotent,
            retry_policy: RetryPolicy::TransientFailures { max_retries: 2 },
            permissions: Vec::new(),
            enabled: true,
        };
        assert!(definition.validate_retry_policy().is_ok());

        definition.idempotency = IdempotencyClass::Unknown;
        assert!(definition.validate_retry_policy().is_err());
        definition.idempotency = IdempotencyClass::Idempotent;
        definition.retry_policy = RetryPolicy::TransientFailures { max_retries: 6 };
        assert!(definition.validate_retry_policy().is_err());
        definition.retry_policy = RetryPolicy::TransientFailures { max_retries: 1 };
        definition.risk_class = RiskClass::Destructive;
        assert!(definition.validate_retry_policy().is_err());
    }

    #[test]
    fn permission_policy_covers_allowed_denied_and_approval_required() {
        let grants = BTreeSet::from(["workspace.read".to_string()]);
        let read_request = PermissionRequest {
            tool_id: "workspace.inspect".to_string(),
            required_capabilities: vec!["workspace.read".to_string()],
            risk_class: RiskClass::ReadOnly,
        };
        assert_eq!(
            evaluate_policy(&read_request, &grants, false),
            PolicyDecision::Allowed
        );

        let missing_grant = PermissionRequest {
            required_capabilities: vec!["filesystem.delete".to_string()],
            ..read_request.clone()
        };
        assert!(matches!(
            evaluate_policy(&missing_grant, &grants, true),
            PolicyDecision::Denied { .. }
        ));

        let change_request = PermissionRequest {
            risk_class: RiskClass::StateChanging,
            ..read_request
        };
        assert!(matches!(
            evaluate_policy(&change_request, &grants, false),
            PolicyDecision::ApprovalRequired { .. }
        ));
        assert_eq!(
            evaluate_policy(&change_request, &grants, true),
            PolicyDecision::Allowed
        );
    }

    #[test]
    fn config_defaults_are_valid_and_schema_mismatch_fails_closed() {
        let config = AppConfig::default();
        assert_eq!(config.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
        assert_eq!(config.theme, ThemeMode::Dark);
        assert_eq!(config.profile, WorkspaceProfile::Developer);
        assert_eq!(config.log_level, ConfigLogLevel::Info);
        assert!(!config.hide_to_tray);
        assert!(config.validate().is_ok());

        let mut invalid = config;
        invalid.schema_version = 99;
        assert!(invalid.validate().is_err());
        assert!(serde_json::from_str::<AppConfig>(r#"{"schemaVersion":1,"theme":"neon","profile":"developer","logLevel":"info","hideToTray":false}"#).is_err());
    }
}
