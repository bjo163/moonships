#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryFieldStatus {
    Available,
    Unsupported,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryField<T> {
    pub value: Option<T>,
    pub status: InventoryFieldStatus,
}

impl<T> InventoryField<T> {
    fn available(value: T) -> Self {
        Self {
            value: Some(value),
            status: InventoryFieldStatus::Available,
        }
    }

    fn unavailable() -> Self {
        Self {
            value: None,
            status: InventoryFieldStatus::Unavailable,
        }
    }

    pub fn unsupported() -> Self {
        Self {
            value: None,
            status: InventoryFieldStatus::Unsupported,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInventory {
    /// Platform family from the compiled runtime target (for example, Windows).
    pub operating_system: InventoryField<String>,
    /// Architecture of this Moonships process, not an inferred hardware model.
    pub process_architecture: InventoryField<String>,
    pub logical_processor_count: InventoryField<u32>,
    pub total_physical_memory_bytes: InventoryField<u64>,
    pub available_physical_memory_bytes: InventoryField<u64>,
    /// Capacity metrics for the volume containing the Windows directory; no path is returned.
    pub windows_volume_total_bytes: InventoryField<u64>,
    pub windows_volume_free_bytes: InventoryField<u64>,
}

/// Reads bounded system metadata only; it does not enumerate files, devices or processes.
pub fn read_system_inventory() -> SystemInventory {
    let operating_system = InventoryField::available(platform_name().to_string());
    let process_architecture = InventoryField::available(std::env::consts::ARCH.to_string());
    let logical_processor_count = std::thread::available_parallelism()
        .ok()
        .and_then(|count| u32::try_from(count.get()).ok())
        .filter(|count| *count > 0)
        .map(InventoryField::available)
        .unwrap_or_else(InventoryField::unavailable);
    let (total_physical_memory_bytes, available_physical_memory_bytes) = memory_fields();
    let (windows_volume_total_bytes, windows_volume_free_bytes) = storage_fields();

    SystemInventory {
        operating_system,
        process_architecture,
        logical_processor_count,
        total_physical_memory_bytes,
        available_physical_memory_bytes,
        windows_volume_total_bytes,
        windows_volume_free_bytes,
    }
}

fn platform_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows",
        "macos" => "macOS",
        "linux" => "Linux",
        other => other,
    }
}

#[cfg(windows)]
fn memory_fields() -> (InventoryField<u64>, InventoryField<u64>) {
    use std::mem::size_of;
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut status = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // GlobalMemoryStatusEx only reads current memory counters into the supplied struct.
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 || status.ullTotalPhys == 0 {
        return (InventoryField::unavailable(), InventoryField::unavailable());
    }
    (
        InventoryField::available(status.ullTotalPhys),
        InventoryField::available(status.ullAvailPhys),
    )
}

#[cfg(not(windows))]
fn memory_fields() -> (InventoryField<u64>, InventoryField<u64>) {
    (InventoryField::unsupported(), InventoryField::unsupported())
}

#[cfg(windows)]
fn storage_fields() -> (InventoryField<u64>, InventoryField<u64>) {
    use windows_sys::Win32::{
        Storage::FileSystem::GetDiskFreeSpaceExW, System::SystemInformation::GetWindowsDirectoryW,
    };

    let mut directory = vec![0u16; 32_768];
    let length = unsafe { GetWindowsDirectoryW(directory.as_mut_ptr(), directory.len() as u32) };
    if length == 0 || length as usize >= directory.len() {
        return (InventoryField::unavailable(), InventoryField::unavailable());
    }
    directory.truncate(length as usize + 1);
    directory[length as usize] = 0;

    let mut available_to_user = 0u64;
    let mut total_bytes = 0u64;
    let mut total_free_bytes = 0u64;
    if unsafe {
        GetDiskFreeSpaceExW(
            directory.as_ptr(),
            &mut available_to_user,
            &mut total_bytes,
            &mut total_free_bytes,
        )
    } == 0
    {
        return (InventoryField::unavailable(), InventoryField::unavailable());
    }
    (
        InventoryField::available(total_bytes),
        InventoryField::available(total_free_bytes),
    )
}

#[cfg(not(windows))]
fn storage_fields() -> (InventoryField<u64>, InventoryField<u64>) {
    (InventoryField::unsupported(), InventoryField::unsupported())
}

#[cfg(test)]
mod tests {
    use super::{
        memory_fields, platform_name, read_system_inventory, storage_fields, InventoryField,
        InventoryFieldStatus,
    };

    #[test]
    fn unavailable_and_unsupported_fields_never_carry_fabricated_values() {
        let unavailable = InventoryField::<u64>::unavailable();
        assert_eq!(unavailable.value, None);
        assert_eq!(unavailable.status, InventoryFieldStatus::Unavailable);

        let unsupported = InventoryField::<u64>::unsupported();
        assert_eq!(unsupported.value, None);
        assert_eq!(unsupported.status, InventoryFieldStatus::Unsupported);
    }

    #[test]
    fn system_metadata_has_explicit_platform_and_field_statuses() {
        let inventory = read_system_inventory();
        assert_eq!(
            inventory.operating_system.value.as_deref(),
            Some(platform_name())
        );
        assert_eq!(
            inventory.operating_system.status,
            InventoryFieldStatus::Available
        );
        assert!(inventory.process_architecture.value.is_some());
        assert!(inventory.logical_processor_count.value.is_some());

        let (memory_total, _) = memory_fields();
        let (storage_total, _) = storage_fields();
        assert_eq!(inventory.total_physical_memory_bytes, memory_total);
        assert_eq!(inventory.windows_volume_total_bytes, storage_total);
        #[cfg(not(windows))]
        {
            assert_eq!(memory_total.status, InventoryFieldStatus::Unsupported);
            assert_eq!(storage_total.status, InventoryFieldStatus::Unsupported);
        }
    }
}
