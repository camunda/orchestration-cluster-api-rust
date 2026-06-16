//! Ergonomic Rust SDK for the Camunda 8 Orchestration Cluster REST API.
//!
//! This crate is a hand-written runtime layered over the generated low-level client
//! (`camunda-orchestration-api-client`). It mirrors the JS / Python / C# SDKs:
//!
//! * **Zero-config construction** from environment variables, with programmatic overrides.
//! * **Authentication strategies**: `OAUTH` (client-credentials), `BASIC`, `NONE`.
//! * **Typed errors** ([`CamundaError`]).
//! * **Job workers** ([`JobWorker`]) — poll for jobs, run a handler, auto-complete/fail.
//! * Access to the full generated API surface via [`CamundaClient::configuration`].
//!
//! # Quick start
//!
//! ```no_run
//! use camunda_orchestration_sdk::CamundaClient;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! // Reads CAMUNDA_REST_ADDRESS, CAMUNDA_AUTH_STRATEGY, CAMUNDA_CLIENT_ID, ... from the env.
//! let client = CamundaClient::from_env()?;
//! let topology = client.topology().await?;
//! println!("cluster version: {:?}", topology.gateway_version);
//! # Ok(())
//! # }
//! ```
//!
//! See the `examples/` directory for worker and authentication examples.

mod runtime;

pub use runtime::auth::{AuthStrategy, Authentication};
pub use runtime::backpressure::{
    BackpressureManager, BackpressureProfile, BackpressureSeverity, BackpressureState,
};
pub use runtime::client::{CamundaClient, CamundaOptions};
pub use runtime::config::{CamundaConfig, LogLevel, RetryConfig, TlsConfig, WorkerDefaults};
pub use runtime::errors::{CamundaError, Result};
pub use runtime::eventual::ConsistencyOptions;
pub use runtime::job_worker::{
    Job, JobAction, JobHandler, JobWorker, JobWorkerConfig, JobWorkerHandle,
};

/// Eventual-consistency polling helpers ([`ConsistencyOptions`]).
pub use runtime::eventual;

/// Re-export of the generated low-level client (models, apis, configuration).
///
/// Use this for operations not yet wrapped by the ergonomic facade. Build a
/// [`camunda_orchestration_api_client::apis::configuration::Configuration`] with
/// [`CamundaClient::configuration`].
pub use camunda_orchestration_api_client as client;

/// The Camunda Domain Type System (semantic keys such as `JobKey`, `ProcessInstanceKey`).
pub use camunda_orchestration_api_client::models;
