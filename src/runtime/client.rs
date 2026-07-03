//! The ergonomic [`CamundaClient`] facade over the generated low-level client.

use std::collections::HashMap;
use std::sync::Arc;

use camunda_orchestration_api_client::apis::configuration::Configuration;
use camunda_orchestration_api_client::apis::{
    cluster_api, decision_definition_api, job_api, message_api, process_instance_api, resource_api,
    signal_api, variable_api,
};
use camunda_orchestration_api_client::models;

use super::auth::Authentication;
use super::backpressure::{is_backpressure_error, BackpressureManager, BackpressureState};
use super::config::CamundaConfig;
use super::errors::{CamundaError, Result};
use super::eventual::ConsistencyOptions;
use super::job_worker::{Job, JobAction, JobWorker, JobWorkerConfig, JobWorkerHandle};
use super::nano::{NanoCaps, NanoProducer};
use super::{retry, tls};

/// Lazily-resolved nanobpmn falcon state, shared across client clones.
#[derive(Clone, Default)]
pub(crate) struct NanoState {
    /// `None` until first probe; `Some(None)` = stock Camunda, `Some(Some(_))` = nano.
    caps: Arc<tokio::sync::OnceCell<Option<NanoCaps>>>,
    /// Shared create producer, built on first nano create.
    producer: Arc<tokio::sync::OnceCell<Arc<NanoProducer>>>,
}

/// Options for constructing a [`CamundaClient`].
#[derive(Default)]
pub struct CamundaOptions {
    /// Programmatic overrides for `CAMUNDA_*` configuration keys. These take precedence
    /// over environment variables.
    pub config: HashMap<String, String>,
    /// A pre-built `reqwest::Client` to use for all requests (including OAuth token
    /// fetches). When `None`, a default client is created.
    pub http_client: Option<reqwest::Client>,
}

impl CamundaOptions {
    /// Create empty options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single `CAMUNDA_*` configuration override.
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }

    /// Use a custom `reqwest::Client`.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }
}

/// The primary entry point of the SDK.
///
/// A `CamundaClient` is cheap to clone — clones share the same configuration,
/// HTTP client, OAuth token cache, and worker registry.
#[derive(Clone)]
pub struct CamundaClient {
    config: CamundaConfig,
    auth: Authentication,
    http: reqwest::Client,
    user_agent: String,
    bp: Arc<BackpressureManager>,
    workers: Arc<std::sync::Mutex<Vec<JobWorkerHandle>>>,
    nano: NanoState,
}

impl CamundaClient {
    /// Construct a client from environment variables only.
    pub fn from_env() -> Result<Self> {
        Self::new(CamundaOptions::new())
    }

    /// Construct a client from [`CamundaOptions`] (environment + overrides).
    pub fn new(options: CamundaOptions) -> Result<Self> {
        let config = CamundaConfig::from_env_with_overrides(&options.config)?;
        let http = match options.http_client {
            Some(client) => client,
            None => tls::apply_tls(reqwest::Client::builder(), &config.tls)?
                .build()
                .map_err(CamundaError::Network)?,
        };
        let auth = Authentication::from_config(&config, http.clone());
        let bp = Arc::new(BackpressureManager::new(config.backpressure_profile));
        Ok(CamundaClient {
            config,
            auth,
            http,
            user_agent: format!(
                "camunda-orchestration-sdk-rust/{}",
                env!("CARGO_PKG_VERSION")
            ),
            bp,
            workers: Arc::new(std::sync::Mutex::new(Vec::new())),
            nano: NanoState::default(),
        })
    }

    /// The resolved configuration.
    pub fn config(&self) -> &CamundaConfig {
        &self.config
    }

    /// The authentication handler.
    pub fn auth(&self) -> &Authentication {
        &self.auth
    }

    /// Install a formatting `tracing` subscriber filtered to the configured
    /// `CAMUNDA_SDK_LOG_LEVEL`. No-op if a global subscriber is already set or logging is
    /// off. Returns `true` if this call installed the subscriber.
    pub fn init_logging(&self) -> bool {
        super::logging::try_init(self.config.log_level)
    }

