use crate::network_diagnostics::{
    validate_network_diagnostic_request, NetworkDiagnosticKind, NetworkDiagnosticRequest,
    DEFAULT_TIMEOUT_MS,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ConnectionProfile {
    pub profile_id: Uuid,
    pub name: String,
    pub connection: ConnectionProfileKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum ConnectionProfileKind {
    Terminal {
        application: TerminalApplication,
    },
    Ssh {
        host: String,
        port: u16,
        username: Option<String>,
        secret_reference: Option<Uuid>,
    },
    Rdp {
        host: String,
        port: u16,
        username: Option<String>,
        secret_reference: Option<Uuid>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TerminalApplication {
    WindowsTerminal,
    PowerShell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProfileValidationError {
    InvalidName,
    InvalidHost,
    InvalidPort,
    InvalidUsername,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProfileExport {
    pub format: &'static str,
    pub schema_version: u32,
    pub profile_id: Uuid,
    pub name: String,
    pub connection: ExportConnectionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportConnectionKind {
    Terminal {
        application: TerminalApplication,
    },
    Ssh {
        host: String,
        port: u16,
        username: Option<String>,
    },
    Rdp {
        host: String,
        port: u16,
        username: Option<String>,
    },
}

pub fn validate_connection_profile(
    profile: &ConnectionProfile,
) -> Result<(), ConnectionProfileValidationError> {
    let name = profile.name.trim();
    if name.is_empty() || name.len() > 80 || name.chars().any(char::is_control) {
        return Err(ConnectionProfileValidationError::InvalidName);
    }
    match &profile.connection {
        ConnectionProfileKind::Terminal { .. } => Ok(()),
        ConnectionProfileKind::Ssh {
            host,
            port,
            username,
            ..
        }
        | ConnectionProfileKind::Rdp {
            host,
            port,
            username,
            ..
        } => {
            let target_valid = validate_network_diagnostic_request(NetworkDiagnosticRequest {
                kind: NetworkDiagnosticKind::DnsLookup,
                target_host: host.clone(),
                port: None,
                timeout_ms: DEFAULT_TIMEOUT_MS,
            })
            .is_ok();
            if !target_valid {
                return Err(ConnectionProfileValidationError::InvalidHost);
            }
            if *port == 0 {
                return Err(ConnectionProfileValidationError::InvalidPort);
            }
            if username.as_ref().is_some_and(|value| {
                value.is_empty() || value.len() > 256 || value.chars().any(char::is_control)
            }) {
                return Err(ConnectionProfileValidationError::InvalidUsername);
            }
            Ok(())
        }
    }
}

/// Creates an explicit export shape that has no field capable of carrying a secret reference.
pub fn export_connection_profile(
    profile: &ConnectionProfile,
) -> Result<ConnectionProfileExport, ConnectionProfileValidationError> {
    validate_connection_profile(profile)?;
    let connection = match &profile.connection {
        ConnectionProfileKind::Terminal { application } => ExportConnectionKind::Terminal {
            application: *application,
        },
        ConnectionProfileKind::Ssh {
            host,
            port,
            username,
            ..
        } => ExportConnectionKind::Ssh {
            host: host.clone(),
            port: *port,
            username: username.clone(),
        },
        ConnectionProfileKind::Rdp {
            host,
            port,
            username,
            ..
        } => ExportConnectionKind::Rdp {
            host: host.clone(),
            port: *port,
            username: username.clone(),
        },
    };
    Ok(ConnectionProfileExport {
        format: "moonships.connection-profile-export",
        schema_version: 1,
        profile_id: profile.profile_id,
        name: profile.name.clone(),
        connection,
    })
}

/// Deliberately excludes profile names, hosts, usernames and secret references from logs.
pub fn safe_profile_log_summary(profile: &ConnectionProfile) -> Value {
    json!({
        "profileId": profile.profile_id,
        "connectionKind": connection_kind_name(&profile.connection),
    })
}

fn connection_kind_name(connection: &ConnectionProfileKind) -> &'static str {
    match connection {
        ConnectionProfileKind::Terminal { .. } => "terminal",
        ConnectionProfileKind::Ssh { .. } => "ssh",
        ConnectionProfileKind::Rdp { .. } => "rdp",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        export_connection_profile, safe_profile_log_summary, validate_connection_profile,
        ConnectionProfile, ConnectionProfileKind, ConnectionProfileValidationError as Error,
        TerminalApplication,
    };
    use uuid::Uuid;

    fn ssh_profile() -> ConnectionProfile {
        ConnectionProfile {
            profile_id: Uuid::from_u128(1),
            name: "Production SSH".to_string(),
            connection: ConnectionProfileKind::Ssh {
                host: "host.example".to_string(),
                port: 22,
                username: Some("operator".to_string()),
                secret_reference: Some(Uuid::from_u128(0xdead_beef)),
            },
        }
    }

    #[test]
    fn profile_types_are_allowlisted_and_validate_single_hosts_and_ports() {
        assert!(validate_connection_profile(&ssh_profile()).is_ok());
        let terminal = ConnectionProfile {
            profile_id: Uuid::from_u128(2),
            name: "Local console".to_string(),
            connection: ConnectionProfileKind::Terminal {
                application: TerminalApplication::WindowsTerminal,
            },
        };
        assert!(validate_connection_profile(&terminal).is_ok());

        let mut bad_host = ssh_profile();
        if let ConnectionProfileKind::Ssh { host, .. } = &mut bad_host.connection {
            *host = "https://host.example/path".to_string();
        }
        assert_eq!(
            validate_connection_profile(&bad_host),
            Err(Error::InvalidHost)
        );

        let mut bad_port = ssh_profile();
        if let ConnectionProfileKind::Ssh { port, .. } = &mut bad_port.connection {
            *port = 0;
        }
        assert_eq!(
            validate_connection_profile(&bad_port),
            Err(Error::InvalidPort)
        );

        let mut bad_name = ssh_profile();
        bad_name.name = "  ".to_string();
        assert_eq!(
            validate_connection_profile(&bad_name),
            Err(Error::InvalidName)
        );
    }

    #[test]
    fn profile_export_never_contains_secret_reference_or_secret_values() {
        let serialized =
            serde_json::to_string(&export_connection_profile(&ssh_profile()).unwrap()).unwrap();
        assert!(serialized.contains("host.example"));
        assert!(serialized.contains("operator"));
        assert!(!serialized.contains("secretReference"));
        assert!(!serialized.contains("deadbeef"));
        assert!(!serialized.contains("password"));
    }

    #[test]
    fn safe_profile_log_summary_omits_profile_content_and_secret_reference() {
        let serialized = safe_profile_log_summary(&ssh_profile()).to_string();
        assert!(serialized.contains("ssh"));
        assert!(!serialized.contains("Production SSH"));
        assert!(!serialized.contains("host.example"));
        assert!(!serialized.contains("operator"));
        assert!(!serialized.contains("deadbeef"));
    }

    #[test]
    fn unknown_terminal_application_is_rejected_by_allowlist_deserialization() {
        let input = r#""arbitrary.exe""#;
        assert!(serde_json::from_str::<TerminalApplication>(input).is_err());
    }

    #[test]
    fn profile_import_shape_rejects_plaintext_secret_fields() {
        let mut value = serde_json::to_value(ssh_profile()).unwrap();
        value["password"] = serde_json::json!("top-secret-sentinel");
        assert!(serde_json::from_value::<ConnectionProfile>(value).is_err());

        let mut value = serde_json::to_value(ssh_profile()).unwrap();
        value["connection"]["secretValue"] = serde_json::json!("top-secret-sentinel");
        assert!(serde_json::from_value::<ConnectionProfile>(value).is_err());
    }
}
