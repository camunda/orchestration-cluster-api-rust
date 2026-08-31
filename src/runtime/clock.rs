//! The clock all SDK runtime cadence resolves through.
//!
//! Worker poll loops, eventual-consistency polling, retry backoff, backpressure decay and
//! OAuth refresh all read time here. Injecting a clock makes that cadence controllable, and
//! is what lets a later engine-bound implementation advance client cadence and engine time
//! together.
//!
//! See the cross-SDK contract in camunda/orchestration-cluster-api-js#450.
//!
//! # Two notions of time, deliberately
//!
//! Unlike the JS, Python and C# SDKs — which expose a single wall-clock reading — this trait
//! exposes both [`Clock::now`] ([`Instant`], monotonic) and [`Clock::now_wall`]
//! ([`SystemTime`], wall). Collapsing them was not an option: [`crate::runtime::auth`] holds
//! both views on purpose, because the on-disk token cache has to survive a process restart
//! (which a monotonic `Instant` cannot) while in-process refresh checks want monotonicity.
//!
//! The split is a feature rather than a tax. Deadlines and elapsed-time measurements use
//! `now()` and inherit `Instant`'s guarantee that it never goes backwards; only state that
//! outlives the process needs `now_wall()`, where a backward jump is possible and must be
//! tolerated.
//!
//! # Why `tokio::time::Instant`
//!
//! `now()` returns tokio's `Instant`, not `std::time::Instant`. Under
//! `#[tokio::test(start_paused = true)]` tokio virtualises its own timer — `tokio::time::sleep`
//! and `tokio::time::Instant::now` move together — but it does not touch `std::time::Instant`.
//! A clock reading `std` while sleeping on tokio would report that no time had passed across
//! a sleep that the runtime believes took thirty seconds, which is worse than either being
//! real. Callers that need the `std` type can convert with `Instant::into_std`.

use std::fmt::Debug;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use tokio::time::Instant;

use super::errors::Result;

/// Time and waiting, as the SDK runtime sees them.
///
/// Implementations must be cheap to clone behind an [`Arc`] and safe to share across tasks
/// and threads.
#[async_trait]
pub trait Clock: Send + Sync + Debug {
    /// A monotonic reading, for deadlines and elapsed-time measurement.
    ///
    /// Must never go backwards.
    fn now(&self) -> Instant;

    /// A wall-clock reading, for state that outlives the process.
    ///
    /// May jump in either direction — only use it where a monotonic reading cannot serve,
    /// such as an expiry written to disk and read back after a restart.
    fn now_wall(&self) -> SystemTime;

    /// Wait for `duration`.
    ///
    /// Must yield to the executor at least once, even for a zero or negative-equivalent
    /// duration: a `sleep` that completes without yielding turns a caller that reschedules
    /// itself into a spin.
    async fn sleep(&self, duration: Duration);
}

/// Real time.
///
/// `now` and `sleep` are both tokio's, so they share one timeline: under
/// `#[tokio::test(start_paused = true)]` this clock is already virtual, and a poll loop
/// settles without waiting. That covers cadence in tests; it does not bind the engine's
/// clock, which is what an engine-bound implementation is for.
#[derive(Debug, Default, Clone, Copy)]
pub struct LiveClock;

// The one place ambient time is legitimate: this impl *is* the adapter onto it.
#[allow(clippy::disallowed_methods)]
#[async_trait]
impl Clock for LiveClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn now_wall(&self) -> SystemTime {
        SystemTime::now()
    }

    async fn sleep(&self, duration: Duration) {
        if duration.is_zero() {
            // `tokio::time::sleep(ZERO)` is Ready on its first poll -- it never yields. A
            // caller that reschedules itself on completion would spin, and callers reach
            // zero routinely by computing `deadline - now()` on an elapsed deadline. A 1ns
            // sleep does yield, so only exactly-zero needs this.
            tokio::task::yield_now().await;
            return;
        }
        tokio::time::sleep(duration).await;
    }
}

/// The shared [`LiveClock`], used when no clock is injected.
pub fn live_clock() -> Arc<dyn Clock> {
    static LIVE: OnceLock<Arc<dyn Clock>> = OnceLock::new();
    LIVE.get_or_init(|| Arc::new(LiveClock)).clone()
}

