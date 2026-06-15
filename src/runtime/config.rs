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
    /// Basic-auth username.
    pub basic_auth_username: Option<String>,
    /// Basic-auth password.
    pub basic_auth_password: Option<String>,
    /// Default tenant id applied to operations that accept one.
    pub default_tenant_id: Option<String>,
    /// Adaptive backpressure profile (`CAMUNDA_SDK_BACKPRESSURE_PROFILE`).
    pub backpressure_profile: BackpressureProfile,
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
            basic_auth_username: get("CAMUNDA_BASIC_AUTH_USERNAME"),
            basic_auth_password: get("CAMUNDA_BASIC_AUTH_PASSWORD"),
            default_tenant_id: get("CAMUNDA_DEFAULT_TENANT_ID")
                .or_else(|| get("CAMUNDA_TENANT_ID")),
            backpressure_profile: match get("CAMUNDA_SDK_BACKPRESSURE_PROFILE") {
                Some(s) => s.parse::<BackpressureProfile>()?,
                None => BackpressureProfile::default(),
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
