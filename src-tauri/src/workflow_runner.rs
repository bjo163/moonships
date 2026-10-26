//! Deterministic sequential workflow engine over explicitly registered handlers.
//! No handlers are registered by the desktop application in this task.

use crate::workflow_schema::{
    workflow_execution_order, WorkflowDefinition, WorkflowErrorPolicy, WorkflowStep,
    WorkflowValidationError, WorkflowValue, WORKFLOW_SCHEMA_VERSION,
};
use rusqlite::Connection;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowToolError {
    Failed,
    Unavailable,
    Rejected,
    TimedOut,
}

/// Trusted in-process handler interface. Handlers must cooperatively enforce the
/// supplied deadline; a non-cooperative synchronous handler cannot be preempted.
/// Production integrations must also enforce registry, capability, scope and
/// approval policy before doing consequential work.
pub trait WorkflowToolHandler {
    fn execute(
        &mut self,
        inputs: &BTreeMap<String, WorkflowValue>,
        deadline: Instant,
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
    pub duration_ms: u64,
    pub outcome: WorkflowStepOutcome,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowRunResult {
    pub run_id: String,
    pub correlation_id: String,
    pub workflow_id: String,
    pub workflow_version: u32,
    pub status: WorkflowRunStatus,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub steps: Vec<WorkflowStepResult>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowRunError {
    InvalidWorkflow(WorkflowValidationError),
    ExecutionPolicyRequired,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorkflowRunAndRecordError {
    Run(WorkflowRunError),
    History(String),
}

pub fn run_workflow_and_record(
    connection: &mut Connection,
    workflow: &WorkflowDefinition,
    registry: &mut WorkflowToolRegistry,
) -> Result<WorkflowRunResult, WorkflowRunAndRecordError> {
    let result = run_workflow(workflow, registry).map_err(WorkflowRunAndRecordError::Run)?;
    crate::workflow_run_history::record(connection, &result)
        .map_err(WorkflowRunAndRecordError::History)?;
    Ok(result)
}

pub fn run_workflow(
    workflow: &WorkflowDefinition,
    registry: &mut WorkflowToolRegistry,
) -> Result<WorkflowRunResult, WorkflowRunError> {
    if workflow.schema_version != WORKFLOW_SCHEMA_VERSION {
        return Err(WorkflowRunError::ExecutionPolicyRequired);
    }
    let order = workflow_execution_order(workflow).map_err(WorkflowRunError::InvalidWorkflow)?;
    let run_started = Instant::now();
    let started_at = chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now())
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let run_id = uuid::Uuid::new_v4().to_string();
    let correlation_id = uuid::Uuid::new_v4().to_string();
    let mut results = Vec::with_capacity(workflow.steps.len());
    let mut failed_steps = BTreeSet::new();
    let mut stopped_after: Option<String> = None;
    let mut has_failure = false;

    for index in order {
        let step_started = Instant::now();
        let step = &workflow.steps[index];
        let outcome = if let Some(failed_id) = &stopped_after {
            WorkflowStepOutcome::Skipped {
                after_step_id: failed_id.clone(),
            }
        } else {
            if let Some(blocked_by) = step
                .depends_on
                .iter()
                .find(|dependency| failed_steps.contains(dependency.as_str()))
            {
                failed_steps.insert(step.id.clone());
                WorkflowStepOutcome::Skipped {
                    after_step_id: blocked_by.clone(),
                }
            } else {
                match execute_step(registry, step) {
                    Ok(output) => WorkflowStepOutcome::Succeeded { output },
                    Err(error) => {
                        failed_steps.insert(step.id.clone());
                        has_failure = true;
                        if step.on_error == Some(WorkflowErrorPolicy::Stop) {
                            stopped_after = Some(step.id.clone());
                        }
                        WorkflowStepOutcome::Failed { error }
                    }
                }
            }
        };
        results.push(WorkflowStepResult {
            step_id: step.id.clone(),
            tool_id: step.tool_id.clone(),
            tool_version: step.tool_version.clone(),
            duration_ms: step_started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            outcome,
        });
    }

    Ok(WorkflowRunResult {
        run_id,
        correlation_id,
        workflow_id: workflow.id.clone(),
        workflow_version: workflow.version,
        status: if has_failure {
            WorkflowRunStatus::Failed
        } else {
            WorkflowRunStatus::Succeeded
        },
        started_at,
        finished_at: chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now())
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        duration_ms: run_started.elapsed().as_millis().min(u64::MAX as u128) as u64,
        steps: results,
    })
}

fn execute_step(
    registry: &mut WorkflowToolRegistry,
    step: &WorkflowStep,
) -> Result<WorkflowValue, WorkflowToolError> {
    let deadline =
        Instant::now() + std::time::Duration::from_millis(step.timeout_ms.unwrap_or_default());
    let result = registry
        .handlers
        .get_mut(&(step.tool_id.clone(), step.tool_version.clone()))
        .ok_or(WorkflowToolError::Unavailable)?
        .execute(&step.inputs, deadline);
    if Instant::now() >= deadline {
        Err(WorkflowToolError::TimedOut)
    } else {
        result
    }
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
        run_workflow, run_workflow_and_record, RegistryError, WorkflowRunStatus,
        WorkflowStepOutcome, WorkflowToolError, WorkflowToolHandler, WorkflowToolRegistry,
    };
    use crate::workflow_schema::{WorkflowDefinition, WorkflowErrorPolicy, WorkflowValue};
    use std::{cell::RefCell, collections::BTreeMap, time::Instant};

    struct FixtureHandler {
        label: String,
        calls: std::rc::Rc<RefCell<Vec<String>>>,
        failure: Option<WorkflowToolError>,
        deadlines: std::rc::Rc<RefCell<Vec<bool>>>,
        delay: std::time::Duration,
    }

    impl WorkflowToolHandler for FixtureHandler {
        fn execute(
            &mut self,
            inputs: &BTreeMap<String, WorkflowValue>,
            deadline: Instant,
        ) -> Result<WorkflowValue, WorkflowToolError> {
            self.calls.borrow_mut().push(self.label.clone());
            self.deadlines.borrow_mut().push(deadline > Instant::now());
            std::thread::sleep(self.delay);
            if let Some(error) = self.failure {
                return Err(error);
            }
            Ok(inputs
                .get("value")
                .cloned()
                .unwrap_or(WorkflowValue::String(self.label.clone())))
        }
    }

    fn workflow() -> WorkflowDefinition {
        let value = serde_json::json!({
            "schemaVersion": 2,
            "id": "fixture.sequence",
            "name": "Fixture sequence",
            "version": 2,
            "steps": [
                {"id":"second","toolId":"fixture.echo","toolVersion":"1.0.0","dependsOn":["first"],"timeoutMs":1000,"onError":"continue","inputs":{"value":{"type":"string","value":"two"}}},
                {"id":"first","toolId":"fixture.echo","toolVersion":"1.0.0","timeoutMs":1000,"onError":"continue","inputs":{"value":{"type":"string","value":"one"}}},
                {"id":"independent","toolId":"fixture.other","toolVersion":"2.0.0","timeoutMs":1000,"onError":"stop","inputs":{}}
            ]
        });
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn fixture_workflow_executes_in_stable_topological_order_and_returns_expected_values() {
        let calls = std::rc::Rc::new(RefCell::new(Vec::new()));
        let deadlines = std::rc::Rc::new(RefCell::new(Vec::new()));
        let mut registry = WorkflowToolRegistry::default();
        registry
            .register(
                "fixture.echo",
                "1.0.0",
                FixtureHandler {
                    label: "echo".to_owned(),
                    calls: calls.clone(),
                    failure: None,
                    deadlines: deadlines.clone(),
                    delay: std::time::Duration::ZERO,
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
                    failure: None,
                    deadlines: deadlines.clone(),
                    delay: std::time::Duration::ZERO,
                },
            )
            .unwrap();

        let result = run_workflow(&workflow(), &mut registry).unwrap();
        assert_eq!(*calls.borrow(), vec!["echo", "echo", "other"]);
        assert_eq!(result.status, WorkflowRunStatus::Succeeded);
        assert!(deadlines
            .borrow()
            .iter()
            .all(|deadline_was_future| *deadline_was_future));
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
    fn continue_policy_runs_independent_steps_after_timeout_and_skips_dependents() {
        let calls = std::rc::Rc::new(RefCell::new(Vec::new()));
        let deadlines = std::rc::Rc::new(RefCell::new(Vec::new()));
        let mut registry = WorkflowToolRegistry::default();
        registry
            .register(
                "fixture.echo",
                "1.0.0",
                FixtureHandler {
                    label: "echo".to_owned(),
                    calls: calls.clone(),
                    failure: Some(WorkflowToolError::TimedOut),
                    deadlines: deadlines.clone(),
                    delay: std::time::Duration::ZERO,
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
                    failure: None,
                    deadlines,
                    delay: std::time::Duration::ZERO,
                },
            )
            .unwrap();

        let mut definition = workflow();
        definition
            .steps
            .iter_mut()
            .find(|step| step.id == "first")
            .unwrap()
            .on_error = Some(WorkflowErrorPolicy::Continue);
        let result = run_workflow(&definition, &mut registry).unwrap();
        assert_eq!(result.status, WorkflowRunStatus::Failed);
        assert_eq!(result.steps[0].step_id, "first");
        assert_eq!(
            result.steps[0].outcome,
            WorkflowStepOutcome::Failed {
                error: WorkflowToolError::TimedOut
            }
        );
        assert_eq!(
            result.steps[1].outcome,
            WorkflowStepOutcome::Skipped {
                after_step_id: "first".to_owned()
            }
        );
        assert!(matches!(
            result.steps[2].outcome,
            WorkflowStepOutcome::Succeeded { .. }
        ));
        assert_eq!(*calls.borrow(), vec!["echo", "other"]);
    }

    #[test]
    fn deadline_exceeded_by_a_handler_is_reported_as_timeout() {
        let calls = std::rc::Rc::new(RefCell::new(Vec::new()));
        let deadlines = std::rc::Rc::new(RefCell::new(Vec::new()));
        let mut registry = WorkflowToolRegistry::default();
        registry
            .register(
                "fixture.echo",
                "1.0.0",
                FixtureHandler {
                    label: "echo".to_owned(),
                    calls: calls.clone(),
                    failure: None,
                    deadlines: deadlines.clone(),
                    delay: std::time::Duration::from_millis(125),
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
                    failure: None,
                    deadlines,
                    delay: std::time::Duration::ZERO,
                },
            )
            .unwrap();

        let mut definition = workflow();
        let first = definition
            .steps
            .iter_mut()
            .find(|step| step.id == "first")
            .unwrap();
        first.timeout_ms = Some(100);
        first.on_error = Some(WorkflowErrorPolicy::Continue);
        let result = run_workflow(&definition, &mut registry).unwrap();

        assert_eq!(
            result.steps[0].outcome,
            WorkflowStepOutcome::Failed {
                error: WorkflowToolError::TimedOut
            }
        );
        assert!(matches!(
            result.steps[1].outcome,
            WorkflowStepOutcome::Skipped { .. }
        ));
        assert!(matches!(
            result.steps[2].outcome,
            WorkflowStepOutcome::Succeeded { .. }
        ));
        assert_eq!(*calls.borrow(), vec!["echo", "other"]);
    }

    #[test]
    fn failed_step_stops_later_execution_and_exact_tool_version_is_required() {
        let calls = std::rc::Rc::new(RefCell::new(Vec::new()));
        let deadlines = std::rc::Rc::new(RefCell::new(Vec::new()));
        let mut registry = WorkflowToolRegistry::default();
        registry
            .register(
                "fixture.echo",
                "0.9.0",
                FixtureHandler {
                    label: "wrong-version".to_owned(),
                    calls: calls.clone(),
                    failure: None,
                    deadlines: deadlines.clone(),
                    delay: std::time::Duration::ZERO,
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
                    failure: None,
                    deadlines: deadlines.clone(),
                    delay: std::time::Duration::ZERO,
                },
            )
            .unwrap();

        let mut definition = workflow();
        definition
            .steps
            .iter_mut()
            .find(|step| step.id == "first")
            .unwrap()
            .on_error = Some(WorkflowErrorPolicy::Stop);
        let result = run_workflow(&definition, &mut registry).unwrap();
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
        let deadlines = std::rc::Rc::new(RefCell::new(Vec::new()));
        let handler = || FixtureHandler {
            label: "same".to_owned(),
            calls: calls.clone(),
            failure: None,
            deadlines: deadlines.clone(),
            delay: std::time::Duration::ZERO,
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

    #[test]
    fn run_and_record_persists_correlation_status_duration_and_redacted_output() {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        crate::run_migrations(&mut connection).unwrap();
        let calls = std::rc::Rc::new(RefCell::new(Vec::new()));
        let deadlines = std::rc::Rc::new(RefCell::new(Vec::new()));
        let mut registry = WorkflowToolRegistry::default();
        registry
            .register(
                "fixture.echo",
                "1.0.0",
                FixtureHandler {
                    label: "echo".to_owned(),
                    calls: calls.clone(),
                    failure: None,
                    deadlines: deadlines.clone(),
                    delay: std::time::Duration::ZERO,
                },
            )
            .unwrap();
        registry
            .register(
                "fixture.other",
                "2.0.0",
                FixtureHandler {
                    label: "other".to_owned(),
                    calls,
                    failure: None,
                    deadlines,
                    delay: std::time::Duration::ZERO,
                },
            )
            .unwrap();

        let result = run_workflow_and_record(&mut connection, &workflow(), &mut registry).unwrap();
        let detail = crate::workflow_run_history::get(&connection, &result.run_id).unwrap();
        assert_eq!(detail.run.status, "succeeded");
        assert_eq!(detail.run.correlation_id, result.correlation_id);
        assert_eq!(detail.run.duration_ms, result.duration_ms);
        assert_eq!(
            detail.steps[0].output_summary.as_deref(),
            Some("[REDACTED]")
        );
        assert!(!serde_json::to_string(&detail).unwrap().contains("one"));
    }
}