/// A clock that records what was asked of it, for tests that need to prove a wait resolved
/// here rather than through an ambient timer.
///
/// Lives beside the trait rather than in one test module because several subsystems need it,
/// and a per-module copy is how implementations drift apart.
/// The engine-side clock an [`EngineClock`] drives.
///
/// Implemented for [`CamundaClient`](super::client::CamundaClient) in terms of the
/// `PUT /clock` and `POST /clock/reset` endpoints. It exists as a trait so the pin
/// semantics can be tested without a running engine.
#[async_trait]
pub trait ClockController: Send + Sync {
    /// Pin the engine clock to an absolute instant, in epoch milliseconds.
    async fn pin(&self, epoch_millis: i64) -> Result<()>;

    /// Return the engine clock to real time.
    async fn reset(&self) -> Result<()>;
}

/// Where the engine clock currently sits, and the monotonic reading that corresponds
/// to it. Both move forward together so `now()` never goes backwards.
#[derive(Debug, Clone, Copy)]
struct Pinned {
    wall_ms: i64,
    mono: Instant,
}

/// A clock bound to the engine's own clock.
///
/// A wait does not pass time locally -- it moves the *engine* forward and reports the new
/// instant. Process instances, timers and the SDK therefore agree on what time it is,
/// which a purely local test clock cannot achieve.
///
/// ```no_run
/// # use camunda_orchestration_sdk::{CamundaClient, Clock, EngineClock, CamundaOptions};
/// # use std::{sync::Arc, time::Duration};
/// # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
/// let control = CamundaClient::from_env()?;
/// let clock: Arc<dyn Clock> = Arc::new(EngineClock::new(Arc::new(control)));
///
/// // Anything the SDK waits on now advances the engine instead of real time.
/// let client = CamundaClient::new(CamundaOptions::new().with_clock(clock))?;
/// # let _ = client;
/// # Ok(())
/// # }
/// ```
///
/// Waits are resolved against an instant fixed *before* the engine is contacted, so
/// overlapping waits settle at one instant rather than summing, while sequential waits
/// compose. Ten concurrent one-second waits advance the engine by one second, not ten.
pub struct EngineClock {
    engine: Arc<dyn ClockController>,
    live: LiveClock,
    /// `None` means unpinned: readings follow live time.
    state: std::sync::Mutex<Option<Pinned>>,
    /// Serialises pin round-trips so concurrent waits collapse into a single request.
    gate: tokio::sync::Mutex<()>,
}

impl std::fmt::Debug for EngineClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineClock")
            .field("pinned", &self.state.lock().map(|s| *s).unwrap_or(None))
            .finish()
    }
}

impl EngineClock {
    /// Bind to an engine. The clock starts unpinned, following real time until the first
    /// wait or [`pin_to`](Self::pin_to).
    ///
    /// Pass the client you want the *pin requests themselves* to go through. That cannot be
    /// a client using this clock: a `CamundaClient` captures its clock when it is built, so
    /// the client given here always predates this clock.
    pub fn new(engine: Arc<dyn ClockController>) -> Self {
        EngineClock {
            engine,
            live: LiveClock,
            state: std::sync::Mutex::new(None),
            gate: tokio::sync::Mutex::new(()),
        }
    }

    /// Move the engine clock to an absolute instant, in epoch milliseconds.
    ///
    /// A no-op when the clock already sits at or past that instant, which is what makes
    /// overlapping waits settle at a single instant. The local reading is published only
    /// after the engine accepts the pin, so a failed request leaves the clock untouched.
    pub async fn pin_to(&self, epoch_millis: i64) -> Result<()> {
        let _gate = self.gate.lock().await;

        let next_mono = {
            let state = self.state.lock().expect("clock state poisoned");
            match *state {
                Some(pinned) if epoch_millis <= pinned.wall_ms => return Ok(()),
                Some(pinned) => {
                    pinned.mono + Duration::from_millis((epoch_millis - pinned.wall_ms) as u64)
                }
                None => {
                    let ahead = epoch_millis.saturating_sub(self.live_wall_millis()).max(0);
                    self.live.now() + Duration::from_millis(ahead as u64)
                }
            }
        };

        self.engine.pin(epoch_millis).await?;

        *self.state.lock().expect("clock state poisoned") = Some(Pinned {
            wall_ms: epoch_millis,
            mono: next_mono,
        });
        Ok(())
    }

    /// Return the engine to real time. Readings follow live time again afterwards, rather
    /// than freezing at the last pinned instant.
    pub async fn reset(&self) -> Result<()> {
        let _gate = self.gate.lock().await;
        self.engine.reset().await?;
        *self.state.lock().expect("clock state poisoned") = None;
        Ok(())
    }

