use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    pub created_at: Option<SystemTime>,
    pub size_bytes: u64,
}

/// A per-scan identity and immutable metadata snapshot. `relative_path` is an
/// internal locator under the allowlisted root, never a display-only identity.
/// Revalidation must compare the saved metadata with a fresh observation before
/// any later mutation; this contract itself has no deletion operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupCandidateIdentity {
    pub id: uuid::Uuid,
    pub relative_path: PathBuf,
    pub metadata: CleanupCandidateMetadata,
    pub platform_identity: Option<CleanupPlatformFileIdentity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CleanupPlatformFileIdentity {
    pub volume_id: u64,
    pub file_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateIdentityError {
    InvalidRelativePath,
}

impl CleanupCandidateIdentity {
    pub fn new(
        relative_path: impl Into<PathBuf>,
        metadata: CleanupCandidateMetadata,
    ) -> Result<Self, CandidateIdentityError> {
        let relative_path = relative_path.into();
        if relative_path.as_os_str().is_empty()
            || !relative_path
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_)))
        {
            return Err(CandidateIdentityError::InvalidRelativePath);
        }
        Ok(Self {
            id: uuid::Uuid::new_v4(),
            relative_path,
            metadata,
            platform_identity: None,
        })
    }

    /// Captures a candidate under a caller-supplied, native-resolved root.
    /// The relative locator must be normal path components only; links are rejected.
    pub fn capture(
        allowed_root: &Path,
        root_id: CleanupRoot,
        relative_path: impl Into<PathBuf>,
    ) -> Result<Self, CleanupRevalidationError> {
        let relative_path = relative_path.into();
        if relative_path.as_os_str().is_empty()
            || !relative_path
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_)))
        {
            return Err(CleanupRevalidationError::InvalidIdentity);
        }
        let target = allowed_root.join(&relative_path);
        let metadata = std::fs::symlink_metadata(&target)
            .map_err(|_| CleanupRevalidationError::PathUnavailable)?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(CleanupRevalidationError::NotRegularFile);
        }
        let initial_platform_identity = capture_platform_file_identity(&target)?;
        let resolved =
            validate_cleanup_path(allowed_root, &target, &[]).map_err(|error| match error {
                CleanupPathError::PathResolutionFailed => CleanupRevalidationError::PathUnavailable,
                CleanupPathError::InvalidRoot => CleanupRevalidationError::InvalidRoot,
                CleanupPathError::OutsideAllowedRoot => {
                    CleanupRevalidationError::OutsideAllowedRoot
                }
                CleanupPathError::ProtectedPath => CleanupRevalidationError::ProtectedPath,
            })?;
        let metadata = std::fs::symlink_metadata(resolved)
            .map_err(|_| CleanupRevalidationError::PathUnavailable)?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(CleanupRevalidationError::NotRegularFile);
        }
        let platform_identity = capture_platform_file_identity(&target)?;
        if platform_identity != initial_platform_identity {
            return Err(CleanupRevalidationError::Changed);
        }
        Ok(Self {
            id: uuid::Uuid::new_v4(),
            relative_path,
            metadata: CleanupCandidateMetadata {
                root: root_id,
                kind: CleanupFileKind::RegularFile,
                modified_at: metadata
                    .modified()
                    .map_err(|_| CleanupRevalidationError::MetadataUnavailable)?,
                created_at: metadata.created().ok(),
                size_bytes: metadata.len(),
            },
            platform_identity: Some(platform_identity),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupScanStage {
    Enumerating,
    Evaluating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupCancellationSupport {
    BestEffort,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupScanTerminalStatus {
    Succeeded,
    Partial,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CleanupScanErrorCode {
    PermissionDenied,
    IoFailure,
    RootUnavailable,
    InternalError,
}

impl From<CleanupScanTerminalStatus> for crate::domain::JobStatus {
    fn from(status: CleanupScanTerminalStatus) -> Self {
        match status {
            CleanupScanTerminalStatus::Succeeded => Self::Succeeded,
            CleanupScanTerminalStatus::Partial => Self::Partial,
            CleanupScanTerminalStatus::Failed => Self::Failed,
            CleanupScanTerminalStatus::Cancelled => Self::Cancelled,
            CleanupScanTerminalStatus::TimedOut => Self::TimedOut,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CleanupScanProgress {
    pub job_id: uuid::Uuid,
    pub stage: CleanupScanStage,
    pub visited_entries: u64,
    pub eligible_candidates: u64,
    pub skipped_entries: u64,
    pub cancellation_support: CleanupCancellationSupport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CleanupScanTerminal {
    pub job_id: uuid::Uuid,
    pub status: CleanupScanTerminalStatus,
    pub visited_entries: u64,
    pub eligible_candidates: u64,
    pub skipped_entries: u64,
    pub error_code: Option<CleanupScanErrorCode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum CleanupScanEvent {
    Progress(CleanupScanProgress),
    Terminal(CleanupScanTerminal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EligibilityExclusion {
    InvalidRule,
    WrongRoot,
    NotRegularFile,
    ModifiedTimeInFuture,
    TooRecent,
    ItemTooLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupPreviewDisposition {
    Eligible,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CleanupPreviewItemDto {
    pub candidate_id: uuid::Uuid,
    pub category: CleanupCategory,
    pub display_name: String,
    pub size_bytes: u64,
    pub modified_at: String,
    pub disposition: CleanupPreviewDisposition,
    pub skip_reason: Option<EligibilityExclusion>,
}

impl CleanupPreviewItemDto {
    pub fn from_candidate(
        candidate: &CleanupCandidateIdentity,
        category: CleanupCategory,
        disposition: CleanupPreviewDisposition,
        skip_reason: Option<EligibilityExclusion>,
    ) -> Result<Self, CandidateIdentityError> {
        let display_name = candidate
            .relative_path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .ok_or(CandidateIdentityError::InvalidRelativePath)?;
        Ok(Self {
            candidate_id: candidate.id,
            category,
            display_name: display_name.to_string(),
            size_bytes: candidate.metadata.size_bytes,
            modified_at: chrono::DateTime::<chrono::Utc>::from(candidate.metadata.modified_at)
                .to_rfc3339(),
            disposition,
            skip_reason,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CleanupPreviewDto {
    pub preview_id: uuid::Uuid,
    pub generated_at: String,
    pub items: Vec<CleanupPreviewItemDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CleanupApprovalRequestDto {
    pub preview_id: uuid::Uuid,
    pub candidate_ids: Vec<uuid::Uuid>,
    pub estimated_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupApprovalValidationError {
    EmptySelection,
    DuplicateCandidate,
    CandidateNotEligible,
    SizeOverflow,
}

impl CleanupApprovalRequestDto {
    /// Builds a request only for unique eligible rows in the supplied immutable preview.
    /// This records approval scope; it does not execute or authorize deletion by itself.
    pub fn from_selection(
        preview: &CleanupPreviewDto,
        selected_ids: &[uuid::Uuid],
    ) -> Result<Self, CleanupApprovalValidationError> {
        if selected_ids.is_empty() {
            return Err(CleanupApprovalValidationError::EmptySelection);
        }
        let mut candidate_ids = Vec::with_capacity(selected_ids.len());
        let mut estimated_bytes = 0_u64;
        for id in selected_ids {
            if candidate_ids.contains(id) {
                return Err(CleanupApprovalValidationError::DuplicateCandidate);
            }
            let item = preview
                .items
                .iter()
                .find(|item| item.candidate_id == *id)
                .filter(|item| item.disposition == CleanupPreviewDisposition::Eligible)
                .ok_or(CleanupApprovalValidationError::CandidateNotEligible)?;
            estimated_bytes = estimated_bytes
                .checked_add(item.size_bytes)
                .ok_or(CleanupApprovalValidationError::SizeOverflow)?;
            candidate_ids.push(*id);
        }
        Ok(Self {
            preview_id: preview.preview_id,
            candidate_ids,
            estimated_bytes,
        })
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupRevalidationError {
    InvalidRule,
    InvalidIdentity,
    WrongRoot,
    Changed,
    NotRegularFile,
    MetadataUnavailable,
    IdentityUnavailable,
    PathUnavailable,
    InvalidRoot,
    OutsideAllowedRoot,
    ProtectedPath,
    NotEligible(EligibilityExclusion),
}

#[cfg(windows)]
fn capture_platform_file_identity(
    path: &Path,
) -> Result<CleanupPlatformFileIdentity, CleanupRevalidationError> {
    use std::os::windows::{ffi::OsStrExt, io::FromRawHandle};
    use windows_sys::Win32::{
        Foundation::INVALID_HANDLE_VALUE,
        Storage::FileSystem::{
            CreateFileW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
            FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ,
            FILE_SHARE_WRITE, OPEN_EXISTING,
        },
    };

    let wide_path = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    // The handle opens the reparse point itself, so a link is identified/rejected
    // without following it. The handle is immediately owned by File and closed.
    let raw_handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if raw_handle == INVALID_HANDLE_VALUE {
        return Err(CleanupRevalidationError::PathUnavailable);
    }
    let file = unsafe { std::fs::File::from_raw_handle(raw_handle) };
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    let succeeded = unsafe { GetFileInformationByHandle(raw_handle, &mut information) };
    if succeeded == 0 {
        return Err(CleanupRevalidationError::IdentityUnavailable);
    }
    if information.dwFileAttributes
        & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
        != 0
    {
        return Err(CleanupRevalidationError::NotRegularFile);
    }
    let file_id =
        (u64::from(information.nFileIndexHigh) << 32) | u64::from(information.nFileIndexLow);
    drop(file);
    Ok(CleanupPlatformFileIdentity {
        volume_id: u64::from(information.dwVolumeSerialNumber),
        file_id,
    })
}

#[cfg(unix)]
fn capture_platform_file_identity(
    path: &Path,
) -> Result<CleanupPlatformFileIdentity, CleanupRevalidationError> {
    use std::os::unix::fs::MetadataExt;

    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| CleanupRevalidationError::PathUnavailable)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(CleanupRevalidationError::NotRegularFile);
    }
    Ok(CleanupPlatformFileIdentity {
        volume_id: metadata.dev(),
        file_id: metadata.ino(),
    })
}

#[cfg(not(any(windows, unix)))]
fn capture_platform_file_identity(
    _path: &Path,
) -> Result<CleanupPlatformFileIdentity, CleanupRevalidationError> {
    Err(CleanupRevalidationError::IdentityUnavailable)
}

/// Re-checks a candidate's native target, kind, metadata snapshot, containment,
/// protected-path policy and current rule eligibility. The caller must resolve
/// `allowed_root` from the rule's closed root identifier and invoke this just
/// before a later action. The returned path does not prevent a subsequent race;
/// an executor must not cache it for later use.
pub fn revalidate_cleanup_candidate(
    identity: &CleanupCandidateIdentity,
    rule: &CleanupRule,
    allowed_root: &Path,
    protected_paths: &[PathBuf],
    observed_at: SystemTime,
) -> Result<PathBuf, CleanupRevalidationError> {
    if rule.validate().is_err() {
        return Err(CleanupRevalidationError::InvalidRule);
    }
    if identity.metadata.root != rule.allowed_root {
        return Err(CleanupRevalidationError::WrongRoot);
    }
    if identity.relative_path.as_os_str().is_empty()
        || !identity
            .relative_path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(CleanupRevalidationError::InvalidIdentity);
    }
    if identity.metadata.kind != CleanupFileKind::RegularFile {
        return Err(CleanupRevalidationError::NotRegularFile);
    }
    let target = allowed_root.join(&identity.relative_path);
    let metadata = std::fs::symlink_metadata(&target)
        .map_err(|_| CleanupRevalidationError::PathUnavailable)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() || !file_type.is_file() {
        return Err(CleanupRevalidationError::NotRegularFile);
    }
    let saved_platform_identity = identity
        .platform_identity
        .ok_or(CleanupRevalidationError::IdentityUnavailable)?;
    if capture_platform_file_identity(&target)? != saved_platform_identity {
        return Err(CleanupRevalidationError::Changed);
    }
    let created_at = metadata
        .created()
        .map_err(|_| CleanupRevalidationError::MetadataUnavailable)?;
    let fresh_metadata = CleanupCandidateMetadata {
        root: rule.allowed_root,
        kind: CleanupFileKind::RegularFile,
        modified_at: metadata
            .modified()
            .map_err(|_| CleanupRevalidationError::MetadataUnavailable)?,
        created_at: Some(created_at),
        size_bytes: metadata.len(),
    };
    if identity.metadata.created_at.is_none() {
        return Err(CleanupRevalidationError::MetadataUnavailable);
    }
    if fresh_metadata != identity.metadata {
        return Err(CleanupRevalidationError::Changed);
    }

    let resolved = validate_cleanup_path(allowed_root, &target, protected_paths).map_err(
        |error| match error {
            CleanupPathError::PathResolutionFailed => CleanupRevalidationError::PathUnavailable,
            CleanupPathError::InvalidRoot => CleanupRevalidationError::InvalidRoot,
            CleanupPathError::OutsideAllowedRoot => CleanupRevalidationError::OutsideAllowedRoot,
            CleanupPathError::ProtectedPath => CleanupRevalidationError::ProtectedPath,
        },
    )?;
    match evaluate_eligibility(rule, fresh_metadata, observed_at) {
        EligibilityDecision::Eligible => Ok(resolved),
        EligibilityDecision::Excluded(reason) => Err(CleanupRevalidationError::NotEligible(reason)),
    }
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
                created_at: metadata.created().ok(),
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
    use std::{fs, io, io::Write, path::Path, path::PathBuf};

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
            created_at: None,
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
                    created_at: None,
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

    #[test]
    fn outside_directory_link_is_skipped_and_cannot_escape_allowed_root() {
        let fixture = std::env::temp_dir().join(format!("moonships-link-{}", uuid::Uuid::new_v4()));
        let root = fixture.join("root");
        let outside = fixture.join("outside");
        fs::create_dir_all(&root).expect("create allowed root");
        fs::create_dir(&outside).expect("create outside target");
        let outside_file = outside.join("keep.txt");
        fs::write(&outside_file, b"outside target remains intact").expect("write outside target");
        let link = root.join("redirect");
        create_directory_symlink(&outside, &link).expect("create directory link fixture");

        assert_eq!(
            super::validate_cleanup_path(&root, &link, &[]),
            Err(super::CleanupPathError::OutsideAllowedRoot),
        );
        let candidates = super::scan_read_only(&root, CleanupRoot::AppTemporary)
            .expect("scan root with external directory link");
        assert!(candidates.is_empty(), "scanner must not follow the link");
        assert_eq!(
            fs::read(&outside_file).expect("read outside target after scan"),
            b"outside target remains intact",
        );

        fs::remove_dir_all(fixture).expect("remove disposable link fixture");
    }

    #[test]
    fn candidate_identity_binds_scan_id_to_relative_locator_and_metadata() {
        let metadata = candidate(
            CleanupRoot::AppTemporary,
            CleanupFileKind::RegularFile,
            300,
            12,
        );
        let first = super::CleanupCandidateIdentity::new("nested/cache.bin", metadata)
            .expect("valid relative candidate locator");
        let second = super::CleanupCandidateIdentity::new("nested/cache.bin", metadata)
            .expect("second scan identity");

        assert_ne!(first.id, second.id);
        assert_eq!(first.relative_path, PathBuf::from("nested/cache.bin"));
        assert_eq!(first.metadata, metadata);
        assert_eq!(
            super::CleanupCandidateIdentity::new("../outside.bin", metadata),
            Err(super::CandidateIdentityError::InvalidRelativePath),
        );
        assert_eq!(
            super::CleanupCandidateIdentity::new(PathBuf::from("/outside.bin"), metadata),
            Err(super::CandidateIdentityError::InvalidRelativePath),
        );
    }

    #[test]
    fn scan_events_serialize_progress_and_truthful_terminal_states() {
        let job_id = uuid::Uuid::new_v4();
        let progress = super::CleanupScanEvent::Progress(super::CleanupScanProgress {
            job_id,
            stage: super::CleanupScanStage::Enumerating,
            visited_entries: 8,
            eligible_candidates: 3,
            skipped_entries: 2,
            cancellation_support: super::CleanupCancellationSupport::BestEffort,
        });
        let progress_json = serde_json::to_value(&progress).expect("serialize progress event");
        assert_eq!(progress_json["type"], "progress");
        assert_eq!(progress_json["data"]["stage"], "enumerating");
        assert_eq!(progress_json["data"]["cancellationSupport"], "best_effort");
        assert_eq!(
            serde_json::from_value::<super::CleanupScanEvent>(progress_json).unwrap(),
            progress,
        );

        let terminal = super::CleanupScanTerminal {
            job_id,
            status: super::CleanupScanTerminalStatus::Cancelled,
            visited_entries: 8,
            eligible_candidates: 3,
            skipped_entries: 2,
            error_code: None,
        };
        let terminal_json =
            serde_json::to_value(super::CleanupScanEvent::Terminal(terminal.clone()))
                .expect("serialize terminal event");
        assert_eq!(terminal_json["type"], "terminal");
        assert_eq!(terminal_json["data"]["status"], "cancelled");
        assert_eq!(
            crate::domain::JobStatus::from(terminal.status),
            crate::domain::JobStatus::Cancelled,
        );
        assert_eq!(
            serde_json::from_value::<super::CleanupScanEvent>(terminal_json).unwrap(),
            super::CleanupScanEvent::Terminal(terminal),
        );
    }

    #[test]
    fn preview_dto_exposes_summary_fields_and_leaf_name_not_absolute_path() {
        let identity = super::CleanupCandidateIdentity::new(
            PathBuf::from("nested/cache.bin"),
            candidate(
                CleanupRoot::AppTemporary,
                CleanupFileKind::RegularFile,
                300,
                24,
            ),
        )
        .expect("valid candidate identity");
        let item = super::CleanupPreviewItemDto::from_candidate(
            &identity,
            CleanupCategory::TemporaryFiles,
            super::CleanupPreviewDisposition::Skipped,
            Some(EligibilityExclusion::TooRecent),
        )
        .expect("create preview row");
        let preview = super::CleanupPreviewDto {
            preview_id: uuid::Uuid::new_v4(),
            generated_at: "2026-10-26T00:00:00Z".to_string(),
            items: vec![item],
        };
        let serialized = serde_json::to_value(&preview).expect("serialize preview DTO");

        assert_eq!(serialized["items"][0]["displayName"], "cache.bin");
        assert_eq!(serialized["items"][0]["sizeBytes"], 24);
        assert_eq!(serialized["items"][0]["disposition"], "skipped");
        assert_eq!(serialized["items"][0]["skipReason"], "too_recent");
        assert!(serialized["items"][0]["displayName"]
            .as_str()
            .unwrap()
            .contains("cache.bin"));
        assert!(!serialized.to_string().contains("nested/cache.bin"));
    }

    #[test]
    fn clean_approval_request_binds_only_unique_eligible_preview_candidates() {
        let eligible_identity = super::CleanupCandidateIdentity::new(
            "cache/eligible.bin",
            candidate(
                CleanupRoot::AppTemporary,
                CleanupFileKind::RegularFile,
                300,
                1_024,
            ),
        )
        .unwrap();
        let skipped_identity = super::CleanupCandidateIdentity::new(
            "cache/recent.bin",
            candidate(
                CleanupRoot::AppTemporary,
                CleanupFileKind::RegularFile,
                1,
                512,
            ),
        )
        .unwrap();
        let eligible = super::CleanupPreviewItemDto::from_candidate(
            &eligible_identity,
            CleanupCategory::TemporaryFiles,
            super::CleanupPreviewDisposition::Eligible,
            None,
        )
        .unwrap();
        let skipped = super::CleanupPreviewItemDto::from_candidate(
            &skipped_identity,
            CleanupCategory::TemporaryFiles,
            super::CleanupPreviewDisposition::Skipped,
            Some(EligibilityExclusion::TooRecent),
        )
        .unwrap();
        let preview = super::CleanupPreviewDto {
            preview_id: uuid::Uuid::new_v4(),
            generated_at: "2026-10-26T00:00:00Z".to_string(),
            items: vec![eligible, skipped],
        };

        let request =
            super::CleanupApprovalRequestDto::from_selection(&preview, &[eligible_identity.id])
                .unwrap();
        assert_eq!(request.preview_id, preview.preview_id);
        assert_eq!(request.candidate_ids, vec![eligible_identity.id]);
        assert_eq!(request.estimated_bytes, 1_024);
        assert_eq!(
            super::CleanupApprovalRequestDto::from_selection(&preview, &[]),
            Err(super::CleanupApprovalValidationError::EmptySelection),
        );
        assert_eq!(
            super::CleanupApprovalRequestDto::from_selection(&preview, &[skipped_identity.id]),
            Err(super::CleanupApprovalValidationError::CandidateNotEligible),
        );
        assert_eq!(
            super::CleanupApprovalRequestDto::from_selection(
                &preview,
                &[eligible_identity.id, eligible_identity.id],
            ),
            Err(super::CleanupApprovalValidationError::DuplicateCandidate),
        );
    }

    #[test]
    fn candidate_revalidation_rejects_changed_and_out_of_policy_files() {
        let fixture =
            std::env::temp_dir().join(format!("moonships-revalidate-{}", uuid::Uuid::new_v4()));
        let root = fixture.join("root");
        fs::create_dir_all(&root).expect("create disposable revalidation root");
        let target = root.join("candidate.bin");
        fs::write(&target, b"original").expect("write candidate fixture");
        let observed_now = SystemTime::now();
        let old_time = observed_now - Duration::from_secs(120);
        fs::OpenOptions::new()
            .write(true)
            .open(&target)
            .expect("open candidate fixture")
            .set_times(fs::FileTimes::new().set_modified(old_time))
            .expect("age candidate fixture");
        let identity = super::CleanupCandidateIdentity::capture(
            &root,
            CleanupRoot::AppTemporary,
            "candidate.bin",
        )
        .expect("capture candidate identity");
        let rule = temporary_rule();

        assert!(
            super::revalidate_cleanup_candidate(&identity, &rule, &root, &[], observed_now,)
                .is_ok()
        );

        fs::write(&target, b"replacement is longer").expect("change candidate fixture");
        assert_eq!(
            super::revalidate_cleanup_candidate(&identity, &rule, &root, &[], observed_now),
            Err(super::CleanupRevalidationError::Changed),
        );

        fs::remove_file(&target).expect("remove changed candidate fixture");
        std::thread::sleep(Duration::from_millis(20));
        fs::write(&target, b"original")
            .expect("replace candidate with identical-size fixture content");
        fs::OpenOptions::new()
            .write(true)
            .open(&target)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(old_time))
            .expect("restore candidate modification time");
        assert_eq!(
            super::revalidate_cleanup_candidate(&identity, &rule, &root, &[], SystemTime::now()),
            Err(super::CleanupRevalidationError::Changed),
        );

        let mut limited_rule = temporary_rule();
        limited_rule.maximum_item_bytes = 4;
        fs::OpenOptions::new()
            .write(true)
            .open(&target)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(old_time))
            .expect("age replacement fixture for policy check");
        let current_observation = SystemTime::now();
        let oversized_identity = super::CleanupCandidateIdentity::capture(
            &root,
            CleanupRoot::AppTemporary,
            "candidate.bin",
        )
        .unwrap();
        assert_eq!(
            super::revalidate_cleanup_candidate(
                &oversized_identity,
                &limited_rule,
                &root,
                &[],
                current_observation,
            ),
            Err(super::CleanupRevalidationError::NotEligible(
                EligibilityExclusion::ItemTooLarge,
            )),
        );

        fs::remove_dir_all(fixture).expect("remove disposable revalidation fixture");
    }

    #[test]
    fn candidate_revalidation_rejects_link_non_file_and_invalid_identity() {
        let fixture =
            std::env::temp_dir().join(format!("moonships-revalidate-{}", uuid::Uuid::new_v4()));
        let root = fixture.join("root");
        fs::create_dir_all(&root).expect("create disposable revalidation root");
        let target = root.join("candidate.bin");
        fs::write(&target, b"candidate").expect("write candidate fixture");
        let metadata = fs::symlink_metadata(&target).unwrap();
        let snapshot = super::CleanupCandidateMetadata {
            root: CleanupRoot::AppTemporary,
            kind: CleanupFileKind::RegularFile,
            modified_at: metadata.modified().unwrap(),
            created_at: Some(metadata.created().unwrap()),
            size_bytes: metadata.len(),
        };
        let mut identity = super::CleanupCandidateIdentity::new("candidate.bin", snapshot).unwrap();
        identity.relative_path = PathBuf::from("../outside.bin");
        assert_eq!(
            super::revalidate_cleanup_candidate(&identity, &temporary_rule(), &root, &[], NOW),
            Err(super::CleanupRevalidationError::InvalidIdentity),
        );
        identity.relative_path = PathBuf::from("candidate.bin");
        assert_eq!(
            super::revalidate_cleanup_candidate(&identity, &temporary_rule(), &root, &[], NOW),
            Err(super::CleanupRevalidationError::IdentityUnavailable),
        );
        fs::remove_file(&target).expect("remove candidate fixture");
        fs::create_dir(&target).expect("replace candidate with directory");
        assert_eq!(
            super::revalidate_cleanup_candidate(&identity, &temporary_rule(), &root, &[], NOW),
            Err(super::CleanupRevalidationError::NotRegularFile),
        );

        fs::remove_dir_all(fixture).expect("remove disposable revalidation fixture");
    }

    #[cfg(windows)]
    fn create_directory_symlink(target: &Path, link: &Path) -> io::Result<()> {
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(
                "could not create Windows junction fixture",
            ))
        }
    }

    #[cfg(unix)]
    fn create_directory_symlink(target: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }
}
