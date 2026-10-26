use crate::network_diagnostics::{
    validate_network_diagnostic_request, NetworkDiagnosticErrorCode, NetworkDiagnosticKind,
    NetworkDiagnosticOutcome, NetworkDiagnosticRequest, NetworkDiagnosticResult,
    NetworkDiagnosticSource, NetworkDiagnosticValidationError, ValidatedNetworkDiagnosticRequest,
    MAX_CONCURRENT_DIAGNOSTICS, MAX_DIAGNOSTICS_PER_MINUTE, MAX_DNS_ADDRESSES,
};
use std::{
    collections::{HashSet, VecDeque},
    io,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    sync::{
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const RATE_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsLookupError {
    InvalidRequest(NetworkDiagnosticValidationError),
    WrongOperation,
    RateLimited,
    ResolverError,
    TimedOut,
    ResolverUnavailable,
}

struct LookupJob {
    host: String,
    deadline: Instant,
    response: mpsc::Sender<io::Result<Vec<SocketAddr>>>,
}

#[derive(Default)]
struct RateWindow {
    started: VecDeque<Instant>,
}

impl RateWindow {
    fn acquire(&mut self, now: Instant) -> bool {
        while self
            .started
            .front()
            .is_some_and(|started| now.duration_since(*started) >= RATE_WINDOW)
        {
            self.started.pop_front();
        }
        if self.started.len() >= MAX_DIAGNOSTICS_PER_MINUTE as usize {
            return false;
        }
        self.started.push_back(now);
        true
    }
}

static RATE_WINDOW_STATE: OnceLock<Mutex<RateWindow>> = OnceLock::new();
static RESOLVER_SENDER: OnceLock<Option<SyncSender<LookupJob>>> = OnceLock::new();

/// Performs one system-resolver lookup, bounded by the shared network contract.
/// The renderer must pass only a typed request and must display its host/timeout.
pub fn lookup_dns(
    request: NetworkDiagnosticRequest,
) -> Result<NetworkDiagnosticResult, DnsLookupError> {
    let validated =
        validate_network_diagnostic_request(request).map_err(DnsLookupError::InvalidRequest)?;
    if validated.kind != NetworkDiagnosticKind::DnsLookup {
        return Err(DnsLookupError::WrongOperation);
    }

    if let Ok(address) = validated.target_host.parse::<IpAddr>() {
        return Ok(NetworkDiagnosticResult {
            kind: validated.kind,
            target_host: validated.target_host,
            target_port: None,
            observed_at_epoch_ms: unix_epoch_milliseconds(),
            duration_ms: 0,
            source: NetworkDiagnosticSource::IpLiteral,
            outcome: NetworkDiagnosticOutcome::Succeeded,
            error_code: None,
            resolved_addresses: vec![address],
        });
    }

    let now = Instant::now();
    let rate_limiter = RATE_WINDOW_STATE.get_or_init(|| Mutex::new(RateWindow::default()));
    if !rate_limiter
        .lock()
        .map(|mut limiter| limiter.acquire(now))
        .unwrap_or(false)
    {
        return Ok(failure_result(
            &validated,
            NetworkDiagnosticOutcome::Failed,
            NetworkDiagnosticErrorCode::RateLimited,
            0,
        ));
    }

    let Some(sender) = resolver_sender() else {
        return Err(DnsLookupError::ResolverUnavailable);
    };
    let (response_tx, response_rx) = mpsc::channel();
    let deadline = now + Duration::from_millis(u64::from(validated.timeout_ms));
    let observed_at_epoch_ms = unix_epoch_milliseconds();
    let started = now;
    let job = LookupJob {
        host: validated.target_host.clone(),
        deadline,
        response: response_tx,
    };
    match sender.try_send(job) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            return Ok(failure_result(
                &validated,
                NetworkDiagnosticOutcome::Failed,
                NetworkDiagnosticErrorCode::RateLimited,
                0,
            ));
        }
        Err(TrySendError::Disconnected(_)) => return Err(DnsLookupError::ResolverUnavailable),
    }

    let remaining = deadline.saturating_duration_since(Instant::now());
    match response_rx.recv_timeout(remaining) {
        Ok(Ok(addresses)) => {
            let (resolved_addresses, truncated) = cap_addresses(addresses);
            if resolved_addresses.is_empty() {
                return Ok(failure_result(
                    &validated,
                    NetworkDiagnosticOutcome::Failed,
                    NetworkDiagnosticErrorCode::ResolverError,
                    elapsed_ms(started),
                ));
            }
            Ok(NetworkDiagnosticResult {
                kind: validated.kind,
                target_host: validated.target_host,
                target_port: None,
                observed_at_epoch_ms,
                duration_ms: elapsed_ms(started),
                source: NetworkDiagnosticSource::SystemResolver,
                outcome: if truncated {
                    NetworkDiagnosticOutcome::Partial
                } else {
                    NetworkDiagnosticOutcome::Succeeded
                },
                error_code: None,
                resolved_addresses,
            })
        }
        Ok(Err(error)) => Ok(resolver_failure_result(
            &validated,
            error.kind(),
            elapsed_ms(started),
        )),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(failure_result(
            &validated,
            NetworkDiagnosticOutcome::TimedOut,
            NetworkDiagnosticErrorCode::TimedOut,
            validated.timeout_ms,
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(DnsLookupError::ResolverUnavailable),
    }
}

fn resolver_sender() -> Option<&'static SyncSender<LookupJob>> {
    RESOLVER_SENDER
        .get_or_init(|| {
            let (sender, receiver) = mpsc::sync_channel(MAX_CONCURRENT_DIAGNOSTICS);
            thread::Builder::new()
                .name("moonships-dns-resolver".to_string())
                .spawn(move || resolver_worker(receiver))
                .ok()
                .map(|_| sender)
        })
        .as_ref()
}

fn resolver_worker(receiver: Receiver<LookupJob>) {
    while let Ok(job) = receiver.recv() {
        if Instant::now() >= job.deadline {
            let _ = job.response.send(Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "lookup deadline expired",
            )));
            continue;
        }
        let result = resolve_host(&job.host);
        let _ = job.response.send(result);
    }
}