    /// A snapshot of the adaptive backpressure controller's state, for observability.
    pub fn backpressure_state(&self) -> BackpressureState {
        self.bp.state()
    }

    /// Apply the configured default tenant id to an optional tenant value, leaving an
    /// explicitly-provided value untouched.
    pub(crate) fn with_default_tenant(
        &self,
        provided: Option<models::TenantId>,
    ) -> Option<models::TenantId> {
        provided.or_else(|| {
            self.config
                .default_tenant_id
                .clone()
                .map(models::TenantId::assume_exists)
        })
    }

    /// Run an initiating operation under the adaptive backpressure gate AND the transient
    /// retry policy: acquire a permit, await the operation (retrying transient failures),
    /// then record the outcome and release the permit.
    ///
    /// Drain operations (job completion / failure / BPMN error) bypass this gate so that
    /// in-flight work can always be drained even while the cluster is shedding new load.
    pub(crate) async fn guarded<T, F, Fut>(&self, op: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.bp.acquire().await?;
        let result = retry::with_retry(&self.config.retry, &op).await;
        match &result {
            Ok(_) => self.bp.record_healthy_hint(),
            Err(e) if is_backpressure_error(e) => self.bp.record_backpressure(),
            Err(_) => {}
        }
        self.bp.release();
        result
    }

    /// Build a generated-client [`Configuration`] with the base URL set and the current
    /// authentication applied (refreshing the OAuth token if needed).
    ///
    /// Use this to call generated operations that the facade does not yet wrap:
    ///
    /// ```no_run
    /// # use camunda_orchestration_sdk::CamundaClient;
    /// use camunda_orchestration_sdk::client::apis::authentication_api;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CamundaClient::from_env()?;
    /// let cfg = client.configuration().await?;
    /// let me = authentication_api::get_authentication(&cfg).await?;
    /// # let _ = me;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn configuration(&self) -> Result<Configuration> {
        let mut cfg = Configuration {
            base_path: self.config.rest_address.clone(),
            user_agent: Some(self.user_agent.clone()),
            client: self.http.clone(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: None,
        };
        self.auth.apply(&mut cfg).await?;
        Ok(cfg)
    }

    // --- Ergonomic operation wrappers ---------------------------------------

    /// Fetch the cluster topology.
    pub async fn topology(&self) -> Result<models::TopologyResponse> {
        self.guarded(|| async {
            let cfg = self.configuration().await?;
            Ok(cluster_api::get_topology(&cfg).await?)
        })
        .await
    }

    /// Probe the gateway once for nanobpmn falcon support. Returns `None` for
    /// stock Camunda (or when disabled by config). Cached for the client's lifetime.
    pub(crate) async fn nano_caps(&self) -> Option<&NanoCaps> {
        if !self.config.nano_falcon {
            return None;
        }
        self.nano
            .caps
            .get_or_init(|| async {
                super::nano::detect(&self.config.rest_address, &self.http).await
            })
            .await
            .as_ref()
    }

    /// The shared, lazily-built nano create producer (one persistent, failover-capable
    /// link per client, dialing the cluster's falcon directory).
    async fn nano_producer(&self, caps: &NanoCaps) -> Result<&Arc<NanoProducer>> {
        let endpoints = caps.endpoints.clone();
        let submit_timeout = self
            .config
            .nano_submit_timeout_ms
            .map(std::time::Duration::from_millis);
        self.nano
            .producer
            .get_or_try_init(|| async { NanoProducer::start(endpoints, submit_timeout).await })
            .await
    }

