//! Environment-driven configuration resolution.
//!
//! Mirrors the JS/Python/C# SDKs: configuration is sourced from environment variables,
//! with optional programmatic overrides supplied via [`CamundaOptions`](super::client::CamundaOptions).
//! The full set of `CAMUNDA_*` variables is documented in the README.

use std::collections::HashMap;

use super::auth::AuthStrategy;
use super::backpressure::BackpressureProfile;
use super::errors::{CamundaError, Result};

/// Default REST address used when none is configured.
const DEFAULT_REST_ADDRESS: &str = "http://localhost:8080";

/// Resolved SDK configuration.
#[derive(Debug, Clone)]
pub struct CamundaConfig {
    /// Base REST address of the Orchestration Cluster, including the `/v2` suffix.
    pub rest_address: String,
    /// Authentication strategy.
    pub auth_strategy: AuthStrategy,
    /// OAuth 2.0 client id (client-credentials grant).
    pub client_id: Option<String>,
    /// OAuth 2.0 client secret.
    pub client_secret: Option<String>,
    /// OAuth 2.0 token endpoint URL.
    pub oauth_url: Option<String>,
    /// OAuth token audience (sent as the `audience` form parameter).
    pub token_audience: Option<String>,
    /// OAuth scope (optional).
    pub oauth_scope: Option<String>,
    /// Directory for the on-disk OAuth token cache (`CAMUNDA_OAUTH_CACHE_DIR`). When set,
    /// fetched tokens are persisted so they survive process restarts.
    pub oauth_cache_dir: Option<String>,
    /// Basic-auth username.
    pub basic_auth_username: Option<String>,
    /// Basic-auth password.
    pub basic_auth_password: Option<String>,
    /// Default tenant id applied to operations that accept one.
    pub default_tenant_id: Option<String>,
    /// Adaptive backpressure profile (`CAMUNDA_SDK_BACKPRESSURE_PROFILE`).
    pub backpressure_profile: BackpressureProfile,
    /// SDK log level (`CAMUNDA_SDK_LOG_LEVEL`).
    pub log_level: LogLevel,
    /// Default per-operation timeout, in milliseconds, for eventual-consistency polling
    /// helpers (`CAMUNDA_SDK_EVENTUAL_POLL_DEFAULT_MS`).
    pub eventual_poll_default_ms: u64,
    /// Transient-error HTTP retry policy.
    pub retry: RetryConfig,
    /// TLS / mutual-TLS configuration.
    pub tls: TlsConfig,
    /// Default job-worker settings sourced from `CAMUNDA_WORKER_*`.
    pub worker_defaults: WorkerDefaults,
    /// Whether to upgrade to the nanobpmn command-stream transport when the gateway
    /// advertises it (`CAMUNDA_FALCON`, default on). When off, the SDK
    /// stays on pure REST even against a nanobpmn gateway.
    pub falcon: bool,
}

/// SDK log level, controlling the verbosity of the SDK's structured logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// Suppress all SDK logging.
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl std::str::FromStr for LogLevel {
    type Err = CamundaError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "off" | "none" | "silent" => Ok(LogLevel::Off),
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "info" | "" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" | "silly" | "verbose" => Ok(LogLevel::Trace),
            other => Err(CamundaError::config(format!(
                "unknown CAMUNDA_SDK_LOG_LEVEL {other:?} (expected off/error/warn/info/debug/trace)"
            ))),
        }
    }
}

impl LogLevel {
    /// The matching [`tracing::Level`], or `None` when logging is off.
    pub fn to_tracing(self) -> Option<tracing::Level> {
        match self {
            LogLevel::Off => None,
            LogLevel::Error => Some(tracing::Level::ERROR),
            LogLevel::Warn => Some(tracing::Level::WARN),
            LogLevel::Info => Some(tracing::Level::INFO),
            LogLevel::Debug => Some(tracing::Level::DEBUG),
            LogLevel::Trace => Some(tracing::Level::TRACE),
        }
    }
}

