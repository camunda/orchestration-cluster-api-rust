//! The ergonomic [`CamundaClient`] facade over the generated low-level client.

use std::collections::HashMap;
use std::sync::Arc;

use camunda_orchestration_api_client::apis::configuration::Configuration;
use camunda_orchestration_api_client::apis::{
    clock_api, cluster_api, conditional_api, decision_definition_api, expression_api, job_api,
    message_api, process_instance_api, resource_api, signal_api, variable_api,
};
use camunda_orchestration_api_client::models;

use super::auth::Authentication;
use super::backpressure::{is_backpressure_error, BackpressureManager, BackpressureState};
use super::clock::{self, Clock};
use super::config::CamundaConfig;
use super::errors::{CamundaError, Result};
use super::eventual::ConsistencyOptions;
use super::falcon::{FalconCaps, FalconProducer};
use super::job_worker::{Job, JobAction, JobWorker, JobWorkerConfig, JobWorkerHandle};
use super::random::{self, Random};
use super::{retry, tls};

/// Lazily-resolved nanobpmn command-stream state, shared across client clones.
#[derive(Clone, Default)]
pub(crate) struct FalconState {
    /// `None` until first probe; `Some(None)` = stock Camunda, `Some(Some(_))` = nanobpmn (Falcon).
    caps: Arc<tokio::sync::OnceCell<Option<FalconCaps>>>,
    /// Shared create producer, built on first falcon create.
    producer: Arc<tokio::sync::OnceCell<Arc<FalconProducer>>>,
}

/// Options for constructing a [`CamundaClient`].
///
/// Non-exhaustive: construct with [`CamundaOptions::new`] and the `with_*` builders, so that
/// later additions here are not breaking changes for callers.
#[derive(Default)]
#[non_exhaustive]
pub struct CamundaOptions {
    /// Programmatic overrides for `CAMUNDA_*` configuration keys. These take precedence
    /// over environment variables.
    pub config: HashMap<String, String>,
    /// A pre-built `reqwest::Client` to use for all requests (including OAuth token
    /// fetches). When `None`, a default client is created.
    pub http_client: Option<reqwest::Client>,
    /// The clock the client's cadence resolves through. When `None`, real time is used.
    pub clock: Option<Arc<dyn Clock>>,
    /// The source jitter is drawn from. When `None`, [`LiveRandom`](super::random::LiveRandom)
    /// is used.
    pub random: Option<Arc<dyn Random>>,
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

