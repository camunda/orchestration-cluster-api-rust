//! Randomness, as the SDK runtime sees it.
//!
//! Every jitter draw — retry backoff, worker startup delay and FALCON endpoint
//! selection — goes through an injected [`Random`], so a [`SeededRandom`] makes them
//! reproducible.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use super::errors::{CamundaError, Result};

/// The environment variable [`SeededRandom::from_env`] reads.
const SEED_ENV_VAR: &str = "CAMUNDA_TEST_SEED";

/// A source of uniformly distributed values in `[0, 1)`.
///
/// Implementations must be safe to share across tasks and threads.
pub trait Random: Send + Sync + fmt::Debug {
    /// A uniformly distributed value in `[0, 1)`.
    fn next_f64(&self) -> f64;
}

/// The default source: a process-wide generator seeded once from OS entropy.
///
/// Rust's standard library has no random-number API, so this seeds the same generator
/// as [`SeededRandom`] from [`std::hash::RandomState`], the only entropy std exposes.
#[derive(Debug, Default, Clone, Copy)]
pub struct LiveRandom;

impl Random for LiveRandom {
    fn next_f64(&self) -> f64 {
        static PROCESS: OnceLock<SeededRandom> = OnceLock::new();
        PROCESS
            .get_or_init(|| {
                use std::hash::{BuildHasher, Hasher};
                #[allow(clippy::disallowed_types)] // the adapter onto ambient entropy
                let entropy = std::hash::RandomState::new();
                SeededRandom::new(entropy.build_hasher().finish())
            })
            .next_f64()
    }
}

/// The shared [`LiveRandom`], used when no source is injected.
pub(crate) fn live_random() -> Arc<dyn Random> {
    static LIVE: OnceLock<Arc<dyn Random>> = OnceLock::new();
    LIVE.get_or_init(|| Arc::new(LiveRandom)).clone()
}

/// A deterministic [`Random`]. Two sources with the same seed produce the same
/// sequence, so a test can assert the exact delays the SDK schedules and replay a
/// failure.
///
/// The generator is SplitMix64, taking the top 53 bits of each output. It is specified
/// across the Camunda SDKs, so one seed yields the same sequence in every language. It
/// is lock-free and safe to share; concurrent callers then share one sequence in
/// nondeterministic order.
#[derive(Debug)]
pub struct SeededRandom {
    seed: u64,
    state: AtomicU64,
}

impl SeededRandom {
    /// A source that replays the sequence for `seed`.
    pub fn new(seed: u64) -> Self {
        SeededRandom {
            seed,
            state: AtomicU64::new(seed),
        }
    }

    /// Seeds from `CAMUNDA_TEST_SEED` when it is set, otherwise from a fresh seed.
    /// Either way the source reports its seed through [`seed`](Self::seed) and its
    /// `Display` form, so a failing test can be replayed.
    ///
    /// Only an unset variable means absent. Any other value that is not a decimal
    /// integer in the `u64` range, including an empty string, is a
    /// [`CamundaError::Config`] rather than a silent switch to a different seed.
    pub fn from_env() -> Result<Self> {
        Self::from_env_value(std::env::var_os(SEED_ENV_VAR))
    }

    fn from_env_value(raw: Option<std::ffi::OsString>) -> Result<Self> {
        let Some(raw) = raw else {
            return Ok(Self::new(
                (LiveRandom.next_f64() * (1u64 << 53) as f64) as u64,
            ));
        };
        let invalid = || {
            CamundaError::Config(format!(
                "{SEED_ENV_VAR} must be a decimal integer in the u64 range, got {raw:?}"
            ))
        };
        let text = raw.to_str().ok_or_else(invalid)?;
        // `u64::from_str` also accepts a leading `+`; the contract is ASCII [0-9]+.
        if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        text.parse().map(Self::new).map_err(|_| invalid())
    }

    /// The seed this source was created with.
    pub fn seed(&self) -> u64 {
        self.seed
    }
}

