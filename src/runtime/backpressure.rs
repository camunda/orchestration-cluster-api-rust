//! Adaptive global backpressure controller.
//!
//! Escalates on broker backpressure signals (HTTP 429 / 503 / `RESOURCE_EXHAUSTED`)
//! and throttles initiating operations via an adaptive concurrency limiter, while
//! letting drain operations (job completion / failure) bypass the gate.
//!
//! This is a Rust port of the Python/TypeScript SDK [`BackpressureManager`], mirroring
//! the same AIMD-style recovery and the two profiles:
//!
//! * [`BackpressureProfile::Balanced`] (default): adaptive gating with multiplicative
//!   shrink on backpressure and additive-then-multiplicative recovery while healthy.
//! * [`BackpressureProfile::Legacy`]: observe-only — record severity but never gate.
//!
//! The manager starts *unlimited* and only boots to an initial permit cap on the first
//! backpressure signal. Once the cluster has been quietly healthy for long enough it
//! returns to unlimited.

use std::str::FromStr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tokio::sync::Notify;

use super::errors::CamundaError;

// --- BALANCED profile tuning constants (matching the Python/TypeScript SDK) -----------

/// Permit cap the manager boots to on the first backpressure signal.
const INITIAL_MAX: u32 = 16;
/// Lowest permit cap the manager will shrink to.
const FLOOR: u32 = 1;
/// Multiplicative shrink factor applied on a `soft` backpressure signal.
const SOFT_FACTOR: f64 = 0.70;
/// Multiplicative shrink factor applied on a `severe` backpressure signal.
const SEVERE_FACTOR: f64 = 0.50;
/// Minimum interval between passive-recovery evaluations.
const RECOVERY_INTERVAL: Duration = Duration::from_secs(1);
/// Additive permit increment applied during phase-1 recovery.
const RECOVERY_STEP: u32 = 1;
/// Consecutive signals required to escalate to `severe`.
const SEVERE_THRESHOLD: u32 = 3;
/// Quiet period after which severity decays one step.
const DECAY_QUIET: Duration = Duration::from_secs(2);
/// Maximum number of queued waiters before [`acquire`](BackpressureManager::acquire) fails fast.
const MAX_WAITERS: u32 = 1000;
/// Multiplicative growth factor applied during phase-2 (healthy) recovery.
const HEALTHY_RECOVERY_MULTIPLIER: f64 = 1.5;
/// Sustained-healthy duration after which the manager returns to unlimited.
const UNLIMITED_AFTER_HEALTHY: Duration = Duration::from_secs(30);

// Backoff-at-floor: rate-limit when concurrency cannot drop any further.
/// Initial backoff applied when stuck at the floor while severe.
const BACKOFF_INITIAL: Duration = Duration::from_millis(25);
/// Ceiling for the backoff-at-floor delay.
const BACKOFF_MAX: Duration = Duration::from_secs(2);
/// Multiplier applied to the backoff-at-floor delay on each severe signal at the floor.
const BACKOFF_ESCALATE: u32 = 2;

/// Backpressure severity level reported by the manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureSeverity {
    /// No recent backpressure.
    Healthy,
    /// Backpressure observed; mild throttling.
    Soft,
    /// Sustained backpressure; aggressive throttling.
    Severe,
}

/// Backpressure tuning profile, configured via `CAMUNDA_SDK_BACKPRESSURE_PROFILE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackpressureProfile {
    /// Adaptive global concurrency control (default).
    #[default]
    Balanced,
    /// Observe-only: record signals but never gate requests.
    Legacy,
}

impl FromStr for BackpressureProfile {
    type Err = CamundaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_uppercase().as_str() {
            "BALANCED" => Ok(BackpressureProfile::Balanced),
            "LEGACY" => Ok(BackpressureProfile::Legacy),
            other => Err(CamundaError::config(format!(
                "invalid CAMUNDA_SDK_BACKPRESSURE_PROFILE '{other}' (expected BALANCED or LEGACY)"
            ))),
        }
    }
}

/// A point-in-time snapshot of the manager's internal state, for observability.
#[derive(Debug, Clone, Copy)]
pub struct BackpressureState {
    /// Current severity level.
    pub severity: BackpressureSeverity,
    /// Consecutive backpressure signals since the last healthy decay.
    pub consecutive: u32,
    /// Current permit cap, or `None` when unlimited.
    pub permits_max: Option<u32>,
    /// Permits currently held.
    pub permits_current: u32,
    /// Number of callers queued waiting for a permit.
    pub waiters: u32,
    /// Current backoff-at-floor delay in milliseconds.
    pub backoff_ms: u64,
}

