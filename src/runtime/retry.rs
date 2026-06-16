//! Transient-error retry with exponential backoff and full jitter.
//!
//! Mirrors the JS / C# SDKs' HTTP retry layer: initiating operations that fail with a
//! retryable signal (HTTP 429/502/503/504, or a network-level error) are retried up to
//! [`RetryConfig::max_attempts`] times with exponentially increasing, fully-jittered delays.
//! Non-retryable errors (4xx other than 429, validation, serialization) fail immediately.

use std::future::Future;
use std::time::Duration;

use super::config::RetryConfig;
use super::errors::{CamundaError, Result};

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
fn backoff_delay(cfg: &RetryConfig, attempt: u32) -> Duration {
    let exp = cfg.base_delay_ms.saturating_mul(1u64 << attempt.min(32));
    let capped = exp.min(cfg.max_delay_ms);
    let jittered = (capped as f64 * super::rand_fraction()) as u64;
    Duration::from_millis(jittered)
}

/// Run `op`, retrying on transient failures per `cfg`.
///
/// `op` is a factory so each attempt gets a fresh future.
pub(crate) async fn with_retry<T, F, Fut>(cfg: &RetryConfig, mut op: F) -> Result<T>
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
                let delay = backoff_delay(cfg, attempt - 1);
                tracing::debug!(
                    attempt,
                    max_attempts,
                    delay_ms = delay.as_millis() as u64,
                    error = %err,
                    "retrying transient error"
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

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
        let result: Result<u32> = with_retry(&cfg(), || {
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
        let result: Result<u32> = with_retry(&cfg(), || {
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
        let result: Result<u32> = with_retry(&cfg(), || {
            calls.fetch_add(1, Ordering::SeqCst);
            async { Err(CamundaError::Validation("nope".into())) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