impl Random for SeededRandom {
    fn next_f64(&self) -> f64 {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut z = self
            .state
            .fetch_add(GOLDEN, Ordering::Relaxed)
            .wrapping_add(GOLDEN);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

impl fmt::Display for SeededRandom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SeededRandom(seed={}; replay with {SEED_ENV_VAR}={})",
            self.seed, self.seed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn seeded_random_matches_the_cross_sdk_vector() {
        let r = SeededRandom::new(42);
        for want in [
            0.7415648787718233,
            0.1599103928769201,
            0.27860113025513866,
            0.34419071652363753,
            0.03803016854024621,
        ] {
            assert_eq!(r.next_f64(), want);
        }
        // Seed 0's first SplitMix64 output is 0xE220A8397B1DCDAF; the draw is its top
        // 53 bits.
        let want = (0xE220_A839_7B1D_CDAFu64 >> 11) as f64 / (1u64 << 53) as f64;
        assert_eq!(SeededRandom::new(0).next_f64(), want);
    }

    /// Concurrent callers share one sequence: every draw is handed out exactly once, so
    /// the multiset of draws equals the sequential one.
    #[test]
    fn seeded_random_is_safe_to_share_across_threads() {
        const THREADS: usize = 8;
        const EACH: usize = 1_000;
        let shared = SeededRandom::new(7);
        let mut got: Vec<u64> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..THREADS)
                .map(|_| {
                    s.spawn(|| {
                        (0..EACH)
                            .map(|_| shared.next_f64().to_bits())
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap())
                .collect()
        });
        let sequential = SeededRandom::new(7);
        let mut want: Vec<u64> = (0..THREADS * EACH)
            .map(|_| sequential.next_f64().to_bits())
            .collect();
        got.sort_unstable();
        want.sort_unstable();
        assert_eq!(got, want, "a state update was lost or repeated");
    }

    #[test]
    fn seeded_random_names_its_seed() {
        let r = SeededRandom::new(42);
        assert_eq!(r.seed(), 42);
        assert_eq!(
            r.to_string(),
            "SeededRandom(seed=42; replay with CAMUNDA_TEST_SEED=42)"
        );
    }

    #[test]
    fn from_env_reads_the_seed() {
        for (raw, want) in [
            ("42", 42),
            ("0", 0),
            ("007", 7),
            ("18446744073709551615", u64::MAX),
        ] {
            let r = SeededRandom::from_env_value(Some(raw.into())).unwrap();
            assert_eq!(r.seed(), want, "{raw:?}");
        }
    }

    /// Only an unset variable means absent. A value that is set but malformed fails
    /// rather than running the test under a seed nobody asked for.
    #[test]
    fn from_env_rejects_a_malformed_seed() {
        for raw in [
            "",
            " 42",
            "42 ",
            "+1",
            "-1",
            "0x2A",
            "1_0",
            "4.2",
            "1e3",
            "18446744073709551616",
            "٤٢",
        ] {
            let err = SeededRandom::from_env_value(Some(raw.into())).unwrap_err();
            assert!(matches!(err, CamundaError::Config(_)), "{raw:?}: {err:?}");
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            let not_utf8 = OsString::from_vec(vec![b'4', 0xFF]);
            assert!(SeededRandom::from_env_value(Some(not_utf8)).is_err());
        }
    }

    #[test]
    fn from_env_draws_a_fresh_seed_when_unset() {
        let a = SeededRandom::from_env_value(None).unwrap();
        let b = SeededRandom::from_env_value(None).unwrap();
        // A 53-bit seed survives a round trip through a double, in the SDKs that hold
        // seeds that way.
        assert!(a.seed() < 1 << 53 && b.seed() < 1 << 53);
        assert_ne!(a.seed(), b.seed(), "the fresh seed is not being drawn");
    }

    #[test]
    fn live_random_draws_in_the_unit_interval() {
        for _ in 0..1_000 {
            let u = LiveRandom.next_f64();
            assert!((0.0..1.0).contains(&u), "{u}");
        }
    }

    #[test]
    fn the_client_draws_from_the_injected_source() {
        let random: Arc<dyn Random> = Arc::new(SeededRandom::new(1));
        let options =
            || crate::CamundaOptions::new().with("CAMUNDA_REST_ADDRESS", "http://localhost:8080");
        let client = crate::CamundaClient::new(options().with_random(random.clone())).unwrap();
        assert!(Arc::ptr_eq(client.random(), &random));

        let client = crate::CamundaClient::new(options()).unwrap();
        assert!(
            Arc::ptr_eq(client.random(), &live_random()),
            "the default is not LiveRandom"
        );
    }

    /// The ban lives in `clippy.toml`, which no code references. Assert the entry is
    /// present so removing it is loud instead of a quiet green build.
    #[test]
    fn ambient_entropy_stays_banned_in_clippy_config() {
        let config = include_str!("../../clippy.toml");
        assert!(
            config
                .lines()
                .map(str::trim)
                .filter(|line| !line.starts_with('#'))
                .any(|line| line.contains("path = \"std::hash::RandomState\"")),
            "`std::hash::RandomState` is no longer banned in clippy.toml"
        );
    }
}