/// Returns `true` if an HTTP status / body pair signals cluster backpressure.
///
/// Mirrors the JS/Python SDKs: 429, a bare 503, any response carrying
/// `RESOURCE_EXHAUSTED`, and 500 + `RESOURCE_EXHAUSTED` are all treated as backpressure.
pub fn is_backpressure_response(status: u16, body: Option<&str>) -> bool {
    let has_resource_exhausted = body.is_some_and(|b| b.contains("RESOURCE_EXHAUSTED"));
    match status {
        429 => true,
        503 => true,
        500 => has_resource_exhausted,
        _ => has_resource_exhausted,
    }
}

/// Returns `true` if an error represents a backpressure signal from the server.
pub fn is_backpressure_error(err: &CamundaError) -> bool {
    match err {
        CamundaError::Api { status, body } => is_backpressure_response(*status, body.as_deref()),
        _ => false,
    }
}

struct Inner {
    severity: BackpressureSeverity,
    consecutive: u32,
    last_event_at: Option<Instant>,
    permits_current: u32,
    /// `None` means unlimited (the gate is open).
    permits_max: Option<u32>,
    last_recover_check: Option<Instant>,
    healthy_since: Option<Instant>,
    waiters: u32,
    backoff: Duration,
}

/// Adaptive backpressure manager shared (via `Arc`) by an SDK client and all its clones.
///
/// Initiating operations call [`acquire`](Self::acquire) before issuing a request and
/// [`release`](Self::release) afterwards, recording the outcome with
/// [`record_healthy_hint`](Self::record_healthy_hint) on success or
/// [`record_backpressure`](Self::record_backpressure) on a backpressure signal. Drain
/// operations (job completion / failure) bypass the gate entirely.
pub struct BackpressureManager {
    observe_only: bool,
    inner: Mutex<Inner>,
    notify: Notify,
}

impl BackpressureManager {
    /// Create a manager for the given profile.
    pub fn new(profile: BackpressureProfile) -> Self {
        BackpressureManager {
            observe_only: profile == BackpressureProfile::Legacy,
            inner: Mutex::new(Inner {
                severity: BackpressureSeverity::Healthy,
                consecutive: 0,
                last_event_at: None,
                permits_current: 0,
                // Start unlimited — only boot to INITIAL_MAX on the first backpressure signal.
                permits_max: None,
                last_recover_check: None,
                healthy_since: None,
                waiters: 0,
                backoff: Duration::ZERO,
            }),
            notify: Notify::new(),
        }
    }

    /// The current severity level.
    pub fn severity(&self) -> BackpressureSeverity {
        self.inner.lock().unwrap().severity
    }

    /// A snapshot of the manager's internal state, for observability.
    pub fn state(&self) -> BackpressureState {
        let st = self.inner.lock().unwrap();
        BackpressureState {
            severity: st.severity,
            consecutive: st.consecutive,
            permits_max: st.permits_max,
            permits_current: st.permits_current,
            waiters: st.waiters,
            backoff_ms: st.backoff.as_millis() as u64,
        }
    }

    /// Acquire a permit, awaiting until one is available.
    ///
    /// Returns [`CamundaError::Backpressure`] if the waiter queue is at capacity
    /// (`MAX_WAITERS`), preventing unbounded memory growth under sustained backpressure.
    /// In the `Legacy` profile this is a no-op.
    pub async fn acquire(&self) -> Result<(), CamundaError> {
        if self.observe_only {
            return Ok(());
        }

        loop {
            // Backoff-at-floor: rate-limit at the concurrency floor. Sleep outside the lock.
            let backoff = {
                let st = self.inner.lock().unwrap();
                if st.permits_max.is_none() {
                    return Ok(()); // unlimited fast path
                }
                st.backoff
            };
            if !backoff.is_zero() {
                tokio::time::sleep(backoff).await;
            }

            // Register interest *before* re-checking, so a concurrent release cannot be lost.
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            {
                let mut st = self.inner.lock().unwrap();
                match st.permits_max {
                    None => return Ok(()), // went unlimited during the backoff sleep
                    Some(max) => {
                        if st.permits_current < max {
                            st.permits_current += 1;
                            return Ok(());
                        }
                        if st.waiters >= MAX_WAITERS {
                            return Err(CamundaError::Backpressure(format!(
                                "backpressure waiter queue full ({MAX_WAITERS}); rejecting to \
                                 prevent unbounded memory growth"
                            )));
                        }
                        st.waiters += 1;
                    }
                }
            }

            notified.await;

            {
                let mut st = self.inner.lock().unwrap();
                st.waiters = st.waiters.saturating_sub(1);
            }
            // Re-evaluate from the top (re-applies any backoff and re-checks permits).
        }
    }