/// Transient-error HTTP retry policy (`CAMUNDA_SDK_HTTP_RETRY_*`).
///
/// Initiating operations that fail with a retryable signal (HTTP 429/503, or a network
/// error) are retried with exponential backoff and full jitter.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts (1 disables retries).
    pub max_attempts: u32,
    /// Base backoff delay, in milliseconds.
    pub base_delay_ms: u64,
    /// Maximum backoff delay, in milliseconds.
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 4,
            base_delay_ms: 100,
            max_delay_ms: 5_000,
        }
    }
}

/// TLS / mutual-TLS configuration (`CAMUNDA_MTLS_*`).
///
/// A client certificate (`cert` + `key`) enables mutual TLS; a `ca` enables trusting a
/// private certificate authority. Inline PEM values take precedence over the `*_path`
/// file locations.
#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    /// Inline client-certificate PEM (`CAMUNDA_MTLS_CERT`).
    pub cert: Option<String>,
    /// Inline client-key PEM (`CAMUNDA_MTLS_KEY`).
    pub key: Option<String>,
    /// Inline CA-certificate PEM (`CAMUNDA_MTLS_CA`).
    pub ca: Option<String>,
    /// Path to a client-certificate PEM (`CAMUNDA_MTLS_CERT_PATH`).
    pub cert_path: Option<String>,
    /// Path to a client-key PEM (`CAMUNDA_MTLS_KEY_PATH`).
    pub key_path: Option<String>,
    /// Path to a CA-certificate PEM (`CAMUNDA_MTLS_CA_PATH`).
    pub ca_path: Option<String>,
    /// Passphrase for an encrypted client key (`CAMUNDA_MTLS_KEY_PASSPHRASE`).
    pub key_passphrase: Option<String>,
}

impl TlsConfig {
    /// Whether any TLS material has been configured.
    pub fn is_configured(&self) -> bool {
        self.cert.is_some()
            || self.key.is_some()
            || self.ca.is_some()
            || self.cert_path.is_some()
            || self.key_path.is_some()
            || self.ca_path.is_some()
    }
}

/// Default job-worker settings sourced from `CAMUNDA_WORKER_*`. Used to seed
/// [`JobWorkerConfig`](super::job_worker::JobWorkerConfig) defaults.
#[derive(Debug, Clone)]
pub struct WorkerDefaults {
    /// Job activation timeout, in milliseconds (`CAMUNDA_WORKER_TIMEOUT`).
    pub timeout_ms: i64,
    /// Maximum concurrent jobs per worker (`CAMUNDA_WORKER_MAX_CONCURRENT_JOBS`).
    pub max_concurrent_jobs: i32,
    /// Long-poll request timeout, in milliseconds (`CAMUNDA_WORKER_REQUEST_TIMEOUT`).
    pub request_timeout_ms: i64,
    /// Worker name reported to the engine (`CAMUNDA_WORKER_NAME`).
    pub name: String,
    /// Maximum random startup delay, in seconds (`CAMUNDA_WORKER_STARTUP_JITTER_MAX_SECONDS`).
    pub startup_jitter_max_seconds: u64,
}

impl Default for WorkerDefaults {
    fn default() -> Self {
        WorkerDefaults {
            timeout_ms: 60_000,
            max_concurrent_jobs: 10,
            request_timeout_ms: 10_000,
            name: "rust-sdk-worker".to_string(),
            startup_jitter_max_seconds: 0,
        }
    }
}

impl CamundaConfig {
    /// Resolve configuration from process environment variables.
    pub fn from_env() -> Result<Self> {
        Self::resolve(&env_lookup, &HashMap::new())
    }

    /// Resolve configuration from environment variables, with `overrides` taking precedence.
    pub fn from_env_with_overrides(overrides: &HashMap<String, String>) -> Result<Self> {
        Self::resolve(&env_lookup, overrides)
    }

