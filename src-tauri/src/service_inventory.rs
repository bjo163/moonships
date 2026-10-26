use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_SERVICE_RECORDS: usize = 2_048;
const SERVICE_ENUM_BUFFER_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceInventoryStatus {
    Available,
    Partial,
    PermissionDenied,
    Unsupported,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceInventoryError {
    AccessDenied,
    ManagerUnavailable,
    EnumerationFailed,
    ResultLimitReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowsServiceState {
    Stopped,
    StartPending,
    StopPending,
    Running,
    ContinuePending,
    PausePending,
    Paused,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsServiceRecord {
    pub service_name: String,
    pub display_name: String,
    pub state: WindowsServiceState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInventorySnapshot {
    pub status: ServiceInventoryStatus,
    pub observed_at_epoch_ms: i64,
    pub services: Vec<WindowsServiceRecord>,
    pub error_code: Option<ServiceInventoryError>,
}

/// Enumerates local Windows service metadata with query-only SCM access.
pub fn read_service_inventory() -> ServiceInventorySnapshot {
    read_platform_service_inventory()
}

#[cfg(windows)]
fn read_platform_service_inventory() -> ServiceInventorySnapshot {
    use std::mem::size_of;
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_ACCESS_DENIED, ERROR_MORE_DATA},
        System::Services::{
            EnumServicesStatusExW, OpenSCManagerW, ENUM_SERVICE_STATUS_PROCESSW,
            SC_ENUM_PROCESS_INFO, SC_MANAGER_ENUMERATE_SERVICE, SERVICE_STATE_ALL, SERVICE_WIN32,
        },
    };

    let manager = unsafe {
        OpenSCManagerW(
            std::ptr::null(),
            std::ptr::null(),
            SC_MANAGER_ENUMERATE_SERVICE,
        )
    };
    if manager.is_null() {
        let access_denied = unsafe { GetLastError() } == ERROR_ACCESS_DENIED;
        return failed_snapshot(
            if access_denied {
                ServiceInventoryStatus::PermissionDenied
            } else {
                ServiceInventoryStatus::Unavailable
            },
            if access_denied {
                ServiceInventoryError::AccessDenied
            } else {
                ServiceInventoryError::ManagerUnavailable
            },
        );
    }
    let _manager_guard = ServiceManagerHandle(manager);

    let buffer_bytes = SERVICE_ENUM_BUFFER_BYTES;
    // u64 backing provides the alignment required by the native service-entry structs.
    let mut buffer = vec![0u64; buffer_bytes / size_of::<u64>()];
    let mut bytes_needed = 0u32;
    let mut services_returned = 0u32;
    let mut resume_handle = 0u32;
    let success = unsafe {
        EnumServicesStatusExW(
            manager,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            buffer.as_mut_ptr().cast::<u8>(),
            buffer_bytes as u32,
            &mut bytes_needed,
            &mut services_returned,
            &mut resume_handle,
            std::ptr::null(),
        )
    };
    let enumeration_error = if success == 0 {
        Some(unsafe { GetLastError() })
    } else {
        None
    };
    if enumeration_error == Some(ERROR_ACCESS_DENIED) {
        return failed_snapshot(
            ServiceInventoryStatus::PermissionDenied,
            ServiceInventoryError::AccessDenied,
        );
    }

    let entries = services_returned.min(MAX_SERVICE_RECORDS as u32) as usize;
    let entry_size = size_of::<ENUM_SERVICE_STATUS_PROCESSW>();
    if entries.saturating_mul(entry_size) > buffer_bytes {
        return failed_snapshot(
            ServiceInventoryStatus::Unavailable,
            ServiceInventoryError::EnumerationFailed,
        );
    }
    let start = buffer.as_ptr().cast::<u8>();
    let mut services = Vec::with_capacity(entries);
    let mut malformed_entry = false;
    for index in 0..entries {
        let entry = unsafe {
            &*start
                .add(index * entry_size)
                .cast::<ENUM_SERVICE_STATUS_PROCESSW>()
        };
        let Some(service_name) =
            (unsafe { bounded_wide_string(entry.lpServiceName, start, buffer_bytes) })
        else {
            malformed_entry = true;
            continue;
        };
        let Some(display_name) =
            (unsafe { bounded_wide_string(entry.lpDisplayName, start, buffer_bytes) })
        else {
            malformed_entry = true;
            continue;
        };
        services.push(WindowsServiceRecord {
            service_name,
            display_name,
            state: service_state(entry.ServiceStatusProcess.dwCurrentState),
        });
    }
    services.sort_unstable_by(|left, right| left.service_name.cmp(&right.service_name));

    let reached_limit = services_returned as usize > MAX_SERVICE_RECORDS;
    let partial = enumeration_error == Some(ERROR_MORE_DATA) || reached_limit || malformed_entry;
    let error_code = if reached_limit {
        Some(ServiceInventoryError::ResultLimitReached)
    } else if partial || enumeration_error.is_some() {
        Some(ServiceInventoryError::EnumerationFailed)
    } else {
        None
    };
    ServiceInventorySnapshot {
        status: if partial {
            ServiceInventoryStatus::Partial
        } else if enumeration_error.is_some() {
            ServiceInventoryStatus::Unavailable
        } else {
            ServiceInventoryStatus::Available
        },
        observed_at_epoch_ms: unix_epoch_milliseconds(),
        services,
        error_code,
    }
}

#[cfg(windows)]
unsafe fn bounded_wide_string(
    value: windows_sys::core::PCWSTR,
    buffer_start: *const u8,
    buffer_len: usize,
) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let start = buffer_start as usize;
    let pointer = value as usize;
    let end = start.checked_add(buffer_len)?;
    if pointer < start || pointer >= end || pointer % size_of::<u16>() != 0 {
        return None;
    }
    let max_units = (end - pointer) / size_of::<u16>();
    let units = std::slice::from_raw_parts(value, max_units);
    let terminator = units.iter().position(|unit| *unit == 0)?;
    String::from_utf16(&units[..terminator]).ok()
}

#[cfg(windows)]
fn service_state(state: u32) -> WindowsServiceState {
    use windows_sys::Win32::System::Services::{
        SERVICE_CONTINUE_PENDING, SERVICE_PAUSED, SERVICE_PAUSE_PENDING, SERVICE_RUNNING,
        SERVICE_START_PENDING, SERVICE_STOPPED, SERVICE_STOP_PENDING,
    };
    match state {
        SERVICE_STOPPED => WindowsServiceState::Stopped,
        SERVICE_START_PENDING => WindowsServiceState::StartPending,
        SERVICE_STOP_PENDING => WindowsServiceState::StopPending,
        SERVICE_RUNNING => WindowsServiceState::Running,
        SERVICE_CONTINUE_PENDING => WindowsServiceState::ContinuePending,
        SERVICE_PAUSE_PENDING => WindowsServiceState::PausePending,
        SERVICE_PAUSED => WindowsServiceState::Paused,
        _ => WindowsServiceState::Unknown,
    }
}

#[cfg(windows)]
struct ServiceManagerHandle(windows_sys::Win32::System::Services::SC_HANDLE);

#[cfg(windows)]
impl Drop for ServiceManagerHandle {
    fn drop(&mut self) {
        let _ = unsafe { windows_sys::Win32::System::Services::CloseServiceHandle(self.0) };
    }
}

#[cfg(not(windows))]
fn read_platform_service_inventory() -> ServiceInventorySnapshot {
    ServiceInventorySnapshot {
        status: ServiceInventoryStatus::Unsupported,
        observed_at_epoch_ms: unix_epoch_milliseconds(),
        services: Vec::new(),
        error_code: None,
    }
}

fn failed_snapshot(
    status: ServiceInventoryStatus,
    error_code: ServiceInventoryError,
) -> ServiceInventorySnapshot {
    ServiceInventorySnapshot {
        status,
        observed_at_epoch_ms: unix_epoch_milliseconds(),
        services: Vec::new(),
        error_code: Some(error_code),
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
    use super::{
        read_service_inventory, ServiceInventoryStatus, WindowsServiceState, MAX_SERVICE_RECORDS,
    };

    #[test]
    fn service_snapshot_is_read_only_bounded_and_explicit() {
        let snapshot = read_service_inventory();
        assert!(snapshot.services.len() <= MAX_SERVICE_RECORDS);
        assert!(snapshot.observed_at_epoch_ms > 0);
        for service in &snapshot.services {
            assert!(!service.service_name.is_empty());
        }
        #[cfg(not(windows))]
        {
            assert_eq!(snapshot.status, ServiceInventoryStatus::Unsupported);
            assert!(snapshot.services.is_empty());
        }
        #[cfg(windows)]
        assert!(matches!(
            snapshot.status,
            ServiceInventoryStatus::Available
                | ServiceInventoryStatus::Partial
                | ServiceInventoryStatus::PermissionDenied
                | ServiceInventoryStatus::Unavailable
        ));
    }

    #[test]
    fn service_record_serialization_has_no_mutation_controls() {
        let record = super::WindowsServiceRecord {
            service_name: "Example".to_string(),
            display_name: "Example service".to_string(),
            state: WindowsServiceState::Running,
        };
        let value = serde_json::to_value(record).unwrap();
        assert_eq!(value["serviceName"], "Example");
        assert_eq!(value["displayName"], "Example service");
        assert_eq!(value["state"], "running");
        assert!(value.get("start").is_none());
        assert!(value.get("stop").is_none());
    }

    #[test]
    fn maps_unknown_native_service_states_without_guessing() {
        #[cfg(windows)]
        assert_eq!(super::service_state(u32::MAX), WindowsServiceState::Unknown);
        #[cfg(not(windows))]
        assert_eq!(WindowsServiceState::Unknown, WindowsServiceState::Unknown);
    }
}
