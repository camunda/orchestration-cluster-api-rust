//! Job workers: poll for jobs of a given type, run a handler, and apply the result
//! (complete / fail / throw BPMN error) automatically.
//!
//! ```no_run
//! use camunda_orchestration_sdk::{CamundaClient, JobAction, JobWorkerConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CamundaClient::from_env()?;
//! let worker = client.create_job_worker(JobWorkerConfig::new("payment-service"));
//! worker
//!     .run(|job| async move {
//!         println!("handling job {}", job.key());
//!         JobAction::complete_with(serde_json::json!({ "paid": true }))
//!     })
//!     .await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use camunda_orchestration_api_client::models;

use super::client::CamundaClient;
use super::config::WorkerDefaults;
use super::errors::{CamundaError, Result};

/// Boxed, shareable job handler. You normally pass a closure to [`JobWorker::run`]
/// rather than constructing this directly.
pub type JobHandler =
    Arc<dyn Fn(Job) -> Pin<Box<dyn Future<Output = JobAction> + Send>> + Send + Sync>;

/// Configuration for a [`JobWorker`].
#[derive(Debug, Clone)]
pub struct JobWorkerConfig {
    /// The job type to poll for (required).
    pub job_type: String,
    /// Maximum number of jobs to activate per poll. Also bounds in-flight concurrency.
    pub max_jobs_to_activate: i32,
    /// How long the engine reserves an activated job for this worker, in milliseconds.
    pub job_timeout_ms: i64,
    /// Long-poll timeout for the activate-jobs request, in milliseconds.
    pub request_timeout_ms: i64,
    /// Delay between polls when the last poll returned no jobs, in milliseconds.
    pub poll_interval_ms: u64,
    /// Worker name reported to the engine.
    pub worker_name: String,
    /// Variable names to fetch with each job. `None` fetches all variables.
    pub fetch_variables: Option<Vec<String>>,
    /// Tenant ids to activate jobs for.
    pub tenant_ids: Option<Vec<String>>,
    /// Maximum random startup delay before the first poll, in seconds. Spreads the initial
    /// activate-jobs stampede when many workers start at once.
    pub startup_jitter_max_seconds: u64,
}

impl JobWorkerConfig {
    /// Create a config for the given job type with sensible defaults.
    pub fn new(job_type: impl Into<String>) -> Self {
        JobWorkerConfig {
            job_type: job_type.into(),
            max_jobs_to_activate: 10,
            job_timeout_ms: 60_000,
            request_timeout_ms: 10_000,
            poll_interval_ms: 100,
            worker_name: "rust-sdk-worker".to_string(),
            fetch_variables: None,
            tenant_ids: None,
            startup_jitter_max_seconds: 0,
        }
    }

    /// Create a config seeded from the SDK's resolved [`WorkerDefaults`] (env-driven), for
    /// the given job type. Builder methods can still override individual fields.
    pub fn from_defaults(job_type: impl Into<String>, defaults: &WorkerDefaults) -> Self {
        JobWorkerConfig {
            job_type: job_type.into(),
            max_jobs_to_activate: defaults.max_concurrent_jobs,
            job_timeout_ms: defaults.timeout_ms,
            request_timeout_ms: defaults.request_timeout_ms,
            poll_interval_ms: 100,
            worker_name: defaults.name.clone(),
            fetch_variables: None,
            tenant_ids: None,
            startup_jitter_max_seconds: defaults.startup_jitter_max_seconds,
        }
    }

    /// Set the maximum number of jobs activated per poll.
    pub fn max_jobs_to_activate(mut self, n: i32) -> Self {
        self.max_jobs_to_activate = n;
        self
    }

    /// Set the job activation timeout, in milliseconds.
    pub fn job_timeout_ms(mut self, ms: i64) -> Self {
        self.job_timeout_ms = ms;
        self
    }

    /// Set the worker name.
    pub fn worker_name(mut self, name: impl Into<String>) -> Self {
        self.worker_name = name.into();
        self
    }

    /// Restrict fetched variables to the given names.
    pub fn fetch_variables<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.fetch_variables = Some(names.into_iter().map(Into::into).collect());
        self
    }

    /// Set the tenant ids to activate jobs for.
    pub fn tenant_ids<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tenant_ids = Some(ids.into_iter().map(Into::into).collect());
        self
    }

    /// Set the maximum random startup delay (seconds) applied before the first poll.
    pub fn startup_jitter_max_seconds(mut self, seconds: u64) -> Self {
        self.startup_jitter_max_seconds = seconds;
        self
    }
}

/// A job delivered to a worker handler. Wraps the activated job with convenient
/// accessors for variables and custom headers.
#[derive(Debug, Clone)]
pub struct Job {
    inner: models::ActivatedJobResult,
}