    fn resolve(
        env: &dyn Fn(&str) -> Option<String>,
        overrides: &HashMap<String, String>,
    ) -> Result<Self> {
        let get = |key: &str| -> Option<String> {
            overrides
                .get(key)
                .cloned()
                .or_else(|| env(key))
                .filter(|v| !v.is_empty())
        };

        // Base address: CAMUNDA_REST_ADDRESS, falling back to ZEEBE_REST_ADDRESS.
        let raw_address = get("CAMUNDA_REST_ADDRESS")
            .or_else(|| get("ZEEBE_REST_ADDRESS"))
            .unwrap_or_else(|| DEFAULT_REST_ADDRESS.to_string());
        let rest_address = normalize_rest_address(&raw_address);

        let client_id = get("CAMUNDA_CLIENT_ID").or_else(|| get("CAMUNDA_CLIENT_AUTH_CLIENTID"));
        let client_secret =
            get("CAMUNDA_CLIENT_SECRET").or_else(|| get("CAMUNDA_CLIENT_AUTH_CLIENTSECRET"));
        let oauth_url = get("CAMUNDA_OAUTH_URL");

        // Strategy: explicit, otherwise inferred (OAuth creds present -> OAUTH, else NONE).
        let auth_strategy = match get("CAMUNDA_AUTH_STRATEGY") {
            Some(s) => s.parse::<AuthStrategy>()?,
            None => {
                if oauth_url.is_some() && client_id.is_some() && client_secret.is_some() {
                    AuthStrategy::OAuth
                } else {
                    AuthStrategy::None
                }
            }
        };

        let config = CamundaConfig {
            rest_address,
            auth_strategy,
            client_id,
            client_secret,
            oauth_url,
            token_audience: get("CAMUNDA_TOKEN_AUDIENCE"),
            oauth_scope: get("CAMUNDA_OAUTH_SCOPE"),
            oauth_cache_dir: get("CAMUNDA_OAUTH_CACHE_DIR"),
            basic_auth_username: get("CAMUNDA_BASIC_AUTH_USERNAME"),
            basic_auth_password: get("CAMUNDA_BASIC_AUTH_PASSWORD"),
            default_tenant_id: get("CAMUNDA_DEFAULT_TENANT_ID")
                .or_else(|| get("CAMUNDA_TENANT_ID")),
            backpressure_profile: match get("CAMUNDA_SDK_BACKPRESSURE_PROFILE") {
                Some(s) => s.parse::<BackpressureProfile>()?,
                None => BackpressureProfile::default(),
            },
            log_level: match get("CAMUNDA_SDK_LOG_LEVEL") {
                Some(s) => s.parse::<LogLevel>()?,
                None => LogLevel::default(),
            },
            eventual_poll_default_ms: parse_u64(
                &get,
                "CAMUNDA_SDK_EVENTUAL_POLL_DEFAULT_MS",
                10_000,
            )?,
            retry: RetryConfig {
                max_attempts: parse_u64(&get, "CAMUNDA_SDK_HTTP_RETRY_MAX_ATTEMPTS", 4)? as u32,
                base_delay_ms: parse_u64(&get, "CAMUNDA_SDK_HTTP_RETRY_BASE_DELAY_MS", 100)?,
                max_delay_ms: parse_u64(&get, "CAMUNDA_SDK_HTTP_RETRY_MAX_DELAY_MS", 5_000)?,
            },
            tls: TlsConfig {
                cert: get("CAMUNDA_MTLS_CERT"),
                key: get("CAMUNDA_MTLS_KEY"),
                ca: get("CAMUNDA_MTLS_CA"),
                cert_path: get("CAMUNDA_MTLS_CERT_PATH"),
                key_path: get("CAMUNDA_MTLS_KEY_PATH"),
                ca_path: get("CAMUNDA_MTLS_CA_PATH"),
                key_passphrase: get("CAMUNDA_MTLS_KEY_PASSPHRASE"),
            },
            worker_defaults: WorkerDefaults {
                timeout_ms: parse_u64(&get, "CAMUNDA_WORKER_TIMEOUT", 60_000)? as i64,
                max_concurrent_jobs: parse_u64(&get, "CAMUNDA_WORKER_MAX_CONCURRENT_JOBS", 10)?
                    as i32,
                request_timeout_ms: parse_u64(&get, "CAMUNDA_WORKER_REQUEST_TIMEOUT", 10_000)?
                    as i64,
                name: get("CAMUNDA_WORKER_NAME").unwrap_or_else(|| "rust-sdk-worker".to_string()),
                startup_jitter_max_seconds: parse_u64(
                    &get,
                    "CAMUNDA_WORKER_STARTUP_JITTER_MAX_SECONDS",
                    0,
                )?,
            },
            falcon: match get("CAMUNDA_FALCON") {
                Some(v) => !matches!(
                    v.trim().to_ascii_lowercase().as_str(),
                    "0" | "off" | "false" | "no"
                ),
                None => true,
            },
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        match self.auth_strategy {
            AuthStrategy::OAuth => {
                if self.client_id.is_none()
                    || self.client_secret.is_none()
                    || self.oauth_url.is_none()
                {
                    return Err(CamundaError::config(
                        "OAUTH strategy requires CAMUNDA_CLIENT_ID, CAMUNDA_CLIENT_SECRET and CAMUNDA_OAUTH_URL",
                    ));
                }
            }
            AuthStrategy::Basic => {
                if self.basic_auth_username.is_none() || self.basic_auth_password.is_none() {
                    return Err(CamundaError::config(
                        "BASIC strategy requires CAMUNDA_BASIC_AUTH_USERNAME and CAMUNDA_BASIC_AUTH_PASSWORD",
                    ));
                }
            }
            AuthStrategy::None => {}
        }
        Ok(())
    }
}

fn env_lookup(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Parse a non-empty numeric env var, falling back to `default` when unset.
fn parse_u64(get: &dyn Fn(&str) -> Option<String>, key: &str, default: u64) -> Result<u64> {
    match get(key) {
        Some(s) => s.trim().parse::<u64>().map_err(|_| {
            CamundaError::config(format!("{key} must be a non-negative integer, got {s:?}"))
        }),
        None => Ok(default),
    }
}

/// Normalize a configured base address into a REST base path ending in `/v2`.
///
/// * trims a trailing slash
/// * appends `/v2` when not already present
fn normalize_rest_address(raw: &str) -> String {
    let trimmed = raw.trim_end_matches('/');
    if trimmed.ends_with("/v2") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(pairs: &[(&str, &str)]) -> Result<CamundaConfig> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        // Use a closure env that returns nothing so only overrides apply.
        CamundaConfig::resolve(&|_| None, &map)
    }

    #[test]
    fn defaults_to_localhost_v2_none_auth() {
        let c = cfg(&[]).unwrap();
        assert_eq!(c.rest_address, "http://localhost:8080/v2");
        assert_eq!(c.auth_strategy, AuthStrategy::None);
    }

    #[test]
    fn appends_v2_suffix_once() {
        assert_eq!(normalize_rest_address("https://x.io"), "https://x.io/v2");
        assert_eq!(normalize_rest_address("https://x.io/"), "https://x.io/v2");
        assert_eq!(normalize_rest_address("https://x.io/v2"), "https://x.io/v2");
        assert_eq!(
            normalize_rest_address("https://x.io/v2/"),
            "https://x.io/v2"
        );
    }

    #[test]
    fn zeebe_rest_address_alias() {
        let c = cfg(&[("ZEEBE_REST_ADDRESS", "https://z.example")]).unwrap();
        assert_eq!(c.rest_address, "https://z.example/v2");
    }

    #[test]
    fn infers_oauth_from_credentials() {
        let c = cfg(&[
            ("CAMUNDA_OAUTH_URL", "https://login/oauth/token"),
            ("CAMUNDA_CLIENT_ID", "id"),
            ("CAMUNDA_CLIENT_SECRET", "secret"),
        ])
        .unwrap();
        assert_eq!(c.auth_strategy, AuthStrategy::OAuth);
    }

    #[test]
    fn oauth_requires_credentials() {
        let err = cfg(&[("CAMUNDA_AUTH_STRATEGY", "OAUTH")]).unwrap_err();
        assert!(matches!(err, CamundaError::Config(_)));
    }

    #[test]
    fn basic_requires_username_password() {
        let err = cfg(&[("CAMUNDA_AUTH_STRATEGY", "BASIC")]).unwrap_err();
        assert!(matches!(err, CamundaError::Config(_)));
        let ok = cfg(&[
            ("CAMUNDA_AUTH_STRATEGY", "BASIC"),
            ("CAMUNDA_BASIC_AUTH_USERNAME", "demo"),
            ("CAMUNDA_BASIC_AUTH_PASSWORD", "demo"),
        ])
        .unwrap();
        assert_eq!(ok.auth_strategy, AuthStrategy::Basic);
    }
}