fn resolve_host(host: &str) -> io::Result<Vec<SocketAddr>> {
    (host, 0)
        .to_socket_addrs()
        .map(|addresses| addresses.collect())
}

fn cap_addresses(addresses: Vec<SocketAddr>) -> (Vec<IpAddr>, bool) {
    let mut seen = HashSet::new();
    let mut bounded = Vec::with_capacity(MAX_DNS_ADDRESSES);
    let mut truncated = false;
    for address in addresses {
        let ip = address.ip();
        if !seen.insert(ip) {
            continue;
        }
        if bounded.len() == MAX_DNS_ADDRESSES {
            truncated = true;
            break;
        }
        bounded.push(ip);
    }
    (bounded, truncated)
}

fn failure_result(
    request: &ValidatedNetworkDiagnosticRequest,
    outcome: NetworkDiagnosticOutcome,
    error_code: NetworkDiagnosticErrorCode,
    duration_ms: u32,
) -> NetworkDiagnosticResult {
    NetworkDiagnosticResult {
        kind: request.kind,
        target_host: request.target_host.clone(),
        target_port: None,
        observed_at_epoch_ms: unix_epoch_milliseconds(),
        duration_ms,
        source: NetworkDiagnosticSource::SystemResolver,
        outcome,
        error_code: Some(error_code),
        resolved_addresses: Vec::new(),
    }
}

fn resolver_failure_result(
    request: &ValidatedNetworkDiagnosticRequest,
    error_kind: io::ErrorKind,
    duration_ms: u32,
) -> NetworkDiagnosticResult {
    if error_kind == io::ErrorKind::TimedOut {
        failure_result(
            request,
            NetworkDiagnosticOutcome::TimedOut,
            NetworkDiagnosticErrorCode::TimedOut,
            duration_ms,
        )
    } else {
        failure_result(
            request,
            NetworkDiagnosticOutcome::Failed,
            NetworkDiagnosticErrorCode::ResolverError,
            duration_ms,
        )
    }
}

