//! Strict, versioned workflow definition contract. Execution and registry validation
//! are deliberately owned by later workflow tasks.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const WORKFLOW_SCHEMA_VERSION: u32 = 1;
pub const MAX_WORKFLOW_DOCUMENT_BYTES: usize = 1024 * 1024;
pub const MAX_WORKFLOW_STEPS: usize = 128;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowDefinition {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: u32,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowStep {
    pub id: String,
    pub tool_id: String,
    pub tool_version: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub inputs: BTreeMap<String, WorkflowValue>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum WorkflowValue {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<WorkflowValue>),
    Object(BTreeMap<String, WorkflowValue>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowSchemaError {
    DocumentTooLarge,
    InvalidJson,
    UnsupportedSchemaVersion,
    InvalidDefinition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowValidationError {
    InvalidSchema,
    TooManySteps,
    DuplicateStepId,
    UnknownDependency,
    DuplicateDependency,
    DependencyCycle,
}

impl WorkflowDefinition {
    pub fn parse_json(input: &str) -> Result<Self, WorkflowSchemaError> {
        if input.len() > MAX_WORKFLOW_DOCUMENT_BYTES {
            return Err(WorkflowSchemaError::DocumentTooLarge);
        }
        let definition: Self =
            serde_json::from_str(input).map_err(|_| WorkflowSchemaError::InvalidJson)?;
        definition.validate_shape()?;
        Ok(definition)
    }

    pub fn validate_shape(&self) -> Result<(), WorkflowSchemaError> {
        if self.schema_version != WORKFLOW_SCHEMA_VERSION {
            return Err(WorkflowSchemaError::UnsupportedSchemaVersion);
        }
        if !valid_identifier(&self.id, 128)
            || self.name.trim().is_empty()
            || self.name.chars().count() > 120
            || self.name.chars().any(char::is_control)
            || self.version == 0
            || self.steps.is_empty()
        {
            return Err(WorkflowSchemaError::InvalidDefinition);
        }
        for step in &self.steps {
            if !valid_identifier(&step.id, 128)
                || !valid_identifier(&step.tool_id, 128)
                || !valid_tool_version(&step.tool_version)
                || step.inputs.len() > 128
                || step.inputs.keys().any(|key| !valid_identifier(key, 128))
            {
                return Err(WorkflowSchemaError::InvalidDefinition);
            }
            for value in step.inputs.values() {
                if !valid_value(value, 0) {
                    return Err(WorkflowSchemaError::InvalidDefinition);
                }
            }
        }
        Ok(())
    }
}

/// Validate step identities and dependency DAG structure without resolving tools
/// or executing any step.
pub fn validate_workflow(workflow: &WorkflowDefinition) -> Result<(), WorkflowValidationError> {
    workflow_execution_order(workflow).map(|_| ())
}

/// Return one deterministic topological order, preferring the earliest declared
/// step whenever several steps are ready.
pub fn workflow_execution_order(
    workflow: &WorkflowDefinition,
) -> Result<Vec<usize>, WorkflowValidationError> {
    workflow
        .validate_shape()
        .map_err(|_| WorkflowValidationError::InvalidSchema)?;
    if workflow.steps.len() > MAX_WORKFLOW_STEPS {
        return Err(WorkflowValidationError::TooManySteps);
    }

    let mut indices = BTreeMap::new();
    for (index, step) in workflow.steps.iter().enumerate() {
        if indices.insert(step.id.as_str(), index).is_some() {
            return Err(WorkflowValidationError::DuplicateStepId);
        }
    }

    let mut indegree = vec![0_usize; workflow.steps.len()];
    let mut dependents = vec![Vec::new(); workflow.steps.len()];
    for (index, step) in workflow.steps.iter().enumerate() {
        let mut seen = BTreeSet::new();
        for dependency in &step.depends_on {
            if !seen.insert(dependency.as_str()) {
                return Err(WorkflowValidationError::DuplicateDependency);
            }
            let Some(&dependency_index) = indices.get(dependency.as_str()) else {
                return Err(WorkflowValidationError::UnknownDependency);
            };
            indegree[index] += 1;
            dependents[dependency_index].push(index);
        }
    }

    let mut ready: BTreeSet<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, count)| (*count == 0).then_some(index))
        .collect();
    let mut order = Vec::with_capacity(workflow.steps.len());
    while let Some(index) = ready.pop_first() {
        order.push(index);
        for dependent in &dependents[index] {
            indegree[*dependent] -= 1;
            if indegree[*dependent] == 0 {
                ready.insert(*dependent);
            }
        }
    }
    if order.len() != workflow.steps.len() {
        return Err(WorkflowValidationError::DependencyCycle);
    }
    Ok(order)
}

fn valid_identifier(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

fn valid_tool_version(value: &str) -> bool {
    let mut parts = value.split('.');
    let valid_part = |part: Option<&str>| {
        part.is_some_and(|part| {
            !part.is_empty()
                && part.len() <= 9
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == "0" || !part.starts_with('0'))
        })
    };
    valid_part(parts.next())
        && valid_part(parts.next())
        && valid_part(parts.next())
        && parts.next().is_none()
}

fn valid_value(value: &WorkflowValue, depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    match value {
        WorkflowValue::String(value) => value.len() <= 16 * 1024,
        WorkflowValue::Integer(_)
        | WorkflowValue::Number(_)
        | WorkflowValue::Boolean(_)
        | WorkflowValue::Null => true,
        WorkflowValue::Array(values) => {
            values.len() <= 256 && values.iter().all(|value| valid_value(value, depth + 1))
        }
        WorkflowValue::Object(values) => {
            values.len() <= 128
                && values
                    .iter()
                    .all(|(key, value)| valid_identifier(key, 128) && valid_value(value, depth + 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        validate_workflow, workflow_execution_order, WorkflowDefinition, WorkflowSchemaError,
        WorkflowValidationError, WorkflowValue, MAX_WORKFLOW_STEPS,
    };

    const VALID: &str = r#"{
      "schemaVersion": 1,
      "id": "ops.daily-check",
      "name": "Daily check",
      "version": 1,
      "steps": [{
        "id": "inspect",
        "toolId": "workspace.inspect",
        "toolVersion": "1.2.0",
        "inputs": {
          "includeArchived": {"type":"boolean","value":false},
          "limit": {"type":"integer","value":10},
          "query": {"type":"string","value":"active"}
        }
      }]
    }"#;

    #[test]
    fn parses_versioned_tool_references_and_explicitly_typed_inputs() {
        let workflow = WorkflowDefinition::parse_json(VALID).unwrap();
        assert_eq!(workflow.schema_version, 1);
        assert_eq!(workflow.steps[0].tool_id, "workspace.inspect");
        assert_eq!(workflow.steps[0].tool_version, "1.2.0");
        assert_eq!(
            workflow.steps[0].inputs["limit"],
            WorkflowValue::Integer(10)
        );
        assert!(serde_json::to_string(&workflow)
            .unwrap()
            .contains("\"type\":\"boolean\""));
    }

    #[test]
    fn rejects_malformed_unknown_fields_and_unknown_typed_values() {
        assert_eq!(
            WorkflowDefinition::parse_json("{"),
            Err(WorkflowSchemaError::InvalidJson)
        );
        assert!(WorkflowDefinition::parse_json(
            &VALID.replace("\"version\": 1", "\"version\": 1, \"future\": true")
        )
        .is_err());
        assert!(WorkflowDefinition::parse_json(
            &VALID.replace("\"type\":\"boolean\"", "\"type\":\"binary\"")
        )
        .is_err());
        assert!(WorkflowDefinition::parse_json(&VALID.replace(
            "\"type\":\"integer\",\"value\":10",
            "\"type\":\"integer\",\"value\":\"10\""
        ))
        .is_err());
    }

    #[test]
    fn rejects_unsupported_versions_bad_tool_versions_and_unsafe_bounds() {
        assert_eq!(
            WorkflowDefinition::parse_json(
                &VALID.replace("\"schemaVersion\": 1", "\"schemaVersion\": 2")
            ),
            Err(WorkflowSchemaError::UnsupportedSchemaVersion)
        );
        assert_eq!(
            WorkflowDefinition::parse_json(&VALID.replace("1.2.0", "1.02.0")),
            Err(WorkflowSchemaError::InvalidDefinition)
        );
        assert_eq!(
            WorkflowDefinition::parse_json(&VALID.replace("workspace.inspect", "bad tool id")),
            Err(WorkflowSchemaError::InvalidDefinition)
        );
        let mut empty_steps: serde_json::Value = serde_json::from_str(VALID).unwrap();
        empty_steps["steps"] = serde_json::json!([]);
        assert_eq!(
            WorkflowDefinition::parse_json(&empty_steps.to_string()),
            Err(WorkflowSchemaError::InvalidDefinition)
        );
        let oversized = " ".repeat(super::MAX_WORKFLOW_DOCUMENT_BYTES + 1);
        assert_eq!(
            WorkflowDefinition::parse_json(&oversized),
            Err(WorkflowSchemaError::DocumentTooLarge)
        );
    }

    #[test]
    fn validator_accepts_dag_and_rejects_cycles_invalid_or_duplicate_dependencies() {
        let mut workflow: WorkflowDefinition = serde_json::from_str(VALID).unwrap();
        let mut second = workflow.steps[0].clone();
        second.id = "summarize".to_owned();
        second.depends_on = vec!["inspect".to_owned()];
        workflow.steps.push(second);
        assert_eq!(validate_workflow(&workflow), Ok(()));
        assert_eq!(workflow_execution_order(&workflow), Ok(vec![0, 1]));

        workflow.steps[0].depends_on = vec!["summarize".to_owned()];
        assert_eq!(
            validate_workflow(&workflow),
            Err(WorkflowValidationError::DependencyCycle)
        );
        workflow.steps[0].depends_on = vec!["missing".to_owned()];
        assert_eq!(
            validate_workflow(&workflow),
            Err(WorkflowValidationError::UnknownDependency)
        );
        workflow.steps[0].depends_on = vec!["summarize".to_owned(), "summarize".to_owned()];
        assert_eq!(
            validate_workflow(&workflow),
            Err(WorkflowValidationError::DuplicateDependency)
        );
    }

    #[test]
    fn validator_rejects_oversized_and_duplicate_step_sets() {
        let mut workflow: WorkflowDefinition = serde_json::from_str(VALID).unwrap();
        let template = workflow.steps[0].clone();
        workflow.steps = (0..=MAX_WORKFLOW_STEPS)
            .map(|index| {
                let mut step = template.clone();
                step.id = format!("step-{index}");
                step
            })
            .collect();
        assert_eq!(
            validate_workflow(&workflow),
            Err(WorkflowValidationError::TooManySteps)
        );

        workflow.steps.truncate(2);
        workflow.steps[1].id = workflow.steps[0].id.clone();
        assert_eq!(
            validate_workflow(&workflow),
            Err(WorkflowValidationError::DuplicateStepId)
        );
    }
}
