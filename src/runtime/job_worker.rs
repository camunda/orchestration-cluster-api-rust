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
use super::clock::Clock;
use super::config::WorkerDefaults;
use super::errors::{CamundaError, Result};

/// Boxed, shareable job handler. You normally pass a closure to [`JobWorker::run`]
/// rather than constructing this directly.
pub type JobHandler =
    Arc<dyn Fn(Job) -> Pin<Box<dyn Future<Output = JobAction> + Send>> + Send + Sync>;

/// Callback invoked once, when a worker becomes ready to receive jobs — i.e. its
/// Falcon command-stream subscription has been established, or (when Falcon is
/// unavailable) its REST poll loop has been entered. Useful for readiness gates and
/// probes; the SDK guarantees it fires at most once per worker run.
pub type ReadyCallback = Arc<dyn Fn() + Send + Sync>;

/// Configuration for a [`JobWorker`].
///
/// Build with [`JobWorkerConfig::new`] and the builder methods rather than a struct
/// literal: the type is `#[non_exhaustive]`, so new fields are added without breaking
/// callers.
#[non_exhaustive]
#[derive(Clone)]
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
    /// Activate jobs with a lease. Each job then carries a lease token that the worker
    /// sends back on complete, fail, and throw-error, so the engine can fence the command
    /// against a superseded activation (for example after the job timed out and another
    /// worker picked it up). Off by default, matching the engine.
    ///
    /// Requires a server that supports job leases: rather than degrade to unfenced
    /// commands, a worker that asked for a lease and is handed a job without a token stops
    /// with [`CamundaError::LeaseNotHonored`]. Has no effect on jobs delivered over the
    /// Falcon command stream, which activates them outside the REST activation API.
    pub with_lease: bool,
    /// Optional callback fired once when the worker becomes ready to receive jobs
    /// (Falcon subscription established, or REST poll loop entered). Set via
    /// [`JobWorkerConfig::on_ready`]. Excluded from [`Debug`] output.
    pub on_ready: Option<ReadyCallback>,
}

impl std::fmt::Debug for JobWorkerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobWorkerConfig")
            .field("job_type", &self.job_type)
            .field("max_jobs_to_activate", &self.max_jobs_to_activate)
            .field("job_timeout_ms", &self.job_timeout_ms)
            .field("request_timeout_ms", &self.request_timeout_ms)
            .field("poll_interval_ms", &self.poll_interval_ms)
            .field("worker_name", &self.worker_name)
            .field("fetch_variables", &self.fetch_variables)
            .field("tenant_ids", &self.tenant_ids)
            .field(
                "startup_jitter_max_seconds",
                &self.startup_jitter_max_seconds,
            )
            .field("with_lease", &self.with_lease)
            .field("on_ready", &self.on_ready.as_ref().map(|_| "<callback>"))
            .finish()
    }
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
            with_lease: false,
            on_ready: None,
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
            with_lease: false,
            on_ready: None,
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

    /// Activate jobs with a lease (off by default). Each job then carries a lease token the
    /// worker sends back on complete, fail, and throw-error, so the engine can fence the
    /// command against a superseded activation. Requires a server that supports job leases;
    /// see [`JobWorkerConfig::with_lease`](Self::with_lease) for the fail-loud behaviour
    /// against one that does not.
    pub fn with_lease(mut self, enabled: bool) -> Self {
        self.with_lease = enabled;
        self
    }

    /// Register a callback fired once when this worker becomes ready to receive jobs
    /// (its Falcon subscription is established, or it has entered the REST poll loop).
    ///
    /// Useful for readiness gates — e.g. deferring load generation until every worker
    /// is actually subscribed. The callback runs on a Tokio worker thread, so keep it
    /// cheap and non-blocking (increment a counter, send on a channel, etc.).
    pub fn on_ready<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_ready = Some(Arc::new(f));
        self
    }
}

/// A job delivered to a worker handler. Wraps the activated job with convenient
/// accessors for variables and custom headers.
#[derive(Debug, Clone)]
pub struct Job {
    inner: models::ActivatedJobResult,
    clock: Arc<dyn Clock>,
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

    /// The lease token for this job, or `None` if it was not activated with a lease.
    ///
    /// Present exactly when the worker set [`JobWorkerConfig::with_lease`]; the worker
    /// threads it back onto the fenced commands automatically, so a handler rarely needs
    /// to read it directly.
    pub fn lease_token(&self) -> Option<&str> {
        self.inner.job_lease_token.as_ref().map(|t| t.value())
    }

