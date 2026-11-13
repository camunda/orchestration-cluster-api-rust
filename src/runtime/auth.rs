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

    /// The policy's "token-endpoint errors and expiry boundaries" baseline case.
    ///
    /// Nothing in `src/` or `tests/` reached `fetch_token`, `read_disk_token` or
    /// `write_disk_token`: the one OAuth test pre-seeds the in-memory cache specifically so
    /// `apply` resolves without network I/O. Everything below drives the real paths — the
    /// endpoint over a local socket, the expiry boundaries over an injected manual clock, the
    /// disk cache over a temp directory — so no test here needs real time or a real server.
    mod token_endpoint_and_expiry {
        use super::super::super::clock::ManualClock;
        use super::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        /// What the stub endpoint should answer with.
        #[derive(Clone, Copy)]
        enum Reply {
            /// A 200 carrying `{"access_token":"token-<n>","expires_in":<secs>}`, where `n`
            /// counts requests — so a test can tell a cache hit from a refetch by the token
            /// it gets back, rather than by counting calls alone.
            Token { expires_in: Option<u64> },
            /// A 200 whose body is valid JSON but not a token response.
            Unparseable,
            /// A non-2xx status.
            Status(&'static str),
        }

        /// A dependency-free in-process token endpoint, in the shape of the activation stub
        /// in `job_worker.rs`. Serves every connection until dropped and counts requests.
        struct TokenEndpoint {
            url: String,
            requests: Arc<AtomicUsize>,
        }

        impl TokenEndpoint {
            async fn start(reply: Reply) -> Self {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let addr = listener.local_addr().unwrap();
                let requests = Arc::new(AtomicUsize::new(0));
                let counter = requests.clone();

                tokio::spawn(async move {
                    loop {
                        let Ok((mut sock, _)) = listener.accept().await else {
                            break;
                        };
                        let counter = counter.clone();
                        tokio::spawn(async move {
                            let mut buf = Vec::new();
                            let mut tmp = [0u8; 1024];
                            // Read to the end of the request headers.
                            let header_end = loop {
                                match sock.read(&mut tmp).await {
                                    Ok(0) | Err(_) => return,
                                    Ok(n) => {
                                        buf.extend_from_slice(&tmp[..n]);
                                        if let Some(pos) =
                                            buf.windows(4).position(|w| w == b"\r\n\r\n")
                                        {
                                            break pos + 4;
                                        }
                                    }
                                }
                            };
                            // Drain the form body so reqwest sees a clean exchange.
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

                            let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
                            let (status, body) = match reply {
                                Reply::Token { expires_in } => {
                                    let expires = expires_in
                                        .map(|s| format!(r#","expires_in":{s}"#))
                                        .unwrap_or_default();
                                    (
                                        "200 OK",
                                        format!(r#"{{"access_token":"token-{n}"{expires}}}"#),
                                    )
                                }
                                Reply::Unparseable => {
                                    ("200 OK", r#"{"no_token_here":true}"#.to_string())
                                }
                                Reply::Status(status) => {
                                    (status, r#"{"error":"invalid_client"}"#.to_string())
                                }
                            };
                            let resp = format!(
                                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                body.len(),
                                body
                            );
                            let _ = sock.write_all(resp.as_bytes()).await;
                            let _ = sock.flush().await;
                        });
                    }
                });

                TokenEndpoint {
                    url: format!("http://{addr}/oauth/token"),
                    requests,
                }
            }

            fn request_count(&self) -> usize {
                self.requests.load(Ordering::SeqCst)
            }
        }

        /// A cache directory of our own under the system temp dir, removed on drop. The
        /// crate has no `tempfile` dependency and a token-cache test should not add one.
        struct CacheDir(PathBuf);

        impl CacheDir {
            fn new(label: &str) -> Self {
                let dir = std::env::temp_dir().join(format!(
                    "camunda-oauth-expiry-{}-{label}",
                    std::process::id()
                ));
                let _ = std::fs::remove_dir_all(&dir);
                CacheDir(dir)
            }
        }

        impl Drop for CacheDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        fn auth_with(
            url: Option<String>,
            clock: Arc<dyn Clock>,
            cache_dir: Option<PathBuf>,
        ) -> Authentication {
            Authentication {
                inner: Arc::new(Inner {
                    strategy: AuthStrategy::OAuth,
                    oauth_url: url,
                    client_id: Some("client".to_string()),
                    client_secret: Some("secret".to_string()),
                    audience: None,
                    scope: None,
                    basic_username: None,
                    basic_password: None,
                    http: reqwest::Client::new(),
                    cache: Mutex::new(None),
                    oauth_cache_dir: cache_dir,
                    clock,
                }),
            }
        }

        // ---- Token-endpoint failures -------------------------------------------------

        /// Every non-2xx must land as `CamundaError::Auth` and carry the status, so a caller
        /// can tell "bad credentials" from "the IdP is down" and a retry policy can act on
        /// it. Driven across the statuses an identity provider actually returns, so the test
        /// covers the class rather than one code.
        #[tokio::test]
        async fn non_success_statuses_map_to_auth_errors() {
            for (status, code) in [
                ("400 Bad Request", "400"),
                ("401 Unauthorized", "401"),
                ("503 Service Unavailable", "503"),
            ] {
                let endpoint = TokenEndpoint::start(Reply::Status(status)).await;
                let auth = auth_with(
                    Some(endpoint.url.clone()),
                    Arc::new(ManualClock::new()),
                    None,
                );

                let err = match auth.access_token().await {
                    Err(e) => e,
                    Ok(token) => panic!("HTTP {status} must fail, got token {token:?}"),
                };

                assert!(
                    matches!(err, CamundaError::Auth(_)),
                    "HTTP {status} should map to CamundaError::Auth, got {err:?}"
                );
                assert!(
                    err.to_string().contains(code),
                    "the error should carry the status so a caller can act on it: {err}"
                );
            }
        }

        #[tokio::test]
        async fn unparseable_token_response_maps_to_an_auth_error() {
            let endpoint = TokenEndpoint::start(Reply::Unparseable).await;
            let auth = auth_with(
                Some(endpoint.url.clone()),
                Arc::new(ManualClock::new()),
                None,
            );

            let err = auth
                .access_token()
                .await
                .expect_err("a 200 without access_token must fail");

            assert!(
                matches!(err, CamundaError::Auth(_)),
                "expected CamundaError::Auth, got {err:?}"
            );
        }

        /// A refused connection is a transport failure, not an authentication one: it maps
        /// to `CamundaError::Network` so a caller can retry it rather than treating the
        /// credentials as wrong. The port is bound and then released, so nothing is
        /// listening on it.
        #[tokio::test]
        async fn connection_failure_maps_to_a_network_error() {
            let url = {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let addr = listener.local_addr().unwrap();
                drop(listener);
                format!("http://{addr}/oauth/token")
            };
            let auth = auth_with(Some(url), Arc::new(ManualClock::new()), None);

            let err = auth
                .access_token()
                .await
                .expect_err("a refused connection must fail");

            assert!(
                matches!(err, CamundaError::Network(_)),
                "expected CamundaError::Network, got {err:?}"
            );
        }

        /// OAuth with no `CAMUNDA_OAUTH_URL` is a configuration error, reported before any
        /// socket is opened.
        #[tokio::test]
        async fn missing_oauth_url_is_a_config_error() {
            let auth = auth_with(None, Arc::new(ManualClock::new()), None);

            let err = auth
                .access_token()
                .await
                .expect_err("OAUTH without a token URL must fail");

            assert!(
                matches!(err, CamundaError::Config(_)),
                "expected CamundaError::Config, got {err:?}"
            );
        }

        // ---- Expiry boundaries -------------------------------------------------------

        /// The refresh lead is `expires_in - 30s`, and the cache boundary is exact: the
        /// token is served right up to `refresh_after` and refetched at it, not a second
        /// either side. Asserted against a manual clock, so the boundary is stated rather
        /// than raced — and by the token value, so a cache hit is distinguishable from a
        /// refetch that happened to return the same thing.
        #[tokio::test]
        async fn token_is_cached_until_the_refresh_lead_elapses() {
            let endpoint = TokenEndpoint::start(Reply::Token {
                expires_in: Some(120),
            })
            .await;
            let clock = Arc::new(ManualClock::new());
            let auth = auth_with(Some(endpoint.url.clone()), clock.clone(), None);

            assert_eq!(auth.access_token().await.unwrap(), "token-1");
            assert_eq!(endpoint.request_count(), 1);

            // One second short of the 90s lead: still the cached token, still one request.
            clock.advance(Duration::from_secs(89));
            assert_eq!(auth.access_token().await.unwrap(), "token-1");
            assert_eq!(
                endpoint.request_count(),
                1,
                "the token endpoint must not be called again before the lead elapses"
            );

            // At the lead: refetched exactly once.
            clock.advance(Duration::from_secs(1));
            assert_eq!(
                auth.access_token().await.unwrap(),
                "token-2",
                "the token must be refreshed once the refresh lead elapses"
            );
            assert_eq!(endpoint.request_count(), 2);
        }

        /// `expires_in` is optional; an omitted one means a 60s lifetime, so the lead is 30s.
        /// A provider that omits the field is common enough that the default is load-bearing.
        #[tokio::test]
        async fn omitted_expires_in_defaults_to_a_thirty_second_lead() {
            let endpoint = TokenEndpoint::start(Reply::Token { expires_in: None }).await;
            let clock = Arc::new(ManualClock::new());
            let auth = auth_with(Some(endpoint.url.clone()), clock.clone(), None);

            assert_eq!(auth.access_token().await.unwrap(), "token-1");

            clock.advance(Duration::from_secs(29));
            assert_eq!(auth.access_token().await.unwrap(), "token-1");

            clock.advance(Duration::from_secs(1));
            assert_eq!(auth.access_token().await.unwrap(), "token-2");
        }

        /// A token that expires sooner than the 30s lead would subtract to zero. The
        /// `saturating_sub(30).max(1)` floor keeps the lead at one second instead, so the
        /// token is used once rather than refetched on every single call — the behaviour
        /// that makes a short-lived-token provider usable at all. Without the `max(1)`, a
        /// zero lead would mean `now < refresh_after` is false immediately.
        #[tokio::test]
        async fn short_lived_tokens_get_a_one_second_floor_not_a_zero_lead() {
            let endpoint = TokenEndpoint::start(Reply::Token {
                expires_in: Some(10),
            })
            .await;
            let clock = Arc::new(ManualClock::new());
            let auth = auth_with(Some(endpoint.url.clone()), clock.clone(), None);

            assert_eq!(auth.access_token().await.unwrap(), "token-1");
            // Immediately afterwards the token is still served: the lead floored at 1s, it
            // did not saturate to 0.
            assert_eq!(auth.access_token().await.unwrap(), "token-1");
            assert_eq!(
                endpoint.request_count(),
                1,
                "a 10s token must not be refetched on every call"
            );

            clock.advance(Duration::from_secs(1));
            assert_eq!(auth.access_token().await.unwrap(), "token-2");
        }

        // ---- The on-disk cross-process cache -----------------------------------------

        /// A token written by one process is served to the next without a fetch — the whole
        /// point of `CAMUNDA_OAUTH_CACHE_DIR`. Two `Authentication`s over one directory stand
        /// in for two processes.
        #[tokio::test]
        async fn a_disk_cached_token_is_served_without_touching_the_endpoint() {
            let dir = CacheDir::new("hit");
            let endpoint = TokenEndpoint::start(Reply::Token {
                expires_in: Some(600),
            })
            .await;
            let clock = Arc::new(ManualClock::new());

            let first = auth_with(
                Some(endpoint.url.clone()),
                clock.clone(),
                Some(dir.0.clone()),
            );
            assert_eq!(first.access_token().await.unwrap(), "token-1");

            let second = auth_with(
                Some(endpoint.url.clone()),
                clock.clone(),
                Some(dir.0.clone()),
            );
            assert_eq!(
                second.access_token().await.unwrap(),
                "token-1",
                "a second client must reuse the token from disk"
            );
            assert_eq!(
                endpoint.request_count(),
                1,
                "the disk cache must spare the second client a token request"
            );
        }

        /// A corrupt cache file must fall through to a fetch, not fail the call. The disk
        /// cache is an optimisation; a bad one is not an outage.
        #[tokio::test]
        async fn a_corrupt_disk_cache_falls_through_to_a_fetch() {
            let dir = CacheDir::new("corrupt");
            let endpoint = TokenEndpoint::start(Reply::Token {
                expires_in: Some(600),
            })
            .await;
            let clock = Arc::new(ManualClock::new());
            let auth = auth_with(
                Some(endpoint.url.clone()),
                clock.clone(),
                Some(dir.0.clone()),
            );

            let path = auth.cache_file_path().expect("a cache dir is configured");
            std::fs::create_dir_all(&dir.0).unwrap();
            std::fs::write(&path, b"this is not json").unwrap();

            assert_eq!(
                auth.access_token().await.unwrap(),
                "token-1",
                "a corrupt cache must not fail the call"
            );
            // And the fetch repaired it.
            assert!(
                auth.read_disk_token().is_some(),
                "the corrupt cache should have been overwritten by the fetch"
            );
        }

        /// An expired entry on disk must not be served. `read_disk_token` maps a wall-clock
        /// expiry onto the monotonic clock with a branch on each side of "now"; the
        /// already-past branch clamps `refresh_after` to `now`, which `access_token` then
        /// rejects because its check is strict (`now < refresh_after`). A clamp to anything
        /// at or after `now` would serve a dead token.
        #[tokio::test]
        async fn an_expired_disk_entry_is_not_served() {
            let dir = CacheDir::new("expired");
            let endpoint = TokenEndpoint::start(Reply::Token {
                expires_in: Some(600),
            })
            .await;
            let clock = Arc::new(ManualClock::new());
            let auth = auth_with(
                Some(endpoint.url.clone()),
                clock.clone(),
                Some(dir.0.clone()),
            );

            // Write a cache entry whose refresh-after is in the past.
            let path = auth.cache_file_path().expect("a cache dir is configured");
            std::fs::create_dir_all(&dir.0).unwrap();
            let stale_ms = clock
                .now_wall()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                - 60_000;
            std::fs::write(
                &path,
                serde_json::to_vec(&DiskToken {
                    access_token: "stale-token".to_string(),
                    refresh_after_unix_ms: stale_ms,
                })
                .unwrap(),
            )
            .unwrap();

            assert_eq!(
                auth.access_token().await.unwrap(),
                "token-1",
                "an expired disk entry must be refetched, not served"
            );
        }

        /// The wall-clock-to-monotonic mapping on the not-yet-expired branch must preserve
        /// the remaining lifetime, not reset it: a token written with 10 minutes left and
        /// read back must still expire 10 minutes from the reader's `now`, which is what
        /// makes the cache survive a process restart correctly.
        #[tokio::test]
        async fn disk_expiry_maps_onto_the_monotonic_clock_preserving_remaining_life() {
            let dir = CacheDir::new("mapping");
            let clock = Arc::new(ManualClock::new());
            let auth = auth_with(None, clock.clone(), Some(dir.0.clone()));

            let path = auth.cache_file_path().expect("a cache dir is configured");
            std::fs::create_dir_all(&dir.0).unwrap();
            let future_ms = (clock.now_wall() + Duration::from_secs(600))
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            std::fs::write(
                &path,
                serde_json::to_vec(&DiskToken {
                    access_token: "disk-token".to_string(),
                    refresh_after_unix_ms: future_ms,
                })
                .unwrap(),
            )
            .unwrap();

            let read = auth.read_disk_token().expect("the entry reads back");
            let remaining = read.refresh_after - clock.now();
            // Allow a second of slack for the millisecond truncation in the on-disk format.
            assert!(
                remaining <= Duration::from_secs(600) && remaining >= Duration::from_secs(599),
                "remaining lifetime should map to ~600s, got {remaining:?}"
            );
        }

        /// No cache directory configured means no disk I/O at all — `read_disk_token`
        /// returns `None` rather than reaching for a path.
        #[test]
        fn without_a_cache_dir_there_is_no_disk_cache() {
            let auth = auth_with(None, Arc::new(ManualClock::new()), None);
            assert!(auth.cache_file_path().is_none());
            assert!(auth.read_disk_token().is_none());
        }
    }
}
