//! Configurable SDK logging.
//!
//! The SDK emits structured [`tracing`] events (request/retry/backpressure/token
//! lifecycle). Applications usually own their own `tracing` subscriber; for parity with
//! the JS / Python / C# SDKs — which honour `CAMUNDA_SDK_LOG_LEVEL` out of the box — this
//! module offers an opt-in initializer that installs a formatting subscriber filtered to
//! the configured level.
//!
//! Call [`CamundaClient::init_logging`](super::client::CamundaClient::init_logging) (or
//! [`try_init`]) once at startup. It is a no-op if a global subscriber is already set, so
//! it never clobbers an application-provided one.

use super::config::LogLevel;

/// Install a formatting `tracing` subscriber filtered to `level`, unless a global
/// subscriber is already installed or `level` is [`LogLevel::Off`].
///
/// Returns `true` if this call installed the subscriber, `false` otherwise. Safe to call
/// multiple times.
pub fn try_init(level: LogLevel) -> bool {
    let Some(level) = level.to_tracing() else {
        return false;
    };
    use tracing_subscriber::fmt;
    fmt()
        .with_max_level(level)
        .with_target(true)
        .try_init()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_level_does_not_install() {
        assert!(!try_init(LogLevel::Off));
    }
}
