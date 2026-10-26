#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::io;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupPathError {
    PathResolutionFailed,
    InvalidRoot,
    OutsideAllowedRoot,
    ProtectedPath,
}

/// Resolves an existing candidate and verifies it is strictly beneath the
/// resolved allowlisted root and outside native policy-protected paths. The
/// protected path list must be constructed by native policy, never by UI input.
/// This check does not mutate files and does not close the race with later use;
/// callers must still revalidate identity immediately before any future action.
pub fn validate_cleanup_path(
    allowed_root: &Path,
    candidate: &Path,
    protected_paths: &[PathBuf],
) -> Result<PathBuf, CleanupPathError> {
    let root = allowed_root
        .canonicalize()
        .map_err(|_| CleanupPathError::PathResolutionFailed)?;
    if !root.is_dir() {
        return Err(CleanupPathError::InvalidRoot);
    }
    let resolved_candidate = candidate
        .canonicalize()
        .map_err(|_| CleanupPathError::PathResolutionFailed)?;
    if resolved_candidate == root || !resolved_candidate.starts_with(&root) {
        return Err(CleanupPathError::OutsideAllowedRoot);
    }
    for protected in protected_paths {
        let resolved_protected = protected
            .canonicalize()
            .map_err(|_| CleanupPathError::PathResolutionFailed)?;
        if resolved_candidate.starts_with(resolved_protected) {
            return Err(CleanupPathError::ProtectedPath);
        }
    }
    Ok(resolved_candidate)
}

/// Reads file metadata beneath a trusted, native-resolved root. This is not an IPC
/// entry point: callers must not pass renderer-controlled paths. It never opens
/// file contents or mutates the filesystem. Link entries are skipped, and are
/// never recursed into. Path containment and race-resistant handles are handled
/// by the later dedicated cleaner tasks.
#[cfg(test)]
pub(crate) fn scan_read_only(
    trusted_root: &Path,
    root_id: CleanupRoot,
) -> io::Result<Vec<CleanupCandidateMetadata>> {
    let mut candidates = Vec::new();
    scan_directory(trusted_root, root_id, &mut candidates)?;
    Ok(candidates)
}

#[cfg(test)]
fn scan_directory(
    directory: &Path,
    root_id: CleanupRoot,
    candidates: &mut Vec<CleanupCandidateMetadata>,
) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            scan_directory(&path, root_id, candidates)?;
        } else if file_type.is_file() {
            candidates.push(CleanupCandidateMetadata {
                root: root_id,
                kind: CleanupFileKind::RegularFile,
                modified_at: metadata.modified()?,
                size_bytes: metadata.len(),
            });
        }
    }
    Ok(())
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
    use std::{fs, io::Write};

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

    #[test]
    fn read_only_scan_leaves_fixture_files_byte_identical() {
        let fixture =
            std::env::temp_dir().join(format!("moonships-cleaner-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&fixture).expect("create disposable scan fixture");
        let nested = fixture.join("nested");
        fs::create_dir(&nested).expect("create nested fixture directory");
        let first_path = fixture.join("first.bin");
        let nested_path = nested.join("second.bin");
        let first_bytes = [0, 1, 2, 0xFE, 0xFF];
        let second_bytes = b"fixture remains unchanged";
        fs::File::create(&first_path)
            .expect("create first fixture")
            .write_all(&first_bytes)
            .expect("write first fixture");
        fs::File::create(&nested_path)
            .expect("create second fixture")
            .write_all(second_bytes)
            .expect("write second fixture");

        let before_first = fs::read(&first_path).expect("read first fixture before scan");
        let before_nested = fs::read(&nested_path).expect("read second fixture before scan");
        let candidates = super::scan_read_only(&fixture, CleanupRoot::AppTemporary)
            .expect("scan disposable fixture");

        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|candidate| {
            candidate.root == CleanupRoot::AppTemporary
                && candidate.kind == CleanupFileKind::RegularFile
        }));
        assert_eq!(fs::read(&first_path).unwrap(), before_first);
        assert_eq!(fs::read(&nested_path).unwrap(), before_nested);
        fs::remove_dir_all(fixture).expect("remove disposable scan fixture");
    }

    #[test]
    fn cleanup_path_validation_rejects_traversal_and_protected_targets() {
        let fixture = std::env::temp_dir().join(format!("moonships-path-{}", uuid::Uuid::new_v4()));
        let root = fixture.join("root");
        let protected = root.join("protected");
        fs::create_dir_all(&protected).expect("create disposable path fixture");
        let safe_file = root.join("safe.txt");
        let protected_file = protected.join("secret.txt");
        let outside = fixture.join("outside.txt");
        fs::write(&safe_file, b"safe").expect("write safe fixture");
        fs::write(&protected_file, b"protected").expect("write protected fixture");
        fs::write(&outside, b"outside").expect("write outside fixture");

        assert_eq!(
            super::validate_cleanup_path(&root, &safe_file, std::slice::from_ref(&protected)),
            Ok(safe_file.canonicalize().unwrap()),
        );
        assert_eq!(
            super::validate_cleanup_path(&root, &root.join("..").join("outside.txt"), &[]),
            Err(super::CleanupPathError::OutsideAllowedRoot),
        );
        assert_eq!(
            super::validate_cleanup_path(&root, &protected_file, std::slice::from_ref(&protected)),
            Err(super::CleanupPathError::ProtectedPath),
        );
        assert_eq!(
            super::validate_cleanup_path(&root, &root, &[]),
            Err(super::CleanupPathError::OutsideAllowedRoot),
        );

        fs::remove_dir_all(fixture).expect("remove disposable path fixture");
    }

    #[test]
    fn cleanup_path_validation_fails_closed_when_resolution_is_incomplete() {
        let fixture = std::env::temp_dir().join(format!("moonships-path-{}", uuid::Uuid::new_v4()));
        let root = fixture.join("root");
        fs::create_dir_all(&root).expect("create disposable root");
        let missing_candidate = root.join("missing.txt");
        let missing_protected = fixture.join("missing-protected");

        assert_eq!(
            super::validate_cleanup_path(&root, &missing_candidate, &[]),
            Err(super::CleanupPathError::PathResolutionFailed),
        );
        let candidate = root.join("candidate.txt");
        fs::write(&candidate, b"candidate").expect("write candidate fixture");
        assert_eq!(
            super::validate_cleanup_path(&root, &candidate, &[missing_protected]),
            Err(super::CleanupPathError::PathResolutionFailed),
        );
        assert_eq!(
            super::validate_cleanup_path(&candidate, &candidate, &[]),
            Err(super::CleanupPathError::InvalidRoot),
        );

        fs::remove_dir_all(fixture).expect("remove disposable path fixture");
    }
}