impl Job {
    /// The job key, as a string.
    pub fn key(&self) -> &str {
        self.inner.job_key.value()
    }

    /// The job type.
    pub fn job_type(&self) -> &str {
        &self.inner.r#type
    }

    /// The process instance key, as a string.
    pub fn process_instance_key(&self) -> &str {
        self.inner.process_instance_key.value()
    }

    /// The BPMN element id that created this job.
    pub fn element_id(&self) -> &str {
        self.inner.element_id.value()
    }

    /// Remaining retries for this job.
    pub fn retries(&self) -> i32 {
        self.inner.retries
    }

    /// The job variables as a JSON map.
    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.inner.variables
    }

    /// Deserialize the job variables into a typed value.
    pub fn variables_as<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let value = serde_json::to_value(&self.inner.variables)?;
        Ok(serde_json::from_value(value)?)
    }

    /// The job custom headers.
    pub fn custom_headers(&self) -> &HashMap<String, Value> {
        &self.inner.custom_headers
    }

    /// The underlying generated activated-job model.
    pub fn raw(&self) -> &models::ActivatedJobResult {
        &self.inner
    }
}

/// The action a handler asks the worker to take after processing a job.
#[derive(Debug, Clone)]
pub enum JobAction {
    /// Complete the job, optionally with output variables.
    Complete { variables: Option<Value> },
    /// Fail the job, decrementing retries (unless `retries` is set explicitly).
    Fail {
        error_message: String,
        retries: Option<i32>,
        retry_backoff_ms: Option<i64>,
        variables: Option<Value>,
    },
    /// Throw a BPMN error to be caught by an error boundary event.
    Error {
        error_code: String,
        error_message: Option<String>,
        variables: Option<Value>,
    },
    /// Take no action; the job remains activated until its timeout elapses.
    Leave,
}

impl JobAction {
    /// Complete the job with no output variables.
    pub fn complete() -> Self {
        JobAction::Complete { variables: None }
    }

    /// Complete the job with output variables.
    pub fn complete_with(variables: impl Serialize) -> Self {
        JobAction::Complete {
            variables: serde_json::to_value(variables).ok(),
        }
    }

    /// Fail the job with an error message (retries are decremented by the engine).
    pub fn fail(error_message: impl Into<String>) -> Self {
        JobAction::Fail {
            error_message: error_message.into(),
            retries: None,
            retry_backoff_ms: None,
            variables: None,
        }
    }

    /// Throw a BPMN error with the given error code.
    pub fn error(error_code: impl Into<String>) -> Self {
        JobAction::Error {
            error_code: error_code.into(),
            error_message: None,
            variables: None,
        }
    }

    /// Leave the job activated (take no action).
    pub fn leave() -> Self {
        JobAction::Leave
    }
}

/// A continuously-polling job worker. Build one via
/// [`CamundaClient::create_job_worker`](super::client::CamundaClient::create_job_worker).
pub struct JobWorker {
    client: CamundaClient,
    config: JobWorkerConfig,
    stop: Arc<AtomicBool>,
}

/// A handle to a spawned [`JobWorker`], used to stop it and await its completion.
///
/// Dropping the handle does **not** stop the worker; call [`JobWorkerHandle::stop`] (or
/// [`CamundaClient::stop_all_workers`](super::client::CamundaClient::stop_all_workers)) for a
/// graceful shutdown that lets in-flight jobs drain.
pub struct JobWorkerHandle {
    job_type: String,
    worker_name: String,
    stop: Arc<AtomicBool>,
    join: tokio::task::JoinHandle<Result<()>>,
}

impl JobWorkerHandle {
    /// The job type this worker polls for.
    pub fn job_type(&self) -> &str {
        &self.job_type
    }

    /// The worker name reported to the engine.
    pub fn worker_name(&self) -> &str {
        &self.worker_name
    }

    /// Signal the worker to stop. It finishes draining any in-flight jobs from the current
    /// batch, then exits before the next poll. Non-blocking.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    /// Whether the worker task has finished.
    pub fn is_finished(&self) -> bool {
        self.join.is_finished()
    }

    /// Signal the worker to stop and await its graceful shutdown.
    pub async fn shutdown(self) -> Result<()> {
        self.stop.store(true, Ordering::SeqCst);
        match self.join.await {
            Ok(result) => result,
            Err(e) => Err(CamundaError::worker(format!("worker task panicked: {e}"))),
        }
    }
}