    /// The clock this job's worker resolves cadence through. Handlers that need to wait
    /// must use this rather than `tokio::time::sleep`, so an injected clock controls them.
    ///
    /// For short in-handler coordination only -- spacing a retry, waiting for a resource to
    /// settle. A long or business wait belongs in the process as a BPMN timer event: a
    /// handler holding a job for minutes occupies a worker slot, risks the job timeout
    /// expiring underneath it, and hides the wait from the process model where it cannot be
    /// seen or changed.
    ///
    /// ```no_run
    /// # use camunda_orchestration_sdk::{Job, JobAction};
    /// # async fn handle(job: Job) -> JobAction {
    /// job.clock().sleep(std::time::Duration::from_secs(1)).await;
    /// JobAction::complete()
    /// # }
    /// ```
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
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

    /// Wrap an activated job for the handler. The single place a job's clock is chosen,
    /// so the REST and Falcon paths cannot drift apart on which clock handlers observe.
    fn wrap_job(&self, activated: models::ActivatedJobResult) -> Job {
        Job {
            inner: activated,
            clock: self.client.clock().clone(),
        }
    }

    /// Invoke the configured `on_ready` callback, if any. Each run path calls this
    /// exactly once at the moment the worker is ready to receive jobs.
    fn fire_ready(&self) {
        if let Some(cb) = &self.config.on_ready {
            cb();
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

    /// Spawn the worker loop on the Tokio runtime, returning a [`tokio::task::JoinHandle`].
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
            self.client
                .clock()
                .sleep(Duration::from_millis(delay))
                .await;
        }

        // Falcon upgrade: when the gateway advertises the command stream, take pushed
        // jobs over a WebSocket subscription instead of REST long-polling. When the
        // subscription cannot be established (e.g. a proxy blocks WebSockets), fall
        // back to the REST poll loop instead of failing the worker.
        if let Some(caps) = self.client.falcon_caps().await {
            let caps = caps.clone();
            match self.try_run_falcon_stream(handler.clone(), caps).await {
                Ok(()) => return Ok(()),
                Err(FalconStartError::SubscribeFailed(e)) => {
                    tracing::warn!(
                        job_type = %self.config.job_type,
                        error = %e,
                        "falcon subscribe failed; falling back to REST job polling",
                    );
                }
            }
        }

        self.run_rest_poll(handler).await
    }

