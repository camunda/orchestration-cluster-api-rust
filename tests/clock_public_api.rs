//! The injected clock has to be usable from *outside* the crate.
//!
//! `with_clock` takes an `Arc<dyn Clock>`, but `runtime` is a private module — so until
//! `Clock` was re-exported from the crate root, downstream code could not name the trait and
//! the injection point was uncallable. Nothing caught that, because every other clock test
//! lives inside the crate where `Clock` is always in scope.
//!
//! This file is compiled as a separate crate, which is the only vantage point from which the
//! gap is visible. Anything the public API needs to be usable belongs here rather than in a
//! `#[cfg(test)]` module.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use camunda_orchestration_sdk::{
    live_clock, CamundaOptions, Clock, ClockController, EngineClock, LiveClock, Result,
};
use tokio::time::Instant;

/// A downstream implementation, written against nothing but the public API.
#[derive(Debug)]
struct CountingClock {
    inner: LiveClock,
    sleeps: std::sync::Mutex<Vec<Duration>>,
}

#[async_trait::async_trait]
impl Clock for CountingClock {
    fn now(&self) -> Instant {
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

#[tokio::test(start_paused = true)]
async fn a_downstream_crate_can_implement_and_inject_a_clock() {
    let clock = Arc::new(CountingClock {
        inner: LiveClock,
        sleeps: std::sync::Mutex::new(Vec::new()),
    });

    clock.sleep(Duration::from_secs(30)).await;
    assert_eq!(
        clock.sleeps.lock().expect("poisoned").as_slice(),
        &[Duration::from_secs(30)]
    );

    // The injection point itself must accept it. Building a client needs an address, so this
    // asserts the option is expressible rather than driving a request.
    let options = CamundaOptions::new()
        .with("CAMUNDA_REST_ADDRESS", "http://localhost:8080")
        .with_clock(clock);
    assert!(
        options.clock.is_some(),
        "with_clock did not record the clock"
    );
}

#[tokio::test(start_paused = true)]
async fn the_shared_live_clock_is_reachable_too() {
    let clock: Arc<dyn Clock> = live_clock();

    let before = clock.now();
    clock.sleep(Duration::from_secs(5)).await;

    assert!(clock.now().duration_since(before) >= Duration::from_secs(5));
}

/// A downstream `ClockController`. `EngineClock` is only useful if a caller outside the
/// crate can both name it and drive it, and the trait is the half that is easy to leave
/// unexported -- the same gap that made `Clock` unnameable before it was re-exported.
#[derive(Debug, Default)]
struct RecordingEngine {
    pins: std::sync::Mutex<Vec<i64>>,
}

#[async_trait::async_trait]
impl ClockController for RecordingEngine {
    async fn pin(&self, epoch_millis: i64) -> Result<()> {
        self.pins.lock().expect("poisoned").push(epoch_millis);
        Ok(())
    }

    async fn reset(&self) -> Result<()> {
        self.pins.lock().expect("poisoned").clear();
        Ok(())
    }
}

#[tokio::test]
async fn an_engine_clock_is_usable_from_outside_the_crate() {
    let engine = Arc::new(RecordingEngine::default());
    let clock = EngineClock::new(engine.clone());

    clock.pin_to(1_700_000_000_000).await.expect("pin");
    clock.sleep(Duration::from_secs(30)).await;

    assert_eq!(
        engine.pins.lock().expect("poisoned").as_slice(),
        &[1_700_000_000_000, 1_700_000_030_000],
        "a wait should have moved the engine forward by thirty seconds"
    );

    // And it satisfies the injection point, which is the whole reason it exists.
    let clock: Arc<dyn Clock> = Arc::new(EngineClock::new(engine));
    let options = CamundaOptions::new()
        .with("CAMUNDA_REST_ADDRESS", "http://localhost:8080")
        .with_clock(clock);
    assert!(options.clock.is_some());
}
