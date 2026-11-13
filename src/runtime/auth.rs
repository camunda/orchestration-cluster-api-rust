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
            if let Err(e) = create_cache_dir(parent) {
                tracing::warn!(error = %e, "failed to create OAuth cache directory");
                return;
            }
        }
        let tmp = staging_path(&path);
        if let Err(e) = write_owner_only(&tmp, &json) {
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

/// A staging path for one `write_disk_token` call, unique to that call.
///
/// The process id alone is not enough. `access_token` releases the cache mutex before
/// fetching, so two refreshes in one process can reach the write concurrently and would
/// otherwise share a staging path: one writer's exclusive create fails, or — worse, had the
/// path been cleared first — one unlinks the file the other is still writing and renames a
/// truncated token into place. A per-call counter makes the two writers independent, and the
/// loser of the rename race simply overwrites with an equally valid token.
fn staging_path(path: &std::path::Path) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    path.with_extension(format!(
        "tmp-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

/// Create the OAuth token-cache directory, owner-only where the platform has a mode.
///
/// A directory the operator already created is left exactly as it is — its permissions are
/// their decision, and silently widening or narrowing them would be worse than respecting
/// them. Only the directories this call creates get `0700`.
fn create_cache_dir(dir: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)
    }
}

/// Write `bytes` to `path`, creating the file readable and writable by its owner only.
///
/// The cached value is a live bearer token, so the mode is set **at creation** (`O_CREAT`
/// with `0600`) rather than applied afterwards: a `write`-then-`chmod` would leave a window
/// in which the credential sits on disk world-readable, and this file is the staging file
/// for the atomic rename, so that window is exactly where it matters.
///
/// Creation is **exclusive** (`O_EXCL`), because a mode is only applied to a file this call
/// actually creates. Opening an existing path would adopt whatever permissions it already
/// had — so a `0644` file left behind by an interrupted write under an older SDK would
/// receive the new token and keep its world-readable mode straight through the rename.
/// Refusing is the right answer rather than deleting first: the caller treats a failed cache
/// write as best-effort and logs it, whereas unlinking a path this call did not create could
/// destroy a file another writer is mid-way through.
///
/// Windows has no mode bit to set. There the file inherits the directory's ACL, which is
/// why `CAMUNDA_OAUTH_CACHE_DIR` should point at a per-user location on that platform.
fn write_owner_only(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    // The rename that follows swaps in this file's inode, and with it this file's mode — so
    // a cache left at 0644 by an older SDK version is replaced rather than written into.
    let mut file = options.open(path)?;
    file.write_all(bytes)
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

    /// The on-disk OAuth cache holds a live bearer token — a credential for the cluster,
    /// valid until its refresh-after time. It must not be readable by other users on the
    /// host, and neither must the temp file it is staged through.
    mod disk_cache_is_owner_only {
        use super::*;

        /// A cache directory of our own, inside the system temp dir. Deliberately not a
        /// `tempfile` dependency: the crate has none, and a token-cache test should not add
        /// one. The process id keeps concurrent test binaries apart, and the label keeps
        /// tests in this module apart from each other.
        struct CacheDir(PathBuf);

        impl CacheDir {
            fn new(label: &str) -> Self {
                let dir = std::env::temp_dir().join(format!(
                    "camunda-oauth-cache-{}-{label}",
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

        fn auth_with_cache(dir: &std::path::Path) -> Authentication {
            Authentication {
                inner: Arc::new(Inner {
                    strategy: AuthStrategy::OAuth,
                    oauth_url: Some("https://login.example/oauth/token".to_string()),
                    client_id: Some("client".to_string()),
                    client_secret: Some("secret".to_string()),
                    audience: None,
                    scope: None,
                    basic_username: None,
                    basic_password: None,
                    http: reqwest::Client::new(),
                    cache: Mutex::new(None),
                    oauth_cache_dir: Some(dir.to_path_buf()),
                    clock: super::super::super::clock::live_clock(),
                }),
            }
        }

        fn token_for(auth: &Authentication) -> CachedToken {
            CachedToken {
                token: "cached-bearer-token".to_string(),
                refresh_after: auth.inner.clock.now() + Duration::from_secs(300),
                refresh_after_wall: auth.inner.clock.now_wall() + Duration::from_secs(300),
            }
        }

        #[cfg(unix)]
        #[test]
        fn cache_file_is_mode_0600() {
            use std::os::unix::fs::PermissionsExt;

            let dir = CacheDir::new("file-mode");
            let auth = auth_with_cache(&dir.0);
            auth.write_disk_token(&token_for(&auth));

            let path = auth.cache_file_path().expect("a cache dir is configured");
            let mode = std::fs::metadata(&path)
                .expect("the token cache file was written")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(
                mode, 0o600,
                "the OAuth token cache must be readable by its owner only, got {mode:o}"
            );
        }

        #[cfg(unix)]
        #[test]
        fn cache_directory_created_by_the_sdk_is_mode_0700() {
            use std::os::unix::fs::PermissionsExt;

            let dir = CacheDir::new("dir-mode");
            let auth = auth_with_cache(&dir.0);
            auth.write_disk_token(&token_for(&auth));

            let mode = std::fs::metadata(&dir.0)
                .expect("the cache directory was created")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(
                mode, 0o700,
                "a cache directory the SDK creates must not be listable by others, got {mode:o}"
            );
        }

        /// The staging file the atomic write goes through is the same credential. Creating
        /// it with default permissions and tightening afterwards would leave a window in
        /// which it is world-readable, so the mode has to be set at creation. Writing to a
        /// directory that already exists (the common case on the second run) must not
        /// change that.
        #[cfg(unix)]
        #[test]
        fn rewriting_into_an_existing_directory_keeps_mode_0600() {
            use std::os::unix::fs::PermissionsExt;

            let dir = CacheDir::new("rewrite");
            std::fs::create_dir_all(&dir.0).expect("pre-create the cache directory");
            let auth = auth_with_cache(&dir.0);
            auth.write_disk_token(&token_for(&auth));
            auth.write_disk_token(&token_for(&auth));

            let path = auth.cache_file_path().expect("a cache dir is configured");
            let mode = std::fs::metadata(&path)
                .expect("the token cache file was written")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(
                mode, 0o600,
                "a rewritten token cache must still be owner-only, got {mode:o}"
            );

            // The staging file must not survive a successful commit.
            let leftovers: Vec<_> = std::fs::read_dir(&dir.0)
                .expect("the cache directory is readable")
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|name| name.contains("tmp-"))
                .collect();
            assert!(
                leftovers.is_empty(),
                "a staging file was left behind: {leftovers:?}"
            );
        }

        /// A mode is only applied to a file the open actually creates, so adopting an
        /// existing file is the hole: a `0644` staging file left by an interrupted write
        /// would be written into and the rename would commit the token still world-readable.
        /// `write_owner_only` refuses such a path outright rather than clearing it, because
        /// a path it did not create may belong to another writer. The caller treats a failed
        /// cache write as best-effort, so refusing costs a cache miss and never a leak.
        #[cfg(unix)]
        #[test]
        fn write_owner_only_refuses_an_existing_path_instead_of_adopting_its_mode() {
            use std::os::unix::fs::PermissionsExt;

            let dir = CacheDir::new("existing-path");
            std::fs::create_dir_all(&dir.0).expect("create the cache directory");
            let victim = dir.0.join("already-here.json");
            std::fs::write(&victim, b"untouched").expect("pre-create the file");
            std::fs::set_permissions(&victim, std::fs::Permissions::from_mode(0o644))
                .expect("make it world-readable");

            let err = write_owner_only(&victim, b"a-bearer-token")
                .expect_err("an existing path must be refused, not adopted");
            assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);

            assert_eq!(
                std::fs::read(&victim).expect("the file still exists"),
                b"untouched",
                "a path this call did not create must be left alone"
            );
        }

        /// Two writes in one process must not share a staging path. `access_token` releases
        /// the cache mutex before fetching, so two refreshes can reach the write at once; a
        /// shared staging path would mean one writer's create fails, or one truncates the
        /// file the other is still writing and renames it into place. Both writes must land
        /// a valid cache at `0600` whichever wins the rename.
        #[cfg(unix)]
        #[test]
        fn concurrent_writes_use_separate_staging_files() {
            use std::os::unix::fs::PermissionsExt;

            let dir = CacheDir::new("concurrent");
            let auth = auth_with_cache(&dir.0);
            let path = auth.cache_file_path().expect("a cache dir is configured");

            assert_ne!(
                staging_path(&path),
                staging_path(&path),
                "each write needs a staging path of its own"
            );

            let auth = Arc::new(auth);
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    let auth = auth.clone();
                    std::thread::spawn(move || auth.write_disk_token(&token_for(&auth)))
                })
                .collect();
            for t in threads {
                t.join().expect("no writer panicked");
            }

            let mode = std::fs::metadata(&path)
                .expect("the token cache file was written")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(
                mode, 0o600,
                "every writer must commit at 0600, got {mode:o}"
            );
            assert_eq!(
                auth.read_disk_token().expect("the cache reads back").token,
                "cached-bearer-token",
                "a concurrent write must never commit a truncated token"
            );

            // No writer may leave its staging file behind.
            let leftovers: Vec<_> = std::fs::read_dir(&dir.0)
                .expect("the cache directory is readable")
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|name| name.contains("tmp-"))
                .collect();
            assert!(
                leftovers.is_empty(),
                "staging files were left behind: {leftovers:?}"
            );
        }

        /// The token must round-trip through the cache it is written to, so the permission
        /// change cannot be "secure because it broke the feature".
        #[test]
        fn a_written_token_is_read_back() {
            let dir = CacheDir::new("roundtrip");
            let auth = auth_with_cache(&dir.0);
            auth.write_disk_token(&token_for(&auth));

            let read = auth.read_disk_token().expect("the token reads back");
            assert_eq!(read.token, "cached-bearer-token");
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
