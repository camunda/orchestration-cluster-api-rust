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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::time::Instant;

use super::clock::Clock;
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
    clock: Arc<dyn Clock>,
}

impl Authentication {
    /// Build an [`Authentication`] from resolved configuration.
    pub fn from_config(
        config: &CamundaConfig,
        http: reqwest::Client,
        clock: Arc<dyn Clock>,
    ) -> Self {
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
                clock,
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
                // The generated operations read the bearer token from
                // `bearer_access_token`; `oauth_access_token` is a dead field they never
                // consult. Writing the fetched token here is what actually sets the
                // `Authorization: Bearer` header on every request.
                config.bearer_access_token = Some(token);
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
                if self.inner.clock.now() < cached.refresh_after {
                    return Ok(cached.token.clone());
                }
            }
        }

        // Try the cross-process disk cache before hitting the token endpoint.
        if let Some(disk) = self.read_disk_token() {
            if self.inner.clock.now() < disk.refresh_after {
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
        let now_wall_ms = self
            .inner
            .clock
            .now_wall()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_millis();
        let refresh_after_wall = UNIX_EPOCH
            + Duration::from_millis(disk.refresh_after_unix_ms.min(u64::MAX as u128) as u64);
        // Map the wall-clock expiry onto the monotonic clock. Both branches must read one
        // `now()`, or they disagree by however far the clock moved between the two calls.
        let now = self.inner.clock.now();
        let refresh_after = if disk.refresh_after_unix_ms > now_wall_ms {
            now + Duration::from_millis((disk.refresh_after_unix_ms - now_wall_ms) as u64)
        } else {
            now
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
            // Deliberately not the body. It is identity-provider-controlled and not
            // guaranteed to be a bare `{"error":"invalid_client"}`: providers echo
            // submitted parameters and assertion fragments, and a misrouted
            // `CAMUNDA_OAUTH_URL` points a POST carrying `client_secret` at an endpoint
            // that may reflect the request back. The status is the diagnostic; the body is
            // not ours to put into an error callers log.
            return Err(CamundaError::auth(format!(
                "token endpoint returned HTTP {status}"
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
            refresh_after: self.inner.clock.now() + lead_duration,
            refresh_after_wall: self.inner.clock.now_wall() + lead_duration,
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

    /// The token endpoint's response body is identity-provider- and attacker-influenced, and
    /// the client-credentials POST that provoked it carries `client_secret` in its form body.
    /// No failure of `fetch_token` may put any of that into a `CamundaError`, which callers
    /// log and report. These tests drive the failure paths against a local stub endpoint and
    /// scan the rendered error for sentinels, so they catch the class — any response field
    /// reaching the message — rather than today's one call site.
    mod token_endpoint_errors_omit_response_body {
        use super::*;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        /// A sentinel the stub endpoint echoes back the way a reflecting or verbose IdP
        /// would. It must never surface in an error.
        const SENTINEL: &str = "SENTINEL-IDP-BODY-d41d8cd98f00";

        /// Serve one HTTP response with the given status and body, then close. Dependency-free
        /// (no mock-server crate), mirroring the activation stub in `job_worker.rs`.
        async fn serve_once(status_line: &'static str, body: String) -> String {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::spawn(async move {
                let Ok((mut sock, _)) = listener.accept().await else {
                    return;
                };
                let mut buf = Vec::new();
                let mut tmp = [0u8; 1024];
                // Read to the end of the request headers.
                let header_end = loop {
                    match sock.read(&mut tmp).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => {
                            buf.extend_from_slice(&tmp[..n]);
                            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                                break pos + 4;
                            }
                        }
                    }
                };
                // Then drain the form body. `fetch_token` POSTs the client-credentials form,
                // which can arrive in a later read than the headers; closing the socket with
                // it unread can reset the connection, and the test would see a transport
                // failure — `CamundaError::Network` — instead of the auth error it asserts
                // on. Reading `Content-Length` bytes first makes the exchange clean and the
                // test deterministic.
                let content_len = String::from_utf8_lossy(&buf[..header_end])
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                    })
                    .unwrap_or(0);
                while buf.len() < header_end + content_len {
                    match sock.read(&mut tmp).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                    }
                }
                let resp = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
                let _ = sock.flush().await;
            });
            format!("http://{addr}/oauth/token")
        }

        fn auth_for(url: String) -> Authentication {
            Authentication {
                inner: Arc::new(Inner {
                    strategy: AuthStrategy::OAuth,
                    oauth_url: Some(url),
                    client_id: Some("sentinel-client-id".to_string()),
                    client_secret: Some(SENTINEL.to_string()),
                    audience: None,
                    scope: None,
                    basic_username: None,
                    basic_password: None,
                    http: reqwest::Client::new(),
                    cache: Mutex::new(None),
                    oauth_cache_dir: None,
                    clock: super::super::super::clock::live_clock(),
                }),
            }
        }

        /// Assert the rendered error — both `Display` and `Debug`, since callers use either —
        /// carries neither the response body nor the credential that was posted.
        fn assert_no_leak(err: &CamundaError) {
            let display = format!("{err}");
            let debug = format!("{err:?}");
            for rendered in [&display, &debug] {
                assert!(
                    !rendered.contains(SENTINEL),
                    "token-endpoint error leaked response-body or credential material: {rendered}"
                );
            }
        }

        #[tokio::test]
        async fn non_success_status_reports_the_status_without_the_body() {
            let url = serve_once(
                "401 Unauthorized",
                format!(r#"{{"error":"invalid_client","detail":"{SENTINEL}"}}"#),
            )
            .await;
            let err = auth_for(url)
                .fetch_token()
                .await
                .expect_err("a 401 from the token endpoint must fail");

            assert!(
                matches!(err, CamundaError::Auth(_)),
                "expected CamundaError::Auth, got {err:?}"
            );
            assert!(
                format!("{err}").contains("401"),
                "the status is the diagnostic we do keep: {err}"
            );
            assert_no_leak(&err);
        }

        /// A 200 whose body is not a token response takes the other error path. serde's
        /// message quotes the input it choked on, so this path can leak a body too.
        #[tokio::test]
        async fn unparseable_success_body_is_not_quoted_back() {
            let url = serve_once("200 OK", format!(r#"{{"not_a_token":"{SENTINEL}"}}"#)).await;
            let err = auth_for(url)
                .fetch_token()
                .await
                .expect_err("a response without access_token must fail");

            assert!(
                matches!(err, CamundaError::Auth(_)),
                "expected CamundaError::Auth, got {err:?}"
            );
            assert_no_leak(&err);
        }
    }

    /// OAuth tokens must be written to `bearer_access_token` — the only field the
    /// generated operations actually read. Writing them to `oauth_access_token` (a dead
    /// field) would send no `Authorization` header and fail against an OAuth cluster.
    #[tokio::test]
    async fn oauth_token_is_applied_as_bearer_access_token() {
        let clock = super::super::clock::live_clock();
        let auth = Authentication {
            inner: Arc::new(Inner {
                strategy: AuthStrategy::OAuth,
                oauth_url: None,
                client_id: None,
                client_secret: None,
                audience: None,
                scope: None,
                basic_username: None,
                basic_password: None,
                http: reqwest::Client::new(),
                // Pre-seed the in-memory cache so `apply` resolves without network I/O.
                cache: Mutex::new(Some(CachedToken {
                    token: "test-token-abc".to_string(),
                    refresh_after: clock.now() + Duration::from_secs(3600),
                    refresh_after_wall: clock.now_wall() + Duration::from_secs(3600),
                })),
                oauth_cache_dir: None,
                clock,
            }),
        };

        let mut cfg = Configuration::default();
        auth.apply(&mut cfg).await.unwrap();

        assert_eq!(
            cfg.bearer_access_token.as_deref(),
            Some("test-token-abc"),
            "OAuth token must populate bearer_access_token (the field operations read)"
        );
        assert!(
            cfg.oauth_access_token.is_none(),
            "oauth_access_token is a dead field and must not be relied upon"
        );
    }
}
