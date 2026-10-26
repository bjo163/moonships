//! One-shot, local-only metrics snapshot. No persistence, polling or network access.
use crate::system_inventory::{self, InventoryField, InventoryFieldStatus};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LocalMetricStatus {
    Available,
    Unsupported,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMetricValue {
    pub value: Option<u64>,
    pub status: LocalMetricStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMetricsSnapshot {
    pub source: &'static str,
    pub sampled_at: String,
    pub logical_processors: LocalMetricValue,
    pub physical_memory_total_bytes: LocalMetricValue,
    pub physical_memory_available_bytes: LocalMetricValue,
    pub windows_volume_total_bytes: LocalMetricValue,
    pub windows_volume_free_bytes: LocalMetricValue,
}

fn convert<T: Into<u64>>(field: InventoryField<T>) -> LocalMetricValue {
    let status = match field.status {
        InventoryFieldStatus::Available => LocalMetricStatus::Available,
        InventoryFieldStatus::Unsupported => LocalMetricStatus::Unsupported,
        InventoryFieldStatus::Unavailable => LocalMetricStatus::Unavailable,
    };
    LocalMetricValue {
        value: field.value.map(Into::into),
        status,
    }
}

pub fn read_local_metrics_snapshot() -> LocalMetricsSnapshot {
    let inventory = system_inventory::read_system_inventory();
    LocalMetricsSnapshot {
        source: "Local operating system APIs",
        sampled_at: chrono::DateTime::<Utc>::from(std::time::SystemTime::now())
            .to_rfc3339_opts(SecondsFormat::Millis, true),
        logical_processors: convert(inventory.logical_processor_count),
        physical_memory_total_bytes: convert(inventory.total_physical_memory_bytes),
        physical_memory_available_bytes: convert(inventory.available_physical_memory_bytes),
        windows_volume_total_bytes: convert(inventory.windows_volume_total_bytes),
        windows_volume_free_bytes: convert(inventory.windows_volume_free_bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::{read_local_metrics_snapshot, LocalMetricStatus};

    #[test]
    fn snapshot_declares_local_source_and_sample_time() {
        let snapshot = read_local_metrics_snapshot();
        assert_eq!(snapshot.source, "Local operating system APIs");
        assert!(chrono::DateTime::parse_from_rfc3339(&snapshot.sampled_at).is_ok());
        assert_eq!(
            snapshot.logical_processors.status,
            LocalMetricStatus::Available
        );
        assert!(snapshot.logical_processors.value.unwrap_or_default() > 0);
    }

    #[test]
    fn metric_serialization_keeps_status_and_does_not_invent_unsupported_values() {
        let snapshot = read_local_metrics_snapshot();
        let json = serde_json::to_value(snapshot).unwrap();
        assert_eq!(json["source"], "Local operating system APIs");
        assert!(json.get("sampledAt").is_some());
        assert_eq!(json["logicalProcessors"]["status"], "available");
        #[cfg(not(windows))]
        {
            assert_eq!(json["physicalMemoryTotalBytes"]["status"], "unsupported");
            assert!(json["physicalMemoryTotalBytes"]["value"].is_null());
            assert_eq!(json["windowsVolumeFreeBytes"]["status"], "unsupported");
        }
    }
}
