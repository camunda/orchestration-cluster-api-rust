//! Eventual-consistency polling helper.
//!
//! Camunda's secondary storage (used by the `search_*` / `get_*` read APIs) is eventually
//! consistent: an entity created or changed via a command may not yet be visible to a
//! subsequent read. This helper repeatedly runs a read operation until a caller-supplied
//! predicate is satisfied (or a timeout elapses), transparently retrying transient `404`
//! results on a freshly-created entity.
//!
//! Mirrors the JS / Python / C# SDKs' `eventualPoll` / `ConsistencyOptions` surface.
//!
//! ```no_run
//! # use camunda_orchestration_sdk::{CamundaClient};
//! # use camunda_orchestration_sdk::eventual::ConsistencyOptions;
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CamundaClient::from_env()?;
//! // Poll until the process instance is visible to the read API.
//! let key = "12345".to_string();
//! let instance = client
//!     .eventual(ConsistencyOptions::default(), || {
//!         let client = client.clone();
//!         let key = key.clone();
//!         async move { client.get_process_instance(&key).await }
//!     })
//!     .await?;
//! # let _ = instance;
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::time::Duration;

use super::clock::Clock;
use super::errors::{CamundaError, Result};

/// Options controlling an eventual-consistency poll.
#[derive(Debug, Clone)]
pub struct ConsistencyOptions {
    /// Maximum time to keep polling before giving up. `None` uses the SDK default
    /// (`CAMUNDA_SDK_EVENTUAL_POLL_DEFAULT_MS`).
    pub timeout: Option<Duration>,
    /// Delay between polling attempts.
    pub interval: Duration,
    /// Treat a `404` (not-found) result as "not yet consistent" and keep polling, rather
    /// than failing. Useful right after creating an entity.
    pub retry_not_found: bool,
}

impl Default for ConsistencyOptions {
    fn default() -> Self {
        ConsistencyOptions {
            timeout: None,
            interval: Duration::from_millis(250),
            retry_not_found: true,
        }
    }
}

impl ConsistencyOptions {
    /// Set an explicit overall timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the delay between polling attempts.
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Whether to treat `404` as "not yet consistent" (default `true`).
    pub fn retry_not_found(mut self, retry: bool) -> Self {
        self.retry_not_found = retry;
        self
    }
}

/// Poll `op` until `predicate` returns `true` for its result, or the timeout elapses.
///
/// `default_timeout_ms` is the SDK-configured fallback used when
/// [`ConsistencyOptions::timeout`] is `None`.
///
/// A `404` result is retried while [`ConsistencyOptions::retry_not_found`] is set; any
/// other error is returned immediately. On timeout this returns
/// [`CamundaError::EventualConsistencyTimeout`].
pub(crate) async fn poll<T, F, Fut, P>(
    opts: &ConsistencyOptions,
    default_timeout_ms: u64,
    clock: &dyn Clock,
    mut op: F,
    mut predicate: P,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
    P: FnMut(&T) -> bool,
{
    let timeout = opts
        .timeout
        .unwrap_or_else(|| Duration::from_millis(default_timeout_ms));
    let started = clock.now();

    loop {
        match op().await {
            Ok(value) if predicate(&value) => return Ok(value),
            Ok(_) => { /* consistent read, predicate not yet met — keep polling */ }
            Err(CamundaError::Api { status: 404, .. }) if opts.retry_not_found => {
                // Entity not visible yet; keep polling.
            }
            Err(e) => return Err(e),
        }

        let elapsed = clock.now().duration_since(started);
        if elapsed >= timeout {
            return Err(CamundaError::EventualConsistencyTimeout {
                elapsed_ms: elapsed.as_millis() as u64,
            });
        }
        clock.sleep(opts.interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    /// These exercise poll ordering, not clock behaviour, so they keep real time \u2014 which is\n    /// already virtual under start_paused.
    static LIVE: LazyLock<std::sync::Arc<dyn Clock>> =
        LazyLock::new(super::super::clock::live_clock);
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn polls_until_predicate_met() {
        let calls = AtomicU32::new(0);
        let opts = ConsistencyOptions::default().interval(Duration::from_millis(1));
        let result: Result<u32> = poll(
            &opts,
            1_000,
            LIVE.as_ref(),
            || {
                let n = calls.fetch_add(1, Ordering::SeqCst);
                async move { Ok(n) }
            },
            |v| *v >= 2,
        )
        .await;
        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn retries_404_then_succeeds() {
        let calls = AtomicU32::new(0);
        let opts = ConsistencyOptions::default().interval(Duration::from_millis(1));
        let result: Result<u32> = poll(
            &opts,
            1_000,
            LIVE.as_ref(),
            || {
                let n = calls.fetch_add(1, Ordering::SeqCst);
                async move {
                    if n < 2 {
                        Err(CamundaError::Api {
                            status: 404,
                            body: None,
                        })
                    } else {
                        Ok(n)
                    }
                }
            },
            |_| true,
        )
        .await;
        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn times_out_when_predicate_never_met() {
        let opts = ConsistencyOptions::default()
            .interval(Duration::from_millis(1))
            .timeout(Duration::from_millis(10));
        let result: Result<u32> = poll(
            &opts,
            1_000,
            LIVE.as_ref(),
            || async { Ok(0u32) },
            |_| false,
        )
        .await;
        assert!(matches!(
            result,
            Err(CamundaError::EventualConsistencyTimeout { .. })
        ));
    }

    #[tokio::test]
    async fn propagates_non_404_errors() {
        let opts = ConsistencyOptions::default().interval(Duration::from_millis(1));
        let result: Result<u32> = poll(
            &opts,
            1_000,
            LIVE.as_ref(),
            || async {
                Err(CamundaError::Api {
                    status: 500,
                    body: None,
                })
            },
            |_| true,
        )
        .await;
        assert!(matches!(result, Err(CamundaError::Api { status: 500, .. })));
    }
    #[tokio::test(start_paused = true)]
    async fn the_poll_loop_waits_and_measures_on_the_injected_clock() {
        let clock = std::sync::Arc::new(super::super::clock::RecordingClock::default());
        let opts = ConsistencyOptions {
            interval: Duration::from_millis(250),
            timeout: Some(Duration::from_secs(30)),
            retry_not_found: false,
        };

        let result: Result<u32> = poll(
            &opts,
            1_000,
            clock.as_ref(),
            || async { Ok(0u32) },
            |_| false,
        )
        .await;

        assert!(matches!(
            result,
            Err(CamundaError::EventualConsistencyTimeout { .. })
        ));
        assert!(
            !clock.sleeps().is_empty(),
            "the poll gap did not wait on the injected clock"
        );
        assert!(
            clock
                .sleeps()
                .iter()
                .all(|d| *d == Duration::from_millis(250)),
            "the poller waited for something other than its configured interval"
        );
        assert!(
            clock.now_calls() > 0,
            "the deadline was decided without reading the injected clock"
        );
    }
}