    /// Release a permit and wake one waiter. In the `Legacy` profile this is a no-op.
    pub fn release(&self) {
        if self.observe_only {
            return;
        }
        let mut st = self.inner.lock().unwrap();
        if st.permits_max.is_none() {
            return;
        }
        if st.permits_current > 0 {
            st.permits_current -= 1;
        }
        drop(st);
        self.notify.notify_one();
    }

    /// Record a backpressure signal from the server.
    pub fn record_backpressure(&self) {
        let now = Instant::now();
        let mut st = self.inner.lock().unwrap();
        st.last_event_at = Some(now);
        st.consecutive += 1;
        st.healthy_since = None;

        if !self.observe_only && st.permits_max.is_none() {
            // Boot to the initial cap on the first signal (was unlimited).
            st.permits_max = Some(INITIAL_MAX);
            st.permits_current = st.permits_current.min(INITIAL_MAX);
        }

        if st.consecutive >= SEVERE_THRESHOLD {
            st.severity = BackpressureSeverity::Severe;
            if !self.observe_only {
                scale_permits(&mut st, SEVERE_FACTOR);
            }
        } else if st.severity == BackpressureSeverity::Healthy {
            st.severity = BackpressureSeverity::Soft;
            if !self.observe_only {
                scale_permits(&mut st, SOFT_FACTOR);
            }
        } else if !self.observe_only {
            // Already soft — keep scaling with the soft factor.
            scale_permits(&mut st, SOFT_FACTOR);
        }

        // Escalate the backoff when stuck at the floor while severe.
        if !self.observe_only
            && st.permits_max == Some(FLOOR)
            && st.severity == BackpressureSeverity::Severe
        {
            st.backoff = if st.backoff.is_zero() {
                BACKOFF_INITIAL
            } else {
                (st.backoff * BACKOFF_ESCALATE).min(BACKOFF_MAX)
            };
        }
    }

    /// Record a successful (non-backpressure) completion, triggering passive recovery.
    pub fn record_healthy_hint(&self) {
        let now = Instant::now();
        let mut st = self.inner.lock().unwrap();
        // Reset backoff immediately on success — the server clearly has capacity.
        st.backoff = Duration::ZERO;
        self.maybe_recover(&mut st, now);
    }

    /// Passive recovery check. Caller must hold the lock.
    fn maybe_recover(&self, st: &mut Inner, now: Instant) {
        if self.observe_only || st.permits_max.is_none() {
            return;
        }
        if let Some(last) = st.last_recover_check {
            if now.duration_since(last) < RECOVERY_INTERVAL {
                return;
            }
        }
        st.last_recover_check = Some(now);

        // Decay severity if the cluster has been quiet (severe -> soft -> healthy).
        let quiet = st
            .last_event_at
            .is_none_or(|t| now.duration_since(t) > DECAY_QUIET);
        if quiet {
            match st.severity {
                BackpressureSeverity::Severe => st.severity = BackpressureSeverity::Soft,
                BackpressureSeverity::Soft => {
                    st.severity = BackpressureSeverity::Healthy;
                    st.healthy_since = Some(now);
                }
                BackpressureSeverity::Healthy => {}
            }
            if st.severity == BackpressureSeverity::Healthy {
                st.consecutive = 0;
            }
        }

        let permits_max = st.permits_max.unwrap_or(FLOOR);

        if st.severity != BackpressureSeverity::Healthy {
            // Phase 1: additive recovery while not yet healthy (bounded by the bootstrap cap).
            if permits_max < INITIAL_MAX {
                st.permits_max = Some((permits_max + RECOVERY_STEP).min(INITIAL_MAX));
                self.notify.notify_one();
            }
            return;
        }

        // Phase 3: sustained healthy -> return to unlimited.
        if let Some(since) = st.healthy_since {
            if now.duration_since(since) >= UNLIMITED_AFTER_HEALTHY {
                st.permits_max = None;
                st.permits_current = 0;
                st.backoff = Duration::ZERO;
                self.notify.notify_waiters();
                return;
            }
        }

        // Phase 2: multiplicative growth while healthy (no ceiling).
        let next_max = (permits_max as f64 * HEALTHY_RECOVERY_MULTIPLIER).ceil() as u32;
        if next_max > permits_max {
            st.permits_max = Some(next_max);
            self.notify.notify_one();
        }
    }
}