    async fn run_rest_poll(self, handler: JobHandler) -> Result<()> {
        // Ready in the REST case = the poll loop is about to start (there is no
        // subscription handshake). Fires whether we're pure-REST or fell back from a
        // failed Falcon subscribe.
        self.fire_ready();
        let poll_interval = Duration::from_millis(self.config.poll_interval_ms);
        loop {
            if self.stop.load(Ordering::SeqCst) {
                return Ok(());
            }
            let jobs = self.poll().await?;
            if jobs.is_empty() {
                self.client.clock().sleep(poll_interval).await;
                continue;
            }

            let mut tasks = Vec::with_capacity(jobs.len());
            for activated in jobs {
                let job = self.wrap_job(activated);
                let client = self.client.clone();
                let handler = handler.clone();
                tasks.push(tokio::spawn(async move {
                    let key = job.key().to_string();
                    // Capture the lease token before the handler consumes the job, so the
                    // fenced command can present it back and be accepted for a leased job.
                    let lease_token = job.lease_token().map(str::to_owned);
                    let action = handler(job).await;
                    apply_action(&client, &key, lease_token.as_deref(), action).await
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
        let request = self.build_activation_request();
        let result = self.client.activate_jobs(request).await?;
        enforce_lease_presence(self.config.with_lease, &result.jobs)?;
        Ok(result.jobs)
    }

    /// Build the activate-jobs request from the worker config. Split out so the request
    /// shape — notably the lease flag — is unit-testable without a live server.
    fn build_activation_request(&self) -> models::JobActivationRequest {
        // Fall back to the SDK's configured default tenant when none is set on the worker.
        let tenant_ids = self.config.tenant_ids.clone().or_else(|| {
            self.client
                .config()
                .default_tenant_id
                .clone()
                .map(|id| vec![id])
        });
        models::JobActivationRequest {
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
            with_lease: if self.config.with_lease {
                Some(Some(true))
            } else {
                None
            },
        }
    }

    /// Attempt to run the falcon push-based worker loop. Returns a
    /// `FalconStartError` when the initial subscribe fails so the caller can
    /// transparently fall back to REST polling.
    async fn try_run_falcon_stream(
        &self,
        handler: JobHandler,
        caps: super::falcon::FalconCaps,
    ) -> std::result::Result<(), FalconStartError> {
        let worker = match super::falcon::FalconStreamWorker::subscribe(
            caps.endpoints.clone(),
            &self.config.job_type,
            self.config.max_jobs_to_activate as i64,
            self.config.fetch_variables.clone(),
            Some(self.config.job_timeout_ms.max(0) as u64),
            Some(self.config.worker_name.clone()),
        )
        .await
        {
            Ok(w) => Arc::new(w),
            Err(e) => return Err(FalconStartError::SubscribeFailed(e)),
        };

        // Ready in the Falcon case = the subscription handshake completed, so the
        // gateway will now push jobs to this worker. Fire before entering the loop.
        self.fire_ready();

        loop {
            if self.stop.load(Ordering::SeqCst) {
                return Ok(());
            }
            let Some(activated) = worker.next_job(Duration::from_millis(500)).await else {
                continue;
            };
            let job = self.wrap_job(activated);
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

/// Falcon worker startup outcomes distinguishable from normal loop errors.
enum FalconStartError {
    /// The initial WebSocket subscribe/handshake failed. The caller falls back
    /// to REST polling.
    SubscribeFailed(CamundaError),
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

async fn apply_action(
    client: &CamundaClient,
    job_key: &str,
    lease_token: Option<&str>,
    action: JobAction,
) -> Result<()> {
    match action {
        JobAction::Leave => Ok(()),
        JobAction::Complete { variables } => {
            let request = build_completion_request(variables, lease_token);
            client.complete_job(job_key, request).await
        }
        JobAction::Fail {
            error_message,
            retries,
            retry_backoff_ms,
            variables,
        } => {
            let request = build_fail_request(
                error_message,
                retries,
                retry_backoff_ms,
                variables,
                lease_token,
            );
            client.fail_job(job_key, Some(request)).await
        }
        JobAction::Error {
            error_code,
            error_message,
            variables,
        } => {
            let request = build_error_request(error_code, error_message, variables, lease_token);
            client.throw_job_error(job_key, request).await
        }
    }
}

/// Brand a lease token for a generated request's `Option<Option<JobLeaseToken>>` field:
/// `Some(Some(token))` when leased, `None` otherwise.
fn lease_token_field(lease_token: Option<&str>) -> Option<Option<models::JobLeaseToken>> {
    lease_token.map(|t| Some(models::JobLeaseToken::assume_exists(t.to_string())))
}

/// Reject a whole activation when a requested lease was not honoured for any job, on the
/// existing error path — before any job reaches a handler that would finish it unfenced.
fn enforce_lease_presence(with_lease: bool, jobs: &[models::ActivatedJobResult]) -> Result<()> {
    for job in jobs {
        super::present_when::require_lease_presence(
            with_lease,
            job.job_key.value(),
            job.job_lease_token.as_ref().map(|t| t.value()),
        )?;
    }
    Ok(())
}

/// Build the optional completion body. A completion needs a body only when it carries
/// output variables or a lease token to present back; without either, `None` lets the
/// worker complete with an empty request.
fn build_completion_request(
    variables: Option<Value>,
    lease_token: Option<&str>,
) -> Option<models::JobCompletionRequest> {
    if variables.is_none() && lease_token.is_none() {
        return None;
    }
    Some(models::JobCompletionRequest {
        variables: variables.map(value_to_map),
        result: None,
        job_lease_token: lease_token_field(lease_token),
        business_id: None,
    })
}

/// Build a fail-job request, threading the lease token so the engine can fence it.
fn build_fail_request(
    error_message: String,
    retries: Option<i32>,
    retry_backoff_ms: Option<i64>,
    variables: Option<Value>,
    lease_token: Option<&str>,
) -> models::JobFailRequest {
    models::JobFailRequest {
        retries,
        error_message: Some(error_message),
        retry_back_off: retry_backoff_ms,
        variables: variables.and_then(value_to_map),
        job_lease_token: lease_token_field(lease_token),
    }
}

/// Build a throw-BPMN-error request, threading the lease token so the engine can fence it.
fn build_error_request(
    error_code: String,
    error_message: Option<String>,
    variables: Option<Value>,
    lease_token: Option<&str>,
) -> models::JobErrorRequest {
    models::JobErrorRequest {
        error_code,
        error_message: Some(error_message),
        variables: Some(value_to_map(variables.unwrap_or(Value::Null))),
        job_lease_token: lease_token_field(lease_token),
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

    /// A client pointed at a local address, enough to construct workers for building and
    /// inspecting requests without any network I/O.
    fn test_client() -> super::super::client::CamundaClient {
        super::super::client::CamundaClient::new(
            super::super::client::CamundaOptions::new()
                .with("CAMUNDA_REST_ADDRESS", "http://localhost:8080"),
        )
        .expect("client should build from an address alone")
    }

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

    #[test]
    fn with_lease_defaults_off_and_builder_sets_it() {
        assert!(!JobWorkerConfig::new("t").with_lease);
        assert!(JobWorkerConfig::new("t").with_lease(true).with_lease);
        assert!(!JobWorkerConfig::new("t").with_lease(false).with_lease);
    }

    #[test]
    fn activation_request_opts_into_lease_only_when_asked() {
        let client = test_client();
        let unleased = client
            .create_job_worker(JobWorkerConfig::new("t"))
            .build_activation_request();
        assert_eq!(unleased.with_lease, None);

        let leased = client
            .create_job_worker(JobWorkerConfig::new("t").with_lease(true))
            .build_activation_request();
        assert_eq!(leased.with_lease, Some(Some(true)));
    }

    #[test]
    fn lease_token_is_threaded_into_every_fenced_command() {
        let expected = Some(Some(models::JobLeaseToken::assume_exists("tok")));

        // Complete: a body is built for the token alone, even with no variables.
        let complete = build_completion_request(None, Some("tok"));
        assert_eq!(complete.and_then(|r| r.job_lease_token), expected);

        // Fail: the token rides the fail request.
        let fail = build_fail_request("boom".into(), None, None, None, Some("tok"));
        assert_eq!(fail.job_lease_token, expected);

        // Error: the token rides the throw-error request.
        let error = build_error_request("E1".into(), None, None, Some("tok"));
        assert_eq!(error.job_lease_token, expected);
    }

    #[test]
    fn no_lease_token_leaves_commands_unbranded() {
        // No variables and no token => no completion body at all (unchanged behaviour).
        assert!(build_completion_request(None, None).is_none());
        assert_eq!(lease_token_field(None), None);
        // Variables but no token => body present, token field empty.
        let complete = build_completion_request(Some(serde_json::json!({ "k": 1 })), None);
        assert_eq!(complete.and_then(|r| r.job_lease_token), None);
        // Fail and error unbranded when no lease.
        assert_eq!(
            build_fail_request("boom".into(), None, None, None, None).job_lease_token,
            None
        );
        assert_eq!(
            build_error_request("E1".into(), None, None, None).job_lease_token,
            None
        );
    }

    #[test]
    fn on_ready_builder_stores_callback() {
        let c = JobWorkerConfig::new("t");
        assert!(c.on_ready.is_none());
        let c = c.on_ready(|| {});
        assert!(c.on_ready.is_some());
    }

    #[test]
    fn on_ready_callback_is_invoked() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let hits = Arc::new(AtomicUsize::new(0));
        let hits2 = hits.clone();
        let c = JobWorkerConfig::new("t").on_ready(move || {
            hits2.fetch_add(1, Ordering::SeqCst);
        });
        // Invoke the stored callback the way the run loop would.
        (c.on_ready.as_ref().unwrap())();
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn debug_omits_callback_body() {
        let c = JobWorkerConfig::new("t").on_ready(|| {});
        let s = format!("{c:?}");
        assert!(s.contains("on_ready"));
        assert!(s.contains("<callback>"));
    }

    /// Build an activated job. Only the clock wiring is under test, so the field values
    /// are arbitrary.
    fn fake_activated() -> models::ActivatedJobResult {
        fake_activated_with_lease(None)
    }

    /// Build an activated job carrying the given lease token (or none), for exercising the
    /// activation-boundary lease guard.
    fn fake_activated_with_lease(token: Option<&str>) -> models::ActivatedJobResult {
        models::ActivatedJobResult::new(
            "test-type".to_string(),
            models::ProcessDefinitionId::assume_exists("proc".to_string()),
            1,
            models::ElementId::assume_exists("element".to_string()),
            HashMap::new(),
            "worker".to_string(),
            3,
            0,
            HashMap::new(),
            models::TenantId::assume_exists("<default>".to_string()),
            "physical".to_string(),
            models::JobKey::assume_exists("1".to_string()),
            models::ProcessInstanceKey::assume_exists("2".to_string()),
            models::ProcessDefinitionKey::assume_exists("3".to_string()),
            models::ElementInstanceKey::assume_exists("4".to_string()),
            models::JobKindEnum::BpmnElement,
            models::JobListenerEventTypeEnum::Assigning,
            None,
            Vec::new(),
            None,
            None,
            0,
            token.map(models::JobLeaseToken::assume_exists),
        )
    }

    #[test]
    fn enforce_lease_presence_rejects_only_an_unhonored_lease() {
        let leased = [fake_activated_with_lease(Some("tok"))];
        let unleased = [fake_activated_with_lease(None)];

        // Not requested: neither shape is rejected.
        assert!(enforce_lease_presence(false, &unleased).is_ok());
        assert!(enforce_lease_presence(false, &leased).is_ok());
        // Requested and honoured: accepted.
        assert!(enforce_lease_presence(true, &leased).is_ok());
        // Requested but a tokenless job is present: the whole activation is rejected.
        assert!(matches!(
            enforce_lease_presence(true, &unleased),
            Err(CamundaError::LeaseNotHonored { .. })
        ));
        // A single unhonoured job in an otherwise-leased batch still fails.
        let mixed = [
            fake_activated_with_lease(Some("tok")),
            fake_activated_with_lease(None),
        ];
        assert!(enforce_lease_presence(true, &mixed).is_err());
    }

    /// A handler that waits must be controllable by an injected clock. That only holds if
    /// the job carries the *client's* clock -- handing it `live_clock()` would compile,
    /// pass every other test, and silently leave handlers on real time.
    #[test]
    fn a_job_carries_the_clients_clock() {
        let clock: Arc<dyn Clock> = Arc::new(super::super::clock::RecordingClock::default());
        let client = super::super::client::CamundaClient::new(
            super::super::client::CamundaOptions::new()
                .with("CAMUNDA_REST_ADDRESS", "http://localhost:8080")
                .with_clock(clock.clone()),
        )
        .expect("client should build from an address alone");

        let worker = client.create_job_worker(JobWorkerConfig::new("test-type"));
        let job = worker.wrap_job(fake_activated());

        assert!(
            Arc::ptr_eq(job.clock(), &clock),
            "the job was handed a different clock than the client's"
        );
    }

    /// `wrap_job` is the only place a job's clock is chosen. A second construction site
    /// could wire the REST path and leave Falcon on real time -- the half-injected failure
    /// this contract exists to prevent -- and no behavioural test would catch it, because
    /// the Falcon path needs a live gateway to reach. Guard it structurally instead.
    #[test]
    fn jobs_are_only_ever_built_by_wrap_job() {
        let source = include_str!("job_worker.rs");
        let constructions = count_job_constructions(source);
        assert_eq!(
            constructions, 1,
            "found {constructions} `Job {{ inner: .. }}` construction sites; expected exactly \
             one, inside `wrap_job`. Route the new site through `wrap_job` so every delivery \
             path hands handlers the same clock."
        );
    }

    /// Counts `Job { inner: .. }` struct literals, tolerating any whitespace before the
    /// field. Skips the `struct Job {` definition, which has the same shape. Kept
    /// dependency-free -- the crate has no regex dev-dep.
    fn count_job_constructions(source: &str) -> usize {
        // Production code only: the test module discusses this pattern in prose, and
        // counting itself would make the guard fail on its own documentation. Matching
        // "mod tests" alone tolerates brace and whitespace reformatting; if it ever stops
        // matching, the guard over-counts and fails loudly rather than going quiet.
        let production = source
            .split_once("mod tests")
            .map_or(source, |(before, _)| before);
        production
            .match_indices("Job {")
            .filter(|(i, _)| !production[..*i].ends_with("struct "))
            .filter(|(i, _)| {
                production[i + "Job {".len()..]
                    .trim_start()
                    .starts_with("inner:")
            })
            .count()
    }
}
