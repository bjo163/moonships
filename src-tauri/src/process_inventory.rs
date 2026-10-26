use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_PROCESS_RECORDS: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProcessInventoryStatus {
    Available,
    Partial,
    PermissionDenied,
    Unsupported,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProcessInventoryError {
    AccessDenied,
    SnapshotFailed,
    EnumerationFailed,
    ResultLimitReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRecord {
    pub process_id: u32,
    pub parent_process_id: u32,
    pub thread_count: u32,
    pub executable_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInventorySnapshot {
    pub status: ProcessInventoryStatus,
    pub observed_at_epoch_ms: i64,
    pub processes: Vec<ProcessRecord>,
    pub error_code: Option<ProcessInventoryError>,
}

/// Reads a bounded OS process snapshot. No process handles, command lines or mutation actions are exposed.
pub fn read_process_inventory() -> ProcessInventorySnapshot {
    read_platform_process_inventory()
}

#[cfg(windows)]
fn read_platform_process_inventory() -> ProcessInventorySnapshot {
    use std::mem::size_of;
    use windows_sys::Win32::{
        Foundation::{
            GetLastError, ERROR_ACCESS_DENIED, ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE,
        },
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        let error_code = if unsafe { GetLastError() } == ERROR_ACCESS_DENIED {
            ProcessInventoryError::AccessDenied
        } else {
            ProcessInventoryError::SnapshotFailed
        };
        return ProcessInventorySnapshot {
            status: if error_code == ProcessInventoryError::AccessDenied {
                ProcessInventoryStatus::PermissionDenied
            } else {
                ProcessInventoryStatus::Unavailable
            },
            observed_at_epoch_ms: unix_epoch_milliseconds(),
            processes: Vec::new(),
            error_code: Some(error_code),
        };
    }
    let _snapshot_guard = SnapshotHandle(snapshot);

    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let first = unsafe { Process32FirstW(snapshot, &mut entry) };
    if first == 0 {
        // ERROR_NO_MORE_FILES means an empty snapshot, not a failure.
        if unsafe { GetLastError() } == ERROR_NO_MORE_FILES {
            return success_snapshot(Vec::new());
        }
        return ProcessInventorySnapshot {
            status: ProcessInventoryStatus::Unavailable,
            observed_at_epoch_ms: unix_epoch_milliseconds(),
            processes: Vec::new(),
            error_code: Some(ProcessInventoryError::EnumerationFailed),
        };
    }

    let mut processes = Vec::new();
    let mut status = ProcessInventoryStatus::Available;
    let mut error_code = None;
    loop {
        if processes.len() == MAX_PROCESS_RECORDS {
            status = ProcessInventoryStatus::Partial;
            error_code = Some(ProcessInventoryError::ResultLimitReached);
            break;
        }
        processes.push(process_record(&entry));
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        if unsafe { Process32NextW(snapshot, &mut entry) } == 0 {
            if unsafe { GetLastError() } != ERROR_NO_MORE_FILES {
                status = ProcessInventoryStatus::Partial;
                error_code = Some(ProcessInventoryError::EnumerationFailed);
            }
            break;
        }
    }
    processes.sort_unstable_by_key(|process| process.process_id);
    ProcessInventorySnapshot {
        status,
        observed_at_epoch_ms: unix_epoch_milliseconds(),
        processes,
        error_code,
    }
}

#[cfg(windows)]
fn process_record(
    entry: &windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W,
) -> ProcessRecord {
    let end = entry
        .szExeFile
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(entry.szExeFile.len());
    ProcessRecord {
        process_id: entry.th32ProcessID,
        parent_process_id: entry.th32ParentProcessID,
        thread_count: entry.cntThreads,
        executable_name: String::from_utf16_lossy(&entry.szExeFile[..end]),
    }
}

#[cfg(windows)]
struct SnapshotHandle(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        let _ = unsafe { windows_sys::Win32::Foundation::CloseHandle(self.0) };
    }
}

#[cfg(not(windows))]
fn read_platform_process_inventory() -> ProcessInventorySnapshot {
    ProcessInventorySnapshot {
        status: ProcessInventoryStatus::Unsupported,
        observed_at_epoch_ms: unix_epoch_milliseconds(),
        processes: Vec::new(),
        error_code: None,
    }
}

fn success_snapshot(processes: Vec<ProcessRecord>) -> ProcessInventorySnapshot {
    ProcessInventorySnapshot {
        status: ProcessInventoryStatus::Available,
        observed_at_epoch_ms: unix_epoch_milliseconds(),
        processes,
        error_code: None,
    }
}

fn unix_epoch_milliseconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{read_process_inventory, ProcessInventoryStatus, MAX_PROCESS_RECORDS};

    #[test]
    fn process_inventory_is_bounded_and_never_contains_mutation_fields() {
        let result = read_process_inventory();
        assert!(result.processes.len() <= MAX_PROCESS_RECORDS);
        assert!(result.observed_at_epoch_ms > 0);
        for process in &result.processes {
            assert!(!process.executable_name.is_empty());
        }
        #[cfg(not(windows))]
        {
            assert_eq!(result.status, ProcessInventoryStatus::Unsupported);
            assert!(result.processes.is_empty());
        }
        #[cfg(windows)]
        assert!(matches!(
            result.status,
            ProcessInventoryStatus::Available
                | ProcessInventoryStatus::Partial
                | ProcessInventoryStatus::PermissionDenied
                | ProcessInventoryStatus::Unavailable
        ));
    }

    #[test]
    fn serializes_read_only_records_in_camel_case() {
        let record = super::ProcessRecord {
            process_id: 10,
            parent_process_id: 1,
            thread_count: 2,
            executable_name: "example.exe".to_string(),
        };
        let value = serde_json::to_value(record).unwrap();
        assert_eq!(value["processId"], 10);
        assert_eq!(value["parentProcessId"], 1);
        assert_eq!(value["threadCount"], 2);
        assert!(value.get("kill").is_none());
        assert!(value.get("commandLine").is_none());
        assert!(value.get("executablePath").is_none());
    }
}
