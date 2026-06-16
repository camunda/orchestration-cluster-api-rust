//! TLS / mutual-TLS support for the HTTP client.
//!
//! Mirrors the JS / Python / C# SDKs' `CAMUNDA_MTLS_*` surface: a client certificate and
//! key enable mutual TLS, and a CA certificate enables trusting a private certificate
//! authority. Inline PEM values take precedence over `*_PATH` file locations.
//!
//! The TLS backend is selected by crate feature: `native-tls` (default) or `rustls`.

use super::config::TlsConfig;
use super::errors::{CamundaError, Result};

/// Apply the configured TLS material to a [`reqwest::ClientBuilder`].
///
/// When no TLS material is configured the builder is returned unchanged.
pub(crate) fn apply_tls(
    mut builder: reqwest::ClientBuilder,
    tls: &TlsConfig,
) -> Result<reqwest::ClientBuilder> {
    if !tls.is_configured() {
        return Ok(builder);
    }

    if let Some(ca_pem) = read_pem(&tls.ca, &tls.ca_path, "CA certificate")? {
        let cert = reqwest::Certificate::from_pem(&ca_pem).map_err(|e| {
            CamundaError::config(format!("invalid CAMUNDA_MTLS_CA certificate: {e}"))
        })?;
        builder = builder.add_root_certificate(cert);
    }

    let cert_pem = read_pem(&tls.cert, &tls.cert_path, "client certificate")?;
    let key_pem = read_pem(&tls.key, &tls.key_path, "client key")?;
    match (cert_pem, key_pem) {
        (Some(cert), Some(key)) => {
            let identity = build_identity(&cert, &key, tls.key_passphrase.as_deref())?;
            builder = builder.identity(identity);
        }
        (None, None) => {}
        _ => {
            return Err(CamundaError::config(
                "mutual TLS requires both a client certificate (CAMUNDA_MTLS_CERT[_PATH]) and key (CAMUNDA_MTLS_KEY[_PATH])",
            ));
        }
    }

    Ok(builder)
}

/// Resolve PEM bytes from an inline value (preferred) or a file path.
fn read_pem(inline: &Option<String>, path: &Option<String>, what: &str) -> Result<Option<Vec<u8>>> {
    if let Some(pem) = inline {
        return Ok(Some(pem.clone().into_bytes()));
    }
    if let Some(p) = path {
        let bytes = std::fs::read(p)
            .map_err(|e| CamundaError::config(format!("failed to read {what} from {p:?}: {e}")))?;
        return Ok(Some(bytes));
    }
    Ok(None)
}

#[cfg(feature = "native-tls")]
fn build_identity(cert: &[u8], key: &[u8], passphrase: Option<&str>) -> Result<reqwest::Identity> {
    if passphrase.is_some() {
        return Err(CamundaError::config(
            "CAMUNDA_MTLS_KEY_PASSPHRASE (encrypted client keys) is not supported with the native-tls backend; provide an unencrypted PEM key or build with the `rustls` feature",
        ));
    }
    reqwest::Identity::from_pkcs8_pem(cert, key)
        .map_err(|e| CamundaError::config(format!("invalid client certificate/key: {e}")))
}

#[cfg(all(feature = "rustls", not(feature = "native-tls")))]
fn build_identity(cert: &[u8], key: &[u8], passphrase: Option<&str>) -> Result<reqwest::Identity> {
    if passphrase.is_some() {
        return Err(CamundaError::config(
            "CAMUNDA_MTLS_KEY_PASSPHRASE (encrypted client keys) is not supported; provide an unencrypted PEM key",
        ));
    }
    // rustls expects the certificate chain and private key concatenated in one PEM buffer.
    let mut pem = Vec::with_capacity(cert.len() + key.len() + 1);
    pem.extend_from_slice(cert);
    pem.push(b'\n');
    pem.extend_from_slice(key);
    reqwest::Identity::from_pem(&pem)
        .map_err(|e| CamundaError::config(format!("invalid client certificate/key: {e}")))
}

#[cfg(not(any(feature = "native-tls", feature = "rustls")))]
fn build_identity(
    _cert: &[u8],
    _key: &[u8],
    _passphrase: Option<&str>,
) -> Result<reqwest::Identity> {
    Err(CamundaError::config(
        "mutual TLS requires a TLS backend; build with the `native-tls` (default) or `rustls` feature",
    ))
}