    /// Whether the engine clock is currently pinned by this clock.
    pub fn is_pinned(&self) -> bool {
        self.state.lock().expect("clock state poisoned").is_some()
    }

    fn pinned(&self) -> Option<Pinned> {
        *self.state.lock().expect("clock state poisoned")
    }

    fn live_wall_millis(&self) -> i64 {
        self.live
            .now_wall()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    fn wall_millis(&self) -> i64 {
        match self.pinned() {
            Some(pinned) => pinned.wall_ms,
            None => self.live_wall_millis(),
        }
    }
}

#[async_trait]
impl Clock for EngineClock {
    fn now(&self) -> Instant {
        match self.pinned() {
            Some(pinned) => pinned.mono,
            None => self.live.now(),
        }
    }

    fn now_wall(&self) -> SystemTime {
        match self.pinned() {
            Some(pinned) => UNIX_EPOCH + Duration::from_millis(pinned.wall_ms.max(0) as u64),
            None => self.live.now_wall(),
        }
    }

    async fn sleep(&self, duration: Duration) {
        if duration.is_zero() {
            // Same contract as `LiveClock`: a wait always yields, even a zero one.
            tokio::task::yield_now().await;
            return;
        }

        // Fix the wake instant before contacting the engine. Computing it from the pinned
        // value at request time instead would make overlapping waits sum.
        let millis = i64::try_from(duration.as_millis()).unwrap_or(i64::MAX);
        let deadline = self.wall_millis().saturating_add(millis);

        // `Clock::sleep` cannot report failure, and continuing as if the wait happened
        // would hand the caller a clock that silently stopped advancing.
        if let Err(e) = self.pin_to(deadline).await {
            panic!("EngineClock could not advance the engine clock to {deadline}ms: {e}");
        }
    }
}
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct RecordingClock {
    inner: LiveClock,
    sleeps: std::sync::Mutex<Vec<Duration>>,
    now_calls: std::sync::Mutex<u32>,
}

#[cfg(test)]
impl RecordingClock {
    pub(crate) fn sleeps(&self) -> Vec<Duration> {
        self.sleeps.lock().expect("poisoned").clone()
    }

    pub(crate) fn now_calls(&self) -> u32 {
        *self.now_calls.lock().expect("poisoned")
    }
}

#[cfg(test)]
#[async_trait]
impl Clock for RecordingClock {
    fn now(&self) -> Instant {
        *self.now_calls.lock().expect("poisoned") += 1;
        self.inner.now()
    }

    fn now_wall(&self) -> SystemTime {
        self.inner.now_wall()
    }

