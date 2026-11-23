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
/// Minimum interval between two `soft` multiplicative shrinks. Like classic AIMD (one
/// decrease per window), a sparse trickle of signals interleaved with successes shrinks
/// the cap at most once per interval instead of once per signal. `severe` signals
/// (consecutive, with no success in between) are not rate-limited.
const SOFT_DECREASE_INTERVAL: Duration = Duration::from_secs(1);
/// Backpressure ratio (signals / outcomes over the last recovery window) below which the
/// cluster counts as recovering even if the window was not completely quiet.
const LOW_BACKPRESSURE_RATIO: f64 = 0.10;
/// Minimum outcomes in a recovery window before its backpressure ratio is trusted.
const MIN_RATIO_SAMPLES: u32 = 10;
/// Maximum number of queued waiters before [`acquire`](BackpressureManager::acquire) fails fast.
const MAX_WAITERS: u32 = 1000;
/// Multiplicative growth factor applied during phase-2 (healthy) recovery.
const HEALTHY_RECOVERY_MULTIPLIER: f64 = 1.5;
/// Sustained-healthy duration after which the manager returns to unlimited.
const UNLIMITED_AFTER_HEALTHY: Duration = Duration::from_secs(30);
/// Idle gap (no outcome of any kind) after which a carried-over, undersampled recovery
/// window is discarded. Longer than any plausible low-rate request cadence, so sparse
/// interleaved pressure keeps accumulating, while a burst followed by real idleness does
/// not taint the next episode's ratio.
const WINDOW_IDLE_RESET: Duration = Duration::from_secs(10);

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
    /// Consecutive backpressure signals since the last successful response.
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
    /// When the last `soft` multiplicative shrink was applied.
    last_soft_decrease_at: Option<Instant>,
    /// Backpressure signals observed in the current recovery window. Only counted
    /// while capped and reset on every cap-era transition, so the ratio reflects the
    /// current capped period only.
    window_backpressure: u32,
    /// Successful outcomes observed in the current recovery window. Only counted
    /// while capped and reset on every cap-era transition, so the ratio reflects the
    /// current capped period only.
    window_healthy: u32,
    /// When the last outcome (signal or success) was recorded; drives the idle reset of
    /// a carried-over recovery window.
    last_outcome_at: Option<Instant>,
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
                last_soft_decrease_at: None,
                window_backpressure: 0,
                window_healthy: 0,
                last_outcome_at: None,
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
        st.window_backpressure = 0;
        st.window_healthy = 0;
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
        st.window_backpressure = 0;
        st.window_healthy = 0;
    }

    /// Record a backpressure signal from the server.
    pub fn record_backpressure(&self) {
        let now = self.clock.now();
        let mut st = self.inner.lock().unwrap();
        // A signal after a long silence while unlimited is a fresh episode, not part
        // of a trickle: expire the stale sustained-health clock so it boots a cap.
        if st.permits_max.is_none()
            && st.healthy_since.is_some()
            && st
                .last_event_at
                .is_none_or(|t| now.duration_since(t) >= UNLIMITED_AFTER_HEALTHY)
        {
            st.healthy_since = None;
        }
        begin_outcome(&mut st, now);
        st.last_event_at = Some(now);
        st.consecutive += 1;
        st.window_backpressure = st.window_backpressure.saturating_add(1);
        self.judge_unlimited_window(&mut st, now);

        // A signal interrupts the sustained-health stretch only when it is real
        // pressure — two or more signals back to back. A lone trickle signal keeps the
        // clock: clearing it would let a 5% trickle starve the phase-3 transition
        // forever, as every signal would reset the stretch 200ms after it restarted.
        // The exemption applies to the *lone* signal only, and it applies while
        // unlimited too: after phase 3 returns to unlimited the clock is left set, so
        // without this a fresh burst of consecutive 503s there would keep severity
        // Severe while the boot below is suppressed by the still-running clock —
        // leaving the gate unlimited under sustained pressure. `maybe_recover` still
        // judges the window by its ratio and restarts the clock on the next quiet or
        // low-ratio window.
        if st.consecutive > 1 {
            st.healthy_since = None;
        }

        if !self.observe_only && st.permits_max.is_none() && st.healthy_since.is_none() {
            // Boot to the initial cap on the first signal (was unlimited). This starts a
            // new cap era: bump the generation so permits acquired while unlimited are not
            // released against this era's count, and start a fresh recovery window so the
            // ratio reflects only outcomes observed under this cap. A signal that arrives
            // while the sustained-health clock is running does not boot a cap at all: the
            // cluster has been coping for 30s+, so a lone trickle signal is noise, and
            // capping here would oscillate unlimited -> boot -> 30s wait -> unlimited
            // forever instead of settling on the stable fixed point.
            st.generation = st.generation.wrapping_add(1);
            st.permits_max = Some(INITIAL_MAX);
            st.permits_current = st.permits_current.min(INITIAL_MAX);
            st.window_backpressure = 0;
            st.window_healthy = 0;
        }

        if st.consecutive >= SEVERE_THRESHOLD {
            st.severity = BackpressureSeverity::Severe;
            if !self.observe_only {
                scale_permits(&mut st, SEVERE_FACTOR);
            }
        } else if st.severity == BackpressureSeverity::Healthy {
            // A signal that arrives while the sustained-health clock is running is part
            // of a trickle, not a fresh episode: `maybe_recover` judges it by the window
            // ratio, so it must not knock severity back to soft — otherwise the healthy
            // clock could never accumulate the 30s needed to return to unlimited.
            if st.healthy_since.is_none() {
                st.severity = BackpressureSeverity::Soft;
            }
        }

        // Soft shrink: at most once per SOFT_DECREASE_INTERVAL, so a sparse trickle of
        // signals between successes cannot ratchet the cap down to the floor. A trickle
        // signal while the sustained-health clock is running does not shrink at all:
        // the window ratio already judged the cluster to be coping, and a per-signal
        // shrink would undo the phase-2 growth between windows, pinning the gate near
        // the floor while the phase-3 transition to unlimited fires. A severe signal
        // (consecutive >= SEVERE_THRESHOLD) is also excluded: it already applied
        // SEVERE_FACTOR above, and severe signals shrink on every signal — applying the
        // soft factor too would shrink the same signal twice whenever the soft-decrease
        // interval happened to elapse.
        if !self.observe_only
            && st.severity != BackpressureSeverity::Healthy
            && st.consecutive < SEVERE_THRESHOLD
            && st.healthy_since.is_none()
            && st
                .last_soft_decrease_at
                .is_none_or(|t| now.duration_since(t) >= SOFT_DECREASE_INTERVAL)
        {
            scale_permits(&mut st, SOFT_FACTOR);
            st.last_soft_decrease_at = Some(now);
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
        begin_outcome(&mut st, now);
        // Reset backoff immediately on success — the server clearly has capacity.
        st.backoff = Duration::ZERO;
        // A success breaks the run: `consecutive` counts consecutive signals only.
        st.consecutive = 0;
        // The recovery window measures the current capped period only, so a success
        // recorded while unlimited is not counted — otherwise a long healthy run
        // before the first signal would dilute the first capped window's ratio.
        // While unlimited with the sustained-health clock still running, successes
        // are counted too, so `judge_unlimited_window` can tell a tolerated trickle
        // from sustained interleaved pressure.
        if st.permits_max.is_some() || st.healthy_since.is_some() {
            st.window_healthy = st.window_healthy.saturating_add(1);
        }
        self.judge_unlimited_window(&mut st, now);
        self.maybe_recover(&mut st, now);
    }

    /// While unlimited, the sustained-health clock left set by phase 3 makes lone
    /// signals tolerated. Judge that tolerance once per recovery interval by the
    /// window ratio: a well-sampled window that is not low-ratio (e.g. alternating
    /// 503s and successes, never two in a row) is sustained pressure, so the clock is
    /// cleared and the next signal boots a cap. Caller must hold the lock.
    fn judge_unlimited_window(&self, st: &mut Inner, now: Instant) {
        if self.observe_only || st.permits_max.is_some() || st.healthy_since.is_none() {
            return;
        }
        if let Some(last) = st.last_recover_check {
            if now.duration_since(last) < RECOVERY_INTERVAL {
                return;
            }
        }
        st.last_recover_check = Some(now);
        let bp = st.window_backpressure;
        let total = bp.saturating_add(st.window_healthy);
        let sampled = total >= MIN_RATIO_SAMPLES;
        // An undersampled window is carried over, not discarded: at a low request rate
        // (e.g. eight alternating outcomes per second) every single interval falls
        // short of MIN_RATIO_SAMPLES, and discarding them would hide sustained 50%
        // pressure forever. A quiet check (no signal for DECAY_QUIET) does not drop it
        // either — at one outcome every few seconds every check is quiet — only a real
        // idle gap does (see begin_outcome).
        if sampled {
            st.window_backpressure = 0;
            st.window_healthy = 0;
        }
        let low_ratio = sampled && (bp as f64) < (total as f64) * LOW_BACKPRESSURE_RATIO;
        if sampled && !low_ratio {
            st.healthy_since = None;
        }
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

        // Backpressure ratio over the window since the previous recovery check.
        // An undersampled window is carried over into the next check (see
        // judge_unlimited_window) so a low-rate high-ratio stream still accumulates
        // enough samples to be judged; only an idle gap with no outcomes at all drops
        // it (see begin_outcome).
        let bp = st.window_backpressure;
        let total = bp.saturating_add(st.window_healthy);
        let sampled = total >= MIN_RATIO_SAMPLES;
        if sampled {
            st.window_backpressure = 0;
            st.window_healthy = 0;
        }
        let low_ratio = sampled && (bp as f64) < (total as f64) * LOW_BACKPRESSURE_RATIO;

        // A well-sampled window whose ratio is NOT low is real, sustained pressure —
        // e.g. alternating 503s and successes, where no two signals are consecutive so
        // record_backpressure never clears the clock and severity never escalates.
        // Invalidate the sustained-health stretch so phase 2 stops growing the cap
        // toward unlimited while the cluster is still failing half its requests, and
        // re-raise severity so throttling resumes. A quiet window (too few samples to
        // trust the ratio) is not pressure and never lands here.
        let worsening = sampled && !low_ratio;
        if worsening {
            st.healthy_since = None;
            if st.severity == BackpressureSeverity::Healthy {
                st.severity = BackpressureSeverity::Soft;
            }
        }

        // Decay severity (severe -> soft -> healthy) if the cluster has been quiet, or if
        // only a small fraction of recent requests hit backpressure — a steady trickle of
        // signals must not pin the gate in `severe` forever. A worsening window never
        // decays, even when it also happens to be quiet: the sampled high ratio is the
        // fresher evidence, and decaying on the quiet clause would undo the escalation
        // above in the same check.
        // The quiet clause only applies while no unjudged signal is carried over: a
        // low-rate stream (one outcome every few seconds) is "quiet" at every check,
        // and decaying on it would walk a half-failing stream back to unlimited before
        // its window ever gathered enough samples to be judged.
        let quiet = is_quiet(st, now) && st.window_backpressure == 0;
        if (quiet || low_ratio) && !worsening {
            match st.severity {
                BackpressureSeverity::Severe => st.severity = BackpressureSeverity::Soft,
                BackpressureSeverity::Soft => st.severity = BackpressureSeverity::Healthy,
                BackpressureSeverity::Healthy => {}
            }
            if st.severity == BackpressureSeverity::Healthy {
                st.consecutive = 0;
                // A window that decays severity to healthy also starts the
                // sustained-health clock: the window just proved the cluster is coping,
                // and the trickle signal that follows would otherwise leave the clock
                // unset until the next window — resetting the stretch every second.
                // This holds for a quiet decay too: with too few samples to trust the
                // ratio (e.g. one 503 + one success per second), `low_ratio` stays
                // false, and without seeding here the clock would never start — phase 3
                // could never return to unlimited even though severity reached Healthy.
                // An already-running clock keeps its timestamp: a lone signal does not
                // interrupt the stretch (see record_backpressure), so re-seeding would
                // wrongly extend it.
                if st.healthy_since.is_none() {
                    st.healthy_since = Some(now);
                }
            }
        }
        let permits_max = st.permits_max.unwrap_or(FLOOR);

        // While the carried window still holds an unjudged backpressure signal, a
        // Healthy gate must not grow or go unlimited: the pending signals may be the
        // start of sustained interleaved pressure that the window has not yet gathered
        // enough samples to judge. Growth resumes once the window is judged.
        if st.severity == BackpressureSeverity::Healthy && st.window_backpressure > 0 {
            return;
        }
        if st.severity == BackpressureSeverity::Healthy {
            // Phase 3: sustained healthy -> return to unlimited. This runs before the
            // growth below so a window that crosses the 30s mark ends the cap era
            // instead of growing the cap once more.
            if now.duration_since(st.healthy_since.unwrap_or(now)) >= UNLIMITED_AFTER_HEALTHY {
                // Ending the cap era: bump the generation so any still-counted permit
                // from this era is not released against a future era's count, and reset
                // the recovery window so the next capped period starts from a clean
                // slate.
                st.generation = st.generation.wrapping_add(1);
                st.permits_max = None;
                st.permits_current = 0;
                st.backoff = Duration::ZERO;
                st.window_backpressure = 0;
                st.window_healthy = 0;
                self.notify.notify_waiters();
                return;
            }
            // Phase 2: multiplicative growth while healthy (no ceiling).
            let next_max = (permits_max as f64 * HEALTHY_RECOVERY_MULTIPLIER).ceil() as u32;
            if next_max > permits_max {
                st.permits_max = Some(next_max);
                self.notify.notify_one();
            }
            return;
        }

        if low_ratio {
            // Mostly-successful window: decaying toward healthy; grow multiplicatively,
            // past the bootstrap cap.
            let next_max = (permits_max as f64 * HEALTHY_RECOVERY_MULTIPLIER).ceil() as u32;
            st.permits_max = Some(next_max.max(permits_max + RECOVERY_STEP));
            self.notify.notify_one();
            return;
        }
        // Phase 1: additive recovery while not yet healthy (bounded by the bootstrap cap).
        if permits_max < INITIAL_MAX {
            st.permits_max = Some((permits_max + RECOVERY_STEP).min(INITIAL_MAX));
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
/// Start recording an outcome: drop a carried-over recovery window if no outcome at all
/// was seen for [`WINDOW_IDLE_RESET`], then stamp the outcome time. Caller holds the lock.
fn begin_outcome(st: &mut Inner, now: Instant) {
    if st
        .last_outcome_at
        .is_some_and(|t| now.duration_since(t) > WINDOW_IDLE_RESET)
    {
        // Discarding unjudged backpressure must not leave a sustained-health stretch
        // standing on evidence that was never judged: otherwise a stream slower than
        // the idle reset (e.g. alternating 503/success every 12s) would keep the
        // unlimited gate's tolerance alive forever.
        if st.window_backpressure > 0 {
            st.healthy_since = None;
        }
        st.window_backpressure = 0;
        st.window_healthy = 0;
    }
    st.last_outcome_at = Some(now);
}

/// No backpressure signal for longer than [`DECAY_QUIET`].
fn is_quiet(st: &Inner, now: Instant) -> bool {
    st.last_event_at
        .is_none_or(|t| now.duration_since(t) > DECAY_QUIET)
}

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

    #[test]
    fn success_resets_consecutive_so_sparse_signals_stay_soft() {
        let bp = BackpressureManager::new(
            BackpressureProfile::Balanced,
            super::super::clock::live_clock(),
        );
        for _ in 0..10 {
            bp.record_backpressure();
            bp.record_healthy_hint();
        }
        assert_eq!(bp.state().consecutive, 0);
        assert_ne!(bp.severity(), BackpressureSeverity::Severe);
    }

    /// Regression for #203: a steady sender where 1 in 20 requests hits backpressure
    /// must recover to (and beyond) the bootstrap cap, not stay pinned near the floor —
    /// and must keep going all the way back to unlimited: a trickle that decays severity
    /// but leaves the gate capped at an ever-growing, meaningless limit is not recovery.
    #[test]
    fn steady_trickle_of_backpressure_recovers() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        // Drive it into severe first, as a burst would.
        for _ in 0..8 {
            bp.record_backpressure();
        }
        assert_eq!(bp.severity(), BackpressureSeverity::Severe);
        // 50 req/s for 300 s; every 20th is a 503.
        for i in 1..=15000u32 {
            clock.advance(Duration::from_millis(20));
            if i % 20 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        let s = bp.state();
        assert_ne!(s.severity, BackpressureSeverity::Severe, "{s:?}");
        assert_eq!(
            s.permits_max, None,
            "a sustained low-ratio trickle must recover all the way to unlimited: {s:?}"
        );
    }

    /// The recovery window measures the current capped period only: successes recorded
    /// while the gate is unlimited must not dilute the first capped window's ratio, or a
    /// burst that caps the gate after a long healthy run would decay Severe on its very
    /// first recovery check (8/1,009 looking "low" when the capped period saw 8/9).
    #[test]
    fn unlimited_successes_do_not_dilute_the_first_capped_window() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        // 1,000 successful requests while unlimited.
        for _ in 0..1000 {
            bp.record_healthy_hint();
        }
        assert_eq!(bp.state().permits_max, None);
        // A burst of backpressure caps the gate and escalates to severe.
        for _ in 0..8 {
            bp.record_backpressure();
        }
        assert_eq!(bp.severity(), BackpressureSeverity::Severe);
        // One success, then let the first recovery window elapse. The window must
        // contain only the capped outcomes (8 signals / 9 outcomes), so severity must
        // NOT decay on a diluted 8/1,009 ratio.
        bp.record_healthy_hint();
        clock.advance(RECOVERY_INTERVAL + Duration::from_millis(1));
        bp.record_healthy_hint();
        assert_eq!(
            bp.severity(),
            BackpressureSeverity::Severe,
            "unlimited-era successes leaked into the capped recovery window"
        );
    }

    /// A sustained low-ratio trickle while already `Healthy` must converge to the same
    /// end state as a fully quiet window — unlimited — instead of growing the cap
    /// multiplicatively forever while phase 3 never fires.
    #[test]
    fn low_ratio_trickle_while_healthy_returns_to_unlimited() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        // Cap the gate, then keep a 5% trickle of signals for well past
        // UNLIMITED_AFTER_HEALTHY: the low-ratio windows must decay severity and then
        // keep the sustained-health clock running until the gate returns to unlimited.
        bp.record_backpressure();
        for i in 1..=3000u32 {
            clock.advance(Duration::from_millis(20));
            if i % 20 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        let s = bp.state();
        assert_eq!(s.severity, BackpressureSeverity::Healthy, "{s:?}");
        assert_eq!(
            s.permits_max, None,
            "a low-ratio trickle must not pin the gate at an ever-growing cap: {s:?}"
        );
    }

    /// Recover to unlimited, then sustain interleaved 50% pressure (never two 503s in
    /// a row): the window judge must clear the stale clock and re-cap the gate.
    #[test]
    fn interleaved_pressure_while_unlimited_recaps() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        bp.record_backpressure();
        for i in 1..=3000u32 {
            clock.advance(Duration::from_millis(20));
            if i % 20 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        assert_eq!(bp.state().permits_max, None, "must be unlimited first");
        let mut capped = false;
        // Start with a success: the warmup ended with a 503, so starting with another
        // would re-cap via the consecutive-signal path, not the window ratio.
        for i in 0..1000u32 {
            clock.advance(Duration::from_millis(10));
            if i % 2 == 1 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
            capped |= bp.state().permits_max.is_some();
        }
        assert!(
            capped,
            "interleaved 50% pressure must re-cap: {:?}",
            bp.state()
        );
    }

    /// Recover to unlimited (ending on a success), then sustain 50% pressure at a low
    /// rate (eight alternating outcomes per second, so every interval is below
    /// MIN_RATIO_SAMPLES): the undersampled windows must accumulate and re-cap.
    #[test]
    fn low_rate_interleaved_pressure_while_unlimited_recaps() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        bp.record_backpressure();
        for i in 1..=3001u32 {
            clock.advance(Duration::from_millis(20));
            if i % 20 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        assert_eq!(bp.state().permits_max, None, "must be unlimited first");
        let mut capped = false;
        for i in 0..400u32 {
            clock.advance(Duration::from_millis(125));
            if i % 2 == 1 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
            capped |= bp.state().permits_max.is_some();
        }
        assert!(
            capped,
            "low-rate 50% pressure must re-cap: {:?}",
            bp.state()
        );
    }

    /// While capped, low-rate 50% pressure (eight alternating outcomes per second)
    /// must never reach unlimited: undersampled windows accumulate into a judged one.
    #[test]
    fn low_rate_interleaved_pressure_while_capped_stays_capped() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        bp.record_backpressure();
        for i in 0..800u32 {
            clock.advance(Duration::from_millis(125));
            if i % 2 == 1 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
            assert!(
                bp.state().permits_max.is_some(),
                "low-rate 50% pressure reached unlimited at step {i}: {:?}",
                bp.state()
            );
        }
    }

    /// A lone signal long after the gate returned to unlimited is a fresh episode:
    /// the stale sustained-health clock must not suppress the boot.
    #[test]
    fn lone_signal_long_after_unlimited_boots_cap() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        bp.record_backpressure();
        for i in 1..=3000u32 {
            clock.advance(Duration::from_millis(20));
            if i % 20 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        assert_eq!(bp.state().permits_max, None, "must be unlimited first");
        clock.advance(Duration::from_secs(3600));
        bp.record_backpressure();
        assert!(bp.state().permits_max.is_some(), "{:?}", bp.state());
    }

    /// After phase 3 returns to unlimited the sustained-health clock is left set. A
    /// fresh burst of *consecutive* 503s there is real pressure, not a lone trickle
    /// signal: it must clear the clock so the boot caps the gate again — otherwise
    /// severity goes Severe while the gate stays unlimited (the boot is suppressed by
    /// the still-running clock and scaling an unlimited cap does nothing).
    #[test]
    fn consecutive_signals_while_unlimited_clear_the_healthy_clock_and_recap() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        // Cap, then recover all the way to unlimited via a low-ratio trickle.
        bp.record_backpressure();
        for i in 1..=3000u32 {
            clock.advance(Duration::from_millis(20));
            if i % 20 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        assert_eq!(bp.state().permits_max, None, "must be unlimited first");
        // A fresh burst of consecutive 503s while unlimited must re-cap the gate: the
        // first signal keeps the stale clock (lone-signal exemption), the second clears
        // it, and the third boots the cap.
        bp.record_backpressure();
        bp.record_backpressure();
        bp.record_backpressure();
        let s = bp.state();
        assert!(
            s.permits_max.is_some(),
            "consecutive 503s while unlimited must boot a fresh cap: {s:?}"
        );
        assert_eq!(s.severity, BackpressureSeverity::Severe, "{s:?}");
    }

    /// A severe signal already shrinks by SEVERE_FACTOR; the rate-limited soft shrink
    /// must not also fire on the same signal just because SOFT_DECREASE_INTERVAL
    /// elapsed. Three consecutive signals one second apart must produce
    /// 12 -> 9 -> 5, not 12 -> 9 -> 4 (both factors on the third signal).
    #[test]
    fn severe_signals_shrink_once_per_signal_not_twice() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        bp.record_backpressure();
        assert_eq!(
            bp.state().permits_max,
            Some((INITIAL_MAX as f64 * SOFT_FACTOR).ceil() as u32) // 16 * 0.7 = 12
        );
        clock.advance(SOFT_DECREASE_INTERVAL + Duration::from_millis(1));
        bp.record_backpressure();
        assert_eq!(
            bp.state().permits_max,
            Some(((INITIAL_MAX as f64 * SOFT_FACTOR).ceil() * SOFT_FACTOR).ceil() as u32) // 9
        );
        clock.advance(SOFT_DECREASE_INTERVAL + Duration::from_millis(1));
        bp.record_backpressure(); // third consecutive: severe
        assert_eq!(bp.severity(), BackpressureSeverity::Severe);
        assert_eq!(
            bp.state().permits_max,
            Some(
                (((INITIAL_MAX as f64 * SOFT_FACTOR).ceil() * SOFT_FACTOR).ceil() * SEVERE_FACTOR)
                    .ceil() as u32,
            ), // 9 * 0.5 = 5
            "a severe signal must apply SEVERE_FACTOR only, not SEVERE_FACTOR * SOFT_FACTOR"
        );
    }

    /// A well-sampled window whose backpressure ratio is NOT low (e.g. alternating
    /// 503s and successes, where no two signals are consecutive so the clock is never
    /// cleared per-signal) must invalidate the sustained-health stretch and resume
    /// throttling — not keep growing the cap toward unlimited while the cluster fails
    /// half its requests.
    #[test]
    fn worsening_interleaved_pressure_invalidates_the_healthy_stretch() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        // Establish a low-ratio trickle long enough to start the healthy clock.
        bp.record_backpressure();
        for i in 1..=400u32 {
            clock.advance(Duration::from_millis(20));
            if i % 20 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        let cap_after_trickle = bp.state().permits_max.unwrap();
        // Switch to alternating 503 / success (50% backpressure): never consecutive,
        // well sampled, high ratio. The healthy stretch must be dropped and the cap
        // must stop growing toward unlimited.
        for _ in 0..40u32 {
            clock.advance(Duration::from_millis(50));
            bp.record_backpressure();
            clock.advance(Duration::from_millis(50));
            bp.record_healthy_hint();
        }
        let s = bp.state();
        assert!(
            s.permits_max.is_some(),
            "50% backpressure must not return the gate to unlimited: {s:?}"
        );
        assert!(
            s.permits_max.unwrap() <= cap_after_trickle,
            "the cap must not keep growing under 50% backpressure: {} -> {}",
            cap_after_trickle,
            s.permits_max.unwrap()
        );
    }

    /// Sparse traffic after a burst — one success every 3s — never fills a recovery
    /// window quickly, so severity decays by quietude alone once the burst's window has
    /// been judged. Reaching Healthy that way must still start the sustained-health
    /// clock, or phase 3 never returns to unlimited and the cap keeps growing forever.
    #[test]
    fn quiet_recovery_to_healthy_starts_the_clock_and_returns_to_unlimited() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        for _ in 0..8 {
            bp.record_backpressure();
        }
        assert_eq!(bp.severity(), BackpressureSeverity::Severe);
        for _ in 0..30u32 {
            clock.advance(Duration::from_secs(3));
            bp.record_healthy_hint();
        }
        let s = bp.state();
        assert_eq!(s.severity, BackpressureSeverity::Healthy, "{s:?}");
        assert_eq!(
            s.permits_max, None,
            "sparse healthy traffic that reaches Healthy must return to unlimited: {s:?}"
        );
    }

    /// A half-failing stream at one outcome every 3s makes every recovery check
    /// "quiet" (> DECAY_QUIET since the last signal). While capped, the quiet clause
    /// must not decay severity on the strength of an undersampled window that still
    /// carries unjudged signals, or the gate walks back to unlimited under 50% failure.
    #[test]
    fn low_rate_half_failing_stream_while_capped_never_goes_unlimited() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        for _ in 0..8 {
            bp.record_backpressure();
        }
        for i in 0..400u32 {
            clock.advance(Duration::from_secs(3));
            if i % 2 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
            assert!(bp.state().permits_max.is_some(), "went unlimited at {i}");
        }
        assert_ne!(bp.severity(), BackpressureSeverity::Healthy);
    }

    /// Same stream after phase 3 has returned to unlimited: the carried-over window
    /// must survive quiet checks until it is sampled, so the half-failing stream
    /// clears the sustained-health clock and re-caps the gate.
    #[test]
    fn low_rate_half_failing_stream_while_unlimited_recaps() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        bp.record_backpressure();
        for _ in 0..40u32 {
            clock.advance(Duration::from_secs(1));
            bp.record_healthy_hint();
        }
        assert_eq!(bp.state().permits_max, None, "precondition: unlimited");
        for i in 0..200u32 {
            clock.advance(Duration::from_secs(3));
            if i % 2 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        assert!(
            bp.state().permits_max.is_some(),
            "a 50% stream must re-cap: {:?}",
            bp.state()
        );
    }

    /// An already-Healthy capped gate whose carried window holds unjudged
    /// backpressure must not grow to unlimited before that window is judged.
    #[test]
    fn healthy_capped_gate_does_not_go_unlimited_with_unjudged_pressure() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        bp.record_backpressure();
        for _ in 0..10u32 {
            clock.advance(Duration::from_secs(1));
            bp.record_healthy_hint();
        }
        assert!(bp.state().permits_max.is_some(), "precondition: capped");
        for i in 0..200u32 {
            clock.advance(Duration::from_secs(3));
            if i % 2 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
            assert!(bp.state().permits_max.is_some(), "went unlimited at {i}");
        }
    }

    /// After recovery to unlimited, a 50% stream slower than WINDOW_IDLE_RESET must
    /// not keep the sustained-health tolerance alive via idle resets.
    #[test]
    fn slow_half_failing_stream_after_idle_resets_recaps() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        bp.record_backpressure();
        for _ in 0..40u32 {
            clock.advance(Duration::from_secs(1));
            bp.record_healthy_hint();
        }
        assert_eq!(bp.state().permits_max, None, "precondition: unlimited");
        for i in 0..20u32 {
            clock.advance(Duration::from_secs(12));
            if i % 2 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        assert!(
            bp.state().permits_max.is_some(),
            "a slow 50% stream must re-cap: {:?}",
            bp.state()
        );
    }

    /// A window can be quiet AND well-sampled at once (a burst of outcomes, then >
    /// DECAY_QUIET of silence before the next success). When its backpressure ratio is
    /// high, the sampled ratio is the fresher evidence: the check must invalidate the
    /// healthy stretch and must NOT decay severity on the quiet clause in the same
    /// breath.
    #[test]
    fn a_quiet_but_high_ratio_window_does_not_decay() {
        let clock = Arc::new(super::super::clock::ManualClock::new());
        let bp = BackpressureManager::new(BackpressureProfile::Balanced, clock.clone());
        // Cap the gate and get the healthy clock running via a low-ratio trickle.
        bp.record_backpressure();
        for i in 1..=400u32 {
            clock.advance(Duration::from_millis(20));
            if i % 20 == 0 {
                bp.record_backpressure();
            } else {
                bp.record_healthy_hint();
            }
        }
        assert_eq!(bp.severity(), BackpressureSeverity::Healthy);
        // A burst of 20 alternating 503s / successes inside one window (50% ratio,
        // well sampled, never consecutive), then 3s of quiet so the next recovery
        // check sees a window that is both quiet and high-ratio.
        for _ in 0..10u32 {
            bp.record_backpressure();
            bp.record_healthy_hint();
        }
        clock.advance(DECAY_QUIET + Duration::from_secs(1));
        bp.record_healthy_hint();
        let s = bp.state();
        assert_ne!(
            s.severity,
            BackpressureSeverity::Healthy,
            "a quiet but high-ratio window must drop severity out of Healthy, not keep it: {s:?}"
        );
        assert!(
            s.permits_max.is_some(),
            "the gate must stay capped under sampled high-ratio pressure: {s:?}"
        );
    }
}
