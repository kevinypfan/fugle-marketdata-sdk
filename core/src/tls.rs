//! TLS configuration for REST (`ureq`) and WebSocket (`tokio-tungstenite`).
//!
//! Both transports share the same [`TlsConfig`] shape and the
//! [`build_native_tls_connector`] helper, so a user-supplied root CA or
//! "accept invalid certs" flag applies uniformly across the SDK.

use crate::errors::MarketDataError;

/// Optional TLS customization. When all fields are default the SDK uses
/// the OS trust store via `native-tls` — the same behaviour as the
/// default `ureq` / `tokio-tungstenite` configuration.
#[derive(Clone, Debug, Default)]
pub struct TlsConfig {
    /// PEM-encoded additional root CA. Pinned via
    /// `TlsConnectorBuilder::add_root_certificate`; the standard system
    /// trust store is still consulted in addition to this certificate.
    pub root_cert_pem: Option<Vec<u8>>,

    /// Disable ALL TLS verification (chain + hostname). Equivalent to
    /// `wscat --no-check` or `curl -k`. Do not use in production —
    /// exposes the client to trivial MITM. Prefer `root_cert_pem`.
    pub accept_invalid_certs: bool,
}

/// Build a [`native_tls::TlsConnector`] honoring any custom root CA or
/// `accept_invalid_certs` flag in `tls`. On default config this returns
/// a connector that uses the system trust store — equivalent to
/// `native_tls::TlsConnector::new()`. ureq's `native-tls` feature is
/// builder-only (no default fallback), so REST callers MUST always
/// install this connector via `AgentBuilder::tls_connector`.
pub fn build_native_tls_connector(
    tls: &TlsConfig,
) -> Result<native_tls::TlsConnector, MarketDataError> {
    let mut builder = native_tls::TlsConnector::builder();

    if let Some(pem) = &tls.root_cert_pem {
        let cert = native_tls::Certificate::from_pem(pem).map_err(|e| {
            MarketDataError::ConfigError(format!("invalid TLS root cert PEM: {e}"))
        })?;
        builder.add_root_certificate(cert);
    }

    if tls.accept_invalid_certs {
        builder.danger_accept_invalid_certs(true);
        builder.danger_accept_invalid_hostnames(true);
        // Core has no logging infra. Binding layers (py warnings, future
        // JS/Java/C#) are responsible for surfacing this to the user.
    }

    builder.build().map_err(|e| {
        MarketDataError::ConfigError(format!("TlsConnector build failed: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_builds_connector() {
        let cfg = TlsConfig::default();
        let _ = build_native_tls_connector(&cfg).expect("default should always build");
    }

    #[test]
    fn accept_invalid_certs_builds_connector() {
        let cfg = TlsConfig {
            accept_invalid_certs: true,
            ..Default::default()
        };
        let _ = build_native_tls_connector(&cfg).expect("should build");
    }

    #[test]
    fn invalid_pem_is_config_error() {
        let cfg = TlsConfig {
            root_cert_pem: Some(b"not a real pem".to_vec()),
            ..Default::default()
        };
        let err = build_native_tls_connector(&cfg).expect_err("bad PEM should error");
        match err {
            MarketDataError::ConfigError(msg) => assert!(msg.contains("invalid TLS root cert PEM")),
            other => panic!("expected ConfigError, got {other:?}"),
        }
    }

}