    async fn sleep(&self, duration: Duration) {
        self.sleeps.lock().expect("poisoned").push(duration);
        self.inner.sleep(duration).await;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// The contract every implementation is held to. Kept as a free function so an
    /// engine-bound or manual clock can be run through the same clauses rather than having
    /// its own hand-written checks — in the JS slice, three broken clocks reached review
    /// because each was tested only against itself.
    pub(crate) async fn assert_clock_contract(clock: &dyn Clock) {
        let first = clock.now();
        clock.sleep(Duration::from_millis(10)).await;
        assert!(
            clock.now() >= first,
            "now() went backwards across a sleep: {first:?}"
        );

        // Already-elapsed durations are ordinary: callers compute `deadline - now()`, which
        // goes to zero the moment a deadline passes. It must still yield.
        assert!(
            poll_once(clock.sleep(Duration::ZERO)).is_pending(),
            "sleep(ZERO) completed without yielding"
        );
        clock.sleep(Duration::ZERO).await;

        let wall = clock.now_wall();
        assert!(
            wall.duration_since(SystemTime::UNIX_EPOCH).is_ok(),
            "now_wall() must be at or after the epoch"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn live_clock_satisfies_the_contract() {
        assert_clock_contract(&LiveClock).await;
    }

    #[tokio::test(start_paused = true)]
    async fn sleeping_advances_now() {
        let clock = LiveClock;
        let before = clock.now();

        clock.sleep(Duration::from_secs(30)).await;

        assert!(
            clock.now().duration_since(before) >= Duration::from_secs(30),
            "a 30s sleep did not move the clock forward by 30s"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn now_and_sleep_share_one_timeline() {
        // The defect this guards: reading `std::time::Instant` while sleeping on
        // `tokio::time` reports that no time passed across a sleep the runtime believes took
        // thirty seconds. Both halves have to come from the same timer.
        let clock = LiveClock;
        let before = clock.now();

        clock.sleep(Duration::from_secs(30)).await;
        let virtual_elapsed = clock.now().duration_since(before);

        assert!(
            virtual_elapsed >= Duration::from_secs(30),
            "now() did not follow the virtual timer that sleep() used: {virtual_elapsed:?}"
        );
    }

    /// Poll a future exactly once. `Pending` means it yielded; `Ready` means it completed
    /// without ever handing control back to the executor.
    fn poll_once<F: std::future::Future>(fut: F) -> std::task::Poll<F::Output> {
        let waker = futures_util::task::noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);
        Box::pin(fut).as_mut().poll(&mut cx)
    }

    #[tokio::test(start_paused = true)]
    async fn sleep_yields_to_the_executor() {
        // The clause hand-rolled clocks break most often: a sleep that returns on the first
        // poll turns a caller that reschedules itself into a spin. Asserted by polling
        // rather than by racing a real sleep, so it is deterministic and states the property
        // directly.
        assert!(
            poll_once(LiveClock.sleep(Duration::from_secs(30))).is_pending(),
            "sleep() completed without yielding"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_zero_sleep_yields_too() {
        // `tokio::time::sleep(ZERO)` is Ready on its first poll. Callers reach zero by
        // computing `deadline - now()` once a deadline has passed, so without special
        // handling an expired poll loop spins instead of waiting.
        assert!(
            poll_once(LiveClock.sleep(Duration::ZERO)).is_pending(),
            "sleep(ZERO) completed without yielding"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_shared_live_clock_is_one_instance() {
        assert!(
            Arc::ptr_eq(&live_clock(), &live_clock()),
            "live_clock() should hand out the same shared clock"
        );
        assert_clock_contract(live_clock().as_ref()).await;
    }

    /// The ban lives in `clippy.toml`, which no code references. Delete that file and
    /// clippy simply stops warning -- green build, ambient time back in the runtime. Assert
    /// the entries are present so removing them is loud instead.
    #[test]
    fn ambient_time_stays_banned_in_clippy_config() {
        let config = include_str!("../../clippy.toml");
        // Live entries only. Prose in this file names the banned paths, and a bare
        // substring check would accept a commented-out entry -- or accept
        // `tokio::time::sleep` on the strength of the `sleep_until` line alone.
        let entries: Vec<&str> = config
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#'))
            .collect();

        for path in [
            "std::time::Instant::now",
            "tokio::time::Instant::now",
            "std::time::SystemTime::now",
            "tokio::time::sleep",
            "tokio::time::sleep_until",
            "std::thread::sleep",
        ] {
            let field = format!("path = \"{path}\"");
            assert!(
                entries.iter().any(|line| line.contains(&field)),
                "`{path}` is no longer banned in clippy.toml; ambient time can re-enter the runtime"
            );
        }
    }

    /// Records what the engine was asked to do, and can be made to refuse.
    #[derive(Debug, Default)]
    struct FakeEngine {
        pins: std::sync::Mutex<Vec<i64>>,
        resets: std::sync::Mutex<u32>,
        refuse: bool,
    }

    impl FakeEngine {
        fn refusing() -> Self {
            FakeEngine {
                refuse: true,
                ..Default::default()
            }
        }
        fn pins(&self) -> Vec<i64> {
            self.pins.lock().expect("poisoned").clone()
        }
        fn resets(&self) -> u32 {
            *self.resets.lock().expect("poisoned")
        }
    }

    #[async_trait]
    impl ClockController for FakeEngine {
        async fn pin(&self, epoch_millis: i64) -> Result<()> {
            if self.refuse {
                return Err(super::super::errors::CamundaError::worker("engine refused"));
            }
            // Yield so concurrent callers genuinely interleave rather than each pin
            // completing before the next one starts.
            tokio::task::yield_now().await;
            self.pins.lock().expect("poisoned").push(epoch_millis);
            Ok(())
        }

        async fn reset(&self) -> Result<()> {
            *self.resets.lock().expect("poisoned") += 1;
            Ok(())
        }
    }

    fn engine_clock() -> (Arc<FakeEngine>, EngineClock) {
        let engine = Arc::new(FakeEngine::default());
        (engine.clone(), EngineClock::new(engine))
    }

    #[tokio::test]
    async fn sequential_waits_compose() {
        let (engine, clock) = engine_clock();

        clock.sleep(Duration::from_secs(1)).await;
        let after_first = clock.now();
        clock.sleep(Duration::from_secs(2)).await;

        let pins = engine.pins();
        assert_eq!(pins.len(), 2, "expected one pin per wait, got {pins:?}");
        assert_eq!(
            pins[1] - pins[0],
            2_000,
            "the second wait did not start from where the first ended"
        );
        assert_eq!(
            clock.now().duration_since(after_first),
            Duration::from_secs(2)
        );
    }

    /// Ten concurrent one-second waits are all satisfied by the *same* instant. Deriving
    /// each wake time from the pinned value at request time instead would advance the
    /// engine ten seconds -- the defect the C# SDK was corrected for.
    #[tokio::test]
    async fn overlapping_waits_settle_at_one_instant() {
        let (engine, clock) = engine_clock();
        let clock = Arc::new(clock);

        // Pin to a known instant first, so every reading below is exact. Starting from
        // live time would make the assertions depend on how much real time elapsed
        // between reads.
        const BASE: i64 = 1_700_000_000_000;
        clock.pin_to(BASE).await.expect("initial pin");

        let waits: Vec<_> = (0..10)
            .map(|_| {
                let clock = clock.clone();
                tokio::spawn(async move { clock.sleep(Duration::from_secs(1)).await })
            })
            .collect();
        for w in waits {
            w.await.expect("wait task panicked");
        }

        let pins = engine.pins();
        assert_eq!(
            pins,
            vec![BASE, BASE + 1_000],
            "overlapping waits should collapse to a single pin one second past the base"
        );
        assert_eq!(
            clock.wall_millis(),
            BASE + 1_000,
            "ten concurrent one-second waits advanced the engine by more than one second"
        );
    }

    #[tokio::test]
    async fn readings_follow_live_time_until_the_first_wait() {
        let (_engine, clock) = engine_clock();
        assert!(!clock.is_pinned());

        let live_gap = clock
            .now_wall()
            .duration_since(UNIX_EPOCH)
            .expect("before epoch");
        let now_gap = LiveClock
            .now_wall()
            .duration_since(UNIX_EPOCH)
            .expect("before epoch");
        assert!(
            now_gap.saturating_sub(live_gap) < Duration::from_secs(1),
            "an unpinned engine clock should read live wall time"
        );

        clock.sleep(Duration::from_secs(5)).await;
        assert!(clock.is_pinned());
    }

    #[tokio::test]
    async fn reset_returns_the_clock_to_live_time() {
        let (engine, clock) = engine_clock();
        clock.sleep(Duration::from_secs(3_600)).await;
        assert!(clock.is_pinned());

        clock.reset().await.expect("reset should succeed");

        assert_eq!(engine.resets(), 1);
        assert!(
            !clock.is_pinned(),
            "reset left the clock frozen at the last pin instead of following live time"
        );
        let drift = clock
            .now_wall()
            .duration_since(UNIX_EPOCH)
            .expect("before epoch")
            .as_millis() as i64
            - LiveClock
                .now_wall()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
        assert!(
            drift.abs() < 1_000,
            "after reset the clock should read live time again, drifted {drift}ms"
        );
    }

    /// A refused pin must leave the clock exactly where it was. Publishing the new reading
    /// first would report time the engine never moved to.
    #[tokio::test]
    async fn a_refused_pin_leaves_the_clock_untouched() {
        let engine = Arc::new(FakeEngine::refusing());
        let clock = EngineClock::new(engine);

        let err = clock.pin_to(1_000_000).await.expect_err("pin should fail");
        assert!(err.to_string().contains("engine refused"));
        assert!(
            !clock.is_pinned(),
            "a failed pin was published locally anyway"
        );
    }

    /// `Clock::sleep` cannot return an error, so a refused pin must be loud rather than
    /// silently handing back a clock that stopped advancing.
    #[tokio::test]
    #[should_panic(expected = "could not advance the engine clock")]
    async fn a_refused_pin_during_a_wait_panics() {
        let clock = EngineClock::new(Arc::new(FakeEngine::refusing()));
        clock.sleep(Duration::from_secs(1)).await;
    }

    #[tokio::test]
    async fn a_zero_wait_yields_without_touching_the_engine() {
        let (engine, clock) = engine_clock();
        clock.sleep(Duration::ZERO).await;
        assert!(
            engine.pins().is_empty(),
            "a zero wait should not move the engine"
        );
    }
}
