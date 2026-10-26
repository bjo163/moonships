//! Deterministic sequential workflow engine over explicitly registered handlers.
//! No handlers are registered by the desktop application in this task.

use crate::workflow_schema::{
    workflow_execution_order, WorkflowDefinition, WorkflowStep, WorkflowValidationError,
    WorkflowValue,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowToolError {
    Failed,
    Unavailable,
    Rejected,
}

/// Trusted in-process handler interface. Any production integration must enforce
/// tool registry, capabilities, scope and approval before doing consequential work.
pub trait WorkflowToolHandler {
    fn execute(
        &mut self,
        inputs: &BTreeMap<String, WorkflowValue>,
    ) -> Result<WorkflowValue, WorkflowToolError>;
}

#[derive(Default)]
pub struct WorkflowToolRegistry {
    handlers: BTreeMap<(String, String), Box<dyn WorkflowToolHandler>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryError {
    InvalidToolReference,
    DuplicateToolVersion,
}

impl WorkflowToolRegistry {
    pub fn register(
        &mut self,
        tool_id: &str,
        tool_version: &str,
        handler: impl WorkflowToolHandler + 'static,
    ) -> Result<(), RegistryError> {
        if tool_id.is_empty()
            || tool_id.len() > 128
            || !tool_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
            || !valid_version(tool_version)
        {
            return Err(RegistryError::InvalidToolReference);
        }
        let key = (tool_id.to_owned(), tool_version.to_owned());
        if self.handlers.contains_key(&key) {
            return Err(RegistryError::DuplicateToolVersion);
        }
        self.handlers.insert(key, Box::new(handler));
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowRunStatus {
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorkflowStepOutcome {
    Succeeded { output: WorkflowValue },
    Failed { error: WorkflowToolError },
    Skipped { after_step_id: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowStepResult {
    pub step_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub outcome: WorkflowStepOutcome,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowRunResult {
    pub workflow_id: String,
    pub workflow_version: u32,
    pub status: WorkflowRunStatus,
    pub steps: Vec<WorkflowStepResult>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowRunError {
    InvalidWorkflow(WorkflowValidationError),
}

pub fn run_workflow(
    workflow: &WorkflowDefinition,
    registry: &mut WorkflowToolRegistry,
) -> Result<WorkflowRunResult, WorkflowRunError> {
    let order = workflow_execution_order(workflow).map_err(WorkflowRunError::InvalidWorkflow)?;
    let mut results = Vec::with_capacity(workflow.steps.len());
    let mut failed_step: Option<String> = None;

    for index in order {
        let step = &workflow.steps[index];
        let outcome = if let Some(failed_id) = &failed_step {
            WorkflowStepOutcome::Skipped {
                after_step_id: failed_id.clone(),
            }
        } else {
            match execute_step(registry, step) {
                Ok(output) => WorkflowStepOutcome::Succeeded { output },
                Err(error) => {
                    failed_step = Some(step.id.clone());
                    WorkflowStepOutcome::Failed { error }
                }
            }
        };
        results.push(WorkflowStepResult {
            step_id: step.id.clone(),
            tool_id: step.tool_id.clone(),
            tool_version: step.tool_version.clone(),
            outcome,
        });
    }

    Ok(WorkflowRunResult {
        workflow_id: workflow.id.clone(),
        workflow_version: workflow.version,
        status: if failed_step.is_some() {
            WorkflowRunStatus::Failed
        } else {
            WorkflowRunStatus::Succeeded
        },
        steps: results,
    })
}

fn execute_step(
    registry: &mut WorkflowToolRegistry,
    step: &WorkflowStep,
) -> Result<WorkflowValue, WorkflowToolError> {
    registry
        .handlers
        .get_mut(&(step.tool_id.clone(), step.tool_version.clone()))
        .ok_or(WorkflowToolError::Unavailable)?
        .execute(&step.inputs)
}

fn valid_version(version: &str) -> bool {
    let mut parts = version.split('.');
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

#[cfg(test)]
mod tests {
    use super::{
        run_workflow, RegistryError, WorkflowRunStatus, WorkflowStepOutcome, WorkflowToolError,
        WorkflowToolHandler, WorkflowToolRegistry,
    };
    use crate::workflow_schema::{WorkflowDefinition, WorkflowValue};
    use std::{cell::RefCell, collections::BTreeMap};

    struct FixtureHandler {
        label: String,
        calls: std::rc::Rc<RefCell<Vec<String>>>,
        fail: bool,
    }

    impl WorkflowToolHandler for FixtureHandler {
        fn execute(
            &mut self,
            inputs: &BTreeMap<String, WorkflowValue>,
        ) -> Result<WorkflowValue, WorkflowToolError> {
            self.calls.borrow_mut().push(self.label.clone());
            if self.fail {
                return Err(WorkflowToolError::Failed);
            }
            Ok(inputs
                .get("value")
                .cloned()
                .unwrap_or(WorkflowValue::String(self.label.clone())))
        }
    }

    fn workflow() -> WorkflowDefinition {
        let value = serde_json::json!({
            "schemaVersion": 1,
            "id": "fixture.sequence",
            "name": "Fixture sequence",
            "version": 2,
            "steps": [
                {"id":"second","toolId":"fixture.echo","toolVersion":"1.0.0","dependsOn":["first"],"inputs":{"value":{"type":"string","value":"two"}}},
                {"id":"first","toolId":"fixture.echo","toolVersion":"1.0.0","inputs":{"value":{"type":"string","value":"one"}}},
                {"id":"independent","toolId":"fixture.other","toolVersion":"2.0.0","inputs":{}}
            ]
        });
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn fixture_workflow_executes_in_stable_topological_order_and_returns_expected_values() {
        let calls = std::rc::Rc::new(RefCell::new(Vec::new()));
        let mut registry = WorkflowToolRegistry::default();
        registry
            .register(
                "fixture.echo",
                "1.0.0",
                FixtureHandler {
                    label: "echo".to_owned(),
                    calls: calls.clone(),
                    fail: false,
                },
            )
            .unwrap();
        registry
            .register(
                "fixture.other",
                "2.0.0",
                FixtureHandler {
                    label: "other".to_owned(),
                    calls: calls.clone(),
                    fail: false,
                },
            )
            .unwrap();

        let result = run_workflow(&workflow(), &mut registry).unwrap();
        assert_eq!(*calls.borrow(), vec!["echo", "echo", "other"]);
        assert_eq!(result.status, WorkflowRunStatus::Succeeded);
        assert_eq!(
            result
                .steps
                .iter()
                .map(|step| step.step_id.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "second", "independent"]
        );
        assert_eq!(
            result.steps[0].outcome,
            WorkflowStepOutcome::Succeeded {
                output: WorkflowValue::String("one".to_owned())
            }
        );
        assert_eq!(
            result.steps[1].outcome,
            WorkflowStepOutcome::Succeeded {
                output: WorkflowValue::String("two".to_owned())
            }
        );
    }

    #[test]
    fn failed_step_stops_later_execution_and_exact_tool_version_is_required() {
        let calls = std::rc::Rc::new(RefCell::new(Vec::new()));
        let mut registry = WorkflowToolRegistry::default();
        registry
            .register(
                "fixture.echo",
                "0.9.0",
                FixtureHandler {
                    label: "wrong-version".to_owned(),
                    calls: calls.clone(),
                    fail: false,
                },
            )
            .unwrap();
        registry
            .register(
                "fixture.other",
                "2.0.0",
                FixtureHandler {
                    label: "other".to_owned(),
                    calls: calls.clone(),
                    fail: false,
                },
            )
            .unwrap();

        let result = run_workflow(&workflow(), &mut registry).unwrap();
        assert_eq!(result.status, WorkflowRunStatus::Failed);
        assert_eq!(
            result.steps[0].outcome,
            WorkflowStepOutcome::Failed {
                error: WorkflowToolError::Unavailable
            }
        );
        assert!(matches!(
            result.steps[1].outcome,
            WorkflowStepOutcome::Skipped { .. }
        ));
        assert_eq!(*calls.borrow(), Vec::<String>::new());
    }

    #[test]
    fn duplicate_handler_registration_is_rejected() {
        let calls = std::rc::Rc::new(RefCell::new(Vec::new()));
        let handler = || FixtureHandler {
            label: "same".to_owned(),
            calls: calls.clone(),
            fail: false,
        };
        let mut registry = WorkflowToolRegistry::default();
        registry
            .register("fixture.echo", "1.0.0", handler())
            .unwrap();
        assert_eq!(
            registry.register("fixture.echo", "1.0.0", handler()),
            Err(RegistryError::DuplicateToolVersion)
        );
    }
}
