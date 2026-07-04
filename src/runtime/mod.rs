//! Hand-written runtime: configuration, authentication, errors, the ergonomic
//! [`client::CamundaClient`] facade, and [`job_worker::JobWorker`].

pub mod auth;
pub mod backpressure;
pub mod client;
pub mod config;
pub mod errors;
pub mod eventual;
pub mod facade_generated;
pub mod falcon;
pub mod job_worker;
pub mod logging;
pub mod retry;
pub mod tls;

use std::cell::Cell;

thread_local! {
    static RNG_STATE: Cell<u64> = Cell::new(seed());
}

fn seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15);
    // Mix in a per-thread address so concurrent threads don't share a stream.
    let local = &nanos as *const _ as u64;
    (nanos ^ local).max(1)
}

/// A small, dependency-free pseudo-random fraction in `[0, 1)`, used for backoff and
/// startup jitter. Not cryptographically secure — adequate for load spreading only.
pub(crate) fn rand_fraction() -> f64 {
    RNG_STATE.with(|state| {
        let mut x = state.get();
        // xorshift64
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        state.set(x);
        // Top 53 bits -> [0, 1).
        ((x >> 11) as f64) / ((1u64 << 53) as f64)
    })
}
