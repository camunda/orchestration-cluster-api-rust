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
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::Notify;
use tokio::time::Instant;

use super::clock::Clock;

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
    /// Monotonic counter identifying the current "cap era". It is bumped on every
    /// transition that resets `permits_current` — unlimited → capped (first backpressure
    /// signal) and capped → unlimited (full recovery). A counted permit belongs to the
    /// era it was acquired in; releasing it in a later era would decrement a different
    /// era's count and let concurrency exceed the cap, so the guard only releases when
    /// its generation still matches.
    generation: u64,
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
    clock: Arc<dyn Clock>,
}

impl BackpressureManager {
    /// Create a manager for the given profile, resolving its cadence through `clock`.
    pub fn new(profile: BackpressureProfile, clock: Arc<dyn Clock>) -> Self {
        BackpressureManager {
            observe_only: profile == BackpressureProfile::Legacy,
            inner: Mutex::new(Inner {
                severity: BackpressureSeverity::Healthy,
                consecutive: 0,
                last_event_at: None,
                permits_current: 0,
                // Start unlimited — only boot to INITIAL_MAX on the first backpressure signal.
                permits_max: None,
                generation: 0,
                last_recover_check: None,
                healthy_since: None,
                waiters: 0,
                backoff: Duration::ZERO,
            }),
            notify: Notify::new(),
            clock,
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
        self.acquire_inner().await.map(|_| ())
    }

    /// Acquire a permit, returning the [`Acquisition`] record describing whether a permit
    /// was actually counted against the cap and which cap era (`generation`) it belongs to.
    async fn acquire_inner(&self) -> Result<Acquisition, CamundaError> {
        if self.observe_only {
            return Ok(Acquisition::uncounted(0));
        }

        loop {
            // Backoff-at-floor: rate-limit at the concurrency floor. Sleep outside the lock.
            let backoff = {
                let st = self.inner.lock().unwrap();
                if st.permits_max.is_none() {
                    return Ok(Acquisition::uncounted(st.generation)); // unlimited fast path
                }
                st.backoff
            };
            if !backoff.is_zero() {
                self.clock.sleep(backoff).await;
            }

            // Register interest *before* re-checking, so a concurrent release cannot be lost.
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            {
                let mut st = self.inner.lock().unwrap();
                match st.permits_max {
                    // went unlimited during the backoff sleep
                    None => return Ok(Acquisition::uncounted(st.generation)),
                    Some(max) => {
                        if st.permits_current < max {
                            st.permits_current += 1;
                            return Ok(Acquisition::counted(st.generation));
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
    ///
    /// This is the bare, generation-unaware counterpart to [`acquire`](Self::acquire):
    /// it unconditionally decrements the current count. Permits held across an `.await`
    /// must use `acquire_guard` instead, whose guard releases only
    /// the permit it actually acquired, in the cap era it acquired it.
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

    /// Release a specific [`Acquisition`], decrementing the count only if that acquisition
    /// actually counted a permit **and** we are still in the same cap era it counted in.
    /// A cap reset (unlimited ↔ capped) bumps the generation and zeroes the counter, so an
    /// acquisition from a stale era must not decrement the live count.
    fn release_acquisition(&self, acq: Acquisition) {
        if self.observe_only || !acq.counted {
            return;
        }
        let mut st = self.inner.lock().unwrap();
        if st.generation != acq.generation {
            return; // stale era — this permit's count no longer exists.
        }
        if st.permits_current > 0 {
            st.permits_current -= 1;
        }
        drop(st);
        self.notify.notify_one();
    }

    /// Acquire a permit, returning an RAII [`PermitGuard`] that releases it when
    /// dropped — including if the holding future is **cancelled** (dropped before it
    /// completes, e.g. under `tokio::time::timeout` or task abortion).
    ///
    /// Prefer this over the bare [`acquire`](Self::acquire) / [`release`](Self::release)
    /// pair for any permit held across an `.await`: a cancellation between the two calls
    /// leaks the permit, and once the manager is capped a leaked permit is counted against
    /// every clone of the client forever, eventually stalling all guarded operations at the
    /// one-permit floor. The guard also only releases the permit it actually acquired, so
    /// an acquisition made while unlimited never decrements a counted permit after the
    /// manager later caps.
    pub(crate) async fn acquire_guard(self: &Arc<Self>) -> Result<PermitGuard, CamundaError> {
        let acq = self.acquire_inner().await?;
        Ok(PermitGuard {
            mgr: Arc::clone(self),
            acq,
        })
    }

    /// Test-only: mimic the Phase-3 recovery transition (capped → unlimited) that
    /// `maybe_recover` performs on sustained health, without having to drive the
    /// time-based recovery loop. Zeroes the count and bumps the cap generation.
    #[cfg(test)]
    fn force_cap_reset_to_unlimited(&self) {
        let mut st = self.inner.lock().unwrap();
        st.generation = st.generation.wrapping_add(1);
        st.permits_max = None;
        st.permits_current = 0;
        st.backoff = Duration::ZERO;
    }

    /// Test-only: pin the gate to a fixed, deterministic capacity (`max` concurrent
    /// permits, none outstanding) so a test can saturate it predictably. Bumps the cap
    /// generation to start a fresh era, matching a real unlimited → capped transition.
    #[cfg(test)]
    pub(crate) fn test_force_cap(&self, max: u32) {
        let mut st = self.inner.lock().unwrap();
        st.generation = st.generation.wrapping_add(1);
        st.permits_max = Some(max);
        st.permits_current = 0;
        st.backoff = Duration::ZERO;
    }

    /// Record a backpressure signal from the server.
    pub fn record_backpressure(&self) {
        let now = self.clock.now();
        let mut st = self.inner.lock().unwrap();
        st.last_event_at = Some(now);
        st.consecutive += 1;
        st.healthy_since = None;

        if !self.observe_only && st.permits_max.is_none() {
            // Boot to the initial cap on the first signal (was unlimited). This starts a
            // new cap era: bump the generation so permits acquired while unlimited are not
            // released against this era's count.
            st.generation = st.generation.wrapping_add(1);
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
        let now = self.clock.now();
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
                // Ending the cap era: bump the generation so any still-counted permit from
                // this era is not released against a future era's count.
                st.generation = st.generation.wrapping_add(1);
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

/// Records the outcome of an acquisition: whether a permit was actually counted against
/// the cap, and which cap era (`generation`) it belongs to. Releasing is a no-op unless a
/// permit was counted and the manager is still in the same era.
#[derive(Clone, Copy)]
struct Acquisition {
    counted: bool,
    generation: u64,
}

impl Acquisition {
    fn counted(generation: u64) -> Self {
        Acquisition {
            counted: true,
            generation,
        }
    }

    fn uncounted(generation: u64) -> Self {
        Acquisition {
            counted: false,
            generation,
        }
    }
}

/// RAII guard returned by [`BackpressureManager::acquire_guard`]. Releases the held
/// permit exactly once when dropped, so a cancelled operation cannot leak it. The release
/// is a no-op unless the acquisition actually counted a permit in the still-current cap
/// era, so a permit taken while unlimited never decrements a counted permit after a later
/// cap, and a permit counted before a full recovery never decrements a new era's count.
#[must_use = "the permit is released as soon as the guard is dropped"]
pub(crate) struct PermitGuard {
    mgr: Arc<BackpressureManager>,
    acq: Acquisition,
}

impl Drop for PermitGuard {
    fn drop(&mut self) {
        self.mgr.release_acquisition(self.acq);
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
        let bp = BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        );
        let s = bp.state();
        assert_eq!(s.permits_max, None);
        assert_eq!(s.severity, BackpressureSeverity::Healthy);
    }

    #[test]
    fn boots_and_shrinks_on_backpressure() {
        let bp = BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        );
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
        let bp = BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        );
        for _ in 0..SEVERE_THRESHOLD {
            bp.record_backpressure();
        }
        assert_eq!(bp.severity(), BackpressureSeverity::Severe);
    }

    #[test]
    fn legacy_profile_never_gates() {
        let bp = BackpressureManager::new(
            BackpressureProfile::Legacy,
            super::super::clock::live_clock(),
        );
        for _ in 0..10 {
            bp.record_backpressure();
        }
        // Severity is still tracked, but permits stay unlimited (observe-only).
        assert_eq!(bp.state().permits_max, None);
    }

    #[tokio::test]
    async fn acquire_release_round_trips_when_capped() {
        let bp = BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        );
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
        let bp = BackpressureManager::new(
            BackpressureProfile::Legacy,
            super::super::clock::live_clock(),
        );
        bp.acquire().await.unwrap();
        assert_eq!(bp.state().permits_current, 0);
    }

    #[tokio::test]
    async fn acquire_guard_releases_on_scope_exit() {
        let bp = Arc::new(BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        ));
        bp.record_backpressure(); // force a cap
        {
            let _permit = bp.acquire_guard().await.unwrap();
            assert_eq!(bp.state().permits_current, 1);
        }
        assert_eq!(
            bp.state().permits_current,
            0,
            "dropping the guard must release the permit"
        );
    }

    /// A permit acquired while **unlimited** must never decrement a counted permit after
    /// the manager later caps. This is the exact "releases permits it did not acquire"
    /// class: the unlimited-era guard counted nothing, so dropping it after a cap must not
    /// steal another request's counted permit and let concurrency exceed the cap.
    #[tokio::test]
    async fn guard_from_unlimited_era_does_not_release_a_counted_permit() {
        let bp = Arc::new(BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        ));
        // Starts unlimited — this acquisition counts nothing.
        assert_eq!(bp.state().permits_max, None);
        let unlimited_guard = bp.acquire_guard().await.unwrap();
        assert_eq!(bp.state().permits_current, 0);

        // Backpressure now caps the manager (new cap era).
        bp.record_backpressure();
        assert!(bp.state().permits_max.is_some());

        // A real request counts a permit in the new era.
        let counted_guard = bp.acquire_guard().await.unwrap();
        assert_eq!(bp.state().permits_current, 1);

        // Dropping the unlimited-era guard must NOT touch the counted permit.
        drop(unlimited_guard);
        assert_eq!(
            bp.state().permits_current,
            1,
            "a guard that counted nothing while unlimited must not release a counted permit"
        );

        // The genuinely counted guard releases its own permit.
        drop(counted_guard);
        assert_eq!(bp.state().permits_current, 0);
    }

    /// A permit counted in one cap era must not be released against a *different* era's
    /// count after a full recovery to unlimited (which zeroes the counter) and a fresh cap.
    #[tokio::test]
    async fn guard_does_not_release_across_a_cap_era_reset() {
        let bp = Arc::new(BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        ));
        bp.record_backpressure(); // era 1: capped
        let stale_guard = bp.acquire_guard().await.unwrap();
        assert_eq!(bp.state().permits_current, 1);

        // Simulate a full recovery to unlimited followed by a fresh cap.
        // `force_cap_reset_to_unlimited` mirrors what maybe_recover's Phase-3 transition
        // does: zero the count and bump the generation, then re-cap.
        bp.force_cap_reset_to_unlimited();
        assert_eq!(bp.state().permits_max, None);
        assert_eq!(bp.state().permits_current, 0);
        bp.record_backpressure(); // era 2: capped again

        // A new request counts a permit in era 2.
        let fresh_guard = bp.acquire_guard().await.unwrap();
        assert_eq!(bp.state().permits_current, 1);

        // Dropping the era-1 guard must not decrement era 2's count.
        drop(stale_guard);
        assert_eq!(
            bp.state().permits_current,
            1,
            "a stale-era guard must not release a later era's counted permit"
        );
        drop(fresh_guard);
        assert_eq!(bp.state().permits_current, 0);
    }

    /// Cancelling an operation that holds a permit (its future dropped before it
    /// completes) must release the permit via the guard's `Drop`. Without the RAII
    /// guard the permit leaks, and once the manager is capped the leak is permanent,
    /// eventually stalling every guarded operation at the one-permit floor.
    #[tokio::test]
    async fn cancelling_a_guarded_operation_releases_the_permit() {
        use std::future::{pending, Future};
        use std::task::{Context, Poll, Waker};

        let bp = Arc::new(BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        ));
        bp.record_backpressure(); // force a cap so leaks are observable

        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        // A guarded operation that acquires the permit, then parks forever. `Box::pin`
        // so we *own* the future and dropping it really drops its captured guard (a
        // `Pin<&mut _>` from `pin!` would only drop the reference, not the future).
        let mut fut = Box::pin(async {
            let _permit = bp.acquire_guard().await.unwrap();
            pending::<()>().await;
        });

        // Drive it once: it acquires the permit and then parks (Pending).
        assert!(matches!(fut.as_mut().poll(&mut cx), Poll::Pending));
        assert_eq!(
            bp.state().permits_current,
            1,
            "permit should be held while the operation is in flight"
        );

        // Cancel the operation: dropping its future must release the permit.
        drop(fut);
        assert_eq!(
            bp.state().permits_current,
            0,
            "cancelling a guarded operation must release the backpressure permit"
        );
    }

    /// The gate's backoff must be spent on the injected clock, not on ambient
    /// time. Without this the seam exists but `acquire` still blocks the whole
    /// runtime for real seconds under test — the defect the JS and C# SDKs
    /// shipped with.
    #[tokio::test(start_paused = true)]
    async fn the_acquire_gate_waits_on_the_injected_clock() {
        let clock = std::sync::Arc::new(super::super::clock::RecordingClock::default());
        let mgr = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());

        // Backoff only engages once the gate is severe *and* pinned at the permit
        // floor, so hammer it until it gets there — one signal is not enough.
        for _ in 0..64 {
            mgr.record_backpressure();
        }
        mgr.acquire().await.unwrap();
        mgr.release();

        assert!(
            !clock.sleeps().is_empty(),
            "acquire backed off without going through the injected clock"
        );
        assert!(
            clock.now_calls() > 0,
            "the recovery window was measured against ambient time"
        );
    }
}