    /// Create (start) a process instance. The configured default tenant id is applied when
    /// the instruction does not already specify one.
    pub async fn create_process_instance(
        &self,
        instruction: models::ProcessInstanceCreationInstruction,
    ) -> Result<models::CreateProcessInstanceResult> {
        let instruction = self.inject_instance_tenant(instruction);

        // nanobpmn upgrade: route the create over the credit-metered Falcon.
        if self.nano_caps().await.is_some() {
            if let Some(result) = self.create_process_instance_nano(&instruction).await? {
                return Ok(result);
            }
        }

        self.guarded(|| {
            let instruction = instruction.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = process_instance_api::CreateProcessInstanceParams {
                    process_instance_creation_instruction: instruction,
                };
                Ok(process_instance_api::create_process_instance(&cfg, params).await?)
            }
        })
        .await
    }

    /// Create a process instance over the nano Falcon. Returns `Ok(None)` to
    /// signal a transparent fall-back to REST (e.g. the socket could not be opened).
    async fn create_process_instance_nano(
        &self,
        instruction: &models::ProcessInstanceCreationInstruction,
    ) -> Result<Option<models::CreateProcessInstanceResult>> {
        use models::ProcessInstanceCreationInstruction as I;

        // Extract the fields the Falcon understands. The stream create takes a
        // process-definition id OR key + variables (+ awaitCompletion/fetchVariables).
        let (
            id,
            key,
            variables,
            await_completion,
            fetch_variables,
            request_timeout,
            version,
            tenant,
        ) = match instruction {
            I::ProcessInstanceCreationInstructionById(b) => (
                Some(b.process_definition_id.value().to_string()),
                None,
                b.variables.clone(),
                b.await_completion.unwrap_or(false),
                b.fetch_variables.clone(),
                b.request_timeout,
                b.process_definition_version,
                b.tenant_id.clone(),
            ),
            I::ProcessInstanceCreationInstructionByKey(b) => (
                None,
                Some(b.process_definition_key.value().to_string()),
                b.variables.clone(),
                b.await_completion.unwrap_or(false),
                b.fetch_variables.clone(),
                b.request_timeout,
                b.process_definition_version,
                b.tenant_id.clone(),
            ),
        };

        let Some(caps) = self.nano_caps().await.cloned() else {
            return Ok(None);
        };
        let producer = match self.nano_producer(&caps).await {
            Ok(p) => p.clone(),
            // Socket unavailable: fall back to REST rather than failing the create.
            Err(e) => {
                tracing::warn!(error = %e, "nano producer unavailable; falling back to REST");
                return Ok(None);
            }
        };

        let variables_map = variables.map(|m| m.into_iter().collect());
        let outcome = producer
            .create(
                id.as_deref(),
                key.as_deref(),
                variables_map,
                await_completion,
                fetch_variables,
                request_timeout,
            )
            .await?;
        tracing::debug!(
            process_instance_key = %outcome.process_instance_key,
            process_completed = outcome.process_completed,
            "created process instance over nano Falcon"
        );

        // The stream create returns the instance key (+ completion outcome). Synthesise a
        // typed result, echoing the request's definition identity (the stream ack does not
        // re-send every definition field) and any awaited output variables.
        let tenant_id = tenant
            .map(|t| models::TenantId::assume_exists(t.value().to_string()))
            .or_else(|| {
                self.config
                    .default_tenant_id
                    .clone()
                    .map(models::TenantId::assume_exists)
            })
            .unwrap_or_else(|| models::TenantId::assume_exists("<default>"));

        let result_variables = outcome
            .variables
            .and_then(|v| match v {
                serde_json::Value::Object(m) => Some(m.into_iter().collect()),
                _ => None,
            })
            .unwrap_or_default();

        Ok(Some(models::CreateProcessInstanceResult {
            process_definition_id: models::ProcessDefinitionId::assume_exists(
                id.unwrap_or_default(),
            ),
            process_definition_version: version.unwrap_or(0),
            tenant_id,
            variables: result_variables,
            process_definition_key: Box::new(models::ProcessDefinitionKey::assume_exists(
                key.unwrap_or_default(),
            )),
            process_instance_key: Box::new(models::ProcessInstanceKey::assume_exists(
                outcome.process_instance_key,
            )),
            tags: Vec::new(),
            business_id: None,
        }))
    }

    /// Fetch a process instance by key (a read; not subject to backpressure). Returns a
    /// `404` [`CamundaError::Api`] if not yet visible — compose with [`CamundaClient::eventual`]
    /// to wait through replication lag.
    pub async fn get_process_instance(
        &self,
        process_instance_key: &str,
    ) -> Result<models::ProcessInstanceResult> {
        let cfg = self.configuration().await?;
        let params = process_instance_api::GetProcessInstanceParams {
            process_instance_key: process_instance_key.to_string(),
        };
        Ok(process_instance_api::get_process_instance(&cfg, params).await?)
    }

    /// Cancel a running process instance by key.
    pub async fn cancel_process_instance(
        &self,
        process_instance_key: &str,
        request: Option<models::CancelProcessInstanceRequest>,
    ) -> Result<()> {
        self.guarded(|| {
            let request = request.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = process_instance_api::CancelProcessInstanceParams {
                    process_instance_key: process_instance_key.to_string(),
                    cancel_process_instance_request: request,
                };
                Ok(process_instance_api::cancel_process_instance(&cfg, params).await?)
            }
        })
        .await
    }

    /// Deploy one or more resources (BPMN, DMN, forms) from local file paths. The
    /// configured default tenant id is applied when `tenant_id` is `None`.
    pub async fn deploy_resources(
        &self,
        resources: Vec<std::path::PathBuf>,
        tenant_id: Option<String>,
    ) -> Result<models::DeploymentResult> {
        let tenant_id = tenant_id.or_else(|| self.config.default_tenant_id.clone());
        self.guarded(|| {
            let resources = resources.clone();
            let tenant_id = tenant_id.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = resource_api::CreateDeploymentParams {
                    resources,
                    tenant_id,
                };
                Ok(resource_api::create_deployment(&cfg, params).await?)
            }
        })
        .await
    }

    // --- Messaging, signals, decisions --------------------------------------

    /// Publish a message (no correlation key matching against active subscriptions only —
    /// buffered). The configured default tenant id is applied when unset.
    pub async fn publish_message(
        &self,
        mut request: models::MessagePublicationRequest,
    ) -> Result<models::MessagePublicationResult> {
        request.tenant_id = self.with_default_tenant(request.tenant_id);
        self.guarded(|| {
            let request = request.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = message_api::PublishMessageParams {
                    message_publication_request: request,
                };
                Ok(message_api::publish_message(&cfg, params).await?)
            }
        })
        .await
    }

    /// Correlate a message to a waiting process instance. The configured default tenant id
    /// is applied when unset.
    pub async fn correlate_message(
        &self,
        mut request: models::MessageCorrelationRequest,
    ) -> Result<models::MessageCorrelationResult> {
        request.tenant_id = self.with_default_tenant(request.tenant_id);
        self.guarded(|| {
            let request = request.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = message_api::CorrelateMessageParams {
                    message_correlation_request: request,
                };
                Ok(message_api::correlate_message(&cfg, params).await?)
            }
        })
        .await
    }

    /// Broadcast a signal. The configured default tenant id is applied when unset.
    pub async fn broadcast_signal(
        &self,
        mut request: models::SignalBroadcastRequest,
    ) -> Result<models::SignalBroadcastResult> {
        request.tenant_id = self.with_default_tenant(request.tenant_id);
        self.guarded(|| {
            let request = request.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = signal_api::BroadcastSignalParams {
                    signal_broadcast_request: request,
                };
                Ok(signal_api::broadcast_signal(&cfg, params).await?)
            }
        })
        .await
    }

    /// Evaluate a decision (DMN). The configured default tenant id is applied when unset.
    pub async fn evaluate_decision(
        &self,
        instruction: models::DecisionEvaluationInstruction,
    ) -> Result<models::EvaluateDecisionResult> {
        let instruction = self.inject_decision_tenant(instruction);
        self.guarded(|| {
            let instruction = instruction.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = decision_definition_api::EvaluateDecisionParams {
                    decision_evaluation_instruction: instruction,
                };
                Ok(decision_definition_api::evaluate_decision(&cfg, params).await?)
            }
        })
        .await
    }

    // --- Variable search ----------------------------------------------------

    /// Search variables, returning the raw generated result.
    pub async fn search_variables(
        &self,
        request: models::VariableSearchQuery,
    ) -> Result<models::VariableSearchQueryResult> {
        self.guarded(|| {
            let request = request.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = variable_api::SearchVariablesParams {
                    variable_search_query: Some(request),
                    truncate_values: None,
                };
                Ok(variable_api::search_variables(&cfg, params).await?)
            }
        })
        .await
    }

    /// Search variables and deserialize each variable's JSON `value` into a typed `T`.
    ///
    /// Returns the deserialized values in result order; variables whose `value` is absent
    /// are skipped.
    pub async fn search_variables_as<T: serde::de::DeserializeOwned>(
        &self,
        request: models::VariableSearchQuery,
    ) -> Result<Vec<T>> {
        let result = self.search_variables(request).await?;
        let mut out = Vec::with_capacity(result.items.len());
        for item in result.items {
            let parsed: T =
                serde_json::from_str(&item.value).map_err(CamundaError::Serialization)?;
            out.push(parsed);
        }
        Ok(out)
    }

    // --- Eventual consistency -----------------------------------------------

    /// Poll a read operation until it returns `Ok`, retrying `404 Not Found` (the typical
    /// symptom of a not-yet-replicated read) until the eventual-consistency window elapses.
    ///
    /// The default timeout is `CAMUNDA_SDK_EVENTUAL_POLL_DEFAULT_MS`; override per-call with
    /// [`ConsistencyOptions`].
    pub async fn eventual<T, F, Fut>(&self, options: ConsistencyOptions, op: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        super::eventual::poll(&options, self.config.eventual_poll_default_ms, op, |_| true).await
    }

    /// Poll a read operation until `predicate` is satisfied by its result, retrying `404`
    /// and ignoring consistent-but-not-yet-matching reads, until the eventual-consistency
    /// window elapses.
    pub async fn eventual_until<T, F, Fut, P>(
        &self,
        options: ConsistencyOptions,
        op: F,
        predicate: P,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
        P: FnMut(&T) -> bool,
    {
        super::eventual::poll(
            &options,
            self.config.eventual_poll_default_ms,
            op,
            predicate,
        )
        .await
    }

    /// Inject the configured default tenant id into a process-instance creation instruction
    /// when none is set.
    fn inject_instance_tenant(
        &self,
        instruction: models::ProcessInstanceCreationInstruction,
    ) -> models::ProcessInstanceCreationInstruction {
        use models::ProcessInstanceCreationInstruction as I;
        match instruction {
            I::ProcessInstanceCreationInstructionById(mut b) => {
                b.tenant_id = self.with_default_tenant(b.tenant_id);
                I::ProcessInstanceCreationInstructionById(b)
            }
            I::ProcessInstanceCreationInstructionByKey(mut b) => {
                b.tenant_id = self.with_default_tenant(b.tenant_id);
                I::ProcessInstanceCreationInstructionByKey(b)
            }
        }
    }

    /// Inject the configured default tenant id into a decision-evaluation instruction when
    /// none is set.
    fn inject_decision_tenant(
        &self,
        instruction: models::DecisionEvaluationInstruction,
    ) -> models::DecisionEvaluationInstruction {
        use models::DecisionEvaluationInstruction as I;
        match instruction {
            I::DecisionEvaluationById(mut b) => {
                b.tenant_id = self.with_default_tenant(b.tenant_id);
                I::DecisionEvaluationById(b)
            }
            I::DecisionEvaluationByKey(mut b) => {
                b.tenant_id = self.with_default_tenant(b.tenant_id);
                I::DecisionEvaluationByKey(b)
            }
        }
    }

    // --- Job operations (also used by the job worker) -----------------------

    /// Activate jobs of a given type. Prefer [`CamundaClient::create_job_worker`] for
    /// continuous polling.
    pub async fn activate_jobs(
        &self,
        request: models::JobActivationRequest,
    ) -> Result<models::JobActivationResult> {
        self.guarded(|| {
            let request = request.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = job_api::ActivateJobsParams {
                    job_activation_request: request,
                };
                Ok(job_api::activate_jobs(&cfg, params).await?)
            }
        })
        .await
    }

    /// Complete a job, optionally with output variables.
    ///
    /// Job completion is a *drain* operation and intentionally bypasses the backpressure
    /// gate so in-flight work can always be drained, even while new load is being shed.
    pub async fn complete_job(
        &self,
        job_key: &str,
        request: Option<models::JobCompletionRequest>,
    ) -> Result<()> {
        let cfg = self.configuration().await?;
        let params = job_api::CompleteJobParams {
            job_key: job_key.to_string(),
            job_completion_request: request,
        };
        Ok(job_api::complete_job(&cfg, params).await?)
    }

    /// Fail a job, decrementing retries.
    pub async fn fail_job(
        &self,
        job_key: &str,
        request: Option<models::JobFailRequest>,
    ) -> Result<()> {
        let cfg = self.configuration().await?;
        let params = job_api::FailJobParams {
            job_key: job_key.to_string(),
            job_fail_request: request,
        };
        Ok(job_api::fail_job(&cfg, params).await?)
    }

    /// Throw a BPMN error from a job.
    pub async fn throw_job_error(
        &self,
        job_key: &str,
        request: models::JobErrorRequest,
    ) -> Result<()> {
        let cfg = self.configuration().await?;
        let params = job_api::ThrowJobErrorParams {
            job_key: job_key.to_string(),
            job_error_request: request,
        };
        Ok(job_api::throw_job_error(&cfg, params).await?)
    }

    // --- Job workers --------------------------------------------------------

    /// Build a [`JobWorkerConfig`] for `job_type` pre-seeded from the SDK's resolved
    /// worker defaults (env-driven: `CAMUNDA_WORKER_*`). Builder methods override fields.
    pub fn worker_config(&self, job_type: impl Into<String>) -> JobWorkerConfig {
        JobWorkerConfig::from_defaults(job_type, &self.config.worker_defaults)
    }

    /// Create a [`JobWorker`] that continuously polls for and processes jobs of the
    /// configured type. Call [`JobWorker::run`] (or [`JobWorker::start`] /
    /// [`JobWorker::spawn`]) to begin.
    pub fn create_job_worker(&self, config: JobWorkerConfig) -> JobWorker {
        JobWorker::new(self.clone(), config)
    }

    /// Spawn a managed job worker and register it for lifecycle control. The returned
    /// handle can stop the individual worker; [`CamundaClient::stop_all_workers`] stops
    /// every registered worker. The worker is also retained by the client so it keeps
    /// running even if the returned handle is dropped.
    pub fn spawn_worker<F, Fut>(&self, config: JobWorkerConfig, handler: F)
    where
        F: Fn(Job) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = JobAction> + Send + 'static,
    {
        let handle = JobWorker::new(self.clone(), config).spawn(handler);
        self.workers
            .lock()
            .expect("worker registry poisoned")
            .push(handle);
    }

    /// The job types of all currently-registered workers that are still running.
    pub fn running_workers(&self) -> Vec<String> {
        self.workers
            .lock()
            .expect("worker registry poisoned")
            .iter()
            .filter(|w| !w.is_finished())
            .map(|w| w.job_type().to_string())
            .collect()
    }

    /// Gracefully stop every registered worker, letting in-flight jobs drain, and await
    /// their completion. Clears the registry.
    pub async fn stop_all_workers(&self) -> Result<()> {
        let handles: Vec<JobWorkerHandle> = {
            let mut guard = self.workers.lock().expect("worker registry poisoned");
            std::mem::take(&mut *guard)
        };
        let mut first_err = None;
        for handle in handles {
            if let Err(e) = handle.shutdown().await {
                tracing::warn!(error = %e, "worker did not shut down cleanly");
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}
