use std::time::{Duration, SystemTime};

/// Stable, application-owned locations. These are identifiers, not renderer-provided paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupRoot {
    AppTemporary,
    AppCache,
    UserTemporary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupCategory {
    TemporaryFiles,
    ApplicationCache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupFileKind {
    RegularFile,
    Directory,
    Link,
    Other,
}

/// Declarative rule contract. The root is resolved only by a future native platform adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupRule {
    pub id: String,
    pub version: u32,
    pub category: CleanupCategory,
    pub allowed_root: CleanupRoot,
    pub minimum_age: Duration,
    pub maximum_item_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleValidationError {
    InvalidId,
    MissingVersion,
    CategoryRootMismatch,
    MissingMinimumAge,
    MissingItemSizeLimit,
}

impl CleanupRule {
    pub fn validate(&self) -> Result<(), RuleValidationError> {
        if !is_valid_rule_id(&self.id) {
            return Err(RuleValidationError::InvalidId);
        }
        if self.version == 0 {
            return Err(RuleValidationError::MissingVersion);
        }
        let matching_root = matches!(
            (self.category, self.allowed_root),
            (CleanupCategory::TemporaryFiles, CleanupRoot::AppTemporary)
                | (CleanupCategory::TemporaryFiles, CleanupRoot::UserTemporary)
                | (CleanupCategory::ApplicationCache, CleanupRoot::AppCache)
        );
        if !matching_root {
            return Err(RuleValidationError::CategoryRootMismatch);
        }
        if self.minimum_age.is_zero() {
            return Err(RuleValidationError::MissingMinimumAge);
        }
        if self.maximum_item_bytes == 0 {
            return Err(RuleValidationError::MissingItemSizeLimit);
        }
        Ok(())
    }
}

fn is_valid_rule_id(id: &str) -> bool {
    let mut characters = id.chars();
    matches!(characters.next(), Some('a'..='z' | '0'..='9'))
        && characters.all(|character| matches!(character, 'a'..='z' | '0'..='9' | '.' | '-'))
}

/// Metadata-only scan result. It deliberately contains no absolute path or deletion handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CleanupCandidateMetadata {
    pub root: CleanupRoot,
    pub kind: CleanupFileKind,
    pub modified_at: SystemTime,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EligibilityExclusion {
    InvalidRule,
    WrongRoot,
    NotRegularFile,
    ModifiedTimeInFuture,
    TooRecent,
    ItemTooLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EligibilityDecision {
    Eligible,
    Excluded(EligibilityExclusion),
}

/// Evaluates a rule against already-observed metadata. This function performs no filesystem I/O.
pub fn evaluate_eligibility(
    rule: &CleanupRule,
    candidate: CleanupCandidateMetadata,
    observed_at: SystemTime,
) -> EligibilityDecision {
    if rule.validate().is_err() {
        return EligibilityDecision::Excluded(EligibilityExclusion::InvalidRule);
    }
    if candidate.root != rule.allowed_root {
        return EligibilityDecision::Excluded(EligibilityExclusion::WrongRoot);
    }
    if candidate.kind != CleanupFileKind::RegularFile {
        return EligibilityDecision::Excluded(EligibilityExclusion::NotRegularFile);
    }
    let age = match observed_at.duration_since(candidate.modified_at) {
        Ok(age) => age,
        Err(_) => return EligibilityDecision::Excluded(EligibilityExclusion::ModifiedTimeInFuture),
    };
    if age < rule.minimum_age {
        return EligibilityDecision::Excluded(EligibilityExclusion::TooRecent);
    }
    if candidate.size_bytes > rule.maximum_item_bytes {
        return EligibilityDecision::Excluded(EligibilityExclusion::ItemTooLarge);
    }
    EligibilityDecision::Eligible
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_eligibility, CleanupCandidateMetadata, CleanupCategory, CleanupFileKind,
        CleanupRoot, CleanupRule, EligibilityDecision, EligibilityExclusion, RuleValidationError,
    };
    use std::time::{Duration, SystemTime};

    const NOW: SystemTime = SystemTime::UNIX_EPOCH;

    fn temporary_rule() -> CleanupRule {
        CleanupRule {
            id: "app-temp-old-files".to_string(),
            version: 1,
            category: CleanupCategory::TemporaryFiles,
            allowed_root: CleanupRoot::AppTemporary,
            minimum_age: Duration::from_secs(60),
            maximum_item_bytes: 1_024,
        }
    }

    fn candidate(
        root: CleanupRoot,
        kind: CleanupFileKind,
        age_seconds: u64,
        size_bytes: u64,
    ) -> CleanupCandidateMetadata {
        CleanupCandidateMetadata {
            root,
            kind,
            modified_at: NOW - Duration::from_secs(age_seconds),
            size_bytes,
        }
    }

    #[test]
    fn cleanup_rule_contract_is_valid_without_a_filesystem_target() {
        let rule = temporary_rule();
        assert_eq!(rule.validate(), Ok(()));
        assert_eq!(
            evaluate_eligibility(
                &rule,
                candidate(
                    CleanupRoot::AppTemporary,
                    CleanupFileKind::RegularFile,
                    60,
                    1_024
                ),
                NOW,
            ),
            EligibilityDecision::Eligible,
        );
    }

    #[test]
    fn eligibility_rejects_a_different_root_and_non_regular_entries() {
        let rule = temporary_rule();
        assert_eq!(
            evaluate_eligibility(
                &rule,
                candidate(CleanupRoot::AppCache, CleanupFileKind::RegularFile, 300, 10),
                NOW,
            ),
            EligibilityDecision::Excluded(EligibilityExclusion::WrongRoot),
        );
        for kind in [
            CleanupFileKind::Directory,
            CleanupFileKind::Link,
            CleanupFileKind::Other,
        ] {
            assert_eq!(
                evaluate_eligibility(
                    &rule,
                    candidate(CleanupRoot::AppTemporary, kind, 300, 10),
                    NOW,
                ),
                EligibilityDecision::Excluded(EligibilityExclusion::NotRegularFile),
            );
        }
    }

    #[test]
    fn eligibility_enforces_age_size_and_clock_bounds() {
        let rule = temporary_rule();
        assert_eq!(
            evaluate_eligibility(
                &rule,
                candidate(
                    CleanupRoot::AppTemporary,
                    CleanupFileKind::RegularFile,
                    59,
                    10
                ),
                NOW,
            ),
            EligibilityDecision::Excluded(EligibilityExclusion::TooRecent),
        );
        assert_eq!(
            evaluate_eligibility(
                &rule,
                candidate(
                    CleanupRoot::AppTemporary,
                    CleanupFileKind::RegularFile,
                    60,
                    1_025
                ),
                NOW,
            ),
            EligibilityDecision::Excluded(EligibilityExclusion::ItemTooLarge),
        );
        assert_eq!(
            evaluate_eligibility(
                &rule,
                CleanupCandidateMetadata {
                    root: CleanupRoot::AppTemporary,
                    kind: CleanupFileKind::RegularFile,
                    modified_at: NOW + Duration::from_secs(1),
                    size_bytes: 1,
                },
                NOW,
            ),
            EligibilityDecision::Excluded(EligibilityExclusion::ModifiedTimeInFuture),
        );
    }

    #[test]
    fn rule_validation_rejects_unbound_or_unbounded_rules() {
        let mut rule = temporary_rule();
        rule.allowed_root = CleanupRoot::AppCache;
        assert_eq!(
            rule.validate(),
            Err(RuleValidationError::CategoryRootMismatch),
        );
        assert_eq!(
            evaluate_eligibility(
                &rule,
                candidate(CleanupRoot::AppCache, CleanupFileKind::RegularFile, 300, 10),
                NOW,
            ),
            EligibilityDecision::Excluded(EligibilityExclusion::InvalidRule),
        );

        let mut rule = temporary_rule();
        rule.id = "../downloads".to_string();
        assert_eq!(rule.validate(), Err(RuleValidationError::InvalidId));

        let mut rule = temporary_rule();
        rule.version = 0;
        assert_eq!(rule.validate(), Err(RuleValidationError::MissingVersion));

        let mut rule = temporary_rule();
        rule.minimum_age = Duration::ZERO;
        assert_eq!(rule.validate(), Err(RuleValidationError::MissingMinimumAge));

        let mut rule = temporary_rule();
        rule.maximum_item_bytes = 0;
        assert_eq!(
            rule.validate(),
            Err(RuleValidationError::MissingItemSizeLimit),
        );
    }

    #[test]
    fn application_cache_rule_uses_its_bound_root() {
        let rule = CleanupRule {
            id: "app-cache-old-files".to_string(),
            version: 1,
            category: CleanupCategory::ApplicationCache,
            allowed_root: CleanupRoot::AppCache,
            minimum_age: Duration::from_secs(60),
            maximum_item_bytes: 1_024,
        };
        assert_eq!(rule.validate(), Ok(()));
        assert_eq!(
            evaluate_eligibility(
                &rule,
                candidate(
                    CleanupRoot::AppCache,
                    CleanupFileKind::RegularFile,
                    60,
                    1_024
                ),
                NOW,
            ),
            EligibilityDecision::Eligible,
        );
    }

    #[test]
    fn temporary_rule_can_bind_to_the_platform_resolved_user_temp_root() {
        let rule = CleanupRule {
            id: "user-temp-old-files".to_string(),
            version: 1,
            category: CleanupCategory::TemporaryFiles,
            allowed_root: CleanupRoot::UserTemporary,
            minimum_age: Duration::from_secs(60),
            maximum_item_bytes: 1_024,
        };
        assert_eq!(rule.validate(), Ok(()));
        assert_eq!(
            evaluate_eligibility(
                &rule,
                candidate(
                    CleanupRoot::UserTemporary,
                    CleanupFileKind::RegularFile,
                    60,
                    1_024,
                ),
                NOW,
            ),
            EligibilityDecision::Eligible,
        );
    }
}
