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
//! real. `tokio::time::Instant` derefs to the `std` type, so callers that need one can still
//! get it.

use std::fmt::Debug;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use tokio::time::Instant;

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
}
