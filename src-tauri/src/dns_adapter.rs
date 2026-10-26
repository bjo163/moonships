use crate::network_diagnostics::{
    validate_network_diagnostic_request, NetworkDiagnosticErrorCode, NetworkDiagnosticKind,
    NetworkDiagnosticOutcome, NetworkDiagnosticRequest, NetworkDiagnosticResult,
    NetworkDiagnosticSource, NetworkDiagnosticValidationError, ValidatedNetworkDiagnosticRequest,
    MAX_CONCURRENT_DIAGNOSTICS, MAX_DIAGNOSTICS_PER_MINUTE, MAX_DNS_ADDRESSES,
};
use std::{
    collections::{HashSet, VecDeque},
    io,
    net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs},
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
    operation: LookupOperation,
    deadline: Instant,
    response: mpsc::Sender<io::Result<Vec<SocketAddr>>>,
}

#[derive(Clone, Copy)]
enum LookupOperation {
    Dns,
    TcpConnect(u16),
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
        operation: LookupOperation::Dns,
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

/// Resolves one target and attempts a TCP connection to one explicit port.
/// DNS resolution and connection share the same bounded worker and rate window.
pub fn test_tcp_connectivity(
    request: NetworkDiagnosticRequest,
) -> Result<NetworkDiagnosticResult, DnsLookupError> {
    let validated =
        validate_network_diagnostic_request(request).map_err(DnsLookupError::InvalidRequest)?;
    if validated.kind != NetworkDiagnosticKind::TcpConnectivity {
        return Err(DnsLookupError::WrongOperation);
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
    let started = now;
    let observed_at_epoch_ms = unix_epoch_milliseconds();
    let job = LookupJob {
        host: validated.target_host.clone(),
        operation: LookupOperation::TcpConnect(validated.port.expect("validated TCP port")),
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
            Ok(NetworkDiagnosticResult {
                kind: validated.kind,
                target_host: validated.target_host,
                target_port: validated.port,
                observed_at_epoch_ms,
                duration_ms: elapsed_ms(started),
                source: NetworkDiagnosticSource::OperatingSystemTcpConnect,
                outcome: if truncated {
                    NetworkDiagnosticOutcome::Partial
                } else {
                    NetworkDiagnosticOutcome::Succeeded
                },
                error_code: None,
                resolved_addresses,
            })
        }
        Ok(Err(error)) => {
            let (outcome, code) = tcp_error(&error);
            Ok(failure_result_with_port(
                &validated,
                outcome,
                code,
                elapsed_ms(started),
            ))
        }
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(failure_result_with_port(
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
        let result = match job.operation {
            LookupOperation::Dns => resolve_host(&job.host),
            LookupOperation::TcpConnect(port) => connect_host(&job.host, port, job.deadline),
        };
        let _ = job.response.send(result);
    }
}

fn resolve_host(host: &str) -> io::Result<Vec<SocketAddr>> {
    (host, 0)
        .to_socket_addrs()
        .map(|addresses| addresses.collect())
}

fn connect_host(host: &str, port: u16, deadline: Instant) -> io::Result<Vec<SocketAddr>> {
    let addresses = resolve_host(host).map_err(|error| {
        if error.kind() == io::ErrorKind::TimedOut {
            error
        } else {
            io::Error::new(io::ErrorKind::NotFound, error.to_string())
        }
    })?;
    let mut last_error = None;
    for address in &addresses {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "connection deadline expired",
            ));
        }
        match TcpStream::connect_timeout(&SocketAddr::new(address.ip(), port), remaining) {
            Ok(stream) => {
                drop(stream);
                return Ok(addresses);
            }
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error
        .unwrap_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no addresses resolved")))
}

fn tcp_error(error: &io::Error) -> (NetworkDiagnosticOutcome, NetworkDiagnosticErrorCode) {
    match error.kind() {
        io::ErrorKind::TimedOut => (
            NetworkDiagnosticOutcome::TimedOut,
            NetworkDiagnosticErrorCode::TimedOut,
        ),
        io::ErrorKind::ConnectionRefused => (
            NetworkDiagnosticOutcome::Failed,
            NetworkDiagnosticErrorCode::ConnectionRefused,
        ),
        io::ErrorKind::PermissionDenied => (
            NetworkDiagnosticOutcome::Failed,
            NetworkDiagnosticErrorCode::PermissionDenied,
        ),
        io::ErrorKind::NotFound => (
            NetworkDiagnosticOutcome::Failed,
            NetworkDiagnosticErrorCode::ResolverError,
        ),
        _ => (
            NetworkDiagnosticOutcome::Failed,
            NetworkDiagnosticErrorCode::HostUnreachable,
        ),
    }
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
    failure_result_with_port(request, outcome, error_code, duration_ms)
}

fn failure_result_with_port(
    request: &ValidatedNetworkDiagnosticRequest,
    outcome: NetworkDiagnosticOutcome,
    error_code: NetworkDiagnosticErrorCode,
    duration_ms: u32,
) -> NetworkDiagnosticResult {
    NetworkDiagnosticResult {
        kind: request.kind,
        target_host: request.target_host.clone(),
        target_port: request.port,
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
        net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
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

    #[test]
    fn dns_request_validation_fails_before_any_resolver_work() {
        let error = lookup_dns(NetworkDiagnosticRequest {
            kind: NetworkDiagnosticKind::DnsLookup,
            target_host: "bad host".to_string(),
            port: None,
            timeout_ms: 2_000,
        })
        .unwrap_err();
        assert_eq!(
            error,
            DnsLookupError::InvalidRequest(NetworkDiagnosticValidationError::InvalidTarget)
        );
    }

    #[test]
    fn expired_local_tcp_deadline_returns_timeout_without_connecting() {
        let error = super::connect_host("127.0.0.1", 9, Instant::now()).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert_eq!(
            super::tcp_error(&error),
            (
                NetworkDiagnosticOutcome::TimedOut,
                NetworkDiagnosticErrorCode::TimedOut
            )
        );
    }

    #[test]
    fn tcp_connect_reports_local_success_and_maps_refusal_and_timeout_separately() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let succeeded = super::test_tcp_connectivity(NetworkDiagnosticRequest {
            kind: NetworkDiagnosticKind::TcpConnectivity,
            target_host: "127.0.0.1".to_string(),
            port: Some(port),
            timeout_ms: 2_000,
        })
        .unwrap();
        assert_eq!(succeeded.outcome, NetworkDiagnosticOutcome::Succeeded);
        assert_eq!(succeeded.target_port, Some(port));
        drop(listener);

        let refused_port = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let refused = super::test_tcp_connectivity(NetworkDiagnosticRequest {
            kind: NetworkDiagnosticKind::TcpConnectivity,
            target_host: "127.0.0.1".to_string(),
            port: Some(refused_port),
            timeout_ms: 2_000,
        })
        .unwrap();
        assert!(matches!(
            (refused.outcome, refused.error_code),
            (
                NetworkDiagnosticOutcome::Failed,
                Some(NetworkDiagnosticErrorCode::ConnectionRefused)
            ) | (
                NetworkDiagnosticOutcome::TimedOut,
                Some(NetworkDiagnosticErrorCode::TimedOut)
            )
        ));

        assert_eq!(
            super::tcp_error(&io::Error::new(io::ErrorKind::ConnectionRefused, "fixture")),
            (
                NetworkDiagnosticOutcome::Failed,
                NetworkDiagnosticErrorCode::ConnectionRefused
            )
        );
        assert_eq!(
            super::tcp_error(&io::Error::new(io::ErrorKind::TimedOut, "fixture")),
            (
                NetworkDiagnosticOutcome::TimedOut,
                NetworkDiagnosticErrorCode::TimedOut
            )
        );
    }
}