    /// Resolve the client's cadence through `clock` instead of real time.
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Draw jitter (retry backoff, worker startup delay, FALCON endpoint selection) from
    /// `random`. Pass a [`SeededRandom`](super::random::SeededRandom) to make it
    /// reproducible.
    pub fn with_random(mut self, random: Arc<dyn Random>) -> Self {
        self.random = Some(random);
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
    falcon: FalconState,
    clock: Arc<dyn Clock>,
    random: Arc<dyn Random>,
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
        let clock = options.clock.unwrap_or_else(clock::live_clock);
        let auth = Authentication::from_config(&config, http.clone(), clock.clone());
        let bp = Arc::new(BackpressureManager::new(
            config.backpressure_profile,
            clock.clone(),
        ));
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
            falcon: FalconState::default(),
            clock,
            random: options.random.unwrap_or_else(random::live_random),
        })
    }

    /// The resolved configuration.
    pub fn config(&self) -> &CamundaConfig {
        &self.config
    }

    /// The clock this client's cadence resolves through.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// The source this client draws jitter from.
    pub fn random(&self) -> &Arc<dyn Random> {
        &self.random
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
        apply_default_tenant(provided, &self.config)
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
        let _permit = self.bp.acquire_guard().await?;
        let result = retry::with_retry(
            &self.config.retry,
            self.clock.as_ref(),
            self.random.as_ref(),
            &op,
        )
        .await;
        match &result {
            Ok(_) => self.bp.record_healthy_hint(),
            Err(e) if is_backpressure_error(e) => self.bp.record_backpressure(),
            Err(_) => {}
        }
        result
    }

    /// Run a **non-idempotent** initiating operation under the adaptive backpressure gate
    /// but *without* the transient-retry policy: acquire a permit, await the operation
    /// exactly once, record the outcome, and release the permit.
    ///
    /// Use this for mutations that must not be auto-retried on an unknown outcome — a
    /// retry after a lost 5xx/network response would apply the effect twice. Backpressure
    /// accounting and auth/backpressure error classification are preserved; only the retry
    /// loop is skipped.
    pub(crate) async fn guarded_once<T, Fut>(&self, op: Fut) -> Result<T>
    where
        Fut: std::future::Future<Output = Result<T>>,
    {
        let _permit = self.bp.acquire_guard().await?;
        let result = op.await;
        match &result {
            Ok(_) => self.bp.record_healthy_hint(),
            Err(e) if is_backpressure_error(e) => self.bp.record_backpressure(),
            Err(_) => {}
        }
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
            rest_address_exact: self.config.rest_address_exact,
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

    /// Add a relative offset to the cluster's internal clock (`POST /clock/add`).
    ///
    /// Unlike most operations this is **not idempotent** — each call shifts the clock by
    /// the supplied offset — so it deliberately runs under the backpressure gate *without*
    /// the transient-retry policy (`Self::guarded_once`). Auto-retrying a lost 5xx or
    /// network response would risk applying the offset twice. Callers that want at-least /
    /// exactly-once semantics must handle an uncertain outcome themselves (e.g. by reading
    /// the clock back).
    pub async fn add_clock(
        &self,
        params: clock_api::AddClockParams,
    ) -> Result<models::ClockResult> {
        self.guarded_once(async move {
            // Build the configuration (which refreshes the OAuth token if needed) *after*
            // the permit is acquired, so a request queued behind a capped gate does not
            // send a token that expired while it waited.
            let cfg = self.configuration().await?;
            Ok(clock_api::add_clock(&cfg, params).await?)
        })
        .await
    }

    /// Probe the gateway once for nanobpmn command-stream support. Returns `None` for
    /// stock Camunda (or when disabled by config). Cached for the client's lifetime.
    pub(crate) async fn falcon_caps(&self) -> Option<&FalconCaps> {
        if !self.config.falcon {
            return None;
        }
        self.falcon
            .caps
            .get_or_init(|| async {
                super::falcon::detect(
                    &self.config.rest_address,
                    self.config.rest_address_exact,
                    &self.http,
                )
                .await
            })
            .await
            .as_ref()
    }

    /// The shared, lazily-built falcon create producer (one persistent, failover-capable
    /// link per client, dialing the cluster's command-stream directory).
    async fn falcon_producer(&self, caps: &FalconCaps) -> Result<&Arc<FalconProducer>> {
        let endpoints = caps.endpoints.clone();
        self.falcon
            .producer
            .get_or_try_init(|| async {
                FalconProducer::start(endpoints, self.random.clone()).await
            })
            .await
    }

    /// Create (start) a process instance. The configured default tenant id is applied when
    /// the instruction does not already specify one.
    pub async fn create_process_instance(
        &self,
        instruction: models::ProcessInstanceCreationInstruction,
    ) -> Result<models::CreateProcessInstanceResult> {
        let instruction = self.inject_instance_tenant(instruction);

        // Falcon upgrade: route the create over the credit-metered command stream.
        if self.falcon_caps().await.is_some() {
            if let Some(result) = self.create_process_instance_falcon(&instruction).await? {
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

    /// Create a process instance over the falcon stream. Returns `Ok(None)` to
    /// signal a transparent fall-back to REST (e.g. the socket could not be opened).
    async fn create_process_instance_falcon(
        &self,
        instruction: &models::ProcessInstanceCreationInstruction,
    ) -> Result<Option<models::CreateProcessInstanceResult>> {
        use models::ProcessInstanceCreationInstruction as I;

        // Extract the fields the command stream understands. The stream create takes a
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
            // The command stream resolves a definition by id or key only; a version-tag
            // lookup is REST-only, so fall back transparently.
            I::ProcessInstanceCreationInstructionByVersionTag(_) => return Ok(None),
        };

        let Some(caps) = self.falcon_caps().await.cloned() else {
            return Ok(None);
        };
        let producer = match self.falcon_producer(&caps).await {
            Ok(p) => p.clone(),
            // Socket unavailable: fall back to REST rather than failing the create.
            Err(e) => {
                tracing::warn!(error = %e, "falcon producer unavailable; falling back to REST");
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
            "created process instance over falcon stream"
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

    /// Deploy resources (`POST /deployments`). The configured default tenant id is applied
    /// when the params carry no `tenant_id`.
    ///
    /// This is the params-struct twin of [`CamundaClient::deploy_resources`], kept so the
    /// generated-facade surface applies the default tenant like the sibling SDKs do.
    pub async fn create_deployment(
        &self,
        params: resource_api::CreateDeploymentParams,
    ) -> Result<models::DeploymentResult> {
        let tenant_id = apply_default_tenant_str(params.tenant_id, &self.config);
        self.guarded(|| {
            let resources = params.resources.clone();
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

    /// Evaluate root level conditional start events (`POST /conditionals/evaluation`). The
    /// configured default tenant id is applied when the instruction carries no `tenant_id`.
    pub async fn evaluate_conditionals(
        &self,
        params: conditional_api::EvaluateConditionalsParams,
    ) -> Result<models::EvaluateConditionalResult> {
        let mut instruction = params.conditional_evaluation_instruction;
        instruction.tenant_id = apply_default_tenant(instruction.tenant_id, &self.config);
        self.guarded(|| {
            let instruction = instruction.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = conditional_api::EvaluateConditionalsParams {
                    conditional_evaluation_instruction: instruction,
                };
                Ok(conditional_api::evaluate_conditionals(&cfg, params).await?)
            }
        })
        .await
    }

    /// Evaluate an expression (`POST /expression/evaluation`). The configured default
    /// tenant id is applied when the request carries no `tenant_id`.
    pub async fn evaluate_expression(
        &self,
        params: expression_api::EvaluateExpressionParams,
    ) -> Result<models::ExpressionEvaluationResult> {
        let mut request = params.expression_evaluation_request;
        request.tenant_id = apply_default_tenant_str(request.tenant_id, &self.config);
        self.guarded(|| {
            let request = request.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = expression_api::EvaluateExpressionParams {
                    expression_evaluation_request: request,
                };
                Ok(expression_api::evaluate_expression(&cfg, params).await?)
            }
        })
        .await
    }

    // --- Variable search ----------------------------------------------------

    /// Search variables, returning the raw generated result.
    ///
    /// Long variable values are truncated in the response (`truncateValues=true`, the
    /// server-side default) to match the sibling SDKs. Use
    /// [`CamundaClient::search_variables_with_options`] to pass an explicit
    /// `truncateValues` (for example `false` to read full values).
    pub async fn search_variables(
        &self,
        request: models::VariableSearchQuery,
    ) -> Result<models::VariableSearchQueryResult> {
        self.search_variables_with_options(request, None).await
    }

    /// Search variables with an explicit `truncateValues` query parameter.
    ///
    /// `truncate_values` defaults to `true` (the server-side default) when `None`, so the
    /// parameter is always sent; pass `Some(false)` to read full, untruncated values.
    pub async fn search_variables_with_options(
        &self,
        request: models::VariableSearchQuery,
        truncate_values: Option<bool>,
    ) -> Result<models::VariableSearchQueryResult> {
        let truncate_values = truncate_values.unwrap_or(true);
        self.guarded(|| {
            let request = request.clone();
            async move {
                let cfg = self.configuration().await?;
                let params = variable_api::SearchVariablesParams {
                    variable_search_query: Some(request),
                    truncate_values: Some(truncate_values),
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
        super::eventual::poll(
            &options,
            self.config.eventual_poll_default_ms,
            self.clock.as_ref(),
            op,
            |_| true,
        )
        .await
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
            self.clock.as_ref(),
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
            I::ProcessInstanceCreationInstructionByVersionTag(mut b) => {
                b.tenant_id = self.with_default_tenant(b.tenant_id);
                I::ProcessInstanceCreationInstructionByVersionTag(b)
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

/// Apply the configured default tenant id to an optional [`models::TenantId`], leaving an
/// explicitly-provided value untouched. Free function (rather than a method) so the
/// injection is unit-testable without standing up a full client.
pub(crate) fn apply_default_tenant(
    provided: Option<models::TenantId>,
    config: &CamundaConfig,
) -> Option<models::TenantId> {
    provided.or_else(|| {
        config
            .default_tenant_id
            .clone()
            .map(models::TenantId::assume_exists)
    })
}

/// Apply the configured default tenant id to an optional `String` tenant value, leaving an
/// explicitly-provided value untouched. Used by operations whose generated request model
/// carries the tenant as a plain `String` rather than a [`models::TenantId`] newtype.
pub(crate) fn apply_default_tenant_str(
    provided: Option<String>,
    config: &CamundaConfig,
) -> Option<String> {
    provided.or_else(|| config.default_tenant_id.clone())
}

#[async_trait::async_trait]
impl super::clock::ClockController for CamundaClient {
    async fn pin(&self, epoch_millis: i64) -> Result<()> {
        self.pin_clock(
            camunda_orchestration_api_client::apis::clock_api::PinClockParams {
                clock_pin_request: camunda_orchestration_api_client::models::ClockPinRequest::new(
                    epoch_millis,
                ),
            },
        )
        .await
    }

    async fn reset(&self) -> Result<()> {
        self.reset_clock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a config carrying the given default tenant id (and nothing else).
    fn cfg_with_tenant(tenant: Option<&str>) -> CamundaConfig {
        let mut map = std::collections::HashMap::new();
        if let Some(t) = tenant {
            map.insert("CAMUNDA_TENANT_ID".to_string(), t.to_string());
        }
        CamundaConfig::resolve(&|_| None, &map).expect("valid config")
    }

    #[test]
    fn default_tenant_fills_missing_newtype() {
        let cfg = cfg_with_tenant(Some("acme"));
        let out = apply_default_tenant(None, &cfg);
        assert_eq!(out.as_ref().map(|t| t.value()), Some("acme"));
    }

    #[test]
    fn default_tenant_keeps_explicit_newtype() {
        let cfg = cfg_with_tenant(Some("acme"));
        let explicit = models::TenantId::assume_exists("other");
        let out = apply_default_tenant(Some(explicit), &cfg);
        assert_eq!(out.as_ref().map(|t| t.value()), Some("other"));
    }

    #[test]
    fn default_tenant_none_when_unconfigured() {
        let cfg = cfg_with_tenant(None);
        assert!(apply_default_tenant(None, &cfg).is_none());
    }

    #[test]
    fn default_tenant_str_fills_missing() {
        let cfg = cfg_with_tenant(Some("acme"));
        assert_eq!(
            apply_default_tenant_str(None, &cfg).as_deref(),
            Some("acme")
        );
    }

    #[test]
    fn default_tenant_str_keeps_explicit() {
        let cfg = cfg_with_tenant(Some("acme"));
        let out = apply_default_tenant_str(Some("other".to_string()), &cfg);
        assert_eq!(out.as_deref(), Some("other"));
    }

    #[test]
    fn default_tenant_str_none_when_unconfigured() {
        let cfg = cfg_with_tenant(None);
        assert!(apply_default_tenant_str(None, &cfg).is_none());
    }

    #[test]
    fn tenant_id_roundtrips_through_string() {
        // The deployment param and expression request carry the tenant as a plain String;
        // the conditional instruction carries it as a TenantId newtype. Both must serialize
        // the same value onto the wire.
        let cfg = cfg_with_tenant(Some("acme"));
        let as_str = apply_default_tenant_str(None, &cfg).expect("injected");
        let as_newtype = apply_default_tenant(None, &cfg).expect("injected");
        assert_eq!(as_str, as_newtype.value());
    }

    fn client_with_tenant(tenant: Option<&str>) -> CamundaClient {
        let mut options = CamundaOptions::new();
        if let Some(t) = tenant {
            options = options.with("CAMUNDA_DEFAULT_TENANT_ID", t);
        }
        CamundaClient::new(options).expect("client builds")
    }

    fn tenant_of(
        instruction: &models::ProcessInstanceCreationInstruction,
    ) -> Option<&models::TenantId> {
        use models::ProcessInstanceCreationInstruction as I;
        match instruction {
            I::ProcessInstanceCreationInstructionById(b) => b.tenant_id.as_ref(),
            I::ProcessInstanceCreationInstructionByKey(b) => b.tenant_id.as_ref(),
            I::ProcessInstanceCreationInstructionByVersionTag(b) => b.tenant_id.as_ref(),
        }
    }

    #[test]
    fn injects_default_tenant_into_every_creation_instruction_variant() {
        use models::ProcessInstanceCreationInstruction as I;
        let client = client_with_tenant(Some("acme"));

        let by_id = I::ProcessInstanceCreationInstructionById(Box::new(
            models::ProcessInstanceCreationInstructionById::new(
                models::ProcessDefinitionId::assume_exists("order-process"),
            ),
        ));
        let by_key = I::ProcessInstanceCreationInstructionByKey(Box::new(
            models::ProcessInstanceCreationInstructionByKey::new(
                models::ProcessDefinitionKey::assume_exists("2251799813686749"),
            ),
        ));
        let by_version_tag = I::ProcessInstanceCreationInstructionByVersionTag(Box::new(
            models::ProcessInstanceCreationInstructionByVersionTag::new(
                models::ProcessDefinitionId::assume_exists("order-process"),
                "v2.1".to_string(),
            ),
        ));

        for instruction in [by_id, by_key, by_version_tag] {
            let injected = client.inject_instance_tenant(instruction);
            assert_eq!(
                tenant_of(&injected).map(|t| t.value()),
                Some("acme"),
                "default tenant must be injected into {injected:?}"
            );
        }
    }

    #[test]
    fn explicit_tenant_wins_over_the_default() {
        use models::ProcessInstanceCreationInstruction as I;
        let client = client_with_tenant(Some("acme"));

        let mut instruction = models::ProcessInstanceCreationInstructionByVersionTag::new(
            models::ProcessDefinitionId::assume_exists("order-process"),
            "v2.1".to_string(),
        );
        instruction.tenant_id = Some(models::TenantId::assume_exists("other"));
        let injected = client.inject_instance_tenant(
            I::ProcessInstanceCreationInstructionByVersionTag(Box::new(instruction)),
        );
        assert_eq!(tenant_of(&injected).map(|t| t.value()), Some("other"));
    }

    // --- Untagged-enum deserialization ordering (hook 16) --------------------

    #[test]
    fn version_tag_variant_wins_over_by_id_during_deserialization() {
        use models::ProcessInstanceCreationInstruction as I;
        // A payload carrying BOTH id and version tag must decode as the more-specific
        // ByVersionTag variant, not greedily as ById (which would silently drop the tag).
        let json = r#"{"processDefinitionId":"orders","processDefinitionVersionTag":"v2"}"#;
        let decoded: I = serde_json::from_str(json).expect("decodes");
        match &decoded {
            I::ProcessInstanceCreationInstructionByVersionTag(b) => {
                assert_eq!(b.process_definition_version_tag, "v2");
            }
            other => panic!("expected ByVersionTag, got {other:?}"),
        }
        // Round-trip must preserve the tag.
        let reencoded = serde_json::to_value(&decoded).expect("encodes");
        assert_eq!(reencoded["processDefinitionVersionTag"], "v2");
    }

    #[test]
    fn id_only_payload_still_decodes_as_by_id() {
        use models::ProcessInstanceCreationInstruction as I;
        let decoded: I =
            serde_json::from_str(r#"{"processDefinitionId":"orders"}"#).expect("decodes");
        assert!(matches!(
            decoded,
            I::ProcessInstanceCreationInstructionById(_)
        ));
    }

    #[test]
    fn key_payload_still_decodes_as_by_key() {
        use models::ProcessInstanceCreationInstruction as I;
        let decoded: I = serde_json::from_str(r#"{"processDefinitionKey":"2251799813686749"}"#)
            .expect("decodes");
        assert!(matches!(
            decoded,
            I::ProcessInstanceCreationInstructionByKey(_)
        ));
    }

    // --- Enum-backed exact-match wrappers (hook 15) --------------------------

    #[test]
    fn protection_mode_exact_match_preserves_the_enum_value() {
        use models::ProtectionModeFilterProperty as P;
        // A bare enum string must decode as the exact-match branch and round-trip.
        let decoded: P = serde_json::from_str(r#""REDACT""#).expect("decodes");
        match &decoded {
            P::ProtectionModeExactMatch(b) => {
                assert_eq!(b.0, models::ProtectionModeEnum::Redact);
            }
            other => panic!("expected ProtectionModeExactMatch, got {other:?}"),
        }
        assert_eq!(serde_json::to_value(&decoded).expect("encodes"), "REDACT");
    }

    #[test]
    fn protection_mode_advanced_filter_is_not_swallowed_by_exact_match() {
        use models::ProtectionModeFilterProperty as P;
        // The advanced-filter object must NOT be captured by the exact-match branch.
        let decoded: P = serde_json::from_str(r#"{"$in":["REDACT"]}"#).expect("decodes");
        match &decoded {
            P::AdvancedProtectionModeFilter(b) => {
                assert_eq!(
                    b.dollar_in.as_deref(),
                    Some(&[models::ProtectionModeEnum::Redact][..])
                );
            }
            other => panic!("expected AdvancedProtectionModeFilter, got {other:?}"),
        }
    }

    // --- Gated operations defer auth until a permit is acquired --------------

    // Regression: a gated, non-idempotent operation (`add_clock`) must build its
    // `Configuration` — which captures / refreshes the OAuth bearer token — *inside* the
    // guarded future, so the token is read only after a backpressure permit is acquired.
    // Capturing it before `guarded_once` meant a request queued behind a capped gate could
    // send a token that expired while it waited. We prove the ordering generically at the
    // `guarded_once` seam: with the only permit held, the op future must not be polled.
    #[tokio::test(flavor = "current_thread")]
    async fn guarded_once_defers_op_until_a_permit_is_available() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let client = client_with_tenant(None);
        // Pin the gate to a single permit and take it, so any further acquire must queue.
        client.bp.test_force_cap(1);
        let hold = client.bp.acquire_guard().await.expect("first permit");

        let polled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&polled);
        let worker = client.clone();
        let task = tokio::spawn(async move {
            worker
                .guarded_once(async move {
                    flag.store(true, Ordering::SeqCst);
                    Ok::<(), CamundaError>(())
                })
                .await
        });

        // Let the spawned task run until it parks waiting for the (held) permit.
        for _ in 0..16 {
            tokio::task::yield_now().await;
        }
        assert!(
            !polled.load(Ordering::SeqCst),
            "op future (which would capture the auth token) was polled before a permit was acquired"
        );

        // Releasing the permit lets the queued op acquire one and only then run.
        drop(hold);
        task.await.expect("task joins").expect("op succeeds");
        assert!(
            polled.load(Ordering::SeqCst),
            "op future must run once a permit is available"
        );
    }
}
