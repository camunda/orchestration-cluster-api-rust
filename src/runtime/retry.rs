//! Transient-error retry with exponential backoff and full jitter.
//!
//! Mirrors the JS / C# SDKs' HTTP retry layer: initiating operations that fail with a
//! retryable signal (HTTP 429/502/503/504, or a network-level error) are retried up to
//! [`RetryConfig::max_attempts`] times with exponentially increasing, fully-jittered delays.
//! Non-retryable errors (4xx other than 429, validation, serialization) fail immediately.

use std::future::Future;
use std::time::Duration;

use super::clock::Clock;
use super::config::RetryConfig;
use super::errors::{CamundaError, Result};
use super::random::Random;

/// Whether an error should trigger a retry.
pub(crate) fn is_retryable(err: &CamundaError) -> bool {
    match err {
        // Network/connection/timeout failures are transient.
        CamundaError::Network(_) => true,
        // Standard transient HTTP statuses.
        CamundaError::Api { status, .. } => matches!(status, 429 | 502 | 503 | 504),
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
                if attempt >= max_attempts || !is_retryable(&err) {
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