fn elapsed_ms(started: Instant) -> u32 {
    started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32
}

fn unix_epoch_milliseconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{cap_addresses, lookup_dns, DnsLookupError, RateWindow};
    use crate::network_diagnostics::{
        validate_network_diagnostic_request, NetworkDiagnosticErrorCode, NetworkDiagnosticKind,
        NetworkDiagnosticOutcome, NetworkDiagnosticRequest, NetworkDiagnosticValidationError,
        MAX_DNS_ADDRESSES,
    };
    use std::{
        io,
        net::{IpAddr, Ipv4Addr, SocketAddr},
        time::{Duration, Instant},
    };

    #[test]
    fn distinguishes_invalid_target_from_system_resolver_failure() {
        let invalid = validate_network_diagnostic_request(NetworkDiagnosticRequest {
            kind: NetworkDiagnosticKind::DnsLookup,
            target_host: "bad host".to_string(),
            port: None,
            timeout_ms: 2_000,
        });
        assert_eq!(
            invalid,
            Err(NetworkDiagnosticValidationError::InvalidTarget)
        );

        let resolver_result = super::resolver_failure_result(
            &crate::network_diagnostics::validate_network_diagnostic_request(
                NetworkDiagnosticRequest {
                    kind: NetworkDiagnosticKind::DnsLookup,
                    target_host: "missing.example".to_string(),
                    port: None,
                    timeout_ms: 2_000,
                },
            )
            .unwrap(),
            io::ErrorKind::NotFound,
            1,
        );
        assert_eq!(resolver_result.outcome, NetworkDiagnosticOutcome::Failed);
        assert_eq!(
            resolver_result.error_code,
            Some(NetworkDiagnosticErrorCode::ResolverError)
        );
        assert_ne!(
            DnsLookupError::InvalidRequest(NetworkDiagnosticValidationError::InvalidTarget),
            DnsLookupError::ResolverUnavailable
        );
    }

    #[test]
    fn deduplicates_and_caps_resolved_addresses_with_explicit_truncation() {
        let mut addresses = vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0); 2];
        addresses.extend(
            (1..=MAX_DNS_ADDRESSES as u8)
                .map(|last| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, last)), 0)),
        );
        let (bounded, truncated) = cap_addresses(addresses);
        assert_eq!(bounded.len(), MAX_DNS_ADDRESSES);
        assert!(truncated);
        assert_eq!(bounded[0], IpAddr::V4(Ipv4Addr::LOCALHOST));

        let (empty, was_truncated) = cap_addresses(Vec::new());
        assert!(empty.is_empty());
        assert!(!was_truncated);
    }

    #[test]
    fn rate_window_limits_runs_and_reopens_after_sixty_seconds() {
        let start = Instant::now();
        let mut limiter = RateWindow::default();
        for second in 0..10 {
            assert!(limiter.acquire(start + Duration::from_secs(second)));
        }
        assert!(!limiter.acquire(start + Duration::from_secs(10)));
        assert!(limiter.acquire(start + Duration::from_secs(60)));
    }

    #[test]
    fn ip_literal_returns_without_calling_the_system_resolver() {
        let result = lookup_dns(NetworkDiagnosticRequest {
            kind: NetworkDiagnosticKind::DnsLookup,
            target_host: "127.0.0.1".to_string(),
            port: None,
            timeout_ms: 2_000,
        })
        .unwrap();
        assert_eq!(
            result.source,
            crate::network_diagnostics::NetworkDiagnosticSource::IpLiteral
        );
        assert_eq!(
            result.resolved_addresses,
            vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]
        );
        assert_eq!(result.outcome, NetworkDiagnosticOutcome::Succeeded);
        assert_eq!(result.duration_ms, 0);
    }
}