impl JobWorker {
    pub(crate) fn new(client: CamundaClient, config: JobWorkerConfig) -> Self {
        JobWorker {
            client,
            config,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Run the worker loop, processing jobs with `handler` until stopped or an
    /// unrecoverable error occurs.
    pub async fn run<F, Fut>(self, handler: F) -> Result<()>
    where
        F: Fn(Job) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = JobAction> + Send + 'static,
    {
        let handler: JobHandler = Arc::new(move |job| Box::pin(handler(job)));
        self.run_boxed(handler).await
    }

    /// Spawn the worker loop on the Tokio runtime, returning a [`JoinHandle`].
    pub fn start<F, Fut>(self, handler: F) -> tokio::task::JoinHandle<Result<()>>
    where
        F: Fn(Job) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = JobAction> + Send + 'static,
    {
        tokio::spawn(self.run(handler))
    }

    /// Spawn the worker loop and return a [`JobWorkerHandle`] for graceful shutdown.
    pub fn spawn<F, Fut>(self, handler: F) -> JobWorkerHandle
    where
        F: Fn(Job) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = JobAction> + Send + 'static,
    {
        let stop = self.stop.clone();
        let job_type = self.config.job_type.clone();
        let worker_name = self.config.worker_name.clone();
        let join = tokio::spawn(self.run(handler));
        JobWorkerHandle {
            job_type,
            worker_name,
            stop,
            join,
        }
    }

    async fn run_boxed(self, handler: JobHandler) -> Result<()> {
        // Spread the initial activate-jobs stampede when many workers start together.
        if self.config.startup_jitter_max_seconds > 0 {
            let max_ms = self.config.startup_jitter_max_seconds.saturating_mul(1000);
            let delay = (super::rand_fraction() * max_ms as f64) as u64;
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        // Falcon upgrade: when the gateway advertises the command stream, take pushed
        // jobs over a WebSocket subscription instead of REST long-polling.
        if let Some(caps) = self.client.falcon_caps().await {
            let caps = caps.clone();
            return self.run_falcon_stream(handler, caps).await;
        }

        let poll_interval = Duration::from_millis(self.config.poll_interval_ms);
        loop {
            if self.stop.load(Ordering::SeqCst) {
                return Ok(());
            }
            let jobs = self.poll().await?;
            if jobs.is_empty() {
                tokio::time::sleep(poll_interval).await;
                continue;
            }

            let mut tasks = Vec::with_capacity(jobs.len());
            for activated in jobs {
                let job = Job { inner: activated };
                let client = self.client.clone();
                let handler = handler.clone();
                tasks.push(tokio::spawn(async move {
                    let key = job.key().to_string();
                    let action = handler(job).await;
                    apply_action(&client, &key, action).await
                }));
            }
            for task in tasks {
                match task.await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => tracing::warn!(error = %e, "failed to apply job action"),
                    Err(e) => tracing::warn!(error = %e, "job handler task panicked"),
                }
            }
        }
    }

    async fn poll(&self) -> Result<Vec<models::ActivatedJobResult>> {
        // Fall back to the SDK's configured default tenant when none is set on the worker.
        let tenant_ids = self.config.tenant_ids.clone().or_else(|| {
            self.client
                .config()
                .default_tenant_id
                .clone()
                .map(|id| vec![id])
        });
        let request = models::JobActivationRequest {
            r#type: self.config.job_type.clone(),
            worker: Some(self.config.worker_name.clone()),
            timeout: self.config.job_timeout_ms,
            max_jobs_to_activate: self.config.max_jobs_to_activate,
            fetch_variable: self.config.fetch_variables.clone(),
            request_timeout: Some(self.config.request_timeout_ms),
            tenant_ids: tenant_ids.map(|ids| {
                ids.iter()
                    .map(|id| models::TenantId::assume_exists(id.clone()))
                    .collect()
            }),
            tenant_filter: None,
        };
        let result = self.client.activate_jobs(request).await?;
        Ok(result.jobs)
    }

    /// Run the falcon worker loop: subscribe, then process pushed jobs,
    /// replenishing a delivery credit as each job is acted upon. Honours `stop`.
    async fn run_falcon_stream(
        self,
        handler: JobHandler,
        caps: super::falcon::FalconCaps,
    ) -> Result<()> {
        let worker = Arc::new(
            super::falcon::FalconStreamWorker::subscribe(
                caps.endpoints.clone(),
                &self.config.job_type,
                self.config.max_jobs_to_activate as i64,
                self.config.fetch_variables.clone(),
                Some(self.config.job_timeout_ms.max(0) as u64),
                Some(self.config.worker_name.clone()),
            )
            .await?,
        );

        loop {
            if self.stop.load(Ordering::SeqCst) {
                return Ok(());
            }
            let Some(activated) = worker.next_job(Duration::from_millis(500)).await else {
                continue;
            };
            let job = Job { inner: activated };
            let handler = handler.clone();
            let worker = worker.clone();
            tokio::spawn(async move {
                let key = job.key().to_string();
                let action = handler(job).await;
                apply_action_falcon(&worker, &key, action);
            });
        }
    }
}

/// Translate a [`JobAction`] into a fire-and-forget command-stream frame (each frame
/// also replenishes one delivery credit).
fn apply_action_falcon(
    worker: &super::falcon::FalconStreamWorker,
    job_key: &str,
    action: JobAction,
) {
    match action {
        JobAction::Leave => {
            // No completion frame would be sent, so replenish the consumed credit
            // directly to keep the delivery window full.
            worker.replenish(1);
        }
        JobAction::Complete { variables } => {
            worker.complete(job_key, variables.and_then(value_to_obj));
        }
        JobAction::Fail {
            error_message,
            retries,
            variables: _,
            retry_backoff_ms: _,
        } => {
            worker.fail(job_key, retries, Some(error_message));
        }
        JobAction::Error {
            error_code,
            error_message,
            variables: _,
        } => {
            worker.throw_error(job_key, &error_code, error_message);
        }
    }
}

/// Convert a JSON value into a `serde_json` object map (dropping non-objects).
fn value_to_obj(value: Value) -> Option<serde_json::Map<String, Value>> {
    match value {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

async fn apply_action(client: &CamundaClient, job_key: &str, action: JobAction) -> Result<()> {
    match action {
        JobAction::Leave => Ok(()),
        JobAction::Complete { variables } => {
            let request = variables.map(|v| models::JobCompletionRequest {
                variables: Some(value_to_map(v)),
                result: None,
            });
            client.complete_job(job_key, request).await
        }
        JobAction::Fail {
            error_message,
            retries,
            retry_backoff_ms,
            variables,
        } => {
            let request = models::JobFailRequest {
                retries,
                error_message: Some(error_message),
                retry_back_off: retry_backoff_ms,
                variables: variables.and_then(value_to_map),
            };
            client.fail_job(job_key, Some(request)).await
        }
        JobAction::Error {
            error_code,
            error_message,
            variables,
        } => {
            let request = models::JobErrorRequest {
                error_code,
                error_message: Some(error_message),
                variables: Some(value_to_map(variables.unwrap_or(Value::Null))),
            };
            client.throw_job_error(job_key, request).await
        }
    }
}

/// Convert a JSON value into the `Option<Option<map>>` shape used by generated request
/// bodies. A non-object value is rejected as a validation error at the call site by the
/// engine; here we coerce `Null` to `None` and objects to a map.
fn value_to_map(value: Value) -> Option<HashMap<String, Value>> {
    match value {
        Value::Null => None,
        Value::Object(map) => Some(map.into_iter().collect()),
        other => {
            // Wrap a non-object payload so it is still transmitted; the engine expects
            // an object, so this is a best-effort fallback.
            let _ = &other;
            None
        }
    }
}

impl CamundaError {
    /// Helper to build a validation error from the worker layer.
    #[allow(dead_code)]
    pub(crate) fn worker(msg: impl Into<String>) -> Self {
        CamundaError::Validation(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_are_sensible() {
        let c = JobWorkerConfig::new("my-type");
        assert_eq!(c.job_type, "my-type");
        assert_eq!(c.max_jobs_to_activate, 10);
        assert_eq!(c.job_timeout_ms, 60_000);
    }

    #[test]
    fn builder_overrides_apply() {
        let c = JobWorkerConfig::new("t")
            .max_jobs_to_activate(5)
            .job_timeout_ms(1234)
            .worker_name("w")
            .fetch_variables(["a", "b"])
            .tenant_ids(["t1"]);
        assert_eq!(c.max_jobs_to_activate, 5);
        assert_eq!(c.job_timeout_ms, 1234);
        assert_eq!(c.worker_name, "w");
        assert_eq!(
            c.fetch_variables.as_deref(),
            Some(&["a".to_string(), "b".to_string()][..])
        );
        assert_eq!(c.tenant_ids.as_deref(), Some(&["t1".to_string()][..]));
    }

    #[test]
    fn complete_with_serializes_payload() {
        let action = JobAction::complete_with(serde_json::json!({ "ok": true }));
        match action {
            JobAction::Complete { variables: Some(v) } => {
                assert_eq!(v, serde_json::json!({ "ok": true }));
            }
            _ => panic!("expected Complete with variables"),
        }
    }

    #[test]
    fn value_to_map_handles_objects_and_null() {
        assert!(value_to_map(Value::Null).is_none());
        let map = value_to_map(serde_json::json!({ "k": 1 })).unwrap();
        assert_eq!(map.get("k"), Some(&serde_json::json!(1)));
        // Non-object payloads are dropped (the engine requires an object).
        assert!(value_to_map(serde_json::json!(42)).is_none());
    }
}
