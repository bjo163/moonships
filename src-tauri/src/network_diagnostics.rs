use std::net::IpAddr;

pub const MIN_TIMEOUT_MS: u32 = 100;
pub const DEFAULT_TIMEOUT_MS: u32 = 2_000;
pub const MAX_TIMEOUT_MS: u32 = 10_000;
pub const MAX_DNS_ADDRESSES: usize = 32;
pub const MAX_DIAGNOSTICS_PER_MINUTE: u32 = 10;
pub const MAX_CONCURRENT_DIAGNOSTICS: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDiagnosticKind {
    DnsLookup,
    TcpConnectivity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDiagnosticRequest {
    pub kind: NetworkDiagnosticKind,
    pub target_host: String,
    pub port: Option<u16>,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedNetworkDiagnosticRequest {
    pub kind: NetworkDiagnosticKind,
    pub target_host: String,
    pub port: Option<u16>,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDiagnosticValidationError {
    MissingTarget,
    InvalidTarget,
    InvalidPort,
    UnexpectedPort,
    InvalidTimeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDiagnosticSource {
    IpLiteral,
    SystemResolver,
    OperatingSystemTcpConnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDiagnosticOutcome {
    Succeeded,
    Partial,
    Failed,
    TimedOut,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDiagnosticErrorCode {
    ResolverError,
    ConnectionRefused,
    HostUnreachable,
    PermissionDenied,
    TimedOut,
    RateLimited,
    Unsupported,
}

/// Provenance-bearing result for bounded native DNS/TCP adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDiagnosticResult {
    pub kind: NetworkDiagnosticKind,
    pub target_host: String,
    pub target_port: Option<u16>,
    /// UTC instant represented as Unix epoch milliseconds.
    pub observed_at_epoch_ms: i64,
    pub duration_ms: u32,
    pub source: NetworkDiagnosticSource,
    pub outcome: NetworkDiagnosticOutcome,
    pub error_code: Option<NetworkDiagnosticErrorCode>,
    /// Bounded to `MAX_DNS_ADDRESSES`; empty for operations with no address result.
    pub resolved_addresses: Vec<IpAddr>,
}

pub fn validate_network_diagnostic_request(
    request: NetworkDiagnosticRequest,
) -> Result<ValidatedNetworkDiagnosticRequest, NetworkDiagnosticValidationError> {
    if request.target_host.is_empty() {
        return Err(NetworkDiagnosticValidationError::MissingTarget);
    }
    if !is_valid_host(&request.target_host) {
        return Err(NetworkDiagnosticValidationError::InvalidTarget);
    }
    if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&request.timeout_ms) {
        return Err(NetworkDiagnosticValidationError::InvalidTimeout);
    }

    match (request.kind, request.port) {
        (NetworkDiagnosticKind::DnsLookup, Some(_)) => {
            return Err(NetworkDiagnosticValidationError::UnexpectedPort);
        }
        (NetworkDiagnosticKind::TcpConnectivity, None) => {
            return Err(NetworkDiagnosticValidationError::InvalidPort);
        }
        (NetworkDiagnosticKind::TcpConnectivity, Some(0)) => {
            return Err(NetworkDiagnosticValidationError::InvalidPort);
        }
        _ => {}
    }

    Ok(ValidatedNetworkDiagnosticRequest {
        kind: request.kind,
        target_host: request.target_host.to_ascii_lowercase(),
        port: request.port,
        timeout_ms: request.timeout_ms,
    })
}

fn is_valid_host(host: &str) -> bool {
    if host.len() > 253 || host.bytes().any(|byte| !byte.is_ascii()) {
        return false;
    }
    if host.parse::<IpAddr>().is_ok() {
        return true;
    }
    if host.contains(':') || host.starts_with('[') || host.ends_with(']') {
        return false;
    }

    let without_final_dot = host.strip_suffix('.').unwrap_or(host);
    if without_final_dot.is_empty()
        || without_final_dot
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return false;
    }

    without_final_dot.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label.as_bytes()[0].is_ascii_alphanumeric()
            && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

#[cfg(test)]
mod tests {
    use super::{
        validate_network_diagnostic_request, NetworkDiagnosticKind, NetworkDiagnosticRequest,
        NetworkDiagnosticValidationError as Error, DEFAULT_TIMEOUT_MS, MAX_DIAGNOSTICS_PER_MINUTE,
        MAX_DNS_ADDRESSES, MAX_TIMEOUT_MS, MIN_TIMEOUT_MS,
    };

    fn request(
        kind: NetworkDiagnosticKind,
        host: &str,
        port: Option<u16>,
        timeout_ms: u32,
    ) -> NetworkDiagnosticRequest {
        NetworkDiagnosticRequest {
            kind,
            target_host: host.to_string(),
            port,
            timeout_ms,
        }
    }

    #[test]
    fn requires_and_validates_a_single_hostname_or_ip_target() {
        assert_eq!(
            validate_network_diagnostic_request(request(
                NetworkDiagnosticKind::DnsLookup,
                "",
                None,
                DEFAULT_TIMEOUT_MS,
            )),
            Err(Error::MissingTarget)
        );
        for host in [
            "https://example.com",
            "example.com/path",
            "user@example.com",
            "bad host",
            "-bad.example",
            "bad-.example",
            "a..example",
            "999.999.1.1",
            "[::1]",
            "*.example.com",
            "example.com:443",
        ] {
            assert_eq!(
                validate_network_diagnostic_request(request(
                    NetworkDiagnosticKind::DnsLookup,
                    host,
                    None,
                    DEFAULT_TIMEOUT_MS,
                )),
                Err(Error::InvalidTarget),
                "host should be rejected: {host}"
            );
        }
        for host in [
            "example.com",
            "NAS",
            "example.com.",
            "192.0.2.5",
            "2001:db8::1",
        ] {
            assert!(validate_network_diagnostic_request(request(
                NetworkDiagnosticKind::DnsLookup,
                host,
                None,
                DEFAULT_TIMEOUT_MS,
            ))
            .is_ok());
        }
    }

    #[test]
    fn requires_one_valid_port_only_for_tcp_connectivity() {
        assert_eq!(
            validate_network_diagnostic_request(request(
                NetworkDiagnosticKind::TcpConnectivity,
                "example.com",
                None,
                DEFAULT_TIMEOUT_MS,
            )),
            Err(Error::InvalidPort)
        );
        assert_eq!(
            validate_network_diagnostic_request(request(
                NetworkDiagnosticKind::TcpConnectivity,
                "example.com",
                Some(0),
                DEFAULT_TIMEOUT_MS,
            )),
            Err(Error::InvalidPort)
        );
        assert_eq!(
            validate_network_diagnostic_request(request(
                NetworkDiagnosticKind::DnsLookup,
                "example.com",
                Some(443),
                DEFAULT_TIMEOUT_MS,
            )),
            Err(Error::UnexpectedPort)
        );
        assert!(validate_network_diagnostic_request(request(
            NetworkDiagnosticKind::TcpConnectivity,
            "example.com",
            Some(u16::MAX),
            DEFAULT_TIMEOUT_MS,
        ))
        .is_ok());
    }

    #[test]
    fn enforces_configured_timeout_bounds_and_normalizes_only_hostname_case() {
        for timeout in [0, MIN_TIMEOUT_MS - 1, MAX_TIMEOUT_MS + 1, u32::MAX] {
            assert_eq!(
                validate_network_diagnostic_request(request(
                    NetworkDiagnosticKind::DnsLookup,
                    "example.com",
                    None,
                    timeout,
                )),
                Err(Error::InvalidTimeout)
            );
        }
        let valid = validate_network_diagnostic_request(request(
            NetworkDiagnosticKind::DnsLookup,
            "ExAmPlE.com",
            None,
            MIN_TIMEOUT_MS,
        ))
        .unwrap();
        assert_eq!(valid.target_host, "example.com");
        assert_eq!(valid.timeout_ms, MIN_TIMEOUT_MS);
    }

    #[test]
    fn declares_explicit_result_and_rate_limits() {
        assert_eq!(MAX_DNS_ADDRESSES, 32);
        assert_eq!(MAX_DIAGNOSTICS_PER_MINUTE, 10);
        assert_eq!(MAX_TIMEOUT_MS, 10_000);
    }
}
