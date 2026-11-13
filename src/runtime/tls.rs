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

#[cfg(test)]
mod tests {
    use super::*;

    /// PEM-shaped but not valid material. Enough to reach the backend's parser, which is
    /// the branch under test; a real key would prove nothing extra here.
    const GARBAGE_PEM: &str =
        "-----BEGIN CERTIFICATE-----\nbm90LWEtY2VydA==\n-----END CERTIFICATE-----\n";

    /// A path that cannot exist, for the unreadable-file branch. Under the system temp
    /// directory so nothing is created anywhere surprising if the assumption ever breaks.
    fn missing_path(label: &str) -> String {
        std::env::temp_dir()
            .join(format!(
                "camunda-tls-absent-{}-{label}/material.pem",
                std::process::id()
            ))
            .to_string_lossy()
            .into_owned()
    }

    fn apply(tls: &TlsConfig) -> Result<reqwest::ClientBuilder> {
        apply_tls(reqwest::Client::builder(), tls)
    }

    /// How a path appears inside `read_pem`'s error message.
    ///
    /// `read_pem` formats the path with `{p:?}`, which quotes it and escapes separators. On
    /// unix that is the raw path in quotes, so a naive substring check passes; on Windows
    /// `C:\dir\material.pem` becomes `"C:\\dir\\material.pem"` and the same check fails
    /// against correct behaviour. Comparing against the Debug form keeps these assertions
    /// about the message and not about the platform's separator.
    fn as_reported(path: &str) -> String {
        format!("{path:?}")
    }

    /// Unwrap the message of a `CamundaError::Config`, failing the test on any other
    /// variant — every misconfiguration branch in this module is a configuration error, and
    /// a branch that started returning something else would be a behaviour change worth
    /// catching.
    fn config_error_message(result: Result<reqwest::ClientBuilder>) -> String {
        match result {
            Err(CamundaError::Config(msg)) => msg,
            Err(other) => panic!("expected CamundaError::Config, got {other:?}"),
            Ok(_) => panic!("expected the misconfiguration to be rejected"),
        }
    }

    #[test]
    fn no_tls_material_is_a_no_op() {
        assert!(
            apply(&TlsConfig::default()).is_ok(),
            "an unconfigured TlsConfig must leave the builder alone"
        );
    }

    /// Half an identity is a misconfiguration, not a partial one: sending a certificate
    /// with no key (or the reverse) cannot authenticate anything, so it must fail loudly at
    /// construction rather than at the first request.
    #[test]
    fn certificate_without_key_is_rejected() {
        let msg = config_error_message(apply(&TlsConfig {
            cert: Some(GARBAGE_PEM.to_string()),
            ..TlsConfig::default()
        }));
        assert!(
            msg.contains("CAMUNDA_MTLS_CERT") && msg.contains("CAMUNDA_MTLS_KEY"),
            "the error should name both variables the user needs to set: {msg}"
        );
    }

    #[test]
    fn key_without_certificate_is_rejected() {
        let msg = config_error_message(apply(&TlsConfig {
            key: Some(GARBAGE_PEM.to_string()),
            ..TlsConfig::default()
        }));
        assert!(
            msg.contains("CAMUNDA_MTLS_CERT") && msg.contains("CAMUNDA_MTLS_KEY"),
            "the error should name both variables the user needs to set: {msg}"
        );
    }

    /// A `*_PATH` pointing at a file that is missing or unreadable must say which file and
    /// what went wrong — this is the single most common mTLS misconfiguration, and a bare
    /// "invalid configuration" would send the user looking in the wrong place.
    #[test]
    fn unreadable_ca_path_names_the_file() {
        let path = missing_path("ca");
        let msg = config_error_message(apply(&TlsConfig {
            ca_path: Some(path.clone()),
            ..TlsConfig::default()
        }));
        assert!(
            msg.contains(&as_reported(&path)),
            "the error should name the unreadable file: {msg}"
        );
        assert!(
            msg.contains("CA certificate"),
            "the error should say which piece of material it is: {msg}"
        );
    }

    #[test]
    fn unreadable_client_certificate_path_names_the_file() {
        let path = missing_path("cert");
        let msg = config_error_message(apply(&TlsConfig {
            cert_path: Some(path.clone()),
            key: Some(GARBAGE_PEM.to_string()),
            ..TlsConfig::default()
        }));
        assert!(
            msg.contains(&as_reported(&path)) && msg.contains("client certificate"),
            "the error should name the unreadable client certificate: {msg}"
        );
    }

    #[test]
    fn unreadable_client_key_path_names_the_file() {
        let path = missing_path("key");
        let msg = config_error_message(apply(&TlsConfig {
            cert: Some(GARBAGE_PEM.to_string()),
            key_path: Some(path.clone()),
            ..TlsConfig::default()
        }));
        assert!(
            msg.contains(&as_reported(&path)) && msg.contains("client key"),
            "the error should name the unreadable client key: {msg}"
        );
    }

