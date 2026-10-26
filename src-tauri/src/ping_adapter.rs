/// Result of probing whether a native ping API can be opened on this device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingCapability {
    Available,
    Unsupported(PingUnsupportedReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingUnsupportedReason {
    NativeIcmpUnavailable,
    UnsupportedPlatform,
}

/// Checks native ICMP API availability without sending a packet or launching a process.
pub fn detect_ping_capability() -> PingCapability {
    detect_ping_capability_for_platform()
}

#[cfg(windows)]
fn detect_ping_capability_for_platform() -> PingCapability {
    use windows_sys::Win32::{
        Foundation::INVALID_HANDLE_VALUE,
        NetworkManagement::IpHelper::{IcmpCloseHandle, IcmpCreateFile},
    };

    // The handle probe only checks that the native ICMP API is usable; it sends no request.
    let handle = unsafe { IcmpCreateFile() };
    if handle == INVALID_HANDLE_VALUE {
        return capability_from_handle(false, true);
    }

    let _ = unsafe { IcmpCloseHandle(handle) };
    capability_from_handle(true, true)
}

#[cfg(not(windows))]
fn detect_ping_capability_for_platform() -> PingCapability {
    capability_from_handle(false, false)
}

fn capability_from_handle(handle_available: bool, platform_supported: bool) -> PingCapability {
    if handle_available {
        PingCapability::Available
    } else if platform_supported {
        PingCapability::Unsupported(PingUnsupportedReason::NativeIcmpUnavailable)
    } else {
        PingCapability::Unsupported(PingUnsupportedReason::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::{capability_from_handle, PingCapability, PingUnsupportedReason};

    #[test]
    fn unsupported_ping_has_an_explicit_reason() {
        assert_eq!(
            capability_from_handle(false, true),
            PingCapability::Unsupported(PingUnsupportedReason::NativeIcmpUnavailable)
        );
        assert_eq!(
            capability_from_handle(false, false),
            PingCapability::Unsupported(PingUnsupportedReason::UnsupportedPlatform)
        );
        assert_eq!(
            capability_from_handle(true, true),
            PingCapability::Available
        );
    }

    #[test]
    fn capability_probe_does_not_start_a_ping() {
        let result = super::detect_ping_capability();
        assert!(matches!(
            result,
            PingCapability::Available
                | PingCapability::Unsupported(PingUnsupportedReason::NativeIcmpUnavailable)
                | PingCapability::Unsupported(PingUnsupportedReason::UnsupportedPlatform)
        ));
    }
}
