//! Canonical Task status/priority policy. The MVP allows explicit changes
//! between any two supported statuses; no undocumented terminal-state rule
//! is enforced. UI views must mutate status through the same service.

pub const TASK_STATUSES: [&str; 7] = [
    "backlog",
    "todo",
    "in_progress",
    "blocked",
    "in_review",
    "done",
    "cancelled",
];

pub const TASK_PRIORITIES: [&str; 4] = ["p0", "p1", "p2", "p3"];

pub fn validate_status(status: &str) -> Result<(), String> {
    if TASK_STATUSES.contains(&status) {
        Ok(())
    } else {
        Err("VALIDATION_FAILED: unsupported task status".to_string())
    }
}

pub fn validate_priority(priority: &str) -> Result<(), String> {
    if TASK_PRIORITIES.contains(&priority) {
        Ok(())
    } else {
        Err("VALIDATION_FAILED: unsupported task priority".to_string())
    }
}

/// True means a real status change; false is a valid, unchanged status.
pub fn validate_transition(previous: &str, next: &str) -> Result<bool, String> {
    validate_status(previous)?;
    validate_status(next)?;
    Ok(previous != next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_values_are_stable_and_unsupported_values_fail_closed() {
        assert_eq!(TASK_STATUSES.len(), 7);
        assert_eq!(TASK_PRIORITIES.len(), 4);
        for status in TASK_STATUSES {
            validate_status(status).expect("canonical status");
        }
        for priority in TASK_PRIORITIES {
            validate_priority(priority).expect("canonical priority");
        }
        for invalid in ["", "archived", "wip", "DONE"] {
            assert!(validate_status(invalid).unwrap_err().starts_with("VALIDATION_FAILED:"));
        }
        assert!(validate_priority("urgent").is_err());
    }

    #[test]
    fn initial_policy_allows_explicit_canonical_transitions_but_not_unknowns() {
        assert_eq!(validate_transition("todo", "in_progress"), Ok(true));
        assert_eq!(validate_transition("done", "backlog"), Ok(true));
        assert_eq!(validate_transition("cancelled", "cancelled"), Ok(false));
        assert!(validate_transition("unknown", "todo").is_err());
        assert!(validate_transition("todo", "unknown").is_err());
    }
}
