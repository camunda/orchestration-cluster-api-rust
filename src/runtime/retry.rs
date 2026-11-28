//! Transient-error retry with exponential backoff and full jitter.
//!
//! Mirrors the JS / C# SDKs' HTTP retry layer: initiating operations that fail with a
//! retryable signal (HTTP 429/502/503/504, or a network-level error; a 504 that reports a probe
//! as over is not) are retried up to
//! [`RetryConfig::max_attempts`] times with exponentially increasing, fully-jittered delays.
//! Non-retryable errors (4xx other than 429, validation, serialization) fail immediately.

use std::future::Future;
use std::time::Duration;

use super::clock::Clock;
use super::config::RetryConfig;
use super::errors::{CamundaError, Result};
use super::random::Random;

/// Problem titles of a 504 that report the *end* of a probe rather than a transient failure:
/// a worker collected it and did not answer by its deadline, or it still waited for the values
/// of its secrets then. Retrying would create the probe again, and run the worker again, only
/// to fail the same way after another full timeout. A 504 titled `DEADLINE_EXCEEDED` (the
/// gateway timed out, for example after a leader change) stays retryable.
const TERMINAL_GATEWAY_TIMEOUTS: [&str; 2] = ["WORKER_DID_NOT_ANSWER", "SECRET_RESOLUTION_TIMEOUT"];

/// Whether an error body is a problem detail whose title is a terminal gateway timeout.
fn is_terminal_gateway_timeout(body: Option<&str>) -> bool {
    body.and_then(|b| serde_json::from_str::<serde_json::Value>(b).ok())
        .and_then(|v| v.get("title")?.as_str().map(str::to_owned))
        .is_some_and(|title| TERMINAL_GATEWAY_TIMEOUTS.contains(&title.as_str()))
}

/// Whether an error should trigger a retry.
pub(crate) fn is_retryable(err: &CamundaError) -> bool {
    match err {
        // Network/connection/timeout failures are transient.
        CamundaError::Network(_) => true,
        // A probe that ran out of time has already had its answer window; do not repeat it.
        CamundaError::Api {
            status: 504, body, ..
        } => !is_terminal_gateway_timeout(body.as_deref()),
        // Standard transient HTTP statuses.
        CamundaError::Api { status, .. } => matches!(status, 429 | 502 | 503),
        _ => false,
    }
}

/// Whether a failed **job command** (`completeJob`, `failJob`, `throwJobError`) may be
/// retried. Job commands are not idempotent: a retry after the command was applied but its
/// response lost gets a 404. So they are retried only when the server says the command was
/// **not applied** -- HTTP 429 or 503 -- or the connection failed before the request was
/// sent (connect refused, DNS, TLS handshake). Never on 504, 502, 500, a read timeout or a
/// connection reset after sending, where the outcome is unknown.
pub(crate) fn is_job_command_retryable(err: &CamundaError) -> bool {
    match err {
        CamundaError::Network(e) => e.is_connect(),
        CamundaError::Api { status, .. } => matches!(status, 429 | 503),
        _ => false,
    }
}

/// Whether a failed job command's outcome is unknown: the server (or the network) failed
/// after the request may have reached the broker, so the command may have been applied.
pub(crate) fn is_unknown_outcome(err: &CamundaError) -> bool {
    match err {
        CamundaError::Network(e) => !e.is_connect(),
        CamundaError::Api { status, .. } => *status >= 500 && *status != 503,
        _ => false,
    }
}

/// Compute the backoff delay for a given (zero-based) attempt using full jitter:
/// `delay = random(0, min(max_delay, base * 2^attempt))`.
fn backoff_delay(cfg: &RetryConfig, attempt: u32, random: &dyn Random) -> Duration {
    let exp = cfg.base_delay_ms.saturating_mul(1u64 << attempt.min(32));
    let capped = exp.min(cfg.max_delay_ms);
    let jittered = (capped as f64 * random.next_f64()) as u64;
    Duration::from_millis(jittered)
}

/// Run `op`, retrying on transient failures per `cfg`.
///
/// `op` is a factory so each attempt gets a fresh future.
pub(crate) async fn with_retry<T, F, Fut>(
    cfg: &RetryConfig,
    clock: &dyn Clock,
    random: &dyn Random,
    op: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    with_retry_if(cfg, clock, random, is_retryable, op).await
}

