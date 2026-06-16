//! Authentication strategies and token management.
//!
//! Supports three strategies, selected via `CAMUNDA_AUTH_STRATEGY`:
//!
//! * [`AuthStrategy::OAuth`] — OAuth 2.0 client-credentials grant. Tokens are fetched
//!   from `CAMUNDA_OAUTH_URL`, cached in memory, and refreshed shortly before expiry.
//! * [`AuthStrategy::Basic`] — HTTP Basic authentication.
//! * [`AuthStrategy::None`] — no authentication (e.g. local development).

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::config::CamundaConfig;
use super::errors::{CamundaError, Result};
use camunda_orchestration_api_client::apis::configuration::Configuration;

/// Authentication strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStrategy {
    /// OAuth 2.0 client-credentials grant.
    OAuth,
    /// HTTP Basic authentication.
    Basic,
    /// No authentication.
    None,
}

impl FromStr for AuthStrategy {
    type Err = CamundaError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "OAUTH" => Ok(AuthStrategy::OAuth),
            "BASIC" => Ok(AuthStrategy::Basic),
            "NONE" | "" => Ok(AuthStrategy::None),
            other => Err(CamundaError::config(format!(
                "unknown CAMUNDA_AUTH_STRATEGY {other:?} (expected OAUTH, BASIC or NONE)"
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    expires_in: Option<u64>,
}

#[derive(Debug)]
struct CachedToken {
    token: String,
    /// Instant after which the token should be considered expired and refreshed.
    refresh_after: Instant,
    /// Wall-clock instant matching `refresh_after`, used to persist the token to disk
    /// (an `Instant` has no portable wall-clock representation).
    refresh_after_wall: SystemTime,
}

/// On-disk representation of a cached OAuth token (shared across processes).
#[derive(Debug, Serialize, Deserialize)]
struct DiskToken {
    access_token: String,
    /// Refresh-after time as milliseconds since the Unix epoch.
    refresh_after_unix_ms: u128,
}

/// Resolves and applies authentication to outgoing requests.
///
/// Cloning an [`Authentication`] shares the same underlying OAuth token cache.
#[derive(Clone)]
pub struct Authentication {
    inner: Arc<Inner>,
}

struct Inner {
    strategy: AuthStrategy,
    // OAuth
    oauth_url: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    audience: Option<String>,
    scope: Option<String>,
    // Basic
    basic_username: Option<String>,
    basic_password: Option<String>,
    // Shared HTTP client for token requests.
    http: reqwest::Client,
    cache: Mutex<Option<CachedToken>>,
    // Optional cross-process token cache directory.
    oauth_cache_dir: Option<PathBuf>,
}

impl Authentication {
    /// Build an [`Authentication`] from resolved configuration.
    pub fn from_config(config: &CamundaConfig, http: reqwest::Client) -> Self {
        Authentication {
            inner: Arc::new(Inner {
                strategy: config.auth_strategy,
                oauth_url: config.oauth_url.clone(),
                client_id: config.client_id.clone(),
                client_secret: config.client_secret.clone(),
                audience: config.token_audience.clone(),
                scope: config.oauth_scope.clone(),
                basic_username: config.basic_auth_username.clone(),
                basic_password: config.basic_auth_password.clone(),
                http,
                cache: Mutex::new(None),
                oauth_cache_dir: config.oauth_cache_dir.clone().map(PathBuf::from),
            }),
        }
    }

    /// The configured strategy.
    pub fn strategy(&self) -> AuthStrategy {
        self.inner.strategy
    }

    /// Apply authentication to a generated-client [`Configuration`], refreshing the
    /// OAuth token if necessary.
    pub async fn apply(&self, config: &mut Configuration) -> Result<()> {
        match self.inner.strategy {
            AuthStrategy::None => {}
            AuthStrategy::Basic => {
                config.basic_auth = Some((
                    self.inner.basic_username.clone().unwrap_or_default(),
                    self.inner.basic_password.clone(),
                ));
            }
            AuthStrategy::OAuth => {
                let token = self.access_token().await?;
                config.oauth_access_token = Some(token);
            }
        }
        Ok(())
    }

    /// Return a valid OAuth access token, fetching or refreshing as needed.
    ///
    /// Token resolution order: in-memory cache → on-disk cache (if
    /// `CAMUNDA_OAUTH_CACHE_DIR` is set) → token endpoint.
    async fn access_token(&self) -> Result<String> {
        {
            let cache = self.inner.cache.lock().await;
            if let Some(cached) = cache.as_ref() {
                if Instant::now() < cached.refresh_after {
                    return Ok(cached.token.clone());
                }
            }
        }

        // Try the cross-process disk cache before hitting the token endpoint.
        if let Some(disk) = self.read_disk_token() {
            if Instant::now() < disk.refresh_after {
                let token = disk.token.clone();
                let mut cache = self.inner.cache.lock().await;
                *cache = Some(disk);
                return Ok(token);
            }
        }

        let fetched = self.fetch_token().await?;
        self.write_disk_token(&fetched);
        let mut cache = self.inner.cache.lock().await;
        let token = fetched.token.clone();
        *cache = Some(fetched);
        Ok(token)
    }

    /// Compute the on-disk cache path for this client's tokens, namespaced by the
    /// credentials so distinct clients do not collide.
    fn cache_file_path(&self) -> Option<PathBuf> {
        use std::hash::{Hash, Hasher};
        let dir = self.inner.oauth_cache_dir.as_ref()?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.inner.oauth_url.hash(&mut hasher);
        self.inner.client_id.hash(&mut hasher);
        self.inner.audience.hash(&mut hasher);
        let key = hasher.finish();
        Some(dir.join(format!("camunda-oauth-{key:016x}.json")))
    }

    /// Read and decode a token from the disk cache, converting its wall-clock expiry into a
    /// monotonic [`Instant`]. Returns `None` on any error (missing/corrupt cache).
    fn read_disk_token(&self) -> Option<CachedToken> {
        let path = self.cache_file_path()?;
        let bytes = std::fs::read(&path).ok()?;
        let disk: DiskToken = serde_json::from_slice(&bytes).ok()?;
        let now_wall_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_millis();
        let refresh_after_wall = UNIX_EPOCH
            + Duration::from_millis(disk.refresh_after_unix_ms.min(u64::MAX as u128) as u64);
        // Map the wall-clock expiry onto the monotonic clock.
        let refresh_after = if disk.refresh_after_unix_ms > now_wall_ms {
            Instant::now()
                + Duration::from_millis((disk.refresh_after_unix_ms - now_wall_ms) as u64)
        } else {
            Instant::now()
        };
        Some(CachedToken {
            token: disk.access_token,
            refresh_after,
            refresh_after_wall,
        })
    }

    /// Persist a token to the disk cache atomically (temp file + rename). Best-effort: any
    /// error is logged and ignored.
    fn write_disk_token(&self, token: &CachedToken) {
        let Some(path) = self.cache_file_path() else {
            return;
        };
        let refresh_after_unix_ms = token
            .refresh_after_wall
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let disk = DiskToken {
            access_token: token.token.clone(),
            refresh_after_unix_ms,
        };
        let Ok(json) = serde_json::to_vec(&disk) else {
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::warn!(error = %e, "failed to create OAuth cache directory");
                return;
            }
        }
        let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
        if let Err(e) = std::fs::write(&tmp, &json) {
            tracing::warn!(error = %e, "failed to write OAuth cache token");
            return;
        }
        if let Err(e) = std::fs::rename(&tmp, &path) {
            tracing::warn!(error = %e, "failed to commit OAuth cache token");
            let _ = std::fs::remove_file(&tmp);
        }
    }

    async fn fetch_token(&self) -> Result<CachedToken> {
        let url = self
            .inner
            .oauth_url
            .as_ref()
            .ok_or_else(|| CamundaError::config("CAMUNDA_OAUTH_URL is required for OAUTH"))?;
        let client_id = self
            .inner
            .client_id
            .as_ref()
            .ok_or_else(|| CamundaError::config("CAMUNDA_CLIENT_ID is required for OAUTH"))?;
        let client_secret =
            self.inner.client_secret.as_ref().ok_or_else(|| {
                CamundaError::config("CAMUNDA_CLIENT_SECRET is required for OAUTH")
            })?;

        let mut form: Vec<(&str, &str)> = vec![
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];
        if let Some(audience) = &self.inner.audience {
            form.push(("audience", audience));
        }
        if let Some(scope) = &self.inner.scope {
            form.push(("scope", scope));
        }

        let response = self
            .inner
            .http
            .post(url)
            .form(&form)
            .send()
            .await
            .map_err(CamundaError::Network)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CamundaError::auth(format!(
                "token endpoint returned HTTP {status}: {body}"
            )));
        }

        let token: TokenResponse = response
            .json()
            .await
            .map_err(|e| CamundaError::auth(format!("failed to parse token response: {e}")))?;

        // Refresh 30s before the stated expiry (default 60s lifetime when unspecified).
        let lifetime = token.expires_in.unwrap_or(60);
        let lead = lifetime.saturating_sub(30).max(1);
        let lead_duration = Duration::from_secs(lead);
        Ok(CachedToken {
            token: token.access_token,
            refresh_after: Instant::now() + lead_duration,
            refresh_after_wall: SystemTime::now() + lead_duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_strategies_case_insensitively() {
        assert_eq!(
            "oauth".parse::<AuthStrategy>().unwrap(),
            AuthStrategy::OAuth
        );
        assert_eq!(
            "Basic".parse::<AuthStrategy>().unwrap(),
            AuthStrategy::Basic
        );
        assert_eq!("NONE".parse::<AuthStrategy>().unwrap(), AuthStrategy::None);
        assert!("nope".parse::<AuthStrategy>().is_err());
    }
}