/// Reduce the permit cap by `factor` (never below the floor). Caller must hold the lock.
fn scale_permits(st: &mut Inner, factor: f64) {
    if let Some(max) = st.permits_max {
        let next = ((max as f64 * factor).ceil() as u32).max(FLOOR);
        if next < max {
            st.permits_max = Some(next);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_parsing_is_case_insensitive() {
        assert_eq!(
            "balanced".parse::<BackpressureProfile>().unwrap(),
            BackpressureProfile::Balanced
        );
        assert_eq!(
            "LEGACY".parse::<BackpressureProfile>().unwrap(),
            BackpressureProfile::Legacy
        );
        assert!("bogus".parse::<BackpressureProfile>().is_err());
    }

    #[test]
    fn detects_backpressure_responses() {
        assert!(is_backpressure_response(429, None));
        assert!(is_backpressure_response(503, None));
        assert!(is_backpressure_response(
            500,
            Some("RESOURCE_EXHAUSTED: ...")
        ));
        assert!(is_backpressure_response(400, Some("RESOURCE_EXHAUSTED")));
        assert!(!is_backpressure_response(500, Some("other error")));
        assert!(!is_backpressure_response(404, None));
        assert!(!is_backpressure_response(200, None));
    }

    #[test]
    fn is_backpressure_error_classifies_api_errors() {
        assert!(is_backpressure_error(&CamundaError::Api {
            status: 429,
            body: None
        }));
        assert!(!is_backpressure_error(&CamundaError::Api {
            status: 404,
            body: None
        }));
        assert!(!is_backpressure_error(&CamundaError::Validation(
            "x".into()
        )));
    }

    #[test]
    fn starts_unlimited() {
        let bp = BackpressureManager::new(BackpressureProfile::Balanced);
        let s = bp.state();
        assert_eq!(s.permits_max, None);
        assert_eq!(s.severity, BackpressureSeverity::Healthy);
    }

    #[test]
    fn boots_and_shrinks_on_backpressure() {
        let bp = BackpressureManager::new(BackpressureProfile::Balanced);
        bp.record_backpressure();
        let s = bp.state();
        // First signal: boots to INITIAL_MAX then shrinks by the soft factor.
        assert_eq!(s.severity, BackpressureSeverity::Soft);
        assert_eq!(
            s.permits_max,
            Some((INITIAL_MAX as f64 * SOFT_FACTOR).ceil() as u32)
        );
    }

    #[test]
    fn escalates_to_severe_after_threshold() {
        let bp = BackpressureManager::new(BackpressureProfile::Balanced);
        for _ in 0..SEVERE_THRESHOLD {
            bp.record_backpressure();
        }
        assert_eq!(bp.severity(), BackpressureSeverity::Severe);
    }

    #[test]
    fn legacy_profile_never_gates() {
        let bp = BackpressureManager::new(BackpressureProfile::Legacy);
        for _ in 0..10 {
            bp.record_backpressure();
        }
        // Severity is still tracked, but permits stay unlimited (observe-only).
        assert_eq!(bp.state().permits_max, None);
    }

    #[tokio::test]
    async fn acquire_release_round_trips_when_capped() {
        let bp = BackpressureManager::new(BackpressureProfile::Balanced);
        // Force a cap.
        bp.record_backpressure();
        // Acquire and release should not deadlock.
        bp.acquire().await.unwrap();
        let held = bp.state().permits_current;
        assert_eq!(held, 1);
        bp.release();
        assert_eq!(bp.state().permits_current, 0);
    }

    #[tokio::test]
    async fn legacy_acquire_is_noop() {
        let bp = BackpressureManager::new(BackpressureProfile::Legacy);
        bp.acquire().await.unwrap();
        assert_eq!(bp.state().permits_current, 0);
    }
}
