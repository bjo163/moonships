/// Traceroute remains unavailable until a reviewed adapter can bound both hops and duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracerouteStatus {
    Deferred(TracerouteDeferredReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracerouteDeferredReason {
    NoReviewedBoundedAdapter,
}

/// Returns an explicit deferred state; this does not launch an operating-system process.
pub fn traceroute_status() -> TracerouteStatus {
    TracerouteStatus::Deferred(TracerouteDeferredReason::NoReviewedBoundedAdapter)
}

#[cfg(test)]
mod tests {
    use super::{traceroute_status, TracerouteDeferredReason, TracerouteStatus};

    #[test]
    fn traceroute_is_explicitly_deferred_without_a_process_fallback() {
        assert_eq!(
            traceroute_status(),
            TracerouteStatus::Deferred(TracerouteDeferredReason::NoReviewedBoundedAdapter)
        );
    }
}