    /// On `native-tls`, `reqwest::Certificate::from_pem` parses eagerly, so a malformed CA
    /// is caught at configuration time and named.
    #[cfg(feature = "native-tls")]
    #[test]
    fn malformed_ca_pem_is_rejected() {
        let msg = config_error_message(apply(&TlsConfig {
            ca: Some("not a pem at all".to_string()),
            ..TlsConfig::default()
        }));
        assert!(
            msg.contains("CAMUNDA_MTLS_CA"),
            "the error should name the variable holding the bad material: {msg}"
        );
    }

    /// The same input on the `rustls` backend is **accepted** at configuration time:
    /// `reqwest::Certificate::from_pem` does not parse eagerly there, so `apply_tls`
    /// returns `Ok` and nothing reports that the configured CA is unusable.
    ///
    /// This is a real divergence between the two backends, not a quirk of the test — the
    /// same `CAMUNDA_MTLS_CA` value is a startup error under `native-tls` and silence under
    /// `rustls`, where it surfaces later as a connection failure or as a root that was never
    /// added. Which behaviour is correct (and whether the SDK should validate the PEM itself
    /// so both backends agree) is a decision beyond this test-coverage change, so the test
    /// pins what actually happens today and makes any future change to it visible in a diff.
    /// CI builds default features only, so without this the `rustls` side goes unexercised.
    #[cfg(all(feature = "rustls", not(feature = "native-tls")))]
    #[test]
    fn malformed_ca_pem_is_accepted_at_configuration_time_on_rustls() {
        assert!(
            apply(&TlsConfig {
                ca: Some("not a pem at all".to_string()),
                ..TlsConfig::default()
            })
            .is_ok(),
            "documenting current rustls behaviour: a malformed CA is not caught here"
        );
    }

    /// Reaching the backend's identity parser with unusable material must surface as a
    /// configuration error, not a panic and not a success that fails later at connect time.
    /// Both backends produce the same message, so this holds whichever feature is enabled.
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    #[test]
    fn malformed_client_identity_is_rejected() {
        let msg = config_error_message(apply(&TlsConfig {
            cert: Some(GARBAGE_PEM.to_string()),
            key: Some(GARBAGE_PEM.to_string()),
            ..TlsConfig::default()
        }));
        assert!(
            msg.contains("client certificate/key"),
            "the error should point at the identity material: {msg}"
        );
    }

    /// Inline PEM takes precedence over `*_PATH`. Pinned by pairing a present inline value
    /// with a path that cannot be read: if precedence were the other way round — or if both
    /// were read — this would fail with a file error instead of reaching the parser.
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    #[test]
    fn inline_pem_takes_precedence_over_path() {
        let path = missing_path("precedence");
        let msg = config_error_message(apply(&TlsConfig {
            cert: Some(GARBAGE_PEM.to_string()),
            cert_path: Some(path.clone()),
            key: Some(GARBAGE_PEM.to_string()),
            key_path: Some(path.clone()),
            ..TlsConfig::default()
        }));
        assert!(
            !msg.contains(&as_reported(&path)),
            "the unreadable path must never have been opened: {msg}"
        );
        assert!(
            msg.contains("client certificate/key"),
            "the inline material should be what reached the parser: {msg}"
        );
    }

    /// Encrypted client keys are rejected on both backends today, and which backend is
    /// compiled in is a `#[cfg]` choice — so a feature-flag change could flip this
    /// behaviour with nothing to catch it. Asserting the rejection on whichever backend is
    /// built keeps that honest.
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    #[test]
    fn key_passphrase_is_rejected_with_a_named_reason() {
        let msg = config_error_message(apply(&TlsConfig {
            cert: Some(GARBAGE_PEM.to_string()),
            key: Some(GARBAGE_PEM.to_string()),
            key_passphrase: Some("hunter2".to_string()),
            ..TlsConfig::default()
        }));
        assert!(
            msg.contains("CAMUNDA_MTLS_KEY_PASSPHRASE"),
            "the error should name the unsupported variable: {msg}"
        );
    }

    /// Documents current behaviour rather than endorsing it: `TlsConfig::is_configured`
    /// deliberately ignores `key_passphrase`, so a passphrase set on its own skips the whole
    /// TLS block and `apply_tls` returns `Ok` — the passphrase is silently ignored rather
    /// than reported. That is a reasonable reading (a passphrase alone configures nothing to
    /// apply it to) and an unhelpful one (the user asked for something and got silence), and
    /// choosing between them is a behaviour decision this test does not make. It exists so
    /// that whichever way it is decided, the change is deliberate and visible in a diff.
    #[test]
    fn passphrase_alone_is_currently_ignored_not_rejected() {
        let tls = TlsConfig {
            key_passphrase: Some("hunter2".to_string()),
            ..TlsConfig::default()
        };
        assert!(
            !tls.is_configured(),
            "is_configured ignores key_passphrase by design"
        );
        assert!(
            apply(&tls).is_ok(),
            "a passphrase with no certificate or key is currently accepted and ignored"
        );
    }
}