/// [`with_retry`] with a custom retry predicate (e.g. [`is_job_command_retryable`]).
pub(crate) async fn with_retry_if<T, F, Fut>(
    cfg: &RetryConfig,
    clock: &dyn Clock,
    random: &dyn Random,
    retryable: fn(&CamundaError) -> bool,
    mut op: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let max_attempts = cfg.max_attempts.max(1);
    let mut attempt = 0u32;
    loop {
        match op().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempt += 1;
                if attempt >= max_attempts || !retryable(&err) {
                    return Err(err);
                }
                let delay = backoff_delay(cfg, attempt - 1, random);
                tracing::debug!(
                    attempt,
                    max_attempts,
                    delay_ms = delay.as_millis() as u64,
                    error = %err,
                    "retrying transient error"
                );
                clock.sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    /// These exercise retry ordering, not clock behaviour, so they keep real time — which is
    /// already virtual under start_paused.
    static LIVE: LazyLock<std::sync::Arc<dyn Clock>> =
        LazyLock::new(super::super::clock::live_clock);
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Draws the same value every time.
    #[derive(Debug)]
    struct Fixed(f64);

    impl Random for Fixed {
        fn next_f64(&self) -> f64 {
            self.0
        }
    }

    /// The backoff is the draw scaled onto the capped exponential window, so a controlled
    /// draw yields the exact delay rather than a range.
    #[test]
    fn backoff_scales_the_draw_onto_the_capped_window() {
        let cfg = RetryConfig {
            max_attempts: 10,
            base_delay_ms: 100,
            max_delay_ms: 5_000,
        };
        for (attempt, want_ms) in [(0, 25), (3, 200), (6, 1_250), (40, 1_250)] {
            assert_eq!(
                backoff_delay(&cfg, attempt, &Fixed(0.25)),
                Duration::from_millis(want_ms),
                "attempt {attempt}"
            );
        }
    }

    fn cfg() -> RetryConfig {
        RetryConfig {
            max_attempts: 4,
            base_delay_ms: 1,
            max_delay_ms: 2,
        }
    }

    #[test]
    fn retryable_classification() {
        assert!(is_retryable(&CamundaError::Api {
            status: 503,
            body: None
        }));
        assert!(is_retryable(&CamundaError::Api {
            status: 429,
            body: None
        }));
        assert!(!is_retryable(&CamundaError::Api {
            status: 404,
            body: None
        }));
        assert!(!is_retryable(&CamundaError::Validation("x".into())));
    }

    #[test]
    fn job_commands_retry_only_when_not_applied() {
        for status in [429, 503] {
            let e = CamundaError::Api { status, body: None };
            assert!(is_job_command_retryable(&e), "{status}");
            assert!(!is_unknown_outcome(&e), "{status}");
        }
        for status in [500, 502, 504] {
            let e = CamundaError::Api { status, body: None };
            assert!(!is_job_command_retryable(&e), "{status}");
            assert!(is_unknown_outcome(&e), "{status}");
        }
        for status in [400, 404, 409] {
            let e = CamundaError::Api { status, body: None };
            assert!(!is_job_command_retryable(&e), "{status}");
            assert!(!is_unknown_outcome(&e), "{status}");
        }
    }

    #[tokio::test]
    async fn a_job_command_504_is_attempted_once() {
        let calls = AtomicU32::new(0);
        let result: Result<u32> = with_retry_if(
            &cfg(),
            LIVE.as_ref(),
            &Fixed(0.5),
            is_job_command_retryable,
            || {
                calls.fetch_add(1, Ordering::SeqCst);
                async { Err(gateway_timeout(Some("DEADLINE_EXCEEDED"))) }
            },
        )
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    /// Run one job command attempt over real transport against `url`, counting attempts,
    /// under the job-command retry predicate. `timeout` bounds each request.
    async fn post_job_command(url: &str, timeout: Duration, calls: &AtomicU32) -> Result<()> {
        let http = reqwest::Client::builder().timeout(timeout).build().unwrap();
        with_retry_if(
            &cfg(),
            LIVE.as_ref(),
            &Fixed(0.5),
            is_job_command_retryable,
            || {
                calls.fetch_add(1, Ordering::SeqCst);
                let http = http.clone();
                async move {
                    http.post(url).body("{}").send().await?;
                    Ok(())
                }
            },
        )
        .await
    }

    /// A server that accepts each connection, reads the request, then either closes the
    /// socket without answering (`hold == false`) or holds it open unanswered. Returns its
    /// URL and the number of requests it received.
    async fn mute_server(hold: bool) -> (String, std::sync::Arc<AtomicU32>) {
        use tokio::io::AsyncReadExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!(
            "http://{}/v2/jobs/7/completion",
            listener.local_addr().unwrap()
        );
        let received = std::sync::Arc::new(AtomicU32::new(0));
        let seen = received.clone();
        tokio::spawn(async move {
            while let Ok((mut sock, _)) = listener.accept().await {
                let seen = seen.clone();
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    let mut tmp = [0u8; 1024];
                    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                        match sock.read(&mut tmp).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        }
                    }
                    seen.fetch_add(1, Ordering::SeqCst);
                    if hold {
                        // Keep the socket open until the client gives up: drain any
                        // remaining request bytes (e.g. a separately sent body) until
                        // EOF or an error, so fragmentation cannot close it early.
                        while let Ok(n) = sock.read(&mut tmp).await {
                            if n == 0 {
                                break;
                            }
                        }
                    }
                });
            }
        });
        (url, received)
    }

    /// Issue #218: a refused connection never reached the server, so the job command is
    /// retried up to `max_attempts`.
    #[tokio::test]
    async fn a_refused_connection_is_retried_up_to_max_attempts() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let calls = AtomicU32::new(0);
        let err = post_job_command(
            &format!("http://{addr}/v2/jobs/7/completion"),
            Duration::from_secs(5),
            &calls,
        )
        .await
        .expect_err("nothing is listening");
        assert!(
            matches!(&err, CamundaError::Network(e) if e.is_connect()),
            "{err:?}"
        );
        assert!(!is_unknown_outcome(&err));
        assert_eq!(calls.load(Ordering::SeqCst), cfg().max_attempts);
    }

    /// Issue #218: the server read the command, then closed without answering. The command
    /// may have been applied, so it is sent exactly once and classed as an unknown outcome.
    #[tokio::test]
    async fn a_connection_closed_after_sending_is_not_retried() {
        let (url, received) = mute_server(false).await;
        let calls = AtomicU32::new(0);
        let err = post_job_command(&url, Duration::from_secs(5), &calls)
            .await
            .expect_err("the server closed without answering");
        assert!(is_unknown_outcome(&err), "{err:?}");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(received.load(Ordering::SeqCst), 1);
    }

    /// Issue #218: a read timeout after sending is an unknown outcome: sent exactly once.
    #[tokio::test]
    async fn a_read_timeout_after_sending_is_not_retried() {
        let (url, received) = mute_server(true).await;
        let calls = AtomicU32::new(0);
        let err = post_job_command(&url, Duration::from_millis(200), &calls)
            .await
            .expect_err("the server never answers");
        assert!(
            matches!(&err, CamundaError::Network(e) if e.is_timeout()),
            "{err:?}"
        );
        assert!(is_unknown_outcome(&err), "{err:?}");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(received.load(Ordering::SeqCst), 1);
    }

    fn gateway_timeout(title: Option<&str>) -> CamundaError {
        CamundaError::Api {
            status: 504,
            body: title.map(|t| format!(r#"{{"type":"about:blank","title":"{t}","status":504}}"#)),
        }
    }

    /// A probe that ran out of time is over: retrying would run its worker again for the same
    /// outcome. Every other 504, including one with no readable body, stays transient.
    #[test]
    fn terminal_probe_timeouts_are_not_retryable() {
        for title in ["WORKER_DID_NOT_ANSWER", "SECRET_RESOLUTION_TIMEOUT"] {
            assert!(!is_retryable(&gateway_timeout(Some(title))), "{title}");
        }
        assert!(is_retryable(&gateway_timeout(Some("DEADLINE_EXCEEDED"))));
        assert!(is_retryable(&gateway_timeout(None)));
        for body in ["not json", "[]", r#"{"title":7}"#, r#"{"detail":"x"}"#, ""] {
            assert!(
                is_retryable(&CamundaError::Api {
                    status: 504,
                    body: Some(body.to_string())
                }),
                "{body:?}"
            );
        }
    }

    /// The titles only mean "terminal" on a 504: the same title on another status is not
    /// special-cased, so 429/502/503 keep their retry behaviour whatever the body says.
    #[test]
    fn the_terminal_titles_do_not_change_other_statuses() {
        for status in [429, 502, 503] {
            assert!(is_retryable(&CamundaError::Api {
                status,
                body: Some(r#"{"title":"WORKER_DID_NOT_ANSWER"}"#.to_string())
            }));
        }
        assert!(!is_retryable(&CamundaError::Api {
            status: 404,
            body: None
        }));
    }

    #[tokio::test]
    async fn a_terminal_probe_timeout_is_attempted_once() {
        let calls = AtomicU32::new(0);
        let result: Result<u32> = with_retry(&cfg(), LIVE.as_ref(), &Fixed(0.5), || {
            calls.fetch_add(1, Ordering::SeqCst);
            async { Err(gateway_timeout(Some("WORKER_DID_NOT_ANSWER"))) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_transient_gateway_timeout_is_still_retried() {
        let calls = AtomicU32::new(0);
        let result: Result<u32> = with_retry(&cfg(), LIVE.as_ref(), &Fixed(0.5), || {
            calls.fetch_add(1, Ordering::SeqCst);
            async { Err(gateway_timeout(Some("DEADLINE_EXCEEDED"))) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn retries_until_success() {
        let calls = AtomicU32::new(0);
        let result: Result<u32> = with_retry(&cfg(), LIVE.as_ref(), &Fixed(0.5), || {
            let n = calls.fetch_add(1, Ordering::SeqCst);
            async move {
                if n < 2 {
                    Err(CamundaError::Api {
                        status: 503,
                        body: None,
                    })
                } else {
                    Ok(n)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 2);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn gives_up_after_max_attempts() {
        let calls = AtomicU32::new(0);
        let result: Result<u32> = with_retry(&cfg(), LIVE.as_ref(), &Fixed(0.5), || {
            calls.fetch_add(1, Ordering::SeqCst);
            async {
                Err(CamundaError::Api {
                    status: 503,
                    body: None,
                })
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn does_not_retry_non_retryable() {
        let calls = AtomicU32::new(0);
        let result: Result<u32> = with_retry(&cfg(), LIVE.as_ref(), &Fixed(0.5), || {
            calls.fetch_add(1, Ordering::SeqCst);
            async { Err(CamundaError::Validation("nope".into())) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[tokio::test(start_paused = true)]
    async fn backoff_waits_on_the_injected_clock() {
        // Slice 1 built the seam and changed nothing; this is the assertion that would have
        // caught it staying unused. The recorded sleeps are the proof the wait came here
        // rather than going to `tokio::time::sleep` directly.
        let clock = std::sync::Arc::new(super::super::clock::RecordingClock::default());
        let calls = AtomicU32::new(0);

        let result: Result<u32> = with_retry(&cfg(), clock.as_ref(), &Fixed(0.5), || {
            let n = calls.fetch_add(1, Ordering::SeqCst);
            async move {
                if n < 2 {
                    Err(CamundaError::Api {
                        status: 503,
                        body: None,
                    })
                } else {
                    Ok(n)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(
            clock.sleeps().len(),
            2,
            "two retries should have produced two waits on the injected clock"
        );
    }

    /// The same property for an arbitrary seed: the wait is exactly the one a replay of the
    /// seed predicts. Set CAMUNDA_TEST_SEED to the reported seed to reproduce.
    #[tokio::test(start_paused = true)]
    async fn backoff_is_replayable_for_any_seed() {
        use super::super::random::SeededRandom;
        let random = SeededRandom::from_env().expect("CAMUNDA_TEST_SEED should be a valid seed");
        let clock = std::sync::Arc::new(super::super::clock::RecordingClock::default());
        let cfg = RetryConfig {
            max_attempts: 2,
            base_delay_ms: 100,
            max_delay_ms: 5_000,
        };

        let _: Result<u32> = with_retry(&cfg, clock.as_ref(), &random, || async {
            Err(CamundaError::Api {
                status: 503,
                body: None,
            })
        })
        .await;

        let want =
            Duration::from_millis((100.0 * SeededRandom::new(random.seed()).next_f64()) as u64);
        assert!(
            want < Duration::from_millis(100),
            "{random}: predicted wait {want:?} is outside the full-jitter window"
        );
        assert_eq!(clock.sleeps(), vec![want], "{random}");
    }
}
